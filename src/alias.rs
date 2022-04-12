use rustc_middle::mir::{
    BasicBlock, Body, Local, Location, MirPass, Operand, RetagKind, Rvalue, StatementKind,
};
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
