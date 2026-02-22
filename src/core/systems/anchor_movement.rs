use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

use crate::core::components::{AnchorMovementMode, Node, NodeType, Playground, ProceduralPathType};
use crate::core::constants::*;
use crate::core::resources::ConstraintGraph;
use crate::core::utils::{get_mouse_world_position, normalize_angle};
use crate::ui::state::PlaybackState;

pub fn anchor_movement_system(
    playback: Res<PlaybackState>,
    time: Res<Time>,
    playground: Res<Playground>,
    graph: Res<ConstraintGraph>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut anchors: Query<(Entity, &mut Node)>,
) {
    if !playback.is_playing() {
        return;
    }

    let total_time = time.elapsed_secs();
    let mouse_world = get_mouse_world_position(&window_query, &camera_query);

    let all_nodes: Vec<(Entity, Vec2, f32)> = anchors.iter().map(|(e, n)| (e, n.position, n.radius)).collect();
    
    let mut child_positions: HashMap<Entity, Vec2> = HashMap::new();
    for (entity, node) in anchors.iter() {
        if let Some(neighbors) = graph.adjacency.get(&entity) {
            if let Some(&(child_entity, _)) = neighbors.first() {
                child_positions.insert(entity, node.position);
            }
        }
    }

    for (entity, mut node) in anchors.iter_mut() {
        if node.node_type != NodeType::Anchor {
            continue;
        }

        match node.movement_mode {
            AnchorMovementMode::None => {
                node.target_position = node.position;
            }
            AnchorMovementMode::FollowTarget => {
                if let Some(target) = mouse_world {
                    node.target_position = target;
                    move_toward_target(entity, &mut node, &graph, &child_positions);
                }
            }
            AnchorMovementMode::Procedural => {
                let dt = time.delta_secs();
                update_procedural_target(entity, &mut node, total_time, dt, &playground, &graph, &all_nodes);
                move_toward_target(entity, &mut node, &graph, &child_positions);
            }
        }
    }
}

// =============================================================================
// Private Methods
// =============================================================================

fn move_toward_target(
    entity: Entity,
    node: &mut Node,
    graph: &ConstraintGraph,
    child_positions: &HashMap<Entity, Vec2>,
) {
    let direction = node.target_position - node.position;
    let distance = direction.length();

    if distance < MIN_TARGET_DISTANCE {
        return;
    }

    let desired_angle = direction.to_angle() + std::f32::consts::PI;
    
    if let Some(neighbors) = graph.adjacency.get(&entity) {
        if let Some(&(child_entity, _)) = neighbors.first() {
            if let Some(&child_pos) = child_positions.get(&child_entity) {
                let current_chain_angle = (child_pos - node.position).to_angle();
                let angle_diff = normalize_angle(desired_angle - current_chain_angle);
                let clamped_diff = angle_diff.clamp(-node.angle_constraint, node.angle_constraint);
                node.chain_angle = normalize_angle(current_chain_angle + clamped_diff);
            } else {
                node.chain_angle = desired_angle;
            }
        } else {
            node.chain_angle = desired_angle;
        }
    } else {
        node.chain_angle = desired_angle;
    }
    
    let step_size = node.movement_speed.min(distance);
    let step = direction.normalize() * step_size;

    node.position += step;
    node.prev_position += step;
}

fn update_procedural_target(
    entity: Entity,
    node: &mut Node,
    time: f32,
    dt: f32,
    playground: &Playground,
    graph: &ConstraintGraph,
    all_nodes: &[(Entity, Vec2, f32)],
) {
    let t = time + node.path_phase;

    node.target_position = match node.path_type {
        ProceduralPathType::Circle => calculate_circle_target(node, t),
        ProceduralPathType::Wave => calculate_wave_target(node, t),
        ProceduralPathType::Wander => calculate_wander_target(entity, node, t, dt, playground, graph, all_nodes),
    };
}

