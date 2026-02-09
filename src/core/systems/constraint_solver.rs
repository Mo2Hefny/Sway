//! Iterative distance-constraint solver.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::core::constants::*;
use crate::core::components::{DistanceConstraint, Node, NodeType};
use crate::core::utils::{normalize_angle_to_positive};
use crate::ui::state::PlaybackState;

pub fn constraint_solving_system(
    playback: Res<PlaybackState>,
    constraint_query: Query<&DistanceConstraint>,
    mut nodes: Query<&mut Node>,
) {
    if !playback.is_playing() {
        return;
    }

    let constraints: Vec<&DistanceConstraint> = constraint_query.iter().collect();
    if constraints.is_empty() {
        return;
    }

    let (chains, standalone) = build_chains(&constraints, &nodes);

    for chain in &chains {
        resolve_chain(chain, &mut nodes);
    }

    for _ in 0..CONSTRAINT_ITERATIONS {
        for constraint in &standalone {
            solve_distance(constraint, &mut nodes);
        }
    }
}

// =============================================================================
// Private Methods
// =============================================================================

struct ChainLink {
    entity: Entity,
    rest_length: f32,
}

fn build_chains(
    constraints: &[&DistanceConstraint],
    nodes: &Query<&mut Node>,
) -> (Vec<Vec<ChainLink>>, Vec<DistanceConstraint>) {
    let adj = build_adjacency_list(constraints);
    let mut visited: HashMap<Entity, bool> = HashMap::new();
    let mut chains: Vec<Vec<ChainLink>> = Vec::new();

    let starts = find_chain_starts(&adj, nodes);

    for &start in &starts {
        if is_visited(&visited, start) {
            continue;
        }

        let neighbors = adj.get(&start).unwrap();
        for &(next, rest_len) in neighbors {
            if is_visited(&visited, next) {
                continue;
            }

            if let Some(chain) = build_chain_from_endpoint(start, next, rest_len, &adj, &mut visited) {
                if chain.len() >= 2 {
                    chains.push(chain);
                }
            }
        }
    }

    collect_cycles(&adj, &mut visited, &mut chains);

    let standalone = find_standalone_constraints(constraints, &visited);

    (chains, standalone)
}

fn build_adjacency_list(
    constraints: &[&DistanceConstraint],
) -> HashMap<Entity, Vec<(Entity, f32)>> {
    let mut adj: HashMap<Entity, Vec<(Entity, f32)>> = HashMap::new();
    for c in constraints {
        adj.entry(c.node_a).or_default().push((c.node_b, c.rest_length));
        adj.entry(c.node_b).or_default().push((c.node_a, c.rest_length));
    }
    adj
}

fn find_chain_starts(
    adj: &HashMap<Entity, Vec<(Entity, f32)>>,
    nodes: &Query<&mut Node>,
) -> Vec<Entity> {
    let mut starts: Vec<Entity> = Vec::new();
    let mut non_anchor_leaves: Vec<Entity> = Vec::new();

    for (&entity, neighbors) in adj {
        if is_anchor_node(entity, nodes) {
            starts.push(entity);
        } else if is_leaf_node(neighbors) {
            non_anchor_leaves.push(entity);
        }
    }

    starts.extend(non_anchor_leaves);
    starts
}

fn is_anchor_node(entity: Entity, nodes: &Query<&mut Node>) -> bool {
    nodes
        .get(entity)
        .map(|n| n.node_type == NodeType::Anchor)
        .unwrap_or(false)
}

fn is_leaf_node(neighbors: &[(Entity, f32)]) -> bool {
    neighbors.len() == 1
}

fn is_visited(visited: &HashMap<Entity, bool>, entity: Entity) -> bool {
    *visited.get(&entity).unwrap_or(&false)
}

fn build_chain_from_endpoint(
    start: Entity,
    next: Entity,
    rest_len: f32,
    adj: &HashMap<Entity, Vec<(Entity, f32)>>,
    visited: &mut HashMap<Entity, bool>,
) -> Option<Vec<ChainLink>> {
    let mut chain: Vec<ChainLink> = vec![ChainLink {
        entity: start,
        rest_length: rest_len,
    }];
    visited.insert(start, true);

    let mut current = next;
    let mut prev = start;

    loop {
        let cur_neighbors = adj.get(&current)?;

        if is_middle_of_chain(cur_neighbors) {
            let (next_node, next_rest) = find_next_neighbor(cur_neighbors, prev);
            chain.push(ChainLink {
                entity: current,
                rest_length: next_rest,
            });
            visited.insert(current, true);
            prev = current;
            current = next_node;
        } else {
            chain.push(ChainLink {
                entity: current,
                rest_length: 0.0,
            });
            visited.insert(current, true);
            break;
        }
    }

    Some(chain)
}

