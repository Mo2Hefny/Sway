//! Physics systems: Verlet integration and boundary collision.

use bevy::prelude::*;

use super::Node;
use super::playground::Playground;
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

        // X axis
        if node.position.x - r < inner_min.x {
            node.position.x = inner_min.x + r;
            node.acceleration.x = node.acceleration.x.abs() * damping;
            node.prev_position.x = node.position.x + vel.x * damping;
        } else if node.position.x + r > inner_max.x {
            node.position.x = inner_max.x - r;
            node.acceleration.x = -node.acceleration.x.abs() * damping;
            node.prev_position.x = node.position.x + vel.x * damping;
        }

        // Y axis
        if node.position.y - r < inner_min.y {
            node.position.y = inner_min.y + r;
            node.acceleration.y = node.acceleration.y.abs() * damping;
            node.prev_position.y = node.position.y + vel.y * damping;
        } else if node.position.y + r > inner_max.y {
            node.position.y = inner_max.y - r;
            node.acceleration.y = -node.acceleration.y.abs() * damping;
            node.prev_position.y = node.position.y + vel.y * damping;
        }
    }
}
