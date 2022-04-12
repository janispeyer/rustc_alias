use rustc_middle::mir::{Body, MirPass};
use rustc_middle::ty::TyCtxt;

pub struct Alias;

impl<'tcx> MirPass<'tcx> for Alias {
    fn is_enabled(&self, sess: &rustc_session::Session) -> bool {
        // Require retag statements for this MirPass to run.
        sess.opts.debugging_opts.mir_emit_retag
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
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
