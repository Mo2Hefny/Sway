//! Selection handling for nodes.

use bevy::prelude::*;


use crate::core::{Node as SimNode, DistanceConstraint};
use crate::ui::state::InputState;
use crate::editor::constants::*;
use crate::editor::components::{NodeVisual, Selected, Selectable};
use crate::editor::visuals::node::get_node_color;
use super::input::{cursor_world_pos, pick_node_at};
use crate::ui::state::{EditorTool, EditorToolState};

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
    if tool_state.active != EditorTool::Cursor
        || !mouse_button.just_pressed(MouseButton::Left)
        || !input_state.can_interact_with_world()
    {
        return;
    }

    let Some(world_pos) = cursor_world_pos(&windows, &cameras) else { return };
    let clicked_node = pick_node_at(world_pos, 0.0, &node_query);

    for prev_selected in selected_query.iter() {
        commands.entity(prev_selected).remove::<Selected>();
    }

    if let Some(entity) = clicked_node {
        selection.select(entity);
        commands.entity(entity).insert(Selected);
    } else {
        selection.deselect();
    }
}

pub fn update_selection_visuals(
    selection: Res<Selection>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    node_query: Query<(Entity, &SimNode, &Children)>,
    visual_query: Query<&MeshMaterial2d<ColorMaterial>, With<NodeVisual>>,
) {
    if !selection.is_changed() {
        return;
    }

    for (entity, node, children) in node_query.iter() {
        let is_selected = selection.is_selected(entity);

        let color = if is_selected { SELECTION_COLOR } else { get_node_color(node.node_type) };

        for child in children.iter() {
            if let Ok(material_handle) = visual_query.get(child) {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.color = color;
                }
            }
        }
    }
}

pub fn handle_delete_selected(
    mut commands: Commands,
    mut selection: ResMut<Selection>,
    keyboard: Res<ButtonInput<KeyCode>>,
    constraints: Query<(Entity, &DistanceConstraint)>,
) {
    if !keyboard.just_pressed(KeyCode::Delete) {
        return;
    }

    if let Some(entity) = selection.entity {
        for (c_entity, constraint) in constraints.iter() {
            if constraint.involves(entity) {
                commands.entity(c_entity).despawn();
            }
        }

        commands.entity(entity).despawn();
        selection.deselect();
    }
}
