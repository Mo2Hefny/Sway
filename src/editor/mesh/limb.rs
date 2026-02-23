//! Limb mesh rendering using Catmull-Rom splines with per-joint width.

use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::core::components::{LimbSet, Node};
use crate::core::resources::ConstraintGraph;
use crate::editor::components::{LimbMesh, LimbOutline, SkinGroupIndex};
use crate::editor::constants::*;
use crate::editor::mesh::skin::{
    build_outline_mesh, build_strip_fill_mesh, evaluate_catmull_rom_closed,
    evaluate_catmull_rom_open,
};
use crate::ui::state::DisplaySettings;

use std::f32::consts::{FRAC_PI_2, PI};
use std::collections::HashMap;


pub fn spawn_limb_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    limb_query: Query<Entity, With<LimbMesh>>,
) {
    if !limb_query.is_empty() {
        return;
    }

    let empty = Mesh::new(PrimitiveTopology::TriangleList, default());

    commands.spawn((
        Name::new("Limb Mesh"),
        LimbMesh,
        SkinGroupIndex(0),
        Mesh2d(meshes.add(empty.clone())),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(SKIN_PALETTE[0]))),
        Transform::from_translation(Vec3::Z * -0.6),
    ));

    commands.spawn((
        Name::new("Limb Outline"),
        LimbOutline,
        SkinGroupIndex(0),
        Mesh2d(meshes.add(empty)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(OUTLINE_COLOR))),
        Transform::from_translation(Vec3::Z * -0.55),
    ));
}

pub fn sync_limb_visual(
    mut commands: Commands,
    display_settings: Res<DisplaySettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    graph: Res<ConstraintGraph>,
    limb_sets: Query<(Entity, &LimbSet)>,
    nodes: Query<&Node>,
    mut fill_query: Query<
        (Entity, &Mesh2d, &MeshMaterial2d<ColorMaterial>, &SkinGroupIndex, &mut Visibility),
        (With<LimbMesh>, Without<LimbOutline>),
    >,
    mut outline_query: Query<
        (Entity, &Mesh2d, &SkinGroupIndex, &mut Visibility),
        (With<LimbOutline>, Without<LimbMesh>),
    >,
) {
    let show = display_settings.show_skin;
    let opaque = !display_settings.show_nodes;

    let mut group_fills: HashMap<u32, Vec<Mesh>> = HashMap::new();
    let mut group_polygons: HashMap<u32, Vec<Vec<Vec2>>> = HashMap::new();

    if show {
        for (body_entity, limb_set) in limb_sets.iter() {
            let body_node = match nodes.get(body_entity) {
                Ok(n) => n,
                Err(_) => continue,
            };
            let body_pos = body_node.position;
            let body_radius = body_node.radius;
            let group_id = graph.get_group(body_entity).unwrap_or(0);

            for limb in &limb_set.limbs {
                let mut positions: Vec<Vec2> = vec![body_pos];
                let mut radii: Vec<f32> = vec![body_radius];
                for &joint_entity in &limb.joints {
                    if let Ok(joint_node) = nodes.get(joint_entity) {
                        positions.push(joint_node.position);
                        radii.push(joint_node.radius);
                    }
                }

                if positions.len() < 2 {
                    continue;
                }

                if let Some(polygon) = build_limb_polygon(&positions, &radii) {
                    group_polygons.entry(group_id).or_default().push(polygon);
                }
                if let Some(fill) = build_limb_fill(&positions, &radii) {
                    group_fills.entry(group_id).or_default().push(fill);
                }
            }
        }
    }

    let mut active_groups: Vec<u32> = group_fills.keys().cloned().collect();
    active_groups.sort();
    let max_group_idx = active_groups.last().copied().map(|g| g as usize + 1).unwrap_or(0);

    let existing_fill_count = fill_query.iter().count();
    let existing_outline_count = outline_query.iter().count();

    if max_group_idx > existing_fill_count {
        let empty = Mesh::new(PrimitiveTopology::TriangleList, default());
        for i in existing_fill_count..max_group_idx {
            commands.spawn((
                Name::new("Limb Mesh"),
                LimbMesh,
                SkinGroupIndex(i),
                Mesh2d(meshes.add(empty.clone())),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(SKIN_PALETTE[i % SKIN_PALETTE.len()]))),
                Transform::from_translation(Vec3::Z * -0.6),
            ));
        }
    }

    if max_group_idx > existing_outline_count {
        let empty = Mesh::new(PrimitiveTopology::TriangleList, default());
        for i in existing_outline_count..max_group_idx {
            commands.spawn((
                Name::new("Limb Outline"),
                LimbOutline,
                SkinGroupIndex(i),
                Mesh2d(meshes.add(empty.clone())),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(OUTLINE_COLOR))),
                Transform::from_translation(Vec3::Z * -0.55),
            ));
        }
    }

    let mut fill_entities_to_despawn = Vec::new();
    for (entity, mesh_handle, mat_handle, group, mut vis) in fill_query.iter_mut() {
        let idx = group.0 as u32;
        if !show || !group_fills.contains_key(&idx) {
            if group.0 >= max_group_idx {
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
            if let Some(fills) = group_fills.get(&idx) {
                *mesh = merge_meshes(fills);
            }
        }
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let color_idx = graph.get_group_min(idx).map(|e| e.index().index() as usize).unwrap_or(idx as usize);
            mat.color = skin_color(color_idx, opaque);
        }
    }

    let mut outline_entities_to_despawn = Vec::new();
    for (entity, mesh_handle, group, mut vis) in outline_query.iter_mut() {
        let idx = group.0 as u32;
        if !show || !group_polygons.contains_key(&idx) {
            if group.0 >= max_group_idx {
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
            if let Some(polygons) = group_polygons.get(&idx) {
                *mesh = build_outline_mesh(polygons, OUTLINE_THICKNESS);
            }
        }
    }

    for entity in fill_entities_to_despawn { commands.entity(entity).despawn(); }
    for entity in outline_entities_to_despawn { commands.entity(entity).despawn(); }
}

