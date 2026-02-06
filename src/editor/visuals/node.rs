//! Node visual spawning and rendering.

use bevy::prelude::*;

use crate::core::{Node, NodeType};
use crate::editor::constants::*;
use crate::editor::components::{NodeVisual, NodeVisualOf, ContactPoint, LookVector, Selectable};
use super::mesh::{create_local_line_mesh, create_hollow_circle_mesh, create_filled_circle_mesh};

// Computes the look direction based on the node type and acceleration.
pub fn get_node_color(node_type: NodeType) -> Color {
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
        let contact_mesh = meshes.add(create_filled_circle_mesh(CONTACT_RADIUS, CONTACT_SEGMENTS));
        let contact_material = materials.add(ColorMaterial::from_color(get_contact_color(node.node_type)));
        let look_mesh = meshes.add(create_local_line_mesh(LOOK_VECTOR_LENGTH, LOOK_VECTOR_THICKNESS));
        let look_material = materials.add(ColorMaterial::from_color(LOOK_VECTOR_COLOR));

        let look = compute_look_direction(node);
        let perp = Vec2::new(-look.y, look.x);
        let look_angle = look.y.atan2(look.x);

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Name::new("Node Visual"),
                NodeVisual,
                NodeVisualOf(entity),
                Mesh2d(meshes.add(mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            ));

            spawn_contact_point(parent, contact_mesh.clone(), contact_material.clone(), perp * node.radius);
            spawn_contact_point(parent, contact_mesh.clone(), contact_material.clone(), -perp * node.radius);
            spawn_look_vector(parent, look_mesh, look_material, look_angle);
        });

        commands.entity(entity).insert((
            Transform::from_translation(node.position.extend(0.0)),
            Selectable,
        ));
    }
}

/// Syncs `Node` position, radius, type changes, contact points, and look vector.
pub fn sync_node_visuals(
    mut meshes: ResMut<Assets<Mesh>>,
    mut node_query: Query<(&Node, &mut Transform, &Children), Changed<Node>>,
    mut visual_query: Query<&mut Mesh2d, With<NodeVisual>>,
    mut contact_query: Query<&mut Transform, (With<ContactPoint>, Without<Node>, Without<LookVector>)>,
    contact_children: Query<Entity, With<ContactPoint>>,
    mut look_query: Query<&mut Transform, (With<LookVector>, Without<Node>, Without<ContactPoint>)>,
    look_children: Query<Entity, With<LookVector>>,
) {
    for (node, mut transform, children) in node_query.iter_mut() {
        transform.translation.x = node.position.x;
        transform.translation.y = node.position.y;

        let look = compute_look_direction(node);
        let perp = Vec2::new(-look.y, look.x);
        let look_angle = look.y.atan2(look.x);

        for child in children.iter() {
            sync_circle_mesh(child, node.radius, &mut meshes, &mut visual_query);
            sync_look_rotation(child, look_angle, &look_children, &mut look_query);
        }

        let offsets = [perp * node.radius, -perp * node.radius];
        sync_contact_positions(children, &offsets, &contact_children, &mut contact_query);
    }
}

// =============================================================================
// Private Methods
// =============================================================================

/// Computes the forward look direction for a node.
fn compute_look_direction(node: &Node) -> Vec2 {
    let velocity = node.position - node.prev_position;
    if velocity.length_squared() > 1e-6 {
        return velocity.normalize();
    }
    if node.acceleration.length_squared() > 1e-6 {
        return node.acceleration.normalize();
    }
    Vec2::X
}

fn get_contact_color(node_type: NodeType) -> Color {
    if node_type == NodeType::Anchor { ANCHOR_CONTACT_COLOR } else { CONTACT_COLOR }
}

fn spawn_contact_point(
    parent: &mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    offset: Vec2,
) {
    parent.spawn((
        Name::new("Contact Point"),
        ContactPoint,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(offset.extend(0.1)),
    ));
}

fn spawn_look_vector(
    parent: &mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    angle: f32,
) {
    parent.spawn((
        Name::new("Look Vector"),
        LookVector,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_rotation(Quat::from_rotation_z(angle))
            .with_translation(Vec3::new(0.0, 0.0, 0.2)),
    ));
}

/// Updates circle mesh for the node visual child.
fn sync_circle_mesh(
    child: Entity,
    radius: f32,
    meshes: &mut Assets<Mesh>,
    visual_query: &mut Query<&mut Mesh2d, With<NodeVisual>>,
) {
    if let Ok(mut mesh_handle) = visual_query.get_mut(child) {
        mesh_handle.0 = meshes.add(create_hollow_circle_mesh(radius, CIRCLE_THICKNESS, CIRCLE_SEGMENTS));
    }
}

/// Updates the look vector child rotation.
fn sync_look_rotation(
    child: Entity,
    angle: f32,
    look_children: &Query<Entity, With<LookVector>>,
    look_query: &mut Query<&mut Transform, (With<LookVector>, Without<Node>, Without<ContactPoint>)>,
) {
    if look_children.get(child).is_ok() {
        if let Ok(mut lt) = look_query.get_mut(child) {
            lt.rotation = Quat::from_rotation_z(angle);
        }
    }
}

/// Updates contact point positions perpendicular to the look direction.
fn sync_contact_positions(
    children: &Children,
    offsets: &[Vec2],
    contact_children: &Query<Entity, With<ContactPoint>>,
    contact_query: &mut Query<&mut Transform, (With<ContactPoint>, Without<Node>, Without<LookVector>)>,
) {
    let mut idx = 0;
    for child in children.iter() {
        if contact_children.get(child).is_ok() {
            if let Ok(mut ct) = contact_query.get_mut(child) {
                if idx < offsets.len() {
                    ct.translation.x = offsets[idx].x;
                    ct.translation.y = offsets[idx].y;
                    idx += 1;
                }
            }
        }
    }
}
