use std::collections::{HashMap, HashSet};

use rustc_index::bit_set::BitSet;

use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::*;

use rustc_middle::ty::TyCtxt;

use rustc_mir_dataflow::{
    Analysis, AnalysisDomain, CallReturnPlaces, GenKill, GenKillAnalysis, ResultsVisitor,
};

pub fn compute_immutability_set<'tcx>(tcx: TyCtxt<'tcx>, body: &Body<'tcx>, retagged: Vec<Local>) {
    let mut maybe_top_visitor = MaybeTopOfBorrowStackVisitor::new();
    let analysis_result = MaybeTopOfBorrowStack { retagged }
        .into_engine(tcx, body)
        .iterate_to_fixpoint()
        .visit_reachable_with(body, &mut maybe_top_visitor);

    for (bb_index, bb) in body.basic_blocks.iter_enumerated() {
        let statements_len = bb.statements.len();
        let mut location: Location = bb_index.start_location();

        while location.statement_index <= statements_len {
            let stmt = body.stmt_at(location);
            let mut top_locals: Vec<Local> = maybe_top_visitor
                .locations
                .iter()
                .filter(|(_, set)| set.contains(&location))
                .map(|(&local, _)| local)
                .collect();
            top_locals.sort();
            println!("{:?} -> {:?}", top_locals, stmt);

            location = location.successor_within_block();
        }
    }
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
                self.trans.kill(borrowed_place.local);
            }

            Rvalue::Cast(..)              // we probably have to cover casts: e.g. reference to pointer (TODO)
            | Rvalue::ShallowInitBox(..)  // performs transmute --> we have to handle this (TODO)
            | Rvalue::Use(..)
            | Rvalue::ThreadLocalRef(..)
            | Rvalue::Repeat(..)          // array initialiser: [value; repetitions]
            | Rvalue::Len(..)             // length of array or slice
            | Rvalue::BinaryOp(..)        // e.g. +, -, ...
            | Rvalue::CheckedBinaryOp(..) // e.g. +, -, ...
            | Rvalue::NullaryOp(..)       // sizeof | alignof
            | Rvalue::UnaryOp(..)         // not | neg
            | Rvalue::Discriminant(..)    // e.g. read tag of variant
            | Rvalue::Aggregate(..)
            | Rvalue::CopyForDeref(..) => {}
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

type TopOfBorrowStackLocations = HashMap<Local, HashSet<Location>>;

struct MaybeTopOfBorrowStackVisitor {
    locations: TopOfBorrowStackLocations,
}

impl MaybeTopOfBorrowStackVisitor {
    fn new() -> Self {
        Self {
            locations: HashMap::new(),
        }
    }

    fn visit_location(&mut self, state: &<Self as ResultsVisitor>::FlowState, location: Location) {
        for top in state.iter() {
            self.locations.entry(top).or_default().insert(location);
        }
    }
}

impl<'mir, 'tcx> ResultsVisitor<'mir, 'tcx> for MaybeTopOfBorrowStackVisitor {
    type FlowState = BitSet<Local>;

    fn visit_statement_after_primary_effect(
        &mut self,
        state: &Self::FlowState,
        _statement: &'mir rustc_middle::mir::Statement<'tcx>,
        location: Location,
    ) {
        self.visit_location(state, location);
    }

    fn visit_terminator_after_primary_effect(
        &mut self,
        state: &Self::FlowState,
        _terminator: &'mir rustc_middle::mir::Terminator<'tcx>,
        location: Location,
    ) {
        self.visit_location(state, location);
    }
}