fn is_middle_of_chain(neighbors: &[(Entity, f32)]) -> bool {
    neighbors.len() == 2
}

fn find_next_neighbor(neighbors: &[(Entity, f32)], prev: Entity) -> (Entity, f32) {
    if neighbors[0].0 == prev {
        neighbors[1]
    } else {
        neighbors[0]
    }
}

fn collect_cycles(
    adj: &HashMap<Entity, Vec<(Entity, f32)>>,
    visited: &mut HashMap<Entity, bool>,
    chains: &mut Vec<Vec<ChainLink>>,
) {
    for (&start, _) in adj {
        if is_visited(visited, start) {
            continue;
        }

        if let Some(chain) = build_cycle_chain(start, adj, visited) {
            if chain.len() >= 2 {
                chains.push(chain);
            }
        }
    }
}

fn build_cycle_chain(
    start: Entity,
    adj: &HashMap<Entity, Vec<(Entity, f32)>>,
    visited: &mut HashMap<Entity, bool>,
) -> Option<Vec<ChainLink>> {
    let mut chain: Vec<ChainLink> = Vec::new();
    let mut current = start;
    let mut prev = Entity::PLACEHOLDER;

    loop {
        let neighbors = adj.get(&current)?;
        let (next_node, rest_len) = select_cycle_neighbor(neighbors, prev);

        chain.push(ChainLink {
            entity: current,
            rest_length: rest_len,
        });
        visited.insert(current, true);
        prev = current;
        current = next_node;

        if current == start {
            chain.push(ChainLink {
                entity: current,
                rest_length: 0.0,
            });
            break;
        }
    }

    Some(chain)
}

fn select_cycle_neighbor(neighbors: &[(Entity, f32)], prev: Entity) -> (Entity, f32) {
    if prev == Entity::PLACEHOLDER || neighbors[0].0 == prev {
        if prev == Entity::PLACEHOLDER {
            neighbors[0]
        } else {
            neighbors[1]
        }
    } else {
        neighbors[0]
    }
}

fn find_standalone_constraints(
    constraints: &[&DistanceConstraint],
    visited: &HashMap<Entity, bool>,
) -> Vec<DistanceConstraint> {
    constraints
        .iter()
        .filter(|c| !is_visited(visited, c.node_a) || !is_visited(visited, c.node_b))
        .map(|c| (*c).clone())
        .collect()
}

fn resolve_chain(chain: &[ChainLink], nodes: &mut Query<&mut Node>) {
    if chain.len() < 2 {
        return;
    }

    let (mut prev_angle, mut prev_pos) = match initialize_chain_resolution(chain, nodes) {
        Some(result) => result,
        None => return,
    };

    set_root_chain_angle(chain[0].entity, prev_angle, nodes);

    for i in 1..chain.len() {
        let result = resolve_chain_link(
            &chain[i],
            &chain[i - 1],
            prev_pos,
            prev_angle,
            nodes,
        );

        if let Some((new_angle, new_pos)) = result {
            prev_angle = new_angle;
            prev_pos = new_pos;
        }
    }
}

fn initialize_chain_resolution(
    chain: &[ChainLink],
    nodes: &Query<&mut Node>,
) -> Option<(f32, Vec2)> {
    let first_pos = nodes.get(chain[0].entity).ok()?.position;
    let second_pos = nodes.get(chain[1].entity).ok()?.position;
    let initial_angle = (second_pos - first_pos).to_angle();
    Some((initial_angle, first_pos))
}

fn set_root_chain_angle(entity: Entity, angle: f32, nodes: &mut Query<&mut Node>) {
    if let Ok(mut root) = nodes.get_mut(entity) {
        root.chain_angle = angle;
    }
}

