//! Body skin mesh rendering using chain angles and Catmull-Rom splines.

use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::core::components::{Node, NodeType};
use crate::core::resources::ConstraintGraph;
use crate::editor::components::{SkinGroupIndex, SkinMesh, SkinOutline};
use crate::editor::constants::*;
use crate::editor::resources::SkinChains;
use crate::editor::mesh::skin::{
    build_outline_mesh, build_strip_fill_mesh, evaluate_catmull_rom_closed, evaluate_catmull_rom_open,
};
use crate::ui::state::DisplaySettings;

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_6, PI};

pub fn spawn_skin_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    skin_query: Query<Entity, With<SkinMesh>>,
) {
    if !skin_query.is_empty() {
        return;
    }

    let empty = Mesh::new(PrimitiveTopology::TriangleList, default());

    commands.spawn((
        Name::new("Skin Mesh"),
        SkinMesh,
        SkinGroupIndex(0),
        Mesh2d(meshes.add(empty.clone())),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(SKIN_PALETTE[0]))),
        Transform::from_translation(Vec3::Z * -0.5),
    ));

    commands.spawn((
        Name::new("Skin Outline"),
        SkinOutline,
        SkinGroupIndex(0),
        Mesh2d(meshes.add(empty)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(OUTLINE_COLOR))),
        Transform::from_translation(Vec3::Z * -0.4),
    ));
}

pub fn update_skin_chains(graph: Res<ConstraintGraph>, nodes: Query<&Node>, mut skin_chains: ResMut<SkinChains>) {
    if !graph.is_changed() {
        return;
    }

    skin_chains.chains = build_ordered_chains(&graph, &nodes);
}

pub fn sync_skin_visual(
    mut commands: Commands,
    display_settings: Res<DisplaySettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    skin_chains: Res<SkinChains>,
    nodes: Query<&Node>,
    mut fill_query: Query<
        (
            Entity,
            &Mesh2d,
            &MeshMaterial2d<ColorMaterial>,
            &SkinGroupIndex,
            &mut Visibility,
        ),
        (With<SkinMesh>, Without<SkinOutline>),
    >,
    mut outline_query: Query<
        (Entity, &Mesh2d, &SkinGroupIndex, &mut Visibility),
        (With<SkinOutline>, Without<SkinMesh>),
    >,
) {
    let show = display_settings.show_skin;
    let opaque = !display_settings.show_nodes;

    let chains = if show { &skin_chains.chains } else { &Vec::new() };

    let chain_count = chains.len();

    let mut polygons: Vec<Vec<Vec2>> = Vec::with_capacity(chain_count);
    let mut fill_meshes: Vec<Mesh> = Vec::with_capacity(chain_count);

    for chain in chains.iter() {
        if let Some((polygon, fill)) = build_body_skin(chain, &nodes) {
            polygons.push(polygon);
            fill_meshes.push(fill);
        }
    }

    let existing_fill_count = fill_query.iter().count();
    let existing_outline_count = outline_query.iter().count();

    if chain_count > existing_fill_count {
        let empty = Mesh::new(PrimitiveTopology::TriangleList, default());
        for i in existing_fill_count..chain_count {
            let color = SKIN_PALETTE[i % SKIN_PALETTE.len()];
            commands.spawn((
                Name::new("Skin Mesh"),
                SkinMesh,
                SkinGroupIndex(i),
                Mesh2d(meshes.add(empty.clone())),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
                Transform::from_translation(Vec3::Z * -0.5),
            ));
        }
    }

    if chain_count > existing_outline_count {
        let empty = Mesh::new(PrimitiveTopology::TriangleList, default());
        for i in existing_outline_count..chain_count {
            commands.spawn((
                Name::new("Skin Outline"),
                SkinOutline,
                SkinGroupIndex(i),
                Mesh2d(meshes.add(empty.clone())),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(OUTLINE_COLOR))),
                Transform::from_translation(Vec3::Z * -0.4),
            ));
        }
    }

    let mut fill_entities_to_despawn: Vec<Entity> = Vec::new();
    for (entity, mesh_handle, mat_handle, group, mut vis) in fill_query.iter_mut() {
        let idx = group.0;

        if !show || idx >= polygons.len() {
            if idx >= chain_count {
                fill_entities_to_despawn.push(entity);
            } else {
                *vis = Visibility::Hidden;
                if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
                    *mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
                }
            }
            continue;
        }

        *vis = Visibility::Inherited;

        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            if idx < fill_meshes.len() {
                *mesh = fill_meshes[idx].clone();
            }
        }

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = skin_color(idx, opaque);
        }
    }

    let mut outline_entities_to_despawn: Vec<Entity> = Vec::new();
    for (entity, mesh_handle, group, mut vis) in outline_query.iter_mut() {
        let idx = group.0;

        if !show || idx >= polygons.len() {
            if idx >= chain_count {
                outline_entities_to_despawn.push(entity);
            } else {
                *vis = Visibility::Hidden;
                if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
                    *mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
                }
            }
            continue;
        }

        *vis = Visibility::Inherited;

        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = build_outline_mesh(&[polygons[idx].clone()], OUTLINE_THICKNESS);
        }
    }

    for entity in fill_entities_to_despawn {
        commands.entity(entity).despawn();
    }
    for entity in outline_entities_to_despawn {
        commands.entity(entity).despawn();
    }
}

