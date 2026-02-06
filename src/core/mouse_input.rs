//! Mouse-driven input systems for simulation nodes.

use bevy::prelude::*;

use super::constants::FOLLOW_SPEED;
use super::{Node, NodeType};
use crate::ui::state::PlaybackState;

/// Sets acceleration on eligible anchor nodes toward the mouse cursor.
pub fn follow_mouse_system(
    playback: Res<PlaybackState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut nodes: Query<&mut Node>,
) {
    if !playback.is_playing() {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    for mut node in nodes.iter_mut() {
        if node.node_type != NodeType::Anchor || !node.follow_mouse {
            continue;
        }

        let direction = world_pos - node.position;
        node.acceleration = direction.normalize_or_zero() * FOLLOW_SPEED;
    }
}
