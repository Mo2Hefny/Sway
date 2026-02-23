//! Shared input helpers for editor tools.

use bevy::prelude::*;

use crate::core::Node as SimNode;
use crate::editor::components::Selectable;

pub fn cursor_world_pos(windows: &Query<&Window>, cameras: &Query<(&Camera, &GlobalTransform)>) -> Option<Vec2> {
    let cursor_pos = cursor_screen_pos(windows)?;
    let (camera, cam_tf) = cameras.single().ok()?;
    camera.viewport_to_world_2d(cam_tf, cursor_pos).ok()
}

pub fn cursor_screen_pos(windows: &Query<&Window>) -> Option<Vec2> {
    windows.single().ok()?.cursor_position()
}

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

/// Returns all entities whose circle contains `world_pos`, sorted by `Entity` for stable ordering.
pub fn pick_all_nodes_at(
    world_pos: Vec2,
    padding: f32,
    node_query: &Query<(Entity, &Transform, &SimNode), With<Selectable>>,
) -> Vec<Entity> {
    let mut hits: Vec<Entity> = node_query
        .iter()
        .filter(|(_, transform, node)| {
            world_pos.distance(transform.translation.truncate()) <= node.radius + padding
        })
        .map(|(entity, _, _)| entity)
        .collect();
    hits.sort();
    hits
}