// =============================================================================
// Private Methods
// =============================================================================

/// Effective radius for joint `i`.
fn joint_radius(i: usize, joint_count: usize, radii: &[f32]) -> f32 {
    let t = i as f32 / (joint_count - 1).max(1) as f32;
    let fallback = LIMB_BASE_WIDTH * (1.0 - t) + LIMB_TIP_WIDTH * t;
    if i < radii.len() {
        radii[i].max(fallback)
    } else {
        fallback
    }
}

/// Direction angle at joint `i` (average of incoming and outgoing segments).
fn joint_angle(positions: &[Vec2], i: usize) -> f32 {
    let n = positions.len();
    if n < 2 {
        return 0.0;
    }
    if i == 0 {
        let d = positions[1] - positions[0];
        d.y.atan2(d.x)
    } else if i == n - 1 {
        let d = positions[n - 1] - positions[n - 2];
        d.y.atan2(d.x)
    } else {
        let d_in = (positions[i] - positions[i - 1]).normalize_or_zero();
        let d_out = (positions[i + 1] - positions[i]).normalize_or_zero();
        let avg = (d_in + d_out).normalize_or_zero();
        avg.y.atan2(avg.x)
    }
}

fn build_limb_polygon(positions: &[Vec2], radii: &[f32]) -> Option<Vec<Vec2>> {
    if positions.len() < 2 {
        return None;
    }

    let jc = positions.len();
    let last = jc - 1;

    let r: Vec<f32> = (0..jc).map(|i| joint_radius(i, jc, radii)).collect();
    let angles: Vec<f32> = (0..jc).map(|i| joint_angle(positions, i)).collect();

    let mut ctrl: Vec<Vec2> = Vec::new();

    for i in 0..jc {
        ctrl.push(positions[i] + Vec2::from_angle(angles[i] + FRAC_PI_2) * r[i]);
    }

    {
        let a = angles[last];
        let ri = r[last];
        for k in 0..=JOINT_ARC_SEGMENTS {
            let t = k as f32 / JOINT_ARC_SEGMENTS as f32;
            let angle = a + FRAC_PI_2 - PI * t;
            ctrl.push(positions[last] + Vec2::from_angle(angle) * ri);
        }
    }

    for i in (0..jc).rev() {
        if i == last { continue; }
        ctrl.push(positions[i] + Vec2::from_angle(angles[i] - FRAC_PI_2) * r[i]);
    }

    let oc = ctrl.len().min(3);
    let overlap: Vec<Vec2> = ctrl[..oc].to_vec();
    ctrl.extend(overlap);

    if ctrl.len() < 4 {
        return None;
    }

    let polygon = evaluate_catmull_rom_closed(&ctrl, LIMB_SPLINE_SAMPLES);

    if polygon.len() < 3 {
        return None;
    }

    Some(polygon)
}

/// Merges multiple meshes into one by concatenating their vertex/index buffers.
fn merge_meshes(meshes: &[Mesh]) -> Mesh {
    use bevy::mesh::Indices;

    let mut all_positions: Vec<[f32; 3]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();

    for mesh in meshes {
        let base = all_positions.len() as u32;

        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            all_positions.extend_from_slice(positions);
        }

        if let Some(Indices::U32(indices)) = mesh.indices() {
            for &idx in indices {
                all_indices.push(base + idx);
            }
        }
    }

    if all_positions.is_empty() {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, all_positions)
        .with_inserted_indices(Indices::U32(all_indices))
}

/// Builds a limb strip-fill mesh with a tip cap; no root cap (body skin covers it).
fn build_limb_fill(positions: &[Vec2], radii: &[f32]) -> Option<Mesh> {
    if positions.len() < 2 {
        return None;
    }

    let jc = positions.len();
    let last = jc - 1;

    let r: Vec<f32> = (0..jc).map(|i| joint_radius(i, jc, radii)).collect();
    let angles: Vec<f32> = (0..jc).map(|i| joint_angle(positions, i)).collect();

    let left_ctrl: Vec<Vec2> = (0..jc)
        .map(|i| positions[i] + Vec2::from_angle(angles[i] + FRAC_PI_2) * r[i])
        .collect();
    let right_ctrl: Vec<Vec2> = (0..jc)
        .map(|i| positions[i] + Vec2::from_angle(angles[i] - FRAC_PI_2) * r[i])
        .collect();

    let left_smooth = evaluate_catmull_rom_open(&left_ctrl, LIMB_SPLINE_SAMPLES);
    let right_smooth = evaluate_catmull_rom_open(&right_ctrl, LIMB_SPLINE_SAMPLES);

    let last_l = left_smooth.len() - 1;
    let last_r = right_smooth.len() - 1;
    let mut tip_cap: Vec<Vec2> = (0..=CAP_SEGMENTS)
        .map(|k| {
            let t = k as f32 / CAP_SEGMENTS as f32;
            let angle = angles[last] + FRAC_PI_2 - PI * t;
            positions[last] + Vec2::from_angle(angle) * r[last]
        })
        .collect();
    tip_cap[0] = left_smooth[last_l];
    *tip_cap.last_mut().unwrap() = right_smooth[last_r];

    Some(build_strip_fill_mesh(
        &left_smooth,
        &right_smooth,
        positions[0],
        &[],
        positions[last],
        &tip_cap,
    ))
}
