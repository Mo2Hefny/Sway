//! Skin mesh rendering using chain angles and Catmull-Rom splines.

use bevy::prelude::*;
use bevy::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use crate::core::components::{DistanceConstraint, Node, NodeType};
use crate::editor::components::{SkinMesh, SkinOutline};
use crate::editor::constants::*;
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
        Mesh2d(meshes.add(empty.clone())),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(SKIN_COLOR))),
        Transform::from_translation(Vec3::Z * -0.5),
    ));

    commands.spawn((
        Name::new("Skin Outline"),
        SkinOutline,
        Mesh2d(meshes.add(empty)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(OUTLINE_COLOR))),
        Transform::from_translation(Vec3::Z * -0.4),
    ));
}

pub fn sync_skin_visual(
    display_settings: Res<DisplaySettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    constraints: Query<&DistanceConstraint>,
    nodes: Query<&Node>,
    mut fill_query: Query<
        (&Mesh2d, &MeshMaterial2d<ColorMaterial>, &mut Visibility),
        (With<SkinMesh>, Without<SkinOutline>),
    >,
    mut outline_query: Query<
        (&Mesh2d, &mut Visibility),
        (With<SkinOutline>, Without<SkinMesh>),
    >,
) {
    let show = display_settings.show_skin;
    let opaque = !display_settings.show_nodes;

    let polygons: Vec<Vec<Vec2>> = if show {
        let chains = build_ordered_chains(&constraints, &nodes);
        chains
            .iter()
            .filter_map(|chain| build_body_polygon(chain, &nodes))
            .collect()
    } else {
        Vec::new()
    };

    for (mesh_handle, mat_handle, mut vis) in fill_query.iter_mut() {
        if !show {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;

        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = build_fill_mesh(&polygons);
        }

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = if opaque { SKIN_COLOR_OPAQUE } else { SKIN_COLOR };
        }
    }

    for (mesh_handle, mut vis) in outline_query.iter_mut() {
        if !show {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;

        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = build_outline_mesh(&polygons, OUTLINE_THICKNESS);
        }
    }
}

// =============================================================================
// Private Methods
// =============================================================================

fn get_offset_pos(node: &Node, angle_offset: f32, length_offset: f32) -> Vec2 {
    node.position
        + Vec2::from_angle(node.chain_angle + PI + angle_offset) * (node.radius + length_offset)
}

fn build_body_polygon(
    chain: &[(Entity, f32)],
    nodes: &Query<&Node>,
) -> Option<Vec<Vec2>> {
    let n = chain.len();
    if n < 2 {
        return None;
    }

    let chain_nodes: Vec<&Node> = chain
        .iter()
        .filter_map(|&(entity, _)| nodes.get(entity).ok())
        .collect();

    if chain_nodes.len() < 2 {
        return None;
    }

    let nn = chain_nodes.len();
    let last = nn - 1;

    let mut ctrl: Vec<Vec2> = Vec::new();

    for i in 0..nn {
        ctrl.push(get_offset_pos(chain_nodes[i], FRAC_PI_2, 0.0));
    }

    ctrl.push(get_offset_pos(chain_nodes[last], PI, 0.0));

    for i in (0..nn).rev() {
        ctrl.push(get_offset_pos(chain_nodes[i], -FRAC_PI_2, 0.0));
    }

    ctrl.push(get_offset_pos(chain_nodes[0], -FRAC_PI_6, 0.0));
    ctrl.push(get_offset_pos(chain_nodes[0], 0.0, 0.0));
    ctrl.push(get_offset_pos(chain_nodes[0], FRAC_PI_6, 0.0));

    let overlap_count = nn.min(3);
    for i in 0..overlap_count {
        ctrl.push(get_offset_pos(chain_nodes[i], FRAC_PI_2, 0.0));
    }

    if ctrl.len() < 4 {
        return None;
    }

    let polygon = evaluate_catmull_rom_closed(&ctrl, SPLINE_SAMPLES);

    if polygon.len() < 3 {
        return None;
    }

    Some(polygon)
}

fn evaluate_catmull_rom_closed(ctrl: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    let n = ctrl.len();
    if n < 4 {
        return ctrl.to_vec();
    }

    let mut points = Vec::new();

    for i in 1..(n - 2) {
        let p0 = ctrl[i - 1];
        let p1 = ctrl[i];
        let p2 = ctrl[i + 1];
        let p3 = if i + 2 < n { ctrl[i + 2] } else { ctrl[0] };

        for s in 0..samples_per_segment {
            let t = s as f32 / samples_per_segment as f32;
            points.push(catmull_rom_point(p0, p1, p2, p3, t));
        }
    }

    if n >= 4 {
        points.push(ctrl[n - 2]);
    }

    points
}

