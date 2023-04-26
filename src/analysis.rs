mod immutability_span;
mod top_of_borrow_stack;

pub use immutability_span::{
    FindImmutabilitySpans, ImmutabilitySpan, ImmutabilitySpanVisitor, ImmutabilitySpans,
};
pub use top_of_borrow_stack::{
    print_top_of_borrow_stack, TopOfBorrowStack, TopOfBorrowStackLocations, TopOfBorrowStackVisitor,
};

use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;
use rustc_mir_dataflow::Analysis;
use rustc_type_ir::TyKind;

pub fn compute_immutability_spans<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &Body<'tcx>,
    retagged: Vec<Local>,
    verbose: bool,
) -> ImmutabilitySpans {
    let param_env = tcx.param_env(body.source.def_id());

    // Only consider mutable references to Copy types in this analysis.
    let retagged = retagged
        .into_iter()
        .filter(|local| match body.local_decls[*local].ty.kind() {
            TyKind::Ref(_, ty, Mutability::Mut) => {
                // ty.is_trivially_pure_clone_copy() // TODO: Also consider non-trivial Copy types
                ty.is_copy_modulo_regions(tcx.at(body.span), param_env) // NOTE: Might not work correctly with some lifetime parameters, see `is_copy_modulo_regions`.
            }
            _ => false,
        })
        .collect();

    if verbose {
        println!("# TopOfBorrowStack Analysis");
    }
    let mut top_of_stack_visitor = TopOfBorrowStackVisitor::new();
    TopOfBorrowStack::new(retagged)
        .into_engine(tcx, body)
        .iterate_to_fixpoint()
        .visit_reachable_with(body, &mut top_of_stack_visitor);

    if verbose {
        print_top_of_borrow_stack(body, &top_of_stack_visitor.top_of_borrow_stack);
    }

    if verbose {
        println!("# FindImmutabilitySpans Analysis");
    }
    let mut immutability_span_visitor = ImmutabilitySpanVisitor::new(verbose);
    FindImmutabilitySpans::new(top_of_stack_visitor.top_of_borrow_stack)
        .into_engine(tcx, body)
        .iterate_to_fixpoint()
        .visit_reachable_with(body, &mut immutability_span_visitor);
    if verbose {
        println!();
    }

    return immutability_span_visitor.immutability_spans;
}