fn build_ordered_chains(graph: &ConstraintGraph, nodes: &Query<&Node>) -> Vec<Vec<(Entity, f32)>> {
    let mut visited: HashMap<Entity, bool> = HashMap::new();
    let mut chains: Vec<Vec<(Entity, f32)>> = Vec::new();
    let starts = find_chain_starts(&graph.adjacency, nodes);

    for &start in &starts {
        if *visited.get(&start).unwrap_or(&false) {
            continue;
        }

        let Some(neighbors) = graph.adjacency.get(&start) else {
            continue;
        };

        for &(next, rest_len) in neighbors {
            if *visited.get(&next).unwrap_or(&false) {
                continue;
            }

            let is_limb = nodes.get(next).map(|n| n.node_type == NodeType::Limb).unwrap_or(false);
            if is_limb {
                continue;
            }

            let chain = trace_chain(start, next, rest_len, &graph.adjacency, &mut visited, nodes);
            if chain.len() >= 2 {
                chains.push(chain);
            }
        }
    }

    chains
}

fn find_chain_starts(adj: &HashMap<Entity, Vec<(Entity, f32)>>, nodes: &Query<&Node>) -> Vec<Entity> {
    let mut starts: Vec<Entity> = Vec::new();
    let mut leaves: Vec<Entity> = Vec::new();

    for (&entity, neighbors) in adj {
        let node_type = nodes.get(entity).map(|n| n.node_type).unwrap_or_default();
        if node_type == NodeType::Limb {
            continue;
        }

        if node_type == NodeType::Anchor {
            starts.push(entity);
        } else if neighbors.len() == 1 {
            leaves.push(entity);
        }
    }

    starts.sort();
    leaves.sort();
    starts.extend(leaves);
    starts
}

fn trace_chain(
    start: Entity,
    first_next: Entity,
    first_rest: f32,
    adj: &HashMap<Entity, Vec<(Entity, f32)>>,
    visited: &mut HashMap<Entity, bool>,
    nodes: &Query<&Node>,
) -> Vec<(Entity, f32)> {
    let mut chain: Vec<(Entity, f32)> = vec![(start, first_rest)];
    visited.insert(start, true);

    let mut current = first_next;
    let mut prev = start;

    loop {
        let is_limb = nodes.get(current).map(|n| n.node_type == NodeType::Limb).unwrap_or(false);
        if is_limb {
            break;
        }

        let cur_neighbors = adj.get(&current).unwrap();
        let non_limb_neighbors: Vec<(Entity, f32)> = cur_neighbors
            .iter()
            .filter(|(e, _)| nodes.get(*e).map(|n| n.node_type != NodeType::Limb).unwrap_or(true))
            .copied()
            .collect();

        if non_limb_neighbors.len() == 2 {
            let (next_node, next_rest) = if non_limb_neighbors[0].0 == prev {
                non_limb_neighbors[1]
            } else {
                non_limb_neighbors[0]
            };
            chain.push((current, next_rest));
            visited.insert(current, true);
            prev = current;
            current = next_node;
        } else {
            chain.push((current, 0.0));
            visited.insert(current, true);
            break;
        }
    }
    chain
}

