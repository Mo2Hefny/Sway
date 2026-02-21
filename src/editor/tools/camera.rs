//! Camera control systems for zoom and pan.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::core::Playground;
use crate::editor::constants::{CAMERA_LERP_FACTOR, ZOOM_MAX, ZOOM_MIN, ZOOM_SPEED};
use crate::ui::state::{EditorTool, EditorToolState, InputState};

use super::input::cursor_screen_pos;

#[derive(Resource, Clone, Debug, Reflect)]
pub struct CameraState {
    pub zoom: f32,
    pub last_drag_pos: Option<Vec2>,
    pub follow_selection: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            last_drag_pos: None,
            follow_selection: false,
        }
    }
}

pub fn handle_camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
    input_state: Res<InputState>,
) {
    if !input_state.can_interact_with_world() {
        return;
    }

    let scroll_delta = calculate_scroll_delta(&mut scroll_events);
    if scroll_delta.abs() < 0.001 {
        return;
    }

    update_zoom(&mut camera_state, scroll_delta);
    apply_zoom_to_camera(&mut camera_query, camera_state.zoom);
}

pub fn handle_camera_pan(
    mut camera_state: ResMut<CameraState>,
    tool_state: Res<EditorToolState>,
    input_state: Res<InputState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    playground: Res<Playground>,
    mut camera_query: Query<(&mut Transform, &Projection), With<Camera2d>>,
) {
    if tool_state.active != EditorTool::Move {
        camera_state.last_drag_pos = None;
        return;
    }

    let Some(cursor_pos) = cursor_screen_pos(&windows) else {
        camera_state.last_drag_pos = None;
        return;
    };

    if handle_drag_state(&mut camera_state, &mouse_button, &input_state, cursor_pos) {
        return;
    }

    let Some(delta) = calculate_drag_delta(&mut camera_state, cursor_pos) else {
        return;
    };

    if delta.length_squared() < 0.01 {
        return;
    }

    let Ok((mut camera_transform, projection)) = camera_query.single_mut() else {
        return;
    };

    camera_state.follow_selection = false;

    let scale = get_camera_scale(projection);
    apply_pan_movement(&mut camera_transform, delta, scale, &playground);
}

pub fn handle_follow_toggle(
    mut camera_state: ResMut<CameraState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        camera_state.follow_selection = !camera_state.follow_selection;
    }
}

pub fn handle_camera_follow(
    camera_state: Res<CameraState>,
    selection: Res<super::selection::Selection>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    node_query: Query<&Transform, (With<crate::core::Node>, Without<Camera2d>)>,
) {
    if !camera_state.follow_selection {
        return;
    }

    let Some(selected_entity) = selection.entity else {
        return;
    };

    let Ok(node_transform) = node_query.get(selected_entity) else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let target = node_transform.translation.truncate();
    let current = camera_transform.translation.truncate();
    let next = current.lerp(target, CAMERA_LERP_FACTOR);

    camera_transform.translation.x = next.x;
    camera_transform.translation.y = next.y;
}

// =============================================================================
// Private Methods
// =============================================================================

fn calculate_scroll_delta(scroll_events: &mut MessageReader<MouseWheel>) -> f32 {
    scroll_events.read().fold(0.0, |acc, event| {
        acc + match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 100.0,
        }
    })
}

fn update_zoom(camera_state: &mut CameraState, scroll_delta: f32) {
    let zoom_change = -scroll_delta * ZOOM_SPEED;
    camera_state.zoom = (camera_state.zoom + zoom_change).clamp(ZOOM_MIN, ZOOM_MAX);
}

fn apply_zoom_to_camera(camera_query: &mut Query<&mut Projection, With<Camera2d>>, zoom: f32) {
    if let Ok(mut projection) = camera_query.single_mut() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = zoom;
        }
    }
}

fn handle_drag_state(
    camera_state: &mut CameraState,
    mouse_button: &ButtonInput<MouseButton>,
    input_state: &InputState,
    cursor_pos: Vec2,
) -> bool {
    if mouse_button.just_pressed(MouseButton::Left) && input_state.can_interact_with_world() {
        camera_state.last_drag_pos = Some(cursor_pos);
        return true;
    }

    if mouse_button.just_released(MouseButton::Left) || !mouse_button.pressed(MouseButton::Left) {
        camera_state.last_drag_pos = None;
        return true;
    }

    false
}

fn calculate_drag_delta(camera_state: &mut CameraState, cursor_pos: Vec2) -> Option<Vec2> {
    let last_pos = camera_state.last_drag_pos?;
    let delta = cursor_pos - last_pos;
    camera_state.last_drag_pos = Some(cursor_pos);
    Some(delta)
}

fn get_camera_scale(projection: &Projection) -> f32 {
    match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    }
}

fn apply_pan_movement(camera_transform: &mut Transform, delta: Vec2, scale: f32, playground: &Playground) {
    let world_delta = Vec3::new(-delta.x * scale, delta.y * scale, 0.0);
    camera_transform.translation += world_delta;

    let bounds = playground.half_size;
    camera_transform.translation.x = camera_transform.translation.x.clamp(-bounds.x, bounds.x);
    camera_transform.translation.y = camera_transform.translation.y.clamp(-bounds.y, bounds.y);
}
