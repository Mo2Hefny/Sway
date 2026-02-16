//! FABRIK (Forward And Backward Reaching Inverse Kinematics) solver system.

use bevy::prelude::*;

use crate::core::components::{LimbSet, Node, Playground};
use crate::core::resources::ConstraintGraph;
use crate::ui::state::PlaybackState;

pub fn fabrik_solving_system(
    playback: Res<PlaybackState>,
    playground: Res<Playground>,
    graph: Res<ConstraintGraph>,
    mut limb_sets: Query<(Entity, &mut LimbSet)>,
    mut nodes: Query<&mut Node>,
    time: Res<Time>,
) {
    if !playback.is_playing() {
        return;
    }

    let dt = time.delta_secs();
    let inner_min = playground.inner_min();
    let inner_max = playground.inner_max();
    let graph_changed = graph.is_changed();

    for (body_entity, mut limb_set) in limb_sets.iter_mut() {
        for limb_idx in 0..limb_set.limbs.len() {
            let body_pos = match nodes.get(body_entity) {
                Ok(n) => n.position,
                Err(_) => continue,
            };

            if limb_set.limbs[limb_idx].joints.is_empty() {
                continue;
            }

            solve_single_limb(
                &mut limb_set.limbs[limb_idx],
                body_entity,
                body_pos,
                &graph,
                &mut nodes,
                inner_min,
                inner_max,
                graph_changed,
                dt,
            );
        }
    }
}

// =============================================================================
// Private Methods
// =============================================================================

fn solve_single_limb(
    limb: &mut crate::core::components::Limb,
    body_entity: Entity,
    body_pos: Vec2,
    graph: &ConstraintGraph,
    nodes: &mut Query<&mut Node>,
    inner_min: Vec2,
    inner_max: Vec2,
    graph_changed: bool,
    dt: f32,
) {
    let joint_count = limb.joints.len();
    if joint_count < 1 {
        return;
    }

    if graph_changed || limb.lengths.len() != joint_count {
        recalculate_lengths(limb, body_entity, graph);
    }

    let ideal_target =
        compute_ideal_target(limb, body_entity, body_pos, graph, nodes, inner_min, inner_max);

    if limb.target == Vec2::ZERO {
        limb.target = ideal_target;
    }

    update_stepping(limb, ideal_target, dt);

    let target = clamp_target(limb.target, inner_min, inner_max);

    let chain_len = joint_count + 1;
    let mut positions = Vec::with_capacity(chain_len);
    positions.push(body_pos);
    for &entity in &limb.joints {
        if let Ok(node) = nodes.get(entity) {
            positions.push(node.position);
        } else {
            return;
        }
    }

    if limb.lengths.len() != chain_len - 1 {
        return;
    }

    let total_length: f32 = limb.lengths.iter().sum();
    let dist_to_target = (target - body_pos).length();

    if dist_to_target >= total_length {
        let dir = (target - body_pos).normalize_or_zero();
        for i in 0..limb.lengths.len() {
            positions[i + 1] = positions[i] + dir * limb.lengths[i];
        }
    } else {
        for _ in 0..limb.iterations {
            let last = positions.len() - 1;
            positions[last] = target;
            for i in (0..limb.lengths.len()).rev() {
                positions[i] =
                    constrain_distance(positions[i], positions[i + 1], limb.lengths[i]);
            }

            positions[0] = body_pos;
            for i in 0..limb.lengths.len() {
                positions[i + 1] =
                    constrain_distance(positions[i + 1], positions[i], limb.lengths[i]);
            }

            if joint_count > 1 {
                apply_joint_constraints(&mut positions, limb);
            }

            let diff = positions[last].distance(target);
            if diff <= limb.tolerance {
                break;
            }
        }
    }

    for (i, &entity) in limb.joints.iter().enumerate() {
        if let Ok(mut node) = nodes.get_mut(entity) {
            node.position = positions[i + 1];

            // Update chain_angle
            let prev_pos = positions[i];
            let curr_pos = positions[i + 1];
            
            // For the last node (tip), use direction from previous
            // For others, use direction to next (or average of prev/next if we wanted to be fancy, but simpler is better for now)
            let angle = if i < limb.joints.len() - 1 {
                 let next_pos = positions[i + 2];
                 (next_pos - curr_pos).to_angle()
            } else {
                 (curr_pos - prev_pos).to_angle()
            };
            
            node.chain_angle = angle;
        }
    }
}

