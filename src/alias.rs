use rustc_middle::mir::*;

use rustc_middle::ty::TyCtxt;

use crate::analysis::compute_immutability_span;

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

        compute_immutability_span(tcx, body, retagged.clone());

        println!("# CFG Before");
        println!("{:#?}", body.basic_blocks.as_mut());

        println!("# CFG After");
        println!("{:#?}", body.basic_blocks.as_mut());
        println!();
    }
}

/// Collect places that should be checked by seeing if they are retagged.
///
/// TODO: Remove types with interior mutability.
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