fn get_offset_pos(node: &Node, angle_offset: f32, length_offset: f32) -> Vec2 {
    node.position + Vec2::from_angle(node.chain_angle + PI + angle_offset) * (node.radius + length_offset)
}

/// Builds both the outline polygon and the strip fill mesh in one pass.
fn build_body_skin(chain: &[(Entity, f32)], nodes: &Query<&Node>) -> Option<(Vec<Vec2>, Mesh)> {
    let chain_nodes: Vec<&Node> = chain
        .iter()
        .filter_map(|&(entity, _)| nodes.get(entity).ok())
        .collect();

    if chain_nodes.len() < 2 {
        return None;
    }

    let node_count = chain_nodes.len();
    let last = node_count - 1;

    let left_ctrl: Vec<Vec2> = (0..node_count)
        .map(|i| get_offset_pos(chain_nodes[i], FRAC_PI_2, 0.0))
        .collect();
    let right_ctrl: Vec<Vec2> = (0..node_count)
        .map(|i| get_offset_pos(chain_nodes[i], -FRAC_PI_2, 0.0))
        .collect();

    let overlap_count = node_count.min(3);
    let ctrl_count = node_count + 1 + node_count + 3 + overlap_count;
    let mut control_points: Vec<Vec2> = Vec::with_capacity(ctrl_count);

    for i in 0..node_count {
        control_points.push(left_ctrl[i]);
    }
    control_points.push(get_offset_pos(chain_nodes[last], PI, 0.0));
    for i in (0..node_count).rev() {
        control_points.push(right_ctrl[i]);
    }
    control_points.push(get_offset_pos(chain_nodes[0], -FRAC_PI_6, 0.0));
    control_points.push(get_offset_pos(chain_nodes[0], 0.0, 0.0));
    control_points.push(get_offset_pos(chain_nodes[0], FRAC_PI_6, 0.0));
    for i in 0..overlap_count {
        control_points.push(left_ctrl[i]);
    }

    let polygon = evaluate_catmull_rom_closed(&control_points, SPLINE_SAMPLES);
    if polygon.len() < 3 {
        return None;
    }

    let left_smooth = evaluate_catmull_rom_open(&left_ctrl, SPLINE_SAMPLES);
    let right_smooth = evaluate_catmull_rom_open(&right_ctrl, SPLINE_SAMPLES);

    // Head cap arc
    let head = chain_nodes[0];
    let head_base = head.chain_angle + PI;
    let mut head_cap: Vec<Vec2> = (0..=CAP_SEGMENTS)
        .map(|k| {
            let t = k as f32 / CAP_SEGMENTS as f32;
            let angle = head_base - FRAC_PI_2 + PI * t;
            head.position + Vec2::from_angle(angle) * head.radius
        })
        .collect();
    head_cap[0] = right_smooth[0];
    *head_cap.last_mut().unwrap() = left_smooth[0];

    // Tail cap arc
    let tail = chain_nodes[last];
    let tail_base = tail.chain_angle + PI;
    let last_l = left_smooth.len() - 1;
    let last_r = right_smooth.len() - 1;
    let mut tail_cap: Vec<Vec2> = (0..=CAP_SEGMENTS)
        .map(|k| {
            let t = k as f32 / CAP_SEGMENTS as f32;
            let angle = tail_base + FRAC_PI_2 + PI * t;
            tail.position + Vec2::from_angle(angle) * tail.radius
        })
        .collect();
    tail_cap[0] = left_smooth[last_l];
    *tail_cap.last_mut().unwrap() = right_smooth[last_r];

    let fill = build_strip_fill_mesh(
        &left_smooth,
        &right_smooth,
        head.position,
        &head_cap,
        tail.position,
        &tail_cap,
    );

    Some((polygon, fill))
}
