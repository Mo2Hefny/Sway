//! Skin mesh rendering using chain angles and Catmull-Rom splines.

use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::core::components::{DistanceConstraint, Node, NodeType};
use crate::editor::components::{SkinGroupIndex, SkinMesh, SkinOutline};
use crate::editor::constants::*;
use crate::ui::state::DisplaySettings;

use std::collections::{BTreeMap, HashMap};
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

pub fn sync_skin_visual(
    mut commands: Commands,
    display_settings: Res<DisplaySettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    constraints: Query<&DistanceConstraint>,
    nodes: Query<&Node>,
    mut fill_query: Query<
        (Entity, &Mesh2d, &MeshMaterial2d<ColorMaterial>, &SkinGroupIndex, &mut Visibility),
        (With<SkinMesh>, Without<SkinOutline>),
    >,
    mut outline_query: Query<
        (Entity, &Mesh2d, &SkinGroupIndex, &mut Visibility),
        (With<SkinOutline>, Without<SkinMesh>),
    >,
) {
    let show = display_settings.show_skin;
    let opaque = !display_settings.show_nodes;

    let chains = if show {
        build_ordered_chains(&constraints, &nodes)
    } else {
        Vec::new()
    };

    let chain_count = chains.len();

    let polygons: Vec<Vec<Vec2>> = chains
        .iter()
        .filter_map(|chain| build_body_polygon(chain, &nodes))
        .collect();

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
            *mesh = build_fill_mesh(&[polygons[idx].clone()]);
        }

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = if opaque {
                SKIN_PALETTE_OPAQUE[idx % SKIN_PALETTE_OPAQUE.len()]
            } else {
                SKIN_PALETTE[idx % SKIN_PALETTE.len()]
            };
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

// =============================================================================
// Private Methods
// =============================================================================

fn build_ordered_chains(constraints: &Query<&DistanceConstraint>, nodes: &Query<&Node>) -> Vec<Vec<(Entity, f32)>> {
    let adj = build_adjacency_map(constraints);
    let mut visited: HashMap<Entity, bool> = HashMap::new();
    let mut chains: Vec<Vec<(Entity, f32)>> = Vec::new();
    let starts = find_chain_starts(&adj, nodes);

    for &start in &starts {
        if *visited.get(&start).unwrap_or(&false) {
            continue;
        }

        let Some(neighbors) = adj.get(&start) else { continue };

        for &(next, rest_len) in neighbors {
            if *visited.get(&next).unwrap_or(&false) {
                continue;
            }

            let chain = trace_chain(start, next, rest_len, &adj, &mut visited);
            if chain.len() >= 2 {
                chains.push(chain);
            }
        }
    }

    chains
}

fn build_adjacency_map(constraints: &Query<&DistanceConstraint>) -> BTreeMap<Entity, Vec<(Entity, f32)>> {
    let mut adj: BTreeMap<Entity, Vec<(Entity, f32)>> = BTreeMap::new();
    for c in constraints.iter() {
        adj.entry(c.node_a).or_default().push((c.node_b, c.rest_length));
        adj.entry(c.node_b).or_default().push((c.node_a, c.rest_length));
    }

    for neighbors in adj.values_mut() {
        neighbors.sort_by_key(|&(e, _)| e);
    }
    adj
}

