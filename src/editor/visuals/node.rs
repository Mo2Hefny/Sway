//! Node visual spawning and rendering.

use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;

use crate::editor::visuals::params::*;
use crate::editor::mesh::primitives::{create_filled_circle_mesh, create_hollow_circle_mesh, create_local_line_mesh, create_x_marker_mesh};
use crate::core::components::LimbSet;
use crate::core::{Node, NodeType};
use crate::editor::components::{
    ContactPoint, DirectionVector, EyeVisual, LookVector, NodeVisual, NodeVisualOf, Selectable, TargetMarker,
};
use crate::editor::constants::*;
use crate::ui::state::DisplaySettings;

pub fn get_node_color(node_type: NodeType) -> Color {
    match node_type {
        NodeType::Anchor => ANCHOR_NODE_COLOR,
        NodeType::Limb => LEG_NODE_COLOR,
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

            let alpha = if node.is_head { 1.0 } else { 0.0 };
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
    mut commands: Commands,
    mut iter_params: NodeIterationParams,
    mut sync_params: NodeSyncParams,
) {
    for (entity, node, mut transform, children, limb_set) in iter_params.query.iter_mut() {
        transform.translation.x = node.position.x;
        transform.translation.y = node.position.y;

        let is_anchor = node.node_type == NodeType::Anchor;
        let show_target = is_anchor || limb_set.is_some();
        let look_angle = node.chain_angle;
        let right_offset = Vec2::from_angle(look_angle + std::f32::consts::FRAC_PI_2) * node.radius;
        let left_offset = Vec2::from_angle(look_angle - std::f32::consts::FRAC_PI_2) * node.radius;

        for child in children.iter() {
            sync_circle_mesh(child, node.radius, &mut sync_params);
            sync_look_rotation(child, look_angle, &mut sync_params);

            if let Ok(mat_handle) = sync_params.material_query.get(child) {
                if let Some(material) = sync_params.materials.get_mut(mat_handle.0.id()) {
                    if let Ok(_) = sync_params.visual_query.get(child) {
                        if limb_set.is_some() {
                             material.color = Color::srgb(0.6, 0.2, 0.8);
                        } else {
                            match node.node_type {
                                NodeType::Anchor => material.color = ANCHOR_NODE_COLOR,
                                NodeType::Limb => material.color = LEG_NODE_COLOR,
                                NodeType::Normal => material.color = NORMAL_NODE_COLOR,
                            }
                        }
                    } else if sync_params.eye_children.contains(child) {
                        material.color = if node.is_head {
                            EYE_COLOR
                        } else {
                            EYE_COLOR.with_alpha(0.0)
                        };
                    } else if sync_params.target_children.contains(child) {
                        material.color = if show_target {
                             TARGET_MARKER_COLOR
                        } else {
                            TARGET_MARKER_COLOR.with_alpha(0.0)
                        };
                    } else if sync_params.dir_children.contains(child) {
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
        sync_contact_positions(children, &offsets, &mut sync_params);
        
        if node.is_head {
            let eye_dist = node.radius * EYE_DISTANCE_RATIO;
            let r_eye = Vec2::from_angle(look_angle + std::f32::consts::FRAC_PI_2) * eye_dist;
            let l_eye = Vec2::from_angle(look_angle - std::f32::consts::FRAC_PI_2) * eye_dist;
            let eye_offsets = [r_eye, l_eye];
            sync_eye_positions(children, &eye_offsets, &mut sync_params);
        }

        if is_anchor || show_target {
            let mut targets = Vec::new();
            if is_anchor {
                targets.push(node.target_position);
            }
            if let Some(ls) = limb_set {
                for limb in &ls.limbs {
                    targets.push(limb.target);
                }
            }

            sync_target_position_markers(
                &mut commands,
                entity,
                &targets,
                node.position,
                &mut sync_params,
            );

            if is_anchor {
                sync_direction_vector(
                    children,
                    node.target_position,
                    node.position,
                    &mut sync_params,
                );
            }
        }
    }
}

pub fn update_node_visibility(mut params: NodeVisibilityParams) {
    if !params.display_settings.is_changed() {
        return;
    }
    let vis = if params.display_settings.show_nodes {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in params.node_visuals.iter_mut() {
        *v = vis;
    }
}

pub fn update_debug_visibility(mut params: NodeVisibilityParams) {
    if !params.display_settings.is_changed() {
        return;
    }
    let vis = if params.display_settings.show_debug {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in params.contacts.iter_mut() {
        *v = vis;
    }
    for mut v in params.looks.iter_mut() {
        *v = vis;
    }
    for mut v in params.targets.iter_mut() {
        *v = vis;
    }
    for mut v in params.dirs.iter_mut() {
        *v = vis;
    }
}

pub fn update_eye_visibility(mut params: NodeVisibilityParams) {
    if !params.display_settings.is_changed() {
        return;
    }
    let vis = if params.display_settings.show_skin {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in params.eyes.iter_mut() {
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
    params: &mut NodeSyncParams,
) {
    if let Ok(mut mesh_handle) = params.visual_query.get_mut(child) {
        mesh_handle.0 = params.meshes.add(create_hollow_circle_mesh(radius, CIRCLE_THICKNESS, CIRCLE_SEGMENTS));
    }
}

fn sync_look_rotation(
    child: Entity,
    angle: f32,
    params: &mut NodeSyncParams,
) {
    if params.look_children.get(child).is_ok() {
        if let Ok(mut lt) = params.look_query.get_mut(child) {
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
    params: &mut NodeSyncParams,
) {
    let mut idx = 0;
    for child in children.iter() {
        if params.eye_children.get(child).is_ok() {
            if let Ok(mut et) = params.eye_query.get_mut(child) {
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
    params: &mut NodeSyncParams,
) {
    let mut idx = 0;
    for child in children.iter() {
        if params.contact_children.get(child).is_ok() {
            if let Ok(mut ct) = params.contact_query.get_mut(child) {
                if idx < offsets.len() {
                    ct.translation.x = offsets[idx].x;
                    ct.translation.y = offsets[idx].y;
                    idx += 1;
                }
            }
        }
    }
}

fn sync_target_position_markers(
    commands: &mut Commands,
    parent_entity: Entity,
    target_positions: &[Vec2],
    current_position: Vec2,
    params: &mut NodeSyncParams,
) {
    let mut marker_entities: Vec<Entity> = Vec::new();
    for (entity, child_of) in params.target_parent_query.iter() {
        if child_of.get() == parent_entity {
            marker_entities.push(entity);
        }
    }

    marker_entities.sort();
    while marker_entities.len() < target_positions.len() {
        let i = marker_entities.len();
        let target_pos = target_positions[i];
        let offset = target_pos - current_position;
        
        let target_mesh = params.meshes.add(create_x_marker_mesh(TARGET_MARKER_SIZE, TARGET_MARKER_THICKNESS));
        let target_mat = params.materials.add(ColorMaterial::from_color(TARGET_MARKER_COLOR));
        let new_marker = commands
            .spawn((
                Name::new("Target Position Marker"),
                TargetMarker,
                Mesh2d(target_mesh),
                MeshMaterial2d(target_mat),
                Transform::from_translation(offset.extend(-0.5)),
            ))
            .id();
        commands.entity(parent_entity).add_child(new_marker);
        marker_entities.push(new_marker);
    }

    for (i, &entity) in marker_entities.iter().enumerate() {
        if i < target_positions.len() {
            if let Ok(mut tt) = params.target_query.get_mut(entity) {
                let offset = target_positions[i] - current_position;
                tt.translation.x = offset.x;
                tt.translation.y = offset.y;
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
}

fn sync_direction_vector(
    children: &Children,
    target_position: Vec2,
    current_position: Vec2,
    params: &mut NodeSyncParams,
) {
    for child in children.iter() {
        if params.dir_children.get(child).is_ok() {
            if let Ok(mut dt) = params.dir_query.get_mut(child) {
                let direction = target_position - current_position;
                let angle = direction.y.atan2(direction.x);
                dt.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}
