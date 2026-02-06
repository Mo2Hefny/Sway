//! Selection handling for nodes.

use bevy::prelude::*;

use crate::core::Node as SimNode;
use crate::ui::state::InputState;
use crate::editor::constants::*;
use crate::ui::state::{EditorTool, EditorToolState};

/// Resource tracking the currently selected entity.
#[derive(Resource, Clone, Debug, Default, Reflect)]
pub struct Selection {
    pub entity: Option<Entity>,
}

impl Selection {
    pub fn select(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    pub fn deselect(&mut self) {
        self.entity = None;
    }

    pub fn is_selected(&self, entity: Entity) -> bool {
        self.entity == Some(entity)
    }
}

/// Marker for entities that are currently selected.
#[derive(Component, Clone, Debug, Reflect)]
pub struct Selected;

/// Marker to make nodes selectable via picking.
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct Selectable;

pub fn handle_node_selection(
    mut commands: Commands,
    mut selection: ResMut<Selection>,
    tool_state: Res<EditorToolState>,
    input_state: Res<InputState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<(Entity, &Transform, &SimNode), With<Selectable>>,
    selected_query: Query<Entity, With<Selected>>,
) {
    if tool_state.active != EditorTool::Cursor {
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

    let mut clicked_node = None;
    for (entity, transform, node) in node_query.iter() {
        let node_pos = transform.translation.truncate();
        let distance = world_pos.distance(node_pos);
        if distance <= node.radius {
            clicked_node = Some(entity);
            info!("Selected node at ({:.1}, {:.1})", node_pos.x, node_pos.y);
            break;
        }
    }

    for prev_selected in selected_query.iter() {
        commands.entity(prev_selected).remove::<Selected>();
    }

    if let Some(entity) = clicked_node {
        selection.select(entity);
        commands.entity(entity).insert(Selected);
    } else {
        info!("Selection cleared");
    }
}

pub fn update_selection_visuals(
    selection: Res<Selection>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    node_query: Query<(Entity, &SimNode, &Children)>,
    visual_query: Query<&MeshMaterial2d<ColorMaterial>>,
) {
    if !selection.is_changed() {
        return;
    }

    for (entity, node, children) in node_query.iter() {
        let is_selected = selection.is_selected(entity);

        for child in children.iter() {
            if let Ok(material_handle) = visual_query.get(child) {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.color = if is_selected {
                        SELECTION_COLOR
                    } else {
                        match node.node_type {
                            crate::core::NodeType::Anchor => ANCHOR_NODE_COLOR,
                            crate::core::NodeType::Leg => LEG_NODE_COLOR,
                            crate::core::NodeType::Normal => NORMAL_NODE_COLOR,
                        }
                    };
                }
            }
        }
    }
}