fn catmull_rom_point(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;

    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

fn build_fill_mesh(polygons: &[Vec<Vec2>]) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for polygon in polygons {
        if polygon.len() < 3 {
            continue;
        }

        let base = positions.len() as u32;
        for pt in polygon {
            positions.push([pt.x, pt.y, 0.0]);
        }

        let tri_indices = ear_clip_triangulate(polygon);
        for idx in tri_indices {
            indices.push(base + idx);
        }
    }

    if positions.is_empty() {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

fn ear_clip_triangulate(polygon: &[Vec2]) -> Vec<u32> {
    let n = polygon.len();
    if n < 3 {
        return vec![];
    }
    if n == 3 {
        return vec![0, 1, 2];
    }

    let mut indices: Vec<u32> = Vec::new();
    let mut remaining: Vec<usize> = (0..n).collect();

    let ccw = signed_polygon_area(polygon) > 0.0;

    let mut safety = remaining.len() * 2;
    while remaining.len() > 2 && safety > 0 {
        safety -= 1;
        let len = remaining.len();
        let mut found_ear = false;

        for i in 0..len {
            let pi = remaining[(i + len - 1) % len];
            let ci = remaining[i];
            let ni = remaining[(i + 1) % len];

            let a = polygon[pi];
            let b = polygon[ci];
            let c = polygon[ni];

            let cross = (b - a).perp_dot(c - b);
            if (ccw && cross <= 0.0) || (!ccw && cross >= 0.0) {
                continue;
            }

            let mut blocked = false;
            for j in 0..len {
                if j == (i + len - 1) % len || j == i || j == (i + 1) % len {
                    continue;
                }
                if point_in_triangle(polygon[remaining[j]], a, b, c) {
                    blocked = true;
                    break;
                }
            }

            if !blocked {
                indices.push(pi as u32);
                indices.push(ci as u32);
                indices.push(ni as u32);
                remaining.remove(i);
                found_ear = true;
                break;
            }
        }

        if !found_ear {
            break;
        }
    }

    indices
}

fn signed_polygon_area(polygon: &[Vec2]) -> f32 {
    let n = polygon.len();
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += polygon[i].x * polygon[j].y;
        area -= polygon[j].x * polygon[i].y;
    }
    area * 0.5
}

fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let d1 = (p - a).perp_dot(b - a);
    let d2 = (p - b).perp_dot(c - b);
    let d3 = (p - c).perp_dot(a - c);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

fn build_outline_mesh(polygons: &[Vec<Vec2>], thickness: f32) -> Mesh {
    let half = thickness * 0.5;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for polygon in polygons {
        if polygon.len() < 3 {
            continue;
        }

        let n = polygon.len();

        for i in 0..n {
            let curr = polygon[i];
            let next = polygon[(i + 1) % n];

            let dir = (next - curr).normalize_or_zero();
            let perp = Vec2::new(-dir.y, dir.x) * half;

            let base = positions.len() as u32;
            positions.push([(curr - perp).x, (curr - perp).y, 0.0]);
            positions.push([(curr + perp).x, (curr + perp).y, 0.0]);
            positions.push([(next + perp).x, (next + perp).y, 0.0]);
            positions.push([(next - perp).x, (next - perp).y, 0.0]);

            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 3);
        }
    }

    if positions.is_empty() {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

fn build_ordered_chains(
    constraints: &Query<&DistanceConstraint>,
    nodes: &Query<&Node>,
) -> Vec<Vec<(Entity, f32)>> {
    let mut adj: HashMap<Entity, Vec<(Entity, f32)>> = HashMap::new();
    for c in constraints.iter() {
        adj.entry(c.node_a).or_default().push((c.node_b, c.rest_length));
        adj.entry(c.node_b).or_default().push((c.node_a, c.rest_length));
    }

    let mut visited: HashMap<Entity, bool> = HashMap::new();
    let mut chains: Vec<Vec<(Entity, f32)>> = Vec::new();

    let mut starts: Vec<Entity> = Vec::new();
    let mut leaves: Vec<Entity> = Vec::new();

    for (&entity, neighbors) in &adj {
        let is_anchor = nodes
            .get(entity)
            .map(|n| n.node_type == NodeType::Anchor)
            .unwrap_or(false);

        if is_anchor {
            starts.push(entity);
        } else if neighbors.len() == 1 {
            leaves.push(entity);
        }
    }
    starts.extend(leaves);

    for &start in &starts {
        if *visited.get(&start).unwrap_or(&false) {
            continue;
        }

        let Some(neighbors) = adj.get(&start) else {
            continue;
        };

        for &(next, rest_len) in neighbors {
            if *visited.get(&next).unwrap_or(&false) {
                continue;
            }

            let mut chain: Vec<(Entity, f32)> = vec![(start, rest_len)];
            visited.insert(start, true);

            let mut current = next;
            let mut prev = start;

            loop {
                let cur_neighbors = adj.get(&current).unwrap();
                if cur_neighbors.len() == 2 {
                    let (next_node, next_rest) = if cur_neighbors[0].0 == prev {
                        cur_neighbors[1]
                    } else {
                        cur_neighbors[0]
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

            if chain.len() >= 2 {
                chains.push(chain);
            }
        }
    }

    chains
}
