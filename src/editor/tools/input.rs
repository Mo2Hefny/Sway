//! Shared input helpers for editor tools.

use bevy::prelude::*;

use crate::core::Node as SimNode;
use crate::editor::components::Selectable;

/// Converts the current cursor screen position to a world-space coordinate.
pub fn cursor_world_pos(windows: &Query<&Window>, cameras: &Query<(&Camera, &GlobalTransform)>) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    let (camera, cam_tf) = cameras.single().ok()?;
    camera.viewport_to_world_2d(cam_tf, cursor_pos).ok()
}

/// Finds the closest selectable node whose circle contains `world_pos`.
pub fn pick_node_at(
    world_pos: Vec2,
    padding: f32,
    node_query: &Query<(Entity, &Transform, &SimNode), With<Selectable>>,
) -> Option<Entity> {
    for (entity, transform, node) in node_query.iter() {
        let dist = world_pos.distance(transform.translation.truncate());
        if dist <= node.radius + padding {
            return Some(entity);
        }
    }
    None
}