fn calculate_circle_target(node: &Node, t: f32) -> Vec2 {
    let x = node.path_center.x + node.path_amplitude.x * t.cos();
    let y = node.path_center.y + node.path_amplitude.y * t.sin();
    Vec2::new(x, y)
}

fn calculate_wave_target(node: &Node, t: f32) -> Vec2 {
    let x = node.path_center.x + node.path_amplitude.x * t.cos();
    let y = node.path_center.y + node.path_amplitude.y * (t * 2.0).sin();
    Vec2::new(x, y)
}

fn calculate_wander_target(
    entity: Entity,
    node: &mut Node,
    t: f32,
    dt: f32,
    playground: &Playground,
    graph: &ConstraintGraph,
    all_nodes: &[(Entity, Vec2, f32)],
) -> Vec2 {
    let bounds = calculate_safe_bounds(playground, node.radius);
    let amplitude = node.path_amplitude.x;

    apply_natural_drift(node, t);

    let wander_angle = calculate_wander_angle(node, t);
    let direction = Vec2::new(wander_angle.cos(), wander_angle.sin());

    apply_lookahead_steering(entity, node, direction, amplitude, dt, &bounds, graph, all_nodes);

    let mut new_target = calculate_new_target(node, t, amplitude);

    handle_boundary_cases(node, &mut new_target, &bounds, dt);
    handle_stuck_detection(node, &new_target, dt);

    smooth_target_position(node, new_target)
}

fn calculate_safe_bounds(playground: &Playground, radius: f32) -> SafeBounds {
    SafeBounds {
        min: playground.inner_min() + Vec2::splat(radius),
        max: playground.inner_max() - Vec2::splat(radius),
    }
}

fn apply_natural_drift(node: &mut Node, t: f32) {
    let direction_drift = (t * 0.3).sin() * 0.15 + (t * 0.17).sin() * 0.08;
    node.wander_direction += direction_drift * 0.008;
}

fn calculate_wander_angle(node: &Node, t: f32) -> f32 {
    let angle_variation = (t * 0.7).sin() * 0.15 + (t * 1.3).sin() * 0.08;
    node.wander_direction + angle_variation
}

fn apply_lookahead_steering(
    entity: Entity,
    node: &mut Node,
    direction: Vec2,
    amplitude: f32,
    dt: f32,
    bounds: &SafeBounds,
    graph: &ConstraintGraph,
    all_nodes: &[(Entity, Vec2, f32)],
) {
    let speed_lookahead = node.movement_speed * LOOKAHEAD_WINDOW;
    let base_lookahead = amplitude;

    let scan_distances = [
        base_lookahead + speed_lookahead * 1.5,
        base_lookahead + speed_lookahead * 1.0,
        base_lookahead + speed_lookahead * 0.5,
        base_lookahead,
        node.radius * 3.0,
        node.radius * 1.5,
    ];

    let wander_angle = node.wander_direction;
    let mut total_steering = 0.0_f32;

    for &scan_dist in &scan_distances {
        let scan_point = node.position + direction * scan_dist;
        let distance_factor = (scan_dist / base_lookahead).max(1.0);
        let base_strength = STEERING_STRENGTH / distance_factor;

        let boundary_steering = calculate_boundary_steering(scan_point, bounds, wander_angle, base_strength);

        if boundary_steering.abs() > STEERING_THRESHOLD {
            total_steering += boundary_steering;
        }

        let node_steering = calculate_node_steering(
            entity,
            node.position,
            scan_point,
            node.radius,
            wander_angle,
            graph,
            all_nodes,
            base_strength,
        );

        if node_steering.abs() > STEERING_THRESHOLD {
            total_steering += node_steering;
        }
    }

    if total_steering.abs() > STEERING_THRESHOLD {
        let steer_amount = total_steering.clamp(-std::f32::consts::PI, std::f32::consts::PI);
        node.wander_direction += steer_amount * STEERING_RESPONSIVENESS * dt;
        node.wander_direction = normalize_angle(node.wander_direction);
    }
}

