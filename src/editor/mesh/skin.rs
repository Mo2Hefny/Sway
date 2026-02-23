//! Shared mesh utilities for skin and limb rendering: triangulation, splines, outlines.

use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::editor::constants::*;

/// Evaluates a Catmull-Rom spline through open points using reflected phantom endpoints.
pub fn evaluate_catmull_rom_open(points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    let n = points.len();
    if n < 2 {
        return points.to_vec();
    }
    if n == 2 {
        let mut result = Vec::with_capacity(samples_per_segment + 1);
        for s in 0..=samples_per_segment {
            let t = s as f32 / samples_per_segment as f32;
            result.push(points[0].lerp(points[1], t));
        }
        return result;
    }

    let phantom_start = 2.0 * points[0] - points[1];
    let phantom_end = 2.0 * points[n - 1] - points[n - 2];

    let mut result = Vec::with_capacity((n - 1) * samples_per_segment + 1);
    for seg in 0..(n - 1) {
        let p0 = if seg == 0 { phantom_start } else { points[seg - 1] };
        let p1 = points[seg];
        let p2 = points[seg + 1];
        let p3 = if seg + 2 < n { points[seg + 2] } else { phantom_end };

        for s in 0..samples_per_segment {
            let t = s as f32 / samples_per_segment as f32;
            result.push(catmull_rom_point(p0, p1, p2, p3, t));
        }
    }
    result.push(points[n - 1]);
    result
}

/// Builds a filled triangle-strip mesh between two parallel curves with optional fan caps at each end.
pub fn build_strip_fill_mesh(
    left: &[Vec2],
    right: &[Vec2],
    head_pos: Vec2,
    head_cap: &[Vec2],
    tail_pos: Vec2,
    tail_cap: &[Vec2],
) -> Mesh {
    let n = left.len().min(right.len());
    if n < 2 {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    let cap_verts = head_cap.len() + tail_cap.len() + 2;
    let strip_verts = n * 2;
    let total_verts = strip_verts + cap_verts;
    let strip_tris = (n - 1) * 6;
    let cap_tris = (head_cap.len().saturating_sub(1) + tail_cap.len().saturating_sub(1)) * 3;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(total_verts);
    let mut indices: Vec<u32> = Vec::with_capacity(strip_tris + cap_tris);

    for i in 0..n {
        positions.push([left[i].x, left[i].y, 0.0]);
        positions.push([right[i].x, right[i].y, 0.0]);
    }

    for i in 0..(n - 1) {
        let li = (i * 2) as u32;
        let ri = li + 1;
        let li1 = ((i + 1) * 2) as u32;
        let ri1 = li1 + 1;

        indices.push(li);
        indices.push(li1);
        indices.push(ri);

        indices.push(ri);
        indices.push(li1);
        indices.push(ri1);
    }

    if head_cap.len() >= 2 {
        let center_idx = positions.len() as u32;
        positions.push([head_pos.x, head_pos.y, 0.0]);
        let cap_base = positions.len() as u32;
        for pt in head_cap {
            positions.push([pt.x, pt.y, 0.0]);
        }
        for i in 0..(head_cap.len() - 1) {
            indices.push(center_idx);
            indices.push(cap_base + i as u32);
            indices.push(cap_base + i as u32 + 1);
        }
    }

    if tail_cap.len() >= 2 {
        let center_idx = positions.len() as u32;
        positions.push([tail_pos.x, tail_pos.y, 0.0]);
        let cap_base = positions.len() as u32;
        for pt in tail_cap {
            positions.push([pt.x, pt.y, 0.0]);
        }
        for i in 0..(tail_cap.len() - 1) {
            indices.push(center_idx);
            indices.push(cap_base + i as u32);
            indices.push(cap_base + i as u32 + 1);
        }
    }

    if positions.is_empty() {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn evaluate_catmull_rom_closed(control_points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    let point_count = control_points.len();
    if point_count < 4 {
        return control_points.to_vec();
    }

    let seg_count = point_count.saturating_sub(3);
    let mut raw_points = Vec::with_capacity(seg_count * samples_per_segment + 1);

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

pub fn build_outline_mesh(polygons: &[Vec<Vec2>], thickness: f32) -> Mesh {
    let half = thickness * 0.5;
    let total_verts: usize = polygons.iter().map(|p| p.len() * 2).sum();
    let total_indices: usize = polygons.iter().map(|p| p.len() * 6).sum();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(total_verts);
    let mut indices: Vec<u32> = Vec::with_capacity(total_indices);

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
