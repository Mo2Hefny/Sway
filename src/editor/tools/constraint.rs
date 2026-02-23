//! Constraint creation tool and preview rendering.

use bevy::prelude::*;

use super::input::{cursor_world_pos, pick_node_at};
use crate::core::{DistanceConstraint, Node as SimNode, NodeType};
use crate::editor::components::{ConstraintPreview, Selectable};
use crate::editor::constants::*;
use crate::editor::mesh::primitives::create_dashed_line_mesh;
use crate::ui::state::{EditorTool, EditorToolState, InputState};

/// Tracks the in-progress edge creation state.
#[derive(Resource, Default, Debug)]
pub struct EdgeCreationState {
    pub first_node: Option<Entity>,
}

// Handles left-clicks to create distance constraints between nodes.
pub fn handle_add_edge_tool(
    mut commands: Commands,
    tool_state: Res<EditorToolState>,
    input_state: Res<InputState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut edge_state: ResMut<EdgeCreationState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<(Entity, &Transform, &SimNode), With<Selectable>>,
    existing: Query<&DistanceConstraint>,
) {
    if tool_state.active != EditorTool::AddEdge {
        if edge_state.first_node.is_some() {
            edge_state.first_node = None;
        }
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Left) || !input_state.can_interact_with_world() {
        return;
    }

    let Some(world_pos) = cursor_world_pos(&windows, &cameras) else {
        return;
    };
    let Some(clicked_entity) = pick_node_at(world_pos, 4.0, &node_query) else {
        return;
    };

    match edge_state.first_node {
        None => {
            edge_state.first_node = Some(clicked_entity);
        }
        Some(first) => {
            edge_state.first_node = None;

            if first == clicked_entity {
                return;
            }

            if constraint_exists(first, clicked_entity, &existing) {
                return;
            }

            let Ok((_, _, node_a)) = node_query.get(first) else {
                return;
            };
            let Ok((_, _, node_b)) = node_query.get(clicked_entity) else {
                return;
            };

            if would_exceed_connection_limit(first, clicked_entity, node_a, node_b, &existing, &node_query) {
                warn!("Cannot add edge: a normal node would exceed the maximum of {} non-limb connections.", MAX_NORMAL_NODE_CONNECTIONS);
                return;
            }

            commands.spawn((
                Name::new("Distance Constraint"),
                DistanceConstraint::new(first, clicked_entity, node_a.position.distance(node_b.position)),
            ));
        }
    }
}

/// Cancel edge creation on right-click or Escape.
pub fn cancel_edge_creation(
    mut edge_state: ResMut<EdgeCreationState>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if edge_state.first_node.is_some()
        && (mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape))
    {
        edge_state.first_node = None;
    }
}

/// Renders a dashed preview line from the first selected node to the cursor.
pub fn render_constraint_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    edge_state: Res<EdgeCreationState>,
    tool_state: Res<EditorToolState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<(Entity, &Transform, &SimNode), With<Selectable>>,
    sim_node_query: Query<&SimNode>,
    preview_query: Query<Entity, With<ConstraintPreview>>,
    existing: Query<&DistanceConstraint>,
) {
    for entity in preview_query.iter() {
        commands.entity(entity).despawn();
    }

    if tool_state.active != EditorTool::AddEdge {
        return;
    }

    let Some(first) = edge_state.first_node else { return };
    let Ok(node_a) = sim_node_query.get(first) else { return };
    let Some(world_pos) = cursor_world_pos(&windows, &cameras) else {
        return;
    };

    let dir = world_pos - node_a.position;
    let dist = dir.length();
    if dist < 1e-3 {
        return;
    }
    let norm = dir / dist;
    let start = node_a.position + norm * node_a.radius;

    let hovered_entity = pick_node_at(world_pos, 4.0, &node_query);
    let preview_color = match hovered_entity {
        Some(target) if target != first => {
            let node_a_full = node_a;
            let node_b_opt = sim_node_query.get(target).ok();
            match node_b_opt {
                Some(node_b) if would_exceed_connection_limit(
                    first, target, node_a_full, node_b, &existing, &node_query,
                ) => CONSTRAINT_PREVIEW_INVALID_COLOR,
                _ => CONSTRAINT_PREVIEW_COLOR,
            }
        }
        _ => CONSTRAINT_PREVIEW_COLOR,
    };

    let mesh = create_dashed_line_mesh(
        start,
        world_pos,
        CONSTRAINT_LINE_THICKNESS,
        CONSTRAINT_DASH_LENGTH,
        CONSTRAINT_GAP_LENGTH,
    );

    commands.spawn((
        Name::new("Constraint Preview"),
        ConstraintPreview,
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(preview_color))),
        Transform::from_translation(Vec3::Z * 1.0),
    ));
}

// =============================================================================
// Private Methods
// =============================================================================

/// Checks whether a constraint already exists between two nodes.
fn constraint_exists(a: Entity, b: Entity, existing: &Query<&DistanceConstraint>) -> bool {
    existing
        .iter()
        .any(|c| (c.node_a == a && c.node_b == b) || (c.node_a == b && c.node_b == a))
}

/// Count non-limb connections for a given entity.
fn count_non_limb_connections(
    entity: Entity,
    existing: &Query<&DistanceConstraint>,
    node_query: &Query<(Entity, &Transform, &SimNode), With<Selectable>>,
) -> usize {
    existing
        .iter()
        .filter(|c| {
            let other = if c.node_a == entity {
                Some(c.node_b)
            } else if c.node_b == entity {
                Some(c.node_a)
            } else {
                None
            };
            match other {
                Some(other_entity) => {
                    node_query.get(other_entity)
                        .map(|(_, _, n)| n.node_type != NodeType::Limb)
                        .unwrap_or(false)
                }
                None => false,
            }
        })
        .count()
}

/// Returns true if adding an edge between `entity_a` and `entity_b` would cause
/// either normal node to exceed `MAX_NORMAL_NODE_CONNECTIONS` non-limb connections.
fn would_exceed_connection_limit(
    entity_a: Entity,
    entity_b: Entity,
    node_a: &SimNode,
    node_b: &SimNode,
    existing: &Query<&DistanceConstraint>,
    node_query: &Query<(Entity, &Transform, &SimNode), With<Selectable>>,
) -> bool {
    let a_is_normal = node_a.node_type != NodeType::Limb;
    let b_is_normal = node_b.node_type != NodeType::Limb;

    if !a_is_normal && !b_is_normal {
        return false;
    }

    if a_is_normal && b_is_normal {
        let count = count_non_limb_connections(entity_a, existing, node_query);
        if count >= MAX_NORMAL_NODE_CONNECTIONS {
            return true;
        }
    }

    if b_is_normal && a_is_normal {
        let count = count_non_limb_connections(entity_b, existing, node_query);
        if count >= MAX_NORMAL_NODE_CONNECTIONS {
            return true;
        }
    }

    false
}
