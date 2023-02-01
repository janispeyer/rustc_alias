mod top_of_borrow_stack;

pub use top_of_borrow_stack::{
    MaybeTopOfBorrowStack, MaybeTopOfBorrowStackVisitor, TopOfBorrowStackLocations,
};

use std::collections::HashMap;

use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;
use rustc_mir_dataflow::{
    fmt::DebugWithContext, Analysis, AnalysisDomain, CallReturnPlaces, JoinSemiLattice,
    ResultsVisitor,
};

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

fn print_top_of_borrow_stack(body: &Body, top_of_borrow_stack: &TopOfBorrowStackLocations) {
    for (block_index, _block) in body.basic_blocks.iter_enumerated() {
        println!("{:?}", block_index);
        let mut location: Location = block_index.start_location();

        loop {
            print!("[{:>2}] ", location.statement_index);
            let statement = body.stmt_at(location);

            let mut top_locals: Vec<_> = top_of_borrow_stack
                .iter()
                .filter(|(_, l)| *l == location)
                .map(|(local, _)| local)
                .collect();
            top_locals.sort();
            print!("{:?} <- ", top_locals);

            statement.either(
                |statement| println!("{:?}", statement),
                |terminator| println!("{:?}", terminator.kind),
            );

            if statement.is_right() {
                break;
            }

            location = location.successor_within_block();
        }
        println!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum ImmutabilitySpanState {
    Top,
    Span(Location),
    // Bottom, // Bottom is represented by not being present in the HashMap in ImmutabilitySpanLattice.
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct ImmutabilitySpanLattice(HashMap<Local, ImmutabilitySpanState>);

impl JoinSemiLattice for ImmutabilitySpanLattice {
    fn join(&mut self, other: &Self) -> bool {
        let mut modified_self = false;

        for (local, other_span) in other.0.iter() {
            let self_span = self.0.get(local);
            match (self_span, other_span) {
                // top join x = top
                (Some(ImmutabilitySpanState::Top), _) => {}
                // bottom join x = x
                (None, _) => {
                    self.0.insert(*local, other_span.clone());
                    modified_self = true;
                }
                // x join top = top
                (_, ImmutabilitySpanState::Top) => {
                    self.0.insert(*local, ImmutabilitySpanState::Top);
                    modified_self = true;
                }
                // Span(x) join Span(x) = Span(x)
                // Span(x) join Span(y) = top
                (Some(ImmutabilitySpanState::Span(x)), ImmutabilitySpanState::Span(y)) => {
                    if x != y {
                        self.0.insert(*local, ImmutabilitySpanState::Top);
                        modified_self = true;
                    }
                }
            }
        }

        // bottom join bottom = bottom
        // Case requires no special handling.

        modified_self
    }
}

impl DebugWithContext<FindImmutabilitySpans> for ImmutabilitySpanLattice {}

struct FindImmutabilitySpans {
    top_of_borrow_stack: TopOfBorrowStackLocations,
}

impl FindImmutabilitySpans {
    pub fn new(top_of_borrow_stack: TopOfBorrowStackLocations) -> Self {
        Self {
            top_of_borrow_stack,
        }
    }
}

impl<'tcx> AnalysisDomain<'tcx> for FindImmutabilitySpans {
    type Domain = ImmutabilitySpanLattice;
    type Direction = rustc_mir_dataflow::Forward;

    const NAME: &'static str = "find_immutability_spans";

    fn bottom_value(&self, _body: &rustc_middle::mir::Body<'tcx>) -> Self::Domain {
        ImmutabilitySpanLattice(HashMap::new())
    }

    fn initialize_start_block(
        &self,
        _body: &rustc_middle::mir::Body<'tcx>,
        _state: &mut Self::Domain,
    ) {
    }
}

impl<'tcx> Analysis<'tcx> for FindImmutabilitySpans {
    fn apply_statement_effect(
        &self,
        state: &mut Self::Domain,
        statement: &rustc_middle::mir::Statement<'tcx>,
        location: Location,
    ) {
        let mut deletions = Vec::new();
        for &local in state.0.keys() {
            let local_on_top = self.top_of_borrow_stack.contains(&(local, location));
            if !local_on_top {
                deletions.push(local);
            }
        }

        for local in deletions {
            state.0.insert(local, ImmutabilitySpanState::Top);
        }

        let StatementKind::Assign(ref assignment) = statement.kind else {
            return;
        };

        let local = assignment.0.local;
        let [ProjectionElem::Deref] = assignment.0.projection.as_slice() else {
            return;
        };

        let local_on_top = self.top_of_borrow_stack.contains(&(local, location));

        if local_on_top {
            state.0.insert(local, ImmutabilitySpanState::Span(location));
        }
    }

    fn apply_terminator_effect(
        &self,
        _state: &mut Self::Domain,
        _terminator: &rustc_middle::mir::Terminator<'tcx>,
        _location: Location,
    ) {
    }

    fn apply_call_return_effect(
        &self,
        _state: &mut Self::Domain,
        _block: BasicBlock,
        _return_places: CallReturnPlaces<'_, 'tcx>,
    ) {
    }
}

#[derive(Debug)]
pub struct ImmutabilitySpan(pub Local, pub Vec<Location>);

pub type ImmutabilitySpans = HashMap<Location, ImmutabilitySpan>;

struct ImmutabilitySpanVisitor {
    verbose: bool,
    immutability_spans: ImmutabilitySpans,
}

impl ImmutabilitySpanVisitor {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            immutability_spans: HashMap::new(),
        }
    }

    fn print_state(state: &<Self as ResultsVisitor>::FlowState, location: Location) {
        let mut state: Vec<_> = state.0.iter().collect();
        state.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        let state = state
            .iter()
            .map(|(local, span)| format!("{:?}: {:?}", local, span))
            .collect::<Vec<_>>()
            .join(", ");
        print!("[{:>2}] {{{}}} <- ", location.statement_index, state);
    }

    fn visit(&mut self, state: &<Self as ResultsVisitor>::FlowState, location: Location) {
        for (local, span) in &state.0 {
            let ImmutabilitySpanState::Span(assignment) = span else {
                continue;
            };

            self.immutability_spans
                .entry(*assignment)
                .or_insert(ImmutabilitySpan(*local, Vec::new()))
                .1
                .push(location);
        }
    }
}

impl<'mir, 'tcx> ResultsVisitor<'mir, 'tcx> for ImmutabilitySpanVisitor {
    type FlowState = ImmutabilitySpanLattice;

    fn visit_statement_after_primary_effect(
        &mut self,
        state: &Self::FlowState,
        statement: &'mir rustc_middle::mir::Statement<'tcx>,
        location: Location,
    ) {
        if self.verbose {
            Self::print_state(state, location);
            println!("{:?}", statement);
        }

        self.visit(state, location);
    }

    fn visit_terminator_after_primary_effect(
        &mut self,
        state: &Self::FlowState,
        terminator: &'mir rustc_middle::mir::Terminator<'tcx>,
        location: Location,
    ) {
        if self.verbose {
            Self::print_state(state, location);
            println!("{:?}", terminator.kind);
        }

        self.visit(state, location);
    }

    fn visit_block_start(
        &mut self,
        _state: &Self::FlowState,
        _block_data: &'mir rustc_middle::mir::BasicBlockData<'tcx>,
        block: BasicBlock,
    ) {
        if self.verbose {
            println!("{:?}", block);
        }
    }

    fn visit_block_end(
        &mut self,
        _state: &Self::FlowState,
        _block_data: &'mir rustc_middle::mir::BasicBlockData<'tcx>,
        _block: BasicBlock,
    ) {
        if self.verbose {
            println!();
        }
    }
}
