use bevy::prelude::*;

use crate::core::components::{CellEntry, Node, NodeType, Playground};
use crate::core::constants::{CELL_SIZE, MIN_COLLISION_DISTANCE};
use crate::core::resources::ConstraintGraph;
use crate::ui::state::PlaybackState;

use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
struct Collider {
    entity: Entity,
    position: Vec2,
    radius: f32,
    group: Option<u32>,
}

pub fn collision_avoidance_system(
    playback: Res<PlaybackState>,
    playground: Res<Playground>,
    graph: Res<ConstraintGraph>,
    mut nodes: Query<(Entity, &mut Node)>,
) {
    if !playback.is_playing() {
        return;
    }

    let mut colliders = collect_colliders(&mut nodes, &playground, &graph);
    let mut grid_entries = generate_grid_entries(&colliders);

    grid_entries.sort_unstable();

    let potential_pairs = find_potential_pairs(&grid_entries);

    let pairs_vec: Vec<(usize, usize)> = potential_pairs.into_iter().collect();

    resolve_collisions(&mut colliders, &pairs_vec);
    apply_updates(&mut nodes, &colliders);
}

// =============================================================================
// Private Methods
// =============================================================================

fn collect_colliders(
    nodes: &mut Query<(Entity, &mut Node)>,
    playground: &Playground,
    graph: &ConstraintGraph,
) -> Vec<Collider> {
    let inner_min = playground.inner_min();
    let inner_max = playground.inner_max();
    let mut colliders = Vec::with_capacity(nodes.iter().len());

    for (entity, mut node) in nodes.iter_mut() {
        if node.node_type == NodeType::Limb {
            continue;
        }

        apply_boundary_collision(&mut node, inner_min, inner_max);

        colliders.push(Collider {
            entity,
            position: node.position,
            radius: node.radius,
            group: graph.get_group(entity),
        });
    }
    colliders
}

fn generate_grid_entries(colliders: &[Collider]) -> Vec<CellEntry> {
    let mut grid_entries = Vec::with_capacity(colliders.len() * 4);

    for (index, collider) in colliders.iter().enumerate() {
        let min_x = ((collider.position.x - collider.radius) / CELL_SIZE).floor() as i32;
        let max_x = ((collider.position.x + collider.radius) / CELL_SIZE).floor() as i32;
        let min_y = ((collider.position.y - collider.radius) / CELL_SIZE).floor() as i32;
        let max_y = ((collider.position.y + collider.radius) / CELL_SIZE).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                grid_entries.push(CellEntry {
                    cell_x: x,
                    cell_y: y,
                    collider_index: index,
                });
            }
        }
    }
    grid_entries
}

fn find_potential_pairs(grid_entries: &[CellEntry]) -> HashSet<(usize, usize)> {
    let mut potential_pairs = HashSet::with_capacity(grid_entries.len());
    let mut start_index = 0;

    while start_index < grid_entries.len() {
        let end_index = grid_entries[start_index..]
            .iter()
            .take_while(|e| {
                e.cell_x == grid_entries[start_index].cell_x && e.cell_y == grid_entries[start_index].cell_y
            })
            .count()
            + start_index;

        for i in start_index..end_index {
            for j in (i + 1)..end_index {
                let idx_a = grid_entries[i].collider_index;
                let idx_b = grid_entries[j].collider_index;

                let (first, second) = if idx_a < idx_b { (idx_a, idx_b) } else { (idx_b, idx_a) };

                potential_pairs.insert((first, second));
            }
        }

        start_index = end_index;
    }

    potential_pairs
}

fn resolve_collisions(colliders: &mut [Collider], potential_pairs: &[(usize, usize)]) {
    let iterations = 4;

    for _ in 0..iterations {
        let mut any_correction = false;

        for &(idx_a, idx_b) in potential_pairs {
            let (part1, part2) = colliders.split_at_mut(idx_b);
            let col_a = &mut part1[idx_a];
            let col_b = &mut part2[0];

            if let (Some(g1), Some(g2)) = (col_a.group, col_b.group) {
                if g1 == g2 {
                    continue;
                }
            }

            if let Some(push) = calculate_collision_push(col_a.position, col_a.radius, col_b.position, col_b.radius) {
                col_a.position += push;
                col_b.position -= push;
                any_correction = true;
            }
        }

        if !any_correction {
            break;
        }
    }
}

fn apply_updates(nodes: &mut Query<(Entity, &mut Node)>, colliders: &[Collider]) {
    for collider in colliders {
        if let Ok((_, mut node)) = nodes.get_mut(collider.entity) {
            node.position = collider.position;
        }
    }
}

fn apply_boundary_collision(node: &mut Node, inner_min: Vec2, inner_max: Vec2) {
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

    if hit_boundary {
        apply_verlet_bounce(node, inner_min, inner_max, r);
    }
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

fn calculate_collision_push(pos: Vec2, radius: f32, other_pos: Vec2, other_radius: f32) -> Option<Vec2> {
    let delta = pos - other_pos;
    let distance_sq = delta.length_squared();
    let min_distance = radius + other_radius;
    let min_distance_sq = min_distance * min_distance;

    if distance_sq < min_distance_sq && distance_sq > MIN_COLLISION_DISTANCE * MIN_COLLISION_DISTANCE {
        let distance = distance_sq.sqrt();
        let overlap = min_distance - distance;
        let separation_dir = delta / distance;
        Some(separation_dir * overlap * 0.5)
    } else {
        None
    }
}
