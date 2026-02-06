//! Iterative distance-constraint solver.

use bevy::prelude::*;

use crate::core::constants::CONSTRAINT_ITERATIONS;
use crate::core::components::{DistanceConstraint, Node, NodeType};
use crate::ui::state::PlaybackState;

/// Solves distance constraints using iterative position projection.
pub fn constraint_solving_system(
    playback: Res<PlaybackState>,
    constraint_query: Query<(Entity, &DistanceConstraint)>,
    mut nodes: Query<&mut Node>,
) {
    if !playback.is_playing() {
        return;
    }

    let constraints: Vec<(Entity, &DistanceConstraint)> = constraint_query.iter().collect();
    if constraints.is_empty() {
        return;
    }

    for _ in 0..CONSTRAINT_ITERATIONS {
        for (_, constraint) in constraints.iter() {
            solve_single_constraint(constraint, &mut nodes);
        }
    }
}

// =============================================================================
// Private Helpers
// =============================================================================

/// Solves a single distance constraint by projecting node positions.
fn solve_single_constraint(
    constraint: &DistanceConstraint,
    nodes: &mut Query<&mut Node>,
) {
    let (pos_a, pos_b, type_a, type_b) = {
        let Ok(a) = nodes.get(constraint.node_a) else { return };
        let Ok(b) = nodes.get(constraint.node_b) else { return };
        (a.position, b.position, a.node_type, b.node_type)
    };

    let (correction, w_a, w_b) = compute_correction(pos_a, pos_b, type_a, type_b, constraint.rest_length);
    let (Some(correction), Some(w_a), Some(w_b)) = (correction, w_a, w_b) else { return };

    if let Ok(mut a) = nodes.get_mut(constraint.node_a) {
        let shift = correction * w_a;
        a.position += shift;
        a.prev_position += shift;
    }
    if let Ok(mut b) = nodes.get_mut(constraint.node_b) {
        let shift = correction * w_b;
        b.position -= shift;
        b.prev_position -= shift;
    }
}

/// Computes the correction vector and weights for a pair of nodes.
fn compute_correction(
    pos_a: Vec2,
    pos_b: Vec2,
    type_a: NodeType,
    type_b: NodeType,
    rest_length: f32,
) -> (Option<Vec2>, Option<f32>, Option<f32>) {
    let delta = pos_b - pos_a;
    let dist = delta.length();
    if dist < 1e-6 {
        return (None, None, None);
    }

    let correction = delta / dist * (dist - rest_length);

    let (w_a, w_b) = match (type_a == NodeType::Anchor, type_b == NodeType::Anchor) {
        (true, true) => return (None, None, None),
        (true, false) => (0.0, 1.0),
        (false, true) => (1.0, 0.0),
        (false, false) => (0.5, 0.5),
    };

    (Some(correction), Some(w_a), Some(w_b))
}