fn resolve_chain_link(
    link: &ChainLink,
    prev_link: &ChainLink,
    prev_pos: Vec2,
    prev_angle: f32,
    nodes: &mut Query<&mut Node>,
) -> Option<(f32, Vec2)> {
    let node = nodes.get(link.entity).ok()?;
    let cur_pos = node.position;
    let is_anchor = node.node_type == NodeType::Anchor;
    let angle_constraint = node.angle_constraint;
    drop(node);

    if is_anchor {
        handle_anchor_in_chain(link.entity, cur_pos, prev_pos, nodes)
    } else {
        handle_movable_in_chain(
            link.entity,
            cur_pos,
            prev_pos,
            prev_angle,
            angle_constraint,
            prev_link.rest_length,
            nodes,
        )
    }
}

fn handle_anchor_in_chain(
    entity: Entity,
    cur_pos: Vec2,
    prev_pos: Vec2,
    nodes: &mut Query<&mut Node>,
) -> Option<(f32, Vec2)> {
    let new_angle = (cur_pos - prev_pos).to_angle();
    if let Ok(mut node) = nodes.get_mut(entity) {
        node.chain_angle = new_angle;
    }
    Some((new_angle, cur_pos))
}

fn handle_movable_in_chain(
    entity: Entity,
    cur_pos: Vec2,
    prev_pos: Vec2,
    prev_angle: f32,
    angle_constraint: f32,
    rest_length: f32,
    nodes: &mut Query<&mut Node>,
) -> Option<(f32, Vec2)> {
    let cur_angle = (cur_pos - prev_pos).to_angle();
    let constrained_angle = constrain_angle(cur_angle, prev_angle, angle_constraint);
    let target = prev_pos + Vec2::from_angle(constrained_angle) * rest_length;
    let shift = target - cur_pos;

    if let Ok(mut node) = nodes.get_mut(entity) {
        node.position = target;
        node.prev_position += shift;
        node.chain_angle = constrained_angle;
    }

    Some((constrained_angle, target))
}

fn constrain_angle(angle: f32, anchor: f32, constraint: f32) -> f32 {
    let diff = relative_angle_diff(angle, anchor);
    if diff.abs() <= constraint {
        normalize_angle_to_positive(angle)
    } else if diff > constraint {
        normalize_angle_to_positive(anchor - constraint)
    } else {
        normalize_angle_to_positive(anchor + constraint)
    }
}

fn relative_angle_diff(angle: f32, anchor: f32) -> f32 {
    let shifted = normalize_angle_to_positive(angle + std::f32::consts::PI - anchor);
    std::f32::consts::PI - shifted
}



fn solve_distance(constraint: &DistanceConstraint, nodes: &mut Query<&mut Node>) {
    let (pos_a, pos_b, type_a, type_b) = match extract_constraint_positions(constraint, nodes) {
        Some(data) => data,
        None => return,
    };

    let delta = pos_b - pos_a;
    let dist = delta.length();
    if dist < 1e-6 {
        return;
    }

    let correction = calculate_position_correction(delta, dist, constraint.rest_length);
    let (w_a, w_b) = calculate_constraint_weights(type_a, type_b);

    if w_a == 0.0 && w_b == 0.0 {
        return;
    }

    apply_constraint_correction(constraint.node_a, correction * w_a, nodes);
    apply_constraint_correction(constraint.node_b, -correction * w_b, nodes);
}

fn extract_constraint_positions(
    constraint: &DistanceConstraint,
    nodes: &Query<&mut Node>,
) -> Option<(Vec2, Vec2, NodeType, NodeType)> {
    let a = nodes.get(constraint.node_a).ok()?;
    let b = nodes.get(constraint.node_b).ok()?;
    Some((a.position, b.position, a.node_type, b.node_type))
}

fn calculate_position_correction(delta: Vec2, dist: f32, rest_length: f32) -> Vec2 {
    delta / dist * (dist - rest_length)
}

fn calculate_constraint_weights(type_a: NodeType, type_b: NodeType) -> (f32, f32) {
    match (type_a == NodeType::Anchor, type_b == NodeType::Anchor) {
        (true, true) => (0.0, 0.0),
        (true, false) => (0.0, 1.0),
        (false, true) => (1.0, 0.0),
        (false, false) => (0.5, 0.5),
    }
}

fn apply_constraint_correction(entity: Entity, correction: Vec2, nodes: &mut Query<&mut Node>) {
    if let Ok(mut node) = nodes.get_mut(entity) {
        node.position += correction;
        node.prev_position += correction;
    }
}