fn calculate_new_target(node: &Node, t: f32, amplitude: f32) -> Vec2 {
    let angle_variation = (t * 0.7).sin() * 0.15 + (t * 1.3).sin() * 0.08;
    let final_angle = node.wander_direction + angle_variation;
    let offset = Vec2::new(final_angle.cos(), final_angle.sin()) * amplitude;
    node.position + offset
}

fn handle_boundary_cases(node: &mut Node, new_target: &mut Vec2, bounds: &SafeBounds, dt: f32) {
    let out_left = new_target.x < bounds.min.x;
    let out_right = new_target.x > bounds.max.x;
    let out_bottom = new_target.y < bounds.min.y;
    let out_top = new_target.y > bounds.max.y;

    if is_in_corner(out_left, out_right, out_bottom, out_top) {
        flip_direction_at_corner(node, new_target, bounds, dt);
    } else {
        handle_single_axis_boundary(node, new_target, bounds, out_left, out_right, out_bottom, out_top, dt);
    }
}

fn is_in_corner(out_left: bool, out_right: bool, out_bottom: bool, out_top: bool) -> bool {
    (out_left || out_right) && (out_bottom || out_top)
}

fn flip_direction_at_corner(node: &mut Node, new_target: &mut Vec2, bounds: &SafeBounds, dt: f32) {
    let center_dir = (bounds.min + (bounds.max - bounds.min) * 0.5 - node.position).normalize_or_zero();
    let target_angle = center_dir.y.atan2(center_dir.x);
    
    node.wander_direction = steer_smoothly(node.wander_direction, target_angle, STUCK_TURN_SPEED * 2.0 * dt);
    node.wander_direction = normalize_angle(node.wander_direction);

    let new_angle = node.wander_direction;
    let amplitude = new_target.distance(node.position);
    let new_offset = Vec2::new(new_angle.cos(), new_angle.sin()) * amplitude;
    *new_target = node.position + new_offset;

    clamp_to_bounds(new_target, bounds);
}

fn handle_single_axis_boundary(
    node: &mut Node,
    new_target: &mut Vec2,
    bounds: &SafeBounds,
    out_left: bool,
    out_right: bool,
    out_bottom: bool,
    out_top: bool,
    dt: f32,
) {
    let turn_amount = STUCK_TURN_SPEED * 3.0 * dt;

    if out_left {
        new_target.x = bounds.min.x;
        node.wander_direction = steer_smoothly(node.wander_direction, 0.0, turn_amount);
    } else if out_right {
        new_target.x = bounds.max.x;
        node.wander_direction = steer_smoothly(node.wander_direction, std::f32::consts::PI, turn_amount);
    }

    if out_bottom {
        new_target.y = bounds.min.y;
        node.wander_direction = steer_smoothly(node.wander_direction, std::f32::consts::FRAC_PI_2, turn_amount);
    } else if out_top {
        new_target.y = bounds.max.y;
        node.wander_direction = steer_smoothly(node.wander_direction, -std::f32::consts::FRAC_PI_2, turn_amount);
    }
}

fn handle_stuck_detection(node: &mut Node, new_target: &Vec2, dt: f32) {
    let distance_to_target = (node.position - *new_target).length();

    if distance_to_target < STUCK_DETECTION_THRESHOLD {
        node.wander_direction += STUCK_TURN_SPEED * dt;
        node.wander_direction = normalize_angle(node.wander_direction);
    }
}

fn smooth_target_position(node: &Node, new_target: Vec2) -> Vec2 {
    let prev_target = node.target_position;
    prev_target.lerp(new_target, TARGET_SMOOTHING)
}

