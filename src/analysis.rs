mod immutability_span;
mod top_of_borrow_stack;

pub use immutability_span::{
    FindImmutabilitySpans, ImmutabilitySpan, ImmutabilitySpanVisitor, ImmutabilitySpans,
};
pub use top_of_borrow_stack::{
    print_top_of_borrow_stack, MaybeTopOfBorrowStack, MaybeTopOfBorrowStackVisitor,
    TopOfBorrowStackLocations,
};

use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;
use rustc_mir_dataflow::Analysis;

pub fn compute_immutability_spans<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &Body<'tcx>,
    retagged: Vec<Local>,
    verbose: bool,
) -> ImmutabilitySpans {
    // Only consider mutable references in this analysis.
    let retagged = retagged
        .into_iter()
        .filter(|local| match body.local_decls[*local].ty.kind() {
            rustc_type_ir::TyKind::Ref(_, ty, Mutability::Mut) => ty.is_primitive(),
            _ => false,
        })
        .collect();

    if verbose {
        println!("# MaybeTopOfBorrowStack Analysis");
    }
    let mut maybe_top_visitor = MaybeTopOfBorrowStackVisitor::new();
    MaybeTopOfBorrowStack::new(retagged)
        .into_engine(tcx, body)
        .iterate_to_fixpoint()
        .visit_reachable_with(body, &mut maybe_top_visitor);

    if verbose {
        print_top_of_borrow_stack(body, &maybe_top_visitor.top_of_borrow_stack);
    }

    if verbose {
        println!("# FindImmutabilitySpans Analysis");
    }
    let mut immutability_span_visitor = ImmutabilitySpanVisitor::new(verbose);
    FindImmutabilitySpans::new(maybe_top_visitor.top_of_borrow_stack)
        .into_engine(tcx, body)
        .iterate_to_fixpoint()
        .visit_reachable_with(body, &mut immutability_span_visitor);
    if verbose {
        println!();
    }

    return immutability_span_visitor.immutability_spans;
}
