//! Limb mesh rendering using Catmull-Rom splines with per-joint width.

use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::core::components::{LimbSet, Node};
use crate::core::resources::ConstraintGraph;
use crate::editor::components::{LimbMesh, LimbOutline};
use crate::editor::constants::*;
use crate::editor::mesh::skin::{build_fill_mesh, build_outline_mesh, evaluate_catmull_rom_closed};
use crate::ui::state::DisplaySettings;


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
        Mesh2d(meshes.add(empty.clone())),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(SKIN_PALETTE[0]))),
        Transform::from_translation(Vec3::Z * -0.6),
    ));

    commands.spawn((
        Name::new("Limb Outline"),
        LimbOutline,
        Mesh2d(meshes.add(empty)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(OUTLINE_COLOR))),
        Transform::from_translation(Vec3::Z * -0.55),
    ));
}

pub fn sync_limb_visual(
    display_settings: Res<DisplaySettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    graph: Res<ConstraintGraph>,
    limb_sets: Query<(Entity, &LimbSet)>,
    nodes: Query<&Node>,
    mut fill_query: Query<
        (&Mesh2d, &MeshMaterial2d<ColorMaterial>, &mut Visibility),
        (With<LimbMesh>, Without<LimbOutline>),
    >,
    mut outline_query: Query<
        (&Mesh2d, &mut Visibility),
        (With<LimbOutline>, Without<LimbMesh>),
    >,
) {
    let show = display_settings.show_skin;
    let opaque = !display_settings.show_nodes;

    let mut all_fill_polygons: Vec<Vec<Vec2>> = Vec::new();
    let mut all_outline_polygons: Vec<Vec<Vec2>> = Vec::new();
    let mut body_group_ids: Vec<Option<u32>> = Vec::new();

    if show {
        for (body_entity, limb_set) in limb_sets.iter() {
            let body_node = match nodes.get(body_entity) {
                Ok(n) => n,
                Err(_) => continue,
            };
            let body_pos = body_node.position;
            let body_radius = body_node.radius;
            let group_id = graph.get_group(body_entity);

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
                    all_fill_polygons.push(polygon.clone());
                    all_outline_polygons.push(polygon);
                    body_group_ids.push(group_id);
                }
            }
        }
    }

    let combined_fill = build_fill_mesh(&all_fill_polygons);
    let combined_outline = build_outline_mesh(&all_outline_polygons, OUTLINE_THICKNESS);

    let first_group = body_group_ids.first().copied().flatten().unwrap_or(0) as usize;

    for (mesh_handle, mat_handle, mut vis) in fill_query.iter_mut() {
        if !show || all_fill_polygons.is_empty() {
            *vis = Visibility::Hidden;
            if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
                *mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
            }
            continue;
        }

        *vis = Visibility::Inherited;
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = combined_fill.clone();
        }
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = if opaque {
                SKIN_PALETTE_OPAQUE[first_group % SKIN_PALETTE_OPAQUE.len()]
            } else {
                SKIN_PALETTE[first_group % SKIN_PALETTE.len()]
            };
        }
    }

    for (mesh_handle, mut vis) in outline_query.iter_mut() {
        if !show || all_outline_polygons.is_empty() {
            *vis = Visibility::Hidden;
            if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
                *mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
            }
            continue;
        }

        *vis = Visibility::Inherited;
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = combined_outline.clone();
        }
    }
}

// =============================================================================
// Private Methods
// =============================================================================

fn limb_half_width(i: usize, joint_count: usize, radii: &[f32]) -> f32 {
    let t = i as f32 / (joint_count - 1).max(1) as f32;
    let fallback = LIMB_BASE_WIDTH * (1.0 - t) + LIMB_TIP_WIDTH * t;
    if i < radii.len() {
        radii[i].max(fallback)
    } else {
        fallback
    }
}

fn limb_dir(positions: &[Vec2], i: usize) -> Vec2 {
    let joint_count = positions.len();
    if i < joint_count - 1 {
        (positions[i + 1] - positions[i]).normalize_or_zero()
    } else {
        (positions[i] - positions[i - 1]).normalize_or_zero()
    }
}

fn build_limb_polygon(positions: &[Vec2], radii: &[f32]) -> Option<Vec<Vec2>> {
    if positions.len() < 2 {
        return None;
    }

    let joint_count = positions.len();
    let mut control_points: Vec<Vec2> = Vec::new();

    for i in 0..joint_count {
        let half_width = limb_half_width(i, joint_count, radii);
        let dir = limb_dir(positions, i);
        let perp = Vec2::new(-dir.y, dir.x);
        control_points.push(positions[i] + perp * half_width);
    }

    let tip = positions[joint_count - 1];
    let tip_dir = limb_dir(positions, joint_count - 1);
    let t_tip = (joint_count - 1) as f32 / (joint_count - 1).max(1) as f32;
    let tip_w = LIMB_BASE_WIDTH * (1.0 - t_tip) + LIMB_TIP_WIDTH * t_tip;

    control_points.push(tip + tip_dir * tip_w);

    for i in (0..joint_count).rev() {
        let half_width = limb_half_width(i, joint_count, radii);
        let dir = limb_dir(positions, i);
        let perp = Vec2::new(-dir.y, dir.x);
        control_points.push(positions[i] - perp * half_width);
    }

    let overlap_count = joint_count.min(3);
    for i in 0..overlap_count {
        let half_width = limb_half_width(i, joint_count, radii);
        let dir = limb_dir(positions, i);
        let perp = Vec2::new(-dir.y, dir.x);
        control_points.push(positions[i] + perp * half_width);
    }

    if control_points.len() < 4 {
        return None;
    }

    let polygon = evaluate_catmull_rom_closed(&control_points, LIMB_SPLINE_SAMPLES);

    if polygon.len() < 3 {
        return None;
    }

    Some(polygon)
}