fn find_chain_starts(adj: &BTreeMap<Entity, Vec<(Entity, f32)>>, nodes: &Query<&Node>) -> Vec<Entity> {
    let mut starts: Vec<Entity> = Vec::new();
    let mut leaves: Vec<Entity> = Vec::new();

    for (&entity, neighbors) in adj {
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
    
    starts.sort();
    leaves.sort();
    starts.extend(leaves);
    starts
}

fn trace_chain(
    start: Entity,
    first_next: Entity,
    first_rest: f32,
    adj: &BTreeMap<Entity, Vec<(Entity, f32)>>,
    visited: &mut HashMap<Entity, bool>,
) -> Vec<(Entity, f32)> {
    let mut chain: Vec<(Entity, f32)> = vec![(start, first_rest)];
    visited.insert(start, true);

    let mut current = first_next;
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
    chain
}

fn get_offset_pos(node: &Node, angle_offset: f32, length_offset: f32) -> Vec2 {
    node.position + Vec2::from_angle(node.chain_angle + PI + angle_offset) * (node.radius + length_offset)
}

fn build_body_polygon(chain: &[(Entity, f32)], nodes: &Query<&Node>) -> Option<Vec<Vec2>> {
    let chain_nodes: Vec<&Node> = chain.iter().filter_map(|&(entity, _)| nodes.get(entity).ok()).collect();

    if chain_nodes.len() < 2 {
        return None;
    }

    let node_count = chain_nodes.len();
    let last = node_count - 1;

    let mut control_points: Vec<Vec2> = Vec::new();

    for i in 0..node_count {
        control_points.push(get_offset_pos(chain_nodes[i], FRAC_PI_2, 0.0));
    }

    control_points.push(get_offset_pos(chain_nodes[last], PI, 0.0));

    for i in (0..node_count).rev() {
        control_points.push(get_offset_pos(chain_nodes[i], -FRAC_PI_2, 0.0));
    }

    control_points.push(get_offset_pos(chain_nodes[0], -FRAC_PI_6, 0.0));
    control_points.push(get_offset_pos(chain_nodes[0], 0.0, 0.0));
    control_points.push(get_offset_pos(chain_nodes[0], FRAC_PI_6, 0.0));

    let overlap_count = node_count.min(3);
    for i in 0..overlap_count {
        control_points.push(get_offset_pos(chain_nodes[i], FRAC_PI_2, 0.0));
    }

    if control_points.len() < 4 {
        return None;
    }

    let polygon = evaluate_catmull_rom_closed(&control_points, SPLINE_SAMPLES);

    if polygon.len() < 3 {
        return None;
    }

    Some(polygon)
}

fn evaluate_catmull_rom_closed(control_points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    let point_count = control_points.len();
    if point_count < 4 {
        return control_points.to_vec();
    }

    let mut raw_points = Vec::new();

    for i in 1..(point_count - 2) {
        let p0 = control_points[i - 1];
        let p1 = control_points[i];
        let p2 = control_points[i + 1];
        let p3 = if i + 2 < point_count { control_points[i + 2] } else { control_points[0] };

        for s in 0..samples_per_segment {
            let t = s as f32 / samples_per_segment as f32;
            raw_points.push(catmull_rom_point(p0, p1, p2, p3, t));
        }
    }

    if point_count >= 4 {
        raw_points.push(control_points[point_count - 2]);
    }

    filter_close_points(&raw_points, MIN_SPLINE_POINT_DISTANCE)
}

fn filter_close_points(points: &[Vec2], min_distance: f32) -> Vec<Vec2> {
    if points.is_empty() {
        return Vec::new();
    }

    let min_dist_sq = min_distance * min_distance;
    let mut filtered = vec![points[0]];

    for &point in &points[1..] {
        if point.distance_squared(*filtered.last().unwrap()) >= min_dist_sq {
            filtered.push(point);
        }
    }

    filtered
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
    let vertex_count = polygon.len();
    if vertex_count < 3 {
        return vec![];
    }
    if vertex_count == 3 {
        return vec![0, 1, 2];
    }

    let mut indices: Vec<u32> = Vec::new();
    let mut remaining: Vec<usize> = (0..vertex_count).collect();

    let ccw = signed_polygon_area(polygon) > 0.0;

    let mut safety = remaining.len() * 2;
    while remaining.len() > 2 && safety > 0 {
        safety -= 1;
        let len = remaining.len();
        let mut found_ear = false;

        for i in 0..len {
            let prev_idx = remaining[(i + len - 1) % len];
            let curr_idx = remaining[i];
            let next_idx = remaining[(i + 1) % len];

            let a = polygon[prev_idx];
            let b = polygon[curr_idx];
            let c = polygon[next_idx];

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
                indices.push(prev_idx as u32);
                indices.push(curr_idx as u32);
                indices.push(next_idx as u32);
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
    let vertex_count = polygon.len();
    let mut area = 0.0;
    for i in 0..vertex_count {
        let j = (i + 1) % vertex_count;
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

fn compute_miter_normals(polygon: &[Vec2]) -> Vec<(Vec2, f32)> {
    let vertex_count = polygon.len();
    let mut normals = Vec::with_capacity(vertex_count);

    for i in 0..vertex_count {
        let prev = polygon[(i + vertex_count - 1) % vertex_count];
        let curr = polygon[i];
        let next = polygon[(i + 1) % vertex_count];

        let edge_prev = (curr - prev).normalize_or_zero();
        let edge_next = (next - curr).normalize_or_zero();

        let normal_prev = Vec2::new(-edge_prev.y, edge_prev.x);
        let normal_next = Vec2::new(-edge_next.y, edge_next.x);

        let miter = (normal_prev + normal_next).normalize_or_zero();

        let dot = miter.dot(normal_prev);
        let miter_length = if dot.abs() > 0.1 {
            (1.0 / dot).clamp(-MITER_LIMIT, MITER_LIMIT)
        } else {
            1.0
        };

        normals.push((miter, miter_length));
    }

    normals
}

fn build_outline_mesh(polygons: &[Vec<Vec2>], thickness: f32) -> Mesh {
    let half = thickness * 0.5;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for polygon in polygons {
        if polygon.len() < 3 {
            continue;
        }

        let vertex_count = polygon.len();
        let miter_normals = compute_miter_normals(polygon);
        let base = positions.len() as u32;

        for i in 0..vertex_count {
            let (miter_dir, miter_len) = miter_normals[i];
            let offset = miter_dir * half * miter_len;
            let inner = polygon[i] - offset;
            let outer = polygon[i] + offset;
            positions.push([inner.x, inner.y, 0.0]);
            positions.push([outer.x, outer.y, 0.0]);
        }

        for i in 0..vertex_count {
            let next = (i + 1) % vertex_count;
            let i0 = base + (i as u32) * 2;
            let i1 = i0 + 1;
            let i2 = base + (next as u32) * 2 + 1;
            let i3 = base + (next as u32) * 2;

            indices.push(i0);
            indices.push(i1);
            indices.push(i2);
            indices.push(i0);
            indices.push(i2);
            indices.push(i3);
        }
    }

    if positions.is_empty() {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}