fn calculate_boundary_steering(point: Vec2, bounds: &SafeBounds, current_angle: f32, strength: f32) -> f32 {
    let mut steering = 0.0_f32;
    let avoidance_strength = strength * 2.0;

    if point.x < bounds.min.x + BOUNDARY_AVOIDANCE_RANGE {
        let dist = (point.x - bounds.min.x).max(0.1);
        let weight = (1.0 - (dist / BOUNDARY_AVOIDANCE_RANGE).min(1.0)).powi(2);
        steering += angle_diff(0.0, current_angle) * avoidance_strength * weight;
    } else if point.x > bounds.max.x - BOUNDARY_AVOIDANCE_RANGE {
        let dist = (bounds.max.x - point.x).max(0.1);
        let weight = (1.0 - (dist / BOUNDARY_AVOIDANCE_RANGE).min(1.0)).powi(2);
        steering += angle_diff(std::f32::consts::PI, current_angle) * avoidance_strength * weight;
    }

    if point.y < bounds.min.y + BOUNDARY_AVOIDANCE_RANGE {
        let dist = (point.y - bounds.min.y).max(0.1);
        let weight = (1.0 - (dist / BOUNDARY_AVOIDANCE_RANGE).min(1.0)).powi(2);
        steering += angle_diff(std::f32::consts::FRAC_PI_2, current_angle) * avoidance_strength * weight;
    } else if point.y > bounds.max.y - BOUNDARY_AVOIDANCE_RANGE {
        let dist = (bounds.max.y - point.y).max(0.1);
        let weight = (1.0 - (dist / BOUNDARY_AVOIDANCE_RANGE).min(1.0)).powi(2);
        steering += angle_diff(-std::f32::consts::FRAC_PI_2, current_angle) * avoidance_strength * weight;
    }

    steering
}

fn calculate_node_steering(
    self_entity: Entity,
    self_pos: Vec2,
    scan_point: Vec2,
    self_radius: f32,
    current_angle: f32,
    graph: &ConstraintGraph,
    all_nodes: &[(Entity, Vec2, f32)],
    strength: f32,
) -> f32 {
    let mut steering = 0.0_f32;
    let group = graph.get_group(self_entity);

    for (other_entity, other_pos, other_radius) in all_nodes {
        if should_skip_node(*other_entity, self_entity, group, graph) {
            continue;
        }

        if let Some(away_steering) = calculate_avoidance_steering(
            self_pos,
            scan_point,
            *other_pos,
            self_radius,
            *other_radius,
            current_angle,
            strength,
        ) {
            steering += away_steering;
        }
    }

    steering
}

fn should_skip_node(other_entity: Entity, self_entity: Entity, my_group: Option<u32>, graph: &ConstraintGraph) -> bool {
    if other_entity == self_entity {
        return true;
    }

    match (my_group, graph.get_group(other_entity)) {
        (Some(g1), Some(g2)) => g1 == g2,
        _ => false,
    }
}

fn calculate_avoidance_steering(
    self_pos: Vec2,
    scan_point: Vec2,
    other_pos: Vec2,
    self_radius: f32,
    other_radius: f32,
    current_angle: f32,
    strength: f32,
) -> Option<f32> {
    let to_scan = scan_point - other_pos;
    let distance = to_scan.length();
    let min_safe = other_radius + self_radius + NODE_AVOIDANCE_BUFFER;

    if distance < min_safe && distance > MIN_COLLISION_DISTANCE {
        let urgency = 1.0 - (distance / min_safe);
        let away_dir = self_pos - other_pos;
        let away_angle = away_dir.y.atan2(away_dir.x);
        Some(angle_diff(away_angle, current_angle) * strength * urgency)
    } else {
        None
    }
}

fn angle_diff(target: f32, current: f32) -> f32 {
    normalize_angle(target - current)
}

fn clamp_to_bounds(target: &mut Vec2, bounds: &SafeBounds) {
    target.x = target.x.clamp(bounds.min.x, bounds.max.x);
    target.y = target.y.clamp(bounds.min.y, bounds.max.y);
}

fn steer_smoothly(current: f32, target: f32, fraction: f32) -> f32 {
    current + angle_diff(target, current) * fraction
}

struct SafeBounds {
    min: Vec2,
    max: Vec2,
}
