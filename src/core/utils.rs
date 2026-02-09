//! Common utility functions used across core systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn find_connected_entities(entity: Entity, constraints: &[(Entity, Entity)]) -> Vec<Entity> {
    let mut connected = vec![entity];
    let mut to_check = vec![entity];

    while let Some(current) = to_check.pop() {
        for (a, b) in constraints {
            if let Some(neighbor) = get_constraint_neighbor(current, *a, *b) {
                if !connected.contains(&neighbor) {
                    connected.push(neighbor);
                    to_check.push(neighbor);
                }
            }
        }
    }

    connected
}

pub fn get_constraint_neighbor(current: Entity, a: Entity, b: Entity) -> Option<Entity> {
    if a == current {
        Some(b)
    } else if b == current {
        Some(a)
    } else {
        None
    }
}

pub fn normalize_angle(angle: f32) -> f32 {
    let two_pi = std::f32::consts::TAU;
    let normalized = angle % two_pi;

    if normalized > std::f32::consts::PI {
        normalized - two_pi
    } else if normalized < -std::f32::consts::PI {
        normalized + two_pi
    } else {
        normalized
    }
}

pub fn normalize_angle_to_positive(angle: f32) -> f32 {
    let normalized = normalize_angle(angle);
    if normalized < 0.0 {
        normalized + std::f32::consts::TAU
    } else {
        normalized
    }
}

pub fn get_mouse_world_position(
    window_query: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = window_query.single().ok()?;
    let cursor_position = window.cursor_position()?;
    let (camera, camera_transform) = camera_query.single().ok()?;
    camera.viewport_to_world_2d(camera_transform, cursor_position).ok()
}
