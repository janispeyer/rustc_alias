use rustc_middle::{
    mir::{
        Body, LocalDecl, Location, Operand, Place, ProjectionElem, Rvalue, Statement, StatementKind,
    },
    ty::{List, TyCtxt},
};

use crate::analysis::{ImmutabilitySpan, ImmutabilitySpans};

pub fn move_up<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mut Body<'tcx>,
    immutability_spans: ImmutabilitySpans,
) {
    MoveUpOptimisation {
        tcx,
        body,
        assignments: Vec::new(),
    }
    .run(immutability_spans);
}

struct MoveUpOptimisation<'tcx, 'a> {
    tcx: TyCtxt<'tcx>,
    body: &'a mut Body<'tcx>,
    assignments: Vec<(Location, Statement<'tcx>)>,
}

impl<'tcx, 'a> MoveUpOptimisation<'tcx, 'a> {
    pub fn run(mut self, immutability_spans: ImmutabilitySpans) {
        // Sorting by location ensures deterministic output.
        let mut immutability_spans: Vec<_> = immutability_spans.into_iter().collect();
        immutability_spans.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        for (location, span) in immutability_spans {
            self.move_up_span(location, span);
        }

        // Add back assignments that were replaced.
        // This will make previously calculated locations invalid,
        // which is alsow why we need to start at the bottom.
        let mut assignments = self.assignments;
        assignments.sort_unstable_by(|(a, _), (b, _)| b.cmp(a));
        for (location, assignment) in assignments {
            let statements = &mut self.body[location.block].statements;
            statements.insert(location.statement_index + 1, assignment);
        }
    }

    // TODO: Check that we only do this optimisations for `Copy`-types.
    fn move_up_span(&mut self, location: Location, span: ImmutabilitySpan) {
        // Get original assignment.
        let basic_blocks = self.body.basic_blocks.as_mut();
        let mut statement = &mut basic_blocks[location.block].statements[location.statement_index];
        let mut original_statement = statement.replace_nop();

        let StatementKind::Assign(mut assignment) = original_statement.kind else {
            panic!("ImmutabiltySpan always has to start at an assignment, but was given one that starts wih {:?}", original_statement);
        };

        // Swap assignment of kind `(*x) = rvalue` with `internal = rvalue`.
        // This is done so we can replace reads of `*x` with `internal` where apropriate.
        let assignment_ty = assignment.0.ty(&self.body.local_decls, self.tcx).ty;
        let internal_local = self
            .body
            .local_decls
            .push(LocalDecl::new(assignment_ty, self.body.span));
        let internal_place = Place {
            local: internal_local,
            projection: List::empty(),
        };
        let internal_rvalue = Rvalue::Use(Operand::Copy(internal_place));

        let rvalue = std::mem::replace(&mut assignment.1, internal_rvalue.clone());
        statement.kind = StatementKind::Assign(Box::new((internal_place, rvalue)));
        original_statement.kind = StatementKind::Assign(assignment);
        self.assignments.push((location, original_statement));

        // Search for reads of `*x` and replace them with `internal`.
        // TODO: Handle more statement kinds than just assignments.
        for span_location in span.1 {
            let statement = self.body.stmt_at(span_location);
            let Some(StatementKind::Assign(assignment)) = statement.left().map(|s| &s.kind) else {
                continue;
            };

            let Rvalue::Use(Operand::Copy(place)) = assignment.1 else {
                continue;
            };

            if span.0 != place.local {
                continue;
            }

            let [ProjectionElem::Deref] = place.projection.as_slice() else {
                continue;
            };

            // At this point we know that the statement is an assignment of the form `? = (*x)`
            // and we can replace `(*x)` with `internal`.
            let statements = &mut self.body[span_location.block].statements;
            let statement = &mut statements[span_location.statement_index];
            let StatementKind::Assign(assignment) = &mut statement.kind else {
                unreachable!();
            };
            assignment.1 = internal_rvalue.clone();
        }
    }
}
