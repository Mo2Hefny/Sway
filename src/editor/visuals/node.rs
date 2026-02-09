//! Node visual spawning and rendering.

use bevy::prelude::*;

use super::mesh::{create_filled_circle_mesh, create_hollow_circle_mesh, create_local_line_mesh, create_x_marker_mesh};
use crate::core::{Node, NodeType};
use crate::editor::components::{
    ContactPoint, DirectionVector, EyeVisual, LookVector, NodeVisual, NodeVisualOf, Selectable, TargetMarker,
};
use crate::editor::constants::*;
use crate::ui::state::DisplaySettings;

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

        let look_angle = node.chain_angle;
        let right_offset = Vec2::from_angle(look_angle + std::f32::consts::FRAC_PI_2) * node.radius;
        let left_offset = Vec2::from_angle(look_angle - std::f32::consts::FRAC_PI_2) * node.radius;

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Name::new("Node Visual"),
                NodeVisual,
                NodeVisualOf(entity),
                Mesh2d(meshes.add(mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            ));

            spawn_contact_point(parent, contact_mesh.clone(), contact_material.clone(), right_offset);
            spawn_contact_point(parent, contact_mesh, contact_material, left_offset);
            spawn_look_vector(parent, look_mesh, look_material, look_angle);

            let eye_dist = node.radius * EYE_DISTANCE_RATIO;
            let r_eye = Vec2::from_angle(look_angle + std::f32::consts::FRAC_PI_2) * eye_dist;
            let l_eye = Vec2::from_angle(look_angle - std::f32::consts::FRAC_PI_2) * eye_dist;

            let alpha = if node.node_type == NodeType::Anchor { 1.0 } else { 0.0 };
            let eye_mesh = meshes.add(create_filled_circle_mesh(EYE_RADIUS, CONTACT_SEGMENTS));
            let eye_mat = materials.add(ColorMaterial::from_color(EYE_COLOR.with_alpha(alpha)));
            spawn_eye(parent, eye_mesh.clone(), eye_mat.clone(), r_eye, 1.0);
            spawn_eye(parent, eye_mesh, eye_mat, l_eye, 1.0);

            let target_mesh = meshes.add(create_x_marker_mesh(TARGET_MARKER_SIZE, TARGET_MARKER_THICKNESS));
            let target_mat = materials.add(ColorMaterial::from_color(TARGET_MARKER_COLOR.with_alpha(alpha)));
            spawn_target_position_marker(parent, target_mesh, target_mat);

            let dir_mesh = meshes.add(create_local_line_mesh(
                DIRECTION_VECTOR_LENGTH,
                DIRECTION_VECTOR_THICKNESS,
            ));
            let dir_mat = materials.add(ColorMaterial::from_color(DIRECTION_VECTOR_COLOR.with_alpha(alpha)));
            spawn_direction_vector(parent, dir_mesh, dir_mat);
        });

        commands
            .entity(entity)
            .insert((Transform::from_translation(node.position.extend(0.0)), Selectable));
    }
}

