//! Common utility functions used across core systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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

pub fn relative_angle_diff(angle: f32, anchor: f32) -> f32 {
    let shifted = normalize_angle_to_positive(angle + std::f32::consts::PI - anchor);
    std::f32::consts::PI - shifted
}

pub fn constrain_angle(angle: f32, anchor: f32, angle_min: f32, angle_max: f32) -> f32 {
    let diff = -relative_angle_diff(angle, anchor);
    let real_min = angle_min.min(angle_max);
    let real_max = angle_max.max(angle_min);
    let clamped_diff = diff.clamp(real_min, real_max);
    normalize_angle_to_positive(anchor + clamped_diff)
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
