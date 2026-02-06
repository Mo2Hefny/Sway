//! Tool action systems for editor tools.

use bevy::prelude::*;

use crate::core::{Node as SimNode, NodeType};
use crate::ui::state::InputState;
use crate::editor::constants::*;
use crate::ui::state::{EditorTool, EditorToolState};

pub fn handle_add_node_tool(
    mut commands: Commands,
    tool_state: Res<EditorToolState>,
    input_state: Res<InputState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if tool_state.active != EditorTool::AddNode {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    if !input_state.can_interact_with_world() {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    info!("Adding node at position: ({:.1}, {:.1})", world_pos.x, world_pos.y);

    commands.spawn((
        Name::new("Node"),
        SimNode::new(world_pos)
            .with_radius(DEFAULT_NODE_RADIUS)
            .with_node_type(NodeType::Normal),
    ));
}