pub fn sync_node_visuals(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut node_query: Query<(&Node, &mut Transform, &Children), Changed<Node>>,
    mut visual_query: Query<&mut Mesh2d, With<NodeVisual>>,
    mut contact_query: Query<
        &mut Transform,
        (
            With<ContactPoint>,
            Without<Node>,
            Without<LookVector>,
            Without<EyeVisual>,
        ),
    >,
    contact_children: Query<Entity, With<ContactPoint>>,
    mut look_query: Query<
        &mut Transform,
        (
            With<LookVector>,
            Without<Node>,
            Without<ContactPoint>,
            Without<EyeVisual>,
        ),
    >,
    look_children: Query<Entity, With<LookVector>>,
    mut eye_query: Query<
        &mut Transform,
        (
            With<EyeVisual>,
            Without<Node>,
            Without<ContactPoint>,
            Without<LookVector>,
        ),
    >,
    eye_children: Query<Entity, With<EyeVisual>>,
    target_children: Query<Entity, With<TargetMarker>>,
    mut target_query: Query<
        &mut Transform,
        (
            With<TargetMarker>,
            Without<Node>,
            Without<ContactPoint>,
            Without<LookVector>,
            Without<EyeVisual>,
            Without<DirectionVector>,
        ),
    >,
    dir_children: Query<Entity, With<DirectionVector>>,
    mut dir_query: Query<
        &mut Transform,
        (
            With<DirectionVector>,
            Without<Node>,
            Without<ContactPoint>,
            Without<LookVector>,
            Without<EyeVisual>,
            Without<TargetMarker>,
        ),
    >,
    visual_material_query: Query<&MeshMaterial2d<ColorMaterial>>,
) {
    for (node, mut transform, children) in node_query.iter_mut() {
        transform.translation.x = node.position.x;
        transform.translation.y = node.position.y;

        let is_anchor = node.node_type == NodeType::Anchor;
        let look_angle = node.chain_angle;
        let right_offset = Vec2::from_angle(look_angle + std::f32::consts::FRAC_PI_2) * node.radius;
        let left_offset = Vec2::from_angle(look_angle - std::f32::consts::FRAC_PI_2) * node.radius;

        for child in children.iter() {
            sync_circle_mesh(child, node.radius, &mut meshes, &mut visual_query);
            sync_look_rotation(child, look_angle, &look_children, &mut look_query);

            if let Ok(mat_handle) = visual_material_query.get(child) {
                if let Some(material) = materials.get_mut(mat_handle.0.id()) {
                    if eye_children.contains(child) {
                        material.color = if is_anchor {
                            EYE_COLOR
                        } else {
                            EYE_COLOR.with_alpha(0.0)
                        };
                    } else if target_children.contains(child) {
                        material.color = if is_anchor {
                            TARGET_MARKER_COLOR
                        } else {
                            TARGET_MARKER_COLOR.with_alpha(0.0)
                        };
                    } else if dir_children.contains(child) {
                        material.color = if is_anchor {
                            DIRECTION_VECTOR_COLOR
                        } else {
                            DIRECTION_VECTOR_COLOR.with_alpha(0.0)
                        };
                    }
                }
            }
        }

        let offsets = [right_offset, left_offset];
        sync_contact_positions(children, &offsets, &contact_children, &mut contact_query);

        if is_anchor {
            let eye_dist = node.radius * EYE_DISTANCE_RATIO;
            let r_eye = Vec2::from_angle(look_angle + std::f32::consts::FRAC_PI_2) * eye_dist;
            let l_eye = Vec2::from_angle(look_angle - std::f32::consts::FRAC_PI_2) * eye_dist;
            let eye_offsets = [r_eye, l_eye];
            sync_eye_positions(children, &eye_offsets, &eye_children, &mut eye_query);
            sync_target_position_marker(
                children,
                node.target_position,
                node.position,
                &target_children,
                &mut target_query,
            );
            sync_direction_vector(
                children,
                node.target_position,
                node.position,
                &dir_children,
                &mut dir_query,
            );
        }
    }
}

pub fn update_node_visibility(
    display_settings: Res<DisplaySettings>,
    mut node_visuals: Query<&mut Visibility, With<NodeVisual>>,
) {
    if !display_settings.is_changed() {
        return;
    }
    let vis = if display_settings.show_nodes {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in node_visuals.iter_mut() {
        *v = vis;
    }
}

pub fn update_debug_visibility(
    display_settings: Res<DisplaySettings>,
    mut contacts: Query<
        &mut Visibility,
        (
            With<ContactPoint>,
            Without<LookVector>,
            Without<EyeVisual>,
            Without<TargetMarker>,
            Without<DirectionVector>,
        ),
    >,
    mut looks: Query<
        &mut Visibility,
        (
            With<LookVector>,
            Without<ContactPoint>,
            Without<EyeVisual>,
            Without<TargetMarker>,
            Without<DirectionVector>,
        ),
    >,
    mut targets: Query<
        &mut Visibility,
        (
            With<TargetMarker>,
            Without<ContactPoint>,
            Without<LookVector>,
            Without<EyeVisual>,
            Without<DirectionVector>,
        ),
    >,
    mut dirs: Query<
        &mut Visibility,
        (
            With<DirectionVector>,
            Without<ContactPoint>,
            Without<LookVector>,
            Without<EyeVisual>,
            Without<TargetMarker>,
        ),
    >,
) {
    if !display_settings.is_changed() {
        return;
    }
    let vis = if display_settings.show_debug {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in contacts.iter_mut() {
        *v = vis;
    }
    for mut v in looks.iter_mut() {
        *v = vis;
    }
    for mut v in targets.iter_mut() {
        *v = vis;
    }
    for mut v in dirs.iter_mut() {
        *v = vis;
    }
}

pub fn update_eye_visibility(
    display_settings: Res<DisplaySettings>,
    mut eyes: Query<&mut Visibility, With<EyeVisual>>,
) {
    if !display_settings.is_changed() {
        return;
    }
    let vis = if display_settings.show_skin {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in eyes.iter_mut() {
        *v = vis;
    }
}

// =============================================================================
// Private Methods
// =============================================================================

fn get_contact_color(node_type: NodeType) -> Color {
    if node_type == NodeType::Anchor {
        ANCHOR_CONTACT_COLOR
    } else {
        CONTACT_COLOR
    }
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
        Transform::from_rotation(Quat::from_rotation_z(angle)).with_translation(Vec3::new(0.0, 0.0, 0.2)),
    ));
}

fn spawn_target_position_marker(
    parent: &mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
) {
    parent.spawn((
        Name::new("Target Position Marker"),
        TargetMarker,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(0.0, 0.0, -0.5)),
    ));
}

