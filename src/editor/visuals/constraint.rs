//! Visual rendering for established distance constraints.

use bevy::prelude::*;

use crate::editor::constants::*;
use crate::editor::components::{ConstraintVisual, ConstraintVisualOf};
use super::mesh::create_line_mesh;
use crate::core::{DistanceConstraint, Node};
use crate::ui::state::DisplaySettings;

pub fn spawn_constraint_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    new_constraints: Query<(Entity, &DistanceConstraint), Added<DistanceConstraint>>,
    nodes: Query<&Node>,
) {
    for (entity, constraint) in new_constraints.iter() {
        let Ok(node_a) = nodes.get(constraint.node_a) else { continue };
        let Ok(node_b) = nodes.get(constraint.node_b) else { continue };

        let (start, end) = edge_endpoints(node_a, node_b);
        let mesh = create_line_mesh(start, end, CONSTRAINT_LINE_THICKNESS);

        commands.spawn((
            Name::new("Constraint Visual"),
            ConstraintVisual,
            ConstraintVisualOf(entity),
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(CONSTRAINT_COLOR))),
            Transform::from_translation(Vec3::Z * 0.5),
        ));
    }
}

pub fn sync_constraint_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    constraints: Query<(Entity, &DistanceConstraint)>,
    nodes: Query<&Node>,
    mut visuals: Query<(Entity, &ConstraintVisualOf, &Mesh2d)>,
) {
    for (vis_entity, vis_of, mesh_handle) in visuals.iter_mut() {
        let Ok((_, constraint)) = constraints.get(vis_of.0) else {
            commands.entity(vis_entity).despawn();
            continue;
        };

        let Ok(node_a) = nodes.get(constraint.node_a) else {
            commands.entity(vis_entity).despawn();
            continue;
        };

        let Ok(node_b) = nodes.get(constraint.node_b) else {
            commands.entity(vis_entity).despawn();
            continue;
        };

        let (start, end) = edge_endpoints(node_a, node_b);
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = create_line_mesh(start, end, CONSTRAINT_LINE_THICKNESS);
        }
    }
}

pub fn update_edge_visibility(
    display_settings: Res<DisplaySettings>,
    mut visuals: Query<&mut Visibility, With<ConstraintVisual>>,
) {
    if !display_settings.is_changed() {
        return;
    }

    let vis = if display_settings.show_edge {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut v in visuals.iter_mut() {
        *v = vis;
    }
}

// =============================================================================
// Private Methods
// =============================================================================

fn edge_endpoints(a: &Node, b: &Node) -> (Vec2, Vec2) {
    let dir = b.position - a.position;
    let dist = dir.length();
    if dist < 1e-6 {
        return (a.position, b.position);
    }
    let norm = dir.normalize();
    (a.position + norm * a.radius, b.position - norm * b.radius)
}
