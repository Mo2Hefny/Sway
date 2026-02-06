//! Playground boundary visual rendering.

use bevy::prelude::*;
use bevy::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use crate::core::playground::Playground;

/// Marker for the outside fill (margin area between window edge and border).
#[derive(Component, Debug)]
pub struct PlaygroundOutside;

/// Marker for the border stroke.
#[derive(Component, Debug)]
pub struct PlaygroundBorder;

/// Marker for the inside fill (playable area).
#[derive(Component, Debug)]
pub struct PlaygroundFill;

fn create_quad_mesh(min: Vec2, max: Vec2) -> Mesh {
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

fn create_hollow_rect_mesh(outer_min: Vec2, outer_max: Vec2, inner_min: Vec2, inner_max: Vec2) -> Mesh {
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

/// Syncs `Playground.half_size` to match the window dimensions in world space.
pub fn sync_playground_to_window(
    mut playground: ResMut<Playground>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else { return };
    let new_half = Vec2::new(window.width() * 0.5, window.height() * 0.5);

    if (playground.half_size - new_half).length_squared() > 0.1 {
        playground.half_size = new_half;
    }
}

/// Spawns three visual layers: outside fill, border stroke, inside fill.
pub fn spawn_playground_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    playground: Res<Playground>,
) {
    let hs = playground.half_size;
    let stroke_outer_min = playground.stroke_outer_min();
    let stroke_outer_max = playground.stroke_outer_max();
    let inner_min = playground.inner_min();
    let inner_max = playground.inner_max();

    let outside_mesh = create_hollow_rect_mesh(-hs, hs, stroke_outer_min, stroke_outer_max);
    let border_mesh = create_hollow_rect_mesh(stroke_outer_min, stroke_outer_max, inner_min, inner_max);
    let fill_mesh = create_quad_mesh(inner_min, inner_max);

    commands.spawn((
        Name::new("Playground Outside"),
        PlaygroundOutside,
        Mesh2d(meshes.add(outside_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(playground.outside_color))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
    ));

    commands.spawn((
        Name::new("Playground Border"),
        PlaygroundBorder,
        Mesh2d(meshes.add(border_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(playground.border_color))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
    ));

    commands.spawn((
        Name::new("Playground Fill"),
        PlaygroundFill,
        Mesh2d(meshes.add(fill_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(playground.fill_color))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.5)),
    ));
}

/// Rebuilds playground meshes when the `Playground` resource changes.
pub fn sync_playground_visual(
    playground: Res<Playground>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    outside_query: Query<(&Mesh2d, &MeshMaterial2d<ColorMaterial>), With<PlaygroundOutside>>,
    border_query: Query<(&Mesh2d, &MeshMaterial2d<ColorMaterial>), (With<PlaygroundBorder>, Without<PlaygroundOutside>, Without<PlaygroundFill>)>,
    fill_query: Query<(&Mesh2d, &MeshMaterial2d<ColorMaterial>), With<PlaygroundFill>>,
) {
    if !playground.is_changed() {
        return;
    }

    let hs = playground.half_size;
    let stroke_outer_min = playground.stroke_outer_min();
    let stroke_outer_max = playground.stroke_outer_max();
    let inner_min = playground.inner_min();
    let inner_max = playground.inner_max();

    for (mesh_handle, mat_handle) in outside_query.iter() {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = create_hollow_rect_mesh(-hs, hs, stroke_outer_min, stroke_outer_max);
        }
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = playground.outside_color;
        }
    }

    for (mesh_handle, mat_handle) in border_query.iter() {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = create_hollow_rect_mesh(stroke_outer_min, stroke_outer_max, inner_min, inner_max);
        }
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = playground.border_color;
        }
    }

    for (mesh_handle, mat_handle) in fill_query.iter() {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = create_quad_mesh(inner_min, inner_max);
        }
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = playground.fill_color;
        }
    }
}