fn spawn_direction_vector(parent: &mut ChildSpawnerCommands, mesh: Handle<Mesh>, material: Handle<ColorMaterial>) {
    parent.spawn((
        Name::new("Direction Vector"),
        DirectionVector,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.25)),
    ));
}

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

fn sync_look_rotation(
    child: Entity,
    angle: f32,
    look_children: &Query<Entity, With<LookVector>>,
    look_query: &mut Query<
        &mut Transform,
        (
            With<LookVector>,
            Without<Node>,
            Without<ContactPoint>,
            Without<EyeVisual>,
        ),
    >,
) {
    if look_children.get(child).is_ok() {
        if let Ok(mut lt) = look_query.get_mut(child) {
            lt.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn spawn_eye(
    parent: &mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    offset: Vec2,
    z: f32,
) {
    parent.spawn((
        Name::new("Eye"),
        EyeVisual,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(offset.extend(z)),
    ));
}

fn sync_eye_positions(
    children: &Children,
    offsets: &[Vec2],
    eye_children: &Query<Entity, With<EyeVisual>>,
    eye_query: &mut Query<
        &mut Transform,
        (
            With<EyeVisual>,
            Without<Node>,
            Without<ContactPoint>,
            Without<LookVector>,
        ),
    >,
) {
    let mut idx = 0;
    for child in children.iter() {
        if eye_children.get(child).is_ok() {
            if let Ok(mut et) = eye_query.get_mut(child) {
                if idx < offsets.len() {
                    et.translation.x = offsets[idx].x;
                    et.translation.y = offsets[idx].y;
                    idx += 1;
                }
            }
        }
    }
}

fn sync_contact_positions(
    children: &Children,
    offsets: &[Vec2],
    contact_children: &Query<Entity, With<ContactPoint>>,
    contact_query: &mut Query<
        &mut Transform,
        (
            With<ContactPoint>,
            Without<Node>,
            Without<LookVector>,
            Without<EyeVisual>,
        ),
    >,
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

fn sync_target_position_marker(
    children: &Children,
    target_position: Vec2,
    current_position: Vec2,
    target_children: &Query<Entity, With<TargetMarker>>,
    target_query: &mut Query<
        &mut Transform,
        (
            With<TargetMarker>,
            Without<Node>,
            Without<ContactPoint>,
            Without<LookVector>,
            Without<EyeVisual>,
            Without<DirectionVector>,
        ),
    >,
) {
    for child in children.iter() {
        if target_children.get(child).is_ok() {
            if let Ok(mut tt) = target_query.get_mut(child) {
                let offset = target_position - current_position;
                tt.translation.x = offset.x;
                tt.translation.y = offset.y;
            }
        }
    }
}

fn sync_direction_vector(
    children: &Children,
    target_position: Vec2,
    current_position: Vec2,
    dir_children: &Query<Entity, With<DirectionVector>>,
    dir_query: &mut Query<
        &mut Transform,
        (
            With<DirectionVector>,
            Without<Node>,
            Without<ContactPoint>,
            Without<LookVector>,
            Without<EyeVisual>,
            Without<TargetMarker>,
        ),
    >,
) {
    for child in children.iter() {
        if dir_children.get(child).is_ok() {
            if let Ok(mut dt) = dir_query.get_mut(child) {
                let direction = target_position - current_position;
                let angle = direction.y.atan2(direction.x);
                dt.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}
