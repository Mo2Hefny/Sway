//! Node visual spawning and rendering.

use bevy::prelude::*;
use bevy::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use crate::core::{Node, NodeType};
use crate::editor::selection::Selectable;
use crate::editor::constants::*;

/// Marker component for node visual entities.
#[derive(Component, Clone, Debug, Reflect)]
pub struct NodeVisual;

/// Links a visual entity back to its parent node.
#[derive(Component, Clone, Debug, Reflect)]
pub struct NodeVisualOf(pub Entity);

fn create_hollow_circle_mesh(radius: f32, thickness: f32, segments: usize) -> Mesh {
    let inner_radius = radius - thickness * 0.5;
    let outer_radius = radius + thickness * 0.5;
    
    let vertex_count = (segments + 1) * 2;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(segments * 6);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        positions.push([cos_a * inner_radius, sin_a * inner_radius, 0.0]);
        positions.push([cos_a * outer_radius, sin_a * outer_radius, 0.0]);
    }

    for i in 0..segments {
        let base = (i * 2) as u32;
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
        
        indices.push(base + 1);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

fn get_node_color(node_type: NodeType) -> Color {
    match node_type {
        NodeType::Anchor => ANCHOR_NODE_COLOR,
        NodeType::Leg => LEG_NODE_COLOR,
        NodeType::Normal => NORMAL_NODE_COLOR,
    }
}

pub fn spawn_node_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &Node), Added<Node>>,
) {
    for (entity, node) in query.iter() {
        let mesh = create_hollow_circle_mesh(node.radius, CIRCLE_THICKNESS, CIRCLE_SEGMENTS);
        let color = get_node_color(node.node_type);

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Name::new("Node Visual"),
                NodeVisual,
                NodeVisualOf(entity),
                Mesh2d(meshes.add(mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            ));
        });

        commands.entity(entity).insert((
            Transform::from_translation(node.position.extend(0.0)),
            Selectable,
        ));
    }
}

/// Syncs `Node` position and radius changes to `Transform` and visual mesh.
pub fn sync_node_visuals(
    mut meshes: ResMut<Assets<Mesh>>,
    mut node_query: Query<(&Node, &mut Transform, &Children), Changed<Node>>,
    mut visual_query: Query<&mut Mesh2d, With<NodeVisual>>,
) {
    for (node, mut transform, children) in node_query.iter_mut() {
        transform.translation.x = node.position.x;
        transform.translation.y = node.position.y;

        for child in children.iter() {
            if let Ok(mut mesh_handle) = visual_query.get_mut(child) {
                let new_mesh = create_hollow_circle_mesh(node.radius, CIRCLE_THICKNESS, CIRCLE_SEGMENTS);
                mesh_handle.0 = meshes.add(new_mesh);
            }
        }
    }
}
