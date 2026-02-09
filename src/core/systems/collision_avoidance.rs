use bevy::prelude::*;

use crate::core::components::{AnchorMovementMode, DistanceConstraint, Node, NodeType, Playground};
use crate::core::constants::MIN_COLLISION_DISTANCE;
use crate::core::utils::{find_connected_entities, get_constraint_neighbor};
use crate::ui::state::PlaybackState;

pub fn collision_avoidance_system(
    playback: Res<PlaybackState>,
    playground: Res<Playground>,
    constraint_query: Query<&DistanceConstraint>,
    mut nodes: Query<(Entity, &mut Node)>,
) {
    if !playback.is_playing() {
        return;
    }

    let inner_min = playground.inner_min();
    let inner_max = playground.inner_max();

    let constraints: Vec<(Entity, Entity)> = constraint_query.iter().map(|c| (c.node_a, c.node_b)).collect();

    let node_data: Vec<(Entity, Vec2, f32, f32)> = nodes
        .iter()
        .map(|(e, n)| (e, n.position, n.radius, n.collision_damping))
        .collect();

    for (entity, mut node) in nodes.iter_mut() {
        let is_procedural_anchor = is_procedural_anchor_node(&node);

        apply_boundary_collision(&mut node, inner_min, inner_max, is_procedural_anchor);

        if is_procedural_anchor {
            apply_node_collision(entity, &mut node, &node_data, &constraints);
        }
    }
}

// =============================================================================
// Private Methods
// =============================================================================

fn is_procedural_anchor_node(node: &Node) -> bool {
    node.node_type == NodeType::Anchor && node.movement_mode == AnchorMovementMode::Procedural
}

fn apply_boundary_collision(node: &mut Node, inner_min: Vec2, inner_max: Vec2, is_procedural_anchor: bool) {
    let r = node.radius;
    let mut hit_boundary = false;

    if node.position.x < inner_min.x + r {
        node.position.x = inner_min.x + r;
        hit_boundary = true;
    } else if node.position.x > inner_max.x - r {
        node.position.x = inner_max.x - r;
        hit_boundary = true;
    }

    if node.position.y < inner_min.y + r {
        node.position.y = inner_min.y + r;
        hit_boundary = true;
    } else if node.position.y > inner_max.y - r {
        node.position.y = inner_max.y - r;
        hit_boundary = true;
    }

    if !hit_boundary {
        return;
    }

    if is_procedural_anchor {
        sync_prev_position_to_current(node);
    } else {
        apply_verlet_bounce(node, inner_min, inner_max, r);
    }
}

fn sync_prev_position_to_current(node: &mut Node) {
    node.prev_position = node.position;
}

fn apply_verlet_bounce(node: &mut Node, inner_min: Vec2, inner_max: Vec2, radius: f32) {
    let velocity = node.position - node.prev_position;
    let mut new_velocity = velocity;
    let damping = node.collision_damping;

    if node.position.x == inner_min.x + radius || node.position.x == inner_max.x - radius {
        new_velocity.x = -new_velocity.x * (1.0 - damping);
    }
    if node.position.y == inner_min.y + radius || node.position.y == inner_max.y - radius {
        new_velocity.y = -new_velocity.y * (1.0 - damping);
    }

    node.prev_position = node.position - new_velocity;
}

fn apply_node_collision(
    entity: Entity,
    node: &mut Node,
    node_data: &[(Entity, Vec2, f32, f32)],
    constraints: &[(Entity, Entity)],
) {
    let connected = find_connected_entities(entity, constraints);

    for (other_entity, other_pos, other_radius, _) in node_data {
        if should_skip_collision(*other_entity, entity, &connected) {
            continue;
        }

        if let Some(push) = calculate_collision_push(node.position, node.radius, *other_pos, *other_radius) {
            node.position += push;
            sync_prev_position_to_current(node);
        }
    }
}

fn should_skip_collision(other_entity: Entity, self_entity: Entity, connected: &[Entity]) -> bool {
    other_entity == self_entity || connected.contains(&other_entity)
}

fn calculate_collision_push(pos: Vec2, radius: f32, other_pos: Vec2, other_radius: f32) -> Option<Vec2> {
    let delta = pos - other_pos;
    let distance = delta.length();
    let min_distance = radius + other_radius;

    if distance < min_distance && distance > MIN_COLLISION_DISTANCE {
        let overlap = min_distance - distance;
        let separation_dir = delta.normalize_or_zero();
        Some(separation_dir * overlap * 0.5)
    } else {
        None
    }
}
