//! Shared mesh utilities for skin and limb rendering: triangulation, splines, outlines.

use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::editor::constants::*;

pub fn evaluate_catmull_rom_closed(control_points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    let point_count = control_points.len();
    if point_count < 4 {
        return control_points.to_vec();
    }

    let mut raw_points = Vec::new();

    for i in 1..(point_count - 2) {
        let p0 = control_points[i - 1];
        let p1 = control_points[i];
        let p2 = control_points[i + 1];
        let p3 = if i + 2 < point_count {
            control_points[i + 2]
        } else {
            control_points[0]
        };

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

pub fn build_fill_mesh(polygons: &[Vec<Vec2>]) -> Mesh {
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

pub fn build_outline_mesh(polygons: &[Vec<Vec2>], thickness: f32) -> Mesh {
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

// =============================================================================
// Private Methods
// =============================================================================

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

    let mut safety = remaining.len() * 3;
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
            let best = find_best_convex_vertex(&remaining, polygon, ccw);
            let len = remaining.len();
            let prev_idx = remaining[(best + len - 1) % len];
            let curr_idx = remaining[best];
            let next_idx = remaining[(best + 1) % len];

            indices.push(prev_idx as u32);
            indices.push(curr_idx as u32);
            indices.push(next_idx as u32);
            remaining.remove(best);
        }
    }

    indices
}

fn find_best_convex_vertex(remaining: &[usize], polygon: &[Vec2], ccw: bool) -> usize {
    let len = remaining.len();
    let mut best_idx = 0;
    let mut best_cross = f32::NEG_INFINITY;

    for i in 0..len {
        let a = polygon[remaining[(i + len - 1) % len]];
        let b = polygon[remaining[i]];
        let c = polygon[remaining[(i + 1) % len]];

        let cross = (b - a).perp_dot(c - b);
        let signed = if ccw { cross } else { -cross };

        if signed > best_cross {
            best_cross = signed;
            best_idx = i;
        }
    }

    best_idx
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