fn update_stepping(
    limb: &mut crate::core::components::Limb,
    ideal_target: Vec2,
    dt: f32,
) {
    if limb.is_stepping {
        limb.step_progress += dt * limb.step_speed;

        let dest_drift = limb.step_dest.distance(ideal_target);
        if dest_drift > limb.step_threshold * 0.5 {
            limb.step_dest = ideal_target;
        }

        if limb.step_progress >= 1.0 {
            limb.is_stepping = false;
            limb.step_progress = 0.0;
            limb.target = limb.step_dest;
        } else {
            let t = limb.step_progress;
            let t = t * t * (3.0 - 2.0 * t);
            let flat_pos = limb.step_start.lerp(limb.step_dest, t);
            let height_offset = (t * std::f32::consts::PI).sin() * limb.step_height;
            limb.target = flat_pos;
            limb.target.y += height_offset;
        }
    } else {
        let dist = limb.target.distance(ideal_target);
        if dist > limb.step_threshold {
            limb.is_stepping = true;
            limb.step_start = limb.target;
            limb.step_dest = ideal_target;
            limb.step_progress = 0.0;
        }
    }
}

fn recalculate_lengths(
    limb: &mut crate::core::components::Limb,
    body_entity: Entity,
    graph: &ConstraintGraph,
) {
    limb.lengths.clear();

    let get_len = |a: Entity, b: Entity| -> f32 {
        if let Some(neighbors) = graph.adjacency.get(&a) {
            for &(n, dist) in neighbors {
                if n == b {
                    return dist;
                }
            }
        }
        if let Some(neighbors) = graph.adjacency.get(&b) {
            for &(n, dist) in neighbors {
                if n == a {
                    return dist;
                }
            }
        }
        1.0
    };

    if let Some(&first) = limb.joints.first() {
        limb.lengths.push(get_len(body_entity, first));
    } else {
        return;
    }

    for i in 0..limb.joints.len() - 1 {
        limb.lengths
            .push(get_len(limb.joints[i], limb.joints[i + 1]));
    }
}

fn compute_ideal_target(
    limb: &mut crate::core::components::Limb,
    body_entity: Entity,
    body_pos: Vec2,
    _graph: &ConstraintGraph,
    nodes: &Query<&mut Node>,
    inner_min: Vec2,
    inner_max: Vec2,
) -> Vec2 {
    if let Some(target_entity) = limb.target_node {
        if let Ok(target_node) = nodes.get(target_entity) {
            return target_node.position;
        }
    }

    let body_angle = if let Ok(body_node) = nodes.get(body_entity) {
        body_node.chain_angle
    } else {
        0.0
    };

    let target_angle = body_angle + limb.target_direction_offset;
    let target_dir = Vec2::from_angle(target_angle);

    let mut ray_end = body_pos + target_dir * limb.max_reach;
    ray_end = ray_cast_aabb(body_pos, ray_end, inner_min, inner_max);

    ray_end
}

fn constrain_distance(pos: Vec2, anchor: Vec2, distance: f32) -> Vec2 {
    let dir = (pos - anchor).normalize_or_zero();
    if dir == Vec2::ZERO {
        anchor + Vec2::X * distance
    } else {
        anchor + dir * distance
    }
}

fn clamp_target(target: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    target.clamp(min, max)
}

fn ray_cast_aabb(start: Vec2, end: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    let dir = end - start;

    if dir.x == 0.0 && dir.y == 0.0 {
        return end;
    }

    let mut t = 1.0;

    if dir.x != 0.0 {
        let tx1 = (min.x - start.x) / dir.x;
        let tx2 = (max.x - start.x) / dir.x;
        let (tmin, tmax) = if tx1 < tx2 { (tx1, tx2) } else { (tx2, tx1) };
        if tmax < 0.0 {
            return end;
        }
        if tmin > t {
            return end;
        }
        if tmin > 0.0 {
            t = t.min(tmin);
        }
    }
    if dir.y != 0.0 {
        let ty1 = (min.y - start.y) / dir.y;
        let ty2 = (max.y - start.y) / dir.y;
        let (tmin, tmax) = if ty1 < ty2 { (ty1, ty2) } else { (ty2, ty1) };
        if tmax < 0.0 {
            return end;
        }
        if tmin > t {
            return end;
        }
        if tmin > 0.0 {
            t = t.min(tmin);
        }
    }

    start + dir * t
}

fn apply_joint_constraints(positions: &mut [Vec2], limb: &crate::core::components::Limb) {
    if positions.len() < 3 {
        return;
    }

    let root = positions[0];
    let tip = *positions.last().unwrap();

    let axis = tip - root;
    let axis_len_sq = axis.length_squared();
    if axis_len_sq < 1e-6 {
        return;
    }

    for i in 1..positions.len() - 1 {
        let joint = positions[i];
        let cross =
            (tip.x - root.x) * (joint.y - root.y) - (tip.y - root.y) * (joint.x - root.x);

        let should_flip = limb.flip_bend.get(i - 1).copied().unwrap_or(false);
        let desired_side = if should_flip { 1.0 } else { -1.0 };

        if cross.signum() != desired_side {
            let t = ((joint.x - root.x) * (tip.x - root.x)
                + (joint.y - root.y) * (tip.y - root.y))
                / axis_len_sq;
            let projection = root + axis * t;
            let perp = joint - projection;
            positions[i] = projection - perp;
        }
    }
}
