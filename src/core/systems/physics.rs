//! Physics systems — Verlet integration and boundary collision.

use bevy::prelude::*;

use crate::core::components::{Node, Playground};
use crate::ui::state::PlaybackState;

/// Applies Verlet integration to all nodes.
pub fn verlet_integration_system(
    playback: Res<PlaybackState>,
    time: Res<Time>,
    mut nodes: Query<&mut Node>,
) {
    if !playback.is_playing() {
        return;
    }

    let dt = time.delta_secs();

    for mut node in nodes.iter_mut() {
        node.verlet_step(dt);
    }
}

/// Detects node-boundary collisions and applies impact response.
pub fn boundary_collision_system(
    playback: Res<PlaybackState>,
    playground: Res<Playground>,
    mut nodes: Query<&mut Node>,
) {
    if !playback.is_playing() {
        return;
    }

    let inner_min = playground.inner_min();
    let inner_max = playground.inner_max();
    let damping = playground.impact_damping;

    for mut node in nodes.iter_mut() {
        let r = node.radius;
        let vel = node.position - node.prev_position;

        let (px, ppx, ax) = clamp_axis(node.position.x, node.prev_position.x, node.acceleration.x, vel.x, inner_min.x + r, inner_max.x - r, damping);
        node.position.x = px;
        node.prev_position.x = ppx;
        node.acceleration.x = ax;

        let (py, ppy, ay) = clamp_axis(node.position.y, node.prev_position.y, node.acceleration.y, vel.y, inner_min.y + r, inner_max.y - r, damping);
        node.position.y = py;
        node.prev_position.y = ppy;
        node.acceleration.y = ay;
    }
}

// =============================================================================
// Private Helpers
// =============================================================================

/// Clamps a single axis to `[min, max]` and reflects velocity on impact.
fn clamp_axis(mut pos: f32, prev: f32, accel: f32, vel: f32, min: f32, max: f32, damping: f32) -> (f32, f32, f32) {
    if pos < min {
        pos = min;
        (pos, pos + vel * damping, accel.abs() * damping)
    } else if pos > max {
        pos = max;
        (pos, pos + vel * damping, -accel.abs() * damping)
    } else {
        (pos, prev, accel)
    }
}
