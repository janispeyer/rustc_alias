use rustc_index::bit_set::BitSet;
use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::*;
use rustc_mir_dataflow::{Analysis, AnalysisDomain, CallReturnPlaces, GenKill, GenKillAnalysis};

use rustc_middle::ty::TyCtxt;

pub struct Alias;

impl<'tcx> MirPass<'tcx> for Alias {
    fn is_enabled(&self, sess: &rustc_session::Session) -> bool {
        // Require retag statements for this MirPass to run.
        sess.opts.debugging_opts.mir_emit_retag
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let retagged = get_retags(body);
        if retagged.is_empty() {
            return; // Abort pass early, if there is nothing to do.
        }

        let mut analysis_result = MaybeTopOfBorrowStack { retagged }
            .into_engine(tcx, body)
            .iterate_to_fixpoint()
            .into_results_cursor(body);

        // Inspect the fixpoint state immediately before each `Drop` terminator.
        for (bb_index, bb) in body.basic_blocks().iter_enumerated() {
            let statements_len = bb.statements.len();
            let mut location: Location = bb_index.start_location();

            while location.statement_index <= statements_len {
                analysis_result.seek_before_primary_effect(location);
                let stmt = body.stmt_at(location);
                println!("{:?} -> {:?}", analysis_result.get(), stmt);

                location = location.successor_within_block();
            }
        }

        let def_id = body.source.def_id();
        let path = tcx.def_path_str(def_id);
        println!("# CFG for {}", path);
        println!("Before");
        println!("{:#?}", body.basic_blocks());

        println!("After");
        println!("{:#?}", body.basic_blocks());
        println!();
    }
}

/// Collect places that should be checked by seeing if they are retagged.
///
/// TODO: Remove types with interior mutability.
fn get_retags<'tcx>(body: &mut Body<'tcx>) -> Vec<Local> {
    let Some(bb0_index) = body.basic_blocks().indices().nth(0) else {
        return Vec::new(); // no basic blocks ==> no retags
    };
    let bb0 = &body.basic_blocks()[bb0_index];

    let mut retagged = Vec::new();
    for (stmt_idx, stmt) in bb0.statements.iter().enumerate() {
        if let StatementKind::Retag(RetagKind::FnEntry, place) = &stmt.kind {
            retagged.push(place.local);
        }
    }
    retagged
}

/// A dataflow analysis that tracks whether a given local is on the top of the borrow stack,
/// given the local is a reference.
pub struct MaybeTopOfBorrowStack {
    retagged: Vec<Local>,
}

impl MaybeTopOfBorrowStack {
    fn transfer_function<'a, T>(&'a self, trans: &'a mut T) -> TransferFunction<'a, T> {
        TransferFunction { trans }
    }
}

impl<'tcx> AnalysisDomain<'tcx> for MaybeTopOfBorrowStack {
    type Domain = BitSet<Local>;
    const NAME: &'static str = "maybe_top_of_borrow_stack";

    fn bottom_value(&self, body: &Body<'tcx>) -> Self::Domain {
        // bottom = TODO
        BitSet::new_empty(body.local_decls().len())
    }

    fn initialize_start_block(&self, _: &Body<'tcx>, bitset: &mut Self::Domain) {
        for &local in &self.retagged {
            bitset.insert(local);
        }
    }
}

impl<'tcx> GenKillAnalysis<'tcx> for MaybeTopOfBorrowStack {
    type Idx = Local;

    fn statement_effect(
        &self,
        trans: &mut impl GenKill<Self::Idx>,
        statement: &Statement<'tcx>,
        location: Location,
    ) {
        self.transfer_function(trans)
            .visit_statement(statement, location);
    }

    fn terminator_effect(
        &self,
        trans: &mut impl GenKill<Self::Idx>,
        terminator: &Terminator<'tcx>,
        location: Location,
    ) {
        self.transfer_function(trans)
            .visit_terminator(terminator, location);
    }

    fn call_return_effect(
        &self,
        _trans: &mut impl GenKill<Self::Idx>,
        _block: BasicBlock,
        _return_places: CallReturnPlaces<'_, 'tcx>,
    ) {
    }
}

/// A `Visitor` that defines the transfer function for `MaybeBorrowedLocals`.
struct TransferFunction<'a, T> {
    trans: &'a mut T,
}

impl<'tcx, T> Visitor<'tcx> for TransferFunction<'_, T>
where
    T: GenKill<Local>,
{
    fn visit_statement(&mut self, stmt: &Statement<'tcx>, location: Location) {
        self.super_statement(stmt, location);

        // When we reach a `StorageDead` statement, we can assume that any pointers to this memory
        // are now invalid.
        if let StatementKind::StorageDead(local) = stmt.kind {
            self.trans.kill(local);
        }
    }

    fn visit_rvalue(&mut self, rvalue: &Rvalue<'tcx>, location: Location) {
        self.super_rvalue(rvalue, location);

        match rvalue {
            Rvalue::AddressOf(_mt, borrowed_place) => {
                if !borrowed_place.is_indirect() {
                    self.trans.kill(borrowed_place.local);
                }
            }

            Rvalue::Ref(_, _kind, borrowed_place) => {
                if !borrowed_place.is_indirect() {
                    self.trans.kill(borrowed_place.local);
                }
            }

            Rvalue::Cast(..)
            | Rvalue::ShallowInitBox(..)
            | Rvalue::Use(..)
            | Rvalue::ThreadLocalRef(..)
            | Rvalue::Repeat(..)
            | Rvalue::Len(..)
            | Rvalue::BinaryOp(..)
            | Rvalue::CheckedBinaryOp(..)
            | Rvalue::NullaryOp(..)
            | Rvalue::UnaryOp(..)
            | Rvalue::Discriminant(..)
            | Rvalue::Aggregate(..) => {}
        }
    }

    fn visit_terminator(&mut self, terminator: &Terminator<'tcx>, location: Location) {
        self.super_terminator(terminator, location);

        match terminator.kind {
            TerminatorKind::Drop {
                place: dropped_place,
                ..
            }
            | TerminatorKind::DropAndReplace {
                place: dropped_place,
                ..
            } => {
                // Drop terminators may call custom drop glue (`Drop::drop`), which takes `&mut
                // self` as a parameter. In the general case, a drop impl could launder that
                // reference into the surrounding environment through a raw pointer, thus creating
                // a valid `*mut` pointing to the dropped local. We are not yet willing to declare
                // this particular case UB, so we must treat all dropped locals as mutably borrowed
                // for now. See discussion on [#61069].
                //
                // [#61069]: https://github.com/rust-lang/rust/pull/61069
                self.trans.kill(dropped_place.local);
            }

            TerminatorKind::Abort
            | TerminatorKind::Assert { .. }
            | TerminatorKind::Call { .. }
            | TerminatorKind::FalseEdge { .. }
            | TerminatorKind::FalseUnwind { .. }
            | TerminatorKind::GeneratorDrop
            | TerminatorKind::Goto { .. }
            | TerminatorKind::InlineAsm { .. }
            | TerminatorKind::Resume
            | TerminatorKind::Return
            | TerminatorKind::SwitchInt { .. }
            | TerminatorKind::Unreachable
            | TerminatorKind::Yield { .. } => {}
        }
    }
}
