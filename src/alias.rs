use rustc_middle::mir::*;

use rustc_middle::mir::visit::Visitor;
use rustc_middle::ty::TyCtxt;

use crate::analysis::compute_immutability_spans;
use crate::optimisation;

pub struct Alias;

impl<'tcx> MirPass<'tcx> for Alias {
    fn is_enabled(&self, sess: &rustc_session::Session) -> bool {
        // Require retag statements for this MirPass to run.
        sess.opts.unstable_opts.mir_emit_retag
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let def_id = body.source.def_id();
        let path = tcx.def_path_str(def_id);
        println!("# Analysing {}", path);

        let retagged = get_retags(body);
        if retagged.is_empty() {
            return; // Abort pass early, if there is nothing to do.
        }

        let immutability_spans = compute_immutability_spans(tcx, body, retagged.clone(), true);

        let mut printer = PrintBodyVisitor;
        println!("# CFG Before");
        printer.visit_body(body);

        optimisation::eliminate_reads(tcx, body, immutability_spans);

        println!("# CFG After");
        printer.visit_body(body);
    }
}

/// Collect places that should be checked by seeing if they are retagged.
fn get_retags<'tcx>(body: &mut Body<'tcx>) -> Vec<Local> {
    let Some(bb0_index) = body.basic_blocks.indices().nth(0) else {
        return Vec::new(); // no basic blocks ==> no retags
    };
    let bb0 = &body.basic_blocks[bb0_index];

    let mut retagged = Vec::new();
    for (_stmt_idx, stmt) in bb0.statements.iter().enumerate() {
        if let StatementKind::Retag(RetagKind::FnEntry, place) = &stmt.kind {
            retagged.push(place.local);
        }
    }
    retagged
}

struct PrintBodyVisitor;

impl<'tcx> Visitor<'tcx> for PrintBodyVisitor {
    fn visit_basic_block_data(&mut self, block: BasicBlock, data: &BasicBlockData<'tcx>) {
        println!("{:?}", block);
        self.super_basic_block_data(block, data);
        println!();
    }

    fn visit_statement(&mut self, statement: &Statement<'tcx>, location: Location) {
        self.super_statement(statement, location);
        println!("{:?}", statement);
    }

    fn visit_terminator(&mut self, terminator: &Terminator<'tcx>, location: Location) {
        self.super_terminator(terminator, location);
        println!("{:?}", terminator.kind);
    }
}
