//! Tool action systems for editor tools.

use bevy::prelude::*;

use crate::core::{Node as SimNode, NodeType};
use crate::ui::state::InputState;
use crate::editor::constants::*;
use super::input::cursor_world_pos;
use crate::ui::state::{EditorTool, EditorToolState};

/// Handles left-clicks to add new nodes to the simulation.
pub fn handle_add_node_tool(
    mut commands: Commands,
    tool_state: Res<EditorToolState>,
    input_state: Res<InputState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if tool_state.active != EditorTool::AddNode
        || !mouse_button.just_pressed(MouseButton::Left)
        || !input_state.can_interact_with_world()
    {
        return;
    }

    let Some(world_pos) = cursor_world_pos(&windows, &cameras) else { return };

    commands.spawn((
        Name::new("Node"),
        SimNode::new(world_pos)
            .with_radius(DEFAULT_NODE_RADIUS)
            .with_node_type(NodeType::Normal),
    ));
}
