//! Shared mesh-building utilities for editor visuals.

use bevy::prelude::*;
use bevy::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

/// Creates a thin rectangle mesh extending along the local +X axis from the origin.
pub fn create_local_line_mesh(length: f32, thickness: f32) -> Mesh {
    let half_t = thickness * 0.5;
    let positions = vec![
        [0.0, -half_t, 0.0],
        [length, -half_t, 0.0],
        [length, half_t, 0.0],
        [0.0, half_t, 0.0],
    ];
    let indices = vec![0u32, 1, 2, 0, 2, 3];
    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

/// Creates a solid line mesh between two world-space points.
pub fn create_line_mesh(from: Vec2, to: Vec2, thickness: f32) -> Mesh {
    let dir = to - from;
    let len = dir.length();
    if len < 1e-6 {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    let norm = dir / len;
    let perp = Vec2::new(-norm.y, norm.x) * thickness * 0.5;

    let positions = vec![
        [(from - perp).x, (from - perp).y, 0.0],
        [(from + perp).x, (from + perp).y, 0.0],
        [(to + perp).x, (to + perp).y, 0.0],
        [(to - perp).x, (to - perp).y, 0.0],
    ];
    let indices = vec![0u32, 1, 2, 0, 2, 3];

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

/// Creates a dashed line mesh between two world-space points.
pub fn create_dashed_line_mesh(
    from: Vec2,
    to: Vec2,
    thickness: f32,
    dash_length: f32,
    gap_length: f32,
) -> Mesh {
    let dir = to - from;
    let total = dir.length();
    if total < 1e-6 {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    let norm = dir / total;
    let perp = Vec2::new(-norm.y, norm.x) * thickness * 0.5;

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let cycle = dash_length + gap_length;
    let mut t = 0.0_f32;

    while t < total {
        let dash_end = (t + dash_length).min(total);
        let p0 = from + norm * t;
        let p1 = from + norm * dash_end;

        let base = positions.len() as u32;
        positions.push([(p0 - perp).x, (p0 - perp).y, 0.0]);
        positions.push([(p0 + perp).x, (p0 + perp).y, 0.0]);
        positions.push([(p1 + perp).x, (p1 + perp).y, 0.0]);
        positions.push([(p1 - perp).x, (p1 - perp).y, 0.0]);

        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 3);

        t += cycle;
    }

    if positions.is_empty() {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

/// Creates a hollow circle (ring) mesh.
pub fn create_hollow_circle_mesh(radius: f32, thickness: f32, segments: usize) -> Mesh {
    let inner_radius = radius - thickness * 0.5;
    let outer_radius = radius + thickness * 0.5;

    let vertex_count = (segments + 1) * 2;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(segments * 6);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        positions.push([cos_a * inner_radius, sin_a * inner_radius, 0.0]);
        positions.push([cos_a * outer_radius, sin_a * outer_radius, 0.0]);
    }

    for i in 0..segments {
        let base = (i * 2) as u32;
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);

        indices.push(base + 1);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

/// Creates a filled circle (disk) mesh via fan triangulation.
pub fn create_filled_circle_mesh(radius: f32, segments: usize) -> Mesh {
    let mut positions = Vec::with_capacity(segments + 2);
    let mut indices = Vec::with_capacity(segments * 3);

    // Center vertex
    positions.push([0.0, 0.0, 0.0]);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        positions.push([angle.cos() * radius, angle.sin() * radius, 0.0]);
    }

    for i in 0..segments {
        indices.push(0);
        indices.push((i + 1) as u32);
        indices.push((i + 2) as u32);
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

/// Creates an axis-aligned filled quad mesh.
pub fn create_quad_mesh(min: Vec2, max: Vec2) -> Mesh {
    let positions = vec![
        [min.x, min.y, 0.0],
        [max.x, min.y, 0.0],
        [max.x, max.y, 0.0],
        [min.x, max.y, 0.0],
    ];
    let indices = vec![0u32, 1, 2, 0, 2, 3];

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

/// Creates a hollow rectangle mesh (frame) from outer and inner bounds.
pub fn create_hollow_rect_mesh(
    outer_min: Vec2,
    outer_max: Vec2,
    inner_min: Vec2,
    inner_max: Vec2,
) -> Mesh {
    let positions = vec![
        [outer_min.x, outer_min.y, 0.0],
        [outer_max.x, outer_min.y, 0.0],
        [outer_max.x, outer_max.y, 0.0],
        [outer_min.x, outer_max.y, 0.0],
        [inner_min.x, inner_min.y, 0.0],
        [inner_max.x, inner_min.y, 0.0],
        [inner_max.x, inner_max.y, 0.0],
        [inner_min.x, inner_max.y, 0.0],
    ];

    #[rustfmt::skip]
    let indices = vec![
        0, 1, 5,  0, 5, 4,
        1, 2, 6,  1, 6, 5,
        2, 3, 7,  2, 7, 6,
        3, 0, 4,  3, 4, 7,
    ];

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

/// Creates an 'X' marker mesh centered at origin.
pub fn create_x_marker_mesh(size: f32, thickness: f32) -> Mesh {
    let half_size = size * 0.5;
    let half_t = thickness * 0.5;
    
    let p1_start = Vec2::new(-half_size, half_size);
    let p1_end = Vec2::new(half_size, -half_size);
    
    let p2_start = Vec2::new(half_size, half_size);
    let p2_end = Vec2::new(-half_size, -half_size);
    
    let dir1 = (p1_end - p1_start).normalize();
    let perp1 = Vec2::new(-dir1.y, dir1.x) * half_t;
    
    let dir2 = (p2_end - p2_start).normalize();
    let perp2 = Vec2::new(-dir2.y, dir2.x) * half_t;
    
    let positions = vec![
        // Line 1
        [(p1_start - perp1).x, (p1_start - perp1).y, 0.0],
        [(p1_start + perp1).x, (p1_start + perp1).y, 0.0],
        [(p1_end + perp1).x, (p1_end + perp1).y, 0.0],
        [(p1_end - perp1).x, (p1_end - perp1).y, 0.0],
        // Line 2
        [(p2_start - perp2).x, (p2_start - perp2).y, 0.0],
        [(p2_start + perp2).x, (p2_start + perp2).y, 0.0],
        [(p2_end + perp2).x, (p2_end + perp2).y, 0.0],
        [(p2_end - perp2).x, (p2_end - perp2).y, 0.0],
    ];
    
    let indices = vec![
        // Line 1
        0u32, 1, 2, 0, 2, 3,
        // Line 2
        4, 5, 6, 4, 6, 7,
    ];
    
    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}
