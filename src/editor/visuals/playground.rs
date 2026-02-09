//! Playground boundary visual rendering.

use bevy::prelude::*;

use super::mesh::{create_hollow_rect_mesh, create_quad_mesh};
use crate::core::Playground;
use crate::editor::components::{PlaygroundBorder, PlaygroundFill, PlaygroundOutside};
use crate::editor::constants::*;

/// Syncs `Playground.half_size` to match the window dimensions in world space.
pub fn sync_playground_to_window(mut playground: ResMut<Playground>, windows: Query<&Window>) {
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
        MeshMaterial2d(materials.add(ColorMaterial::from_color(PLAYGROUND_OUTSIDE_COLOR))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
    ));

    commands.spawn((
        Name::new("Playground Border"),
        PlaygroundBorder,
        Mesh2d(meshes.add(border_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(PLAYGROUND_BORDER_COLOR))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
    ));

    commands.spawn((
        Name::new("Playground Fill"),
        PlaygroundFill,
        Mesh2d(meshes.add(fill_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(PLAYGROUND_FILL_COLOR))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.5)),
    ));
}

/// Updates mesh and color for a single playground layer.
fn update_layer(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    mesh_handle: &Mesh2d,
    mat_handle: &MeshMaterial2d<ColorMaterial>,
    new_mesh: Mesh,
    color: Color,
) {
    if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
        *mesh = new_mesh;
    }
    if let Some(mat) = materials.get_mut(&mat_handle.0) {
        mat.color = color;
    }
}

/// Rebuilds playground meshes when the `Playground` resource changes.
pub fn sync_playground_visual(
    playground: Res<Playground>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    outside_query: Query<(&Mesh2d, &MeshMaterial2d<ColorMaterial>), With<PlaygroundOutside>>,
    border_query: Query<
        (&Mesh2d, &MeshMaterial2d<ColorMaterial>),
        (
            With<PlaygroundBorder>,
            Without<PlaygroundOutside>,
            Without<PlaygroundFill>,
        ),
    >,
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

    for (mh, mat) in outside_query.iter() {
        update_layer(
            &mut meshes,
            &mut materials,
            mh,
            mat,
            create_hollow_rect_mesh(-hs, hs, stroke_outer_min, stroke_outer_max),
            PLAYGROUND_OUTSIDE_COLOR,
        );
    }

    for (mh, mat) in border_query.iter() {
        update_layer(
            &mut meshes,
            &mut materials,
            mh,
            mat,
            create_hollow_rect_mesh(stroke_outer_min, stroke_outer_max, inner_min, inner_max),
            PLAYGROUND_BORDER_COLOR,
        );
    }

    for (mh, mat) in fill_query.iter() {
        update_layer(
            &mut meshes,
            &mut materials,
            mh,
            mat,
            create_quad_mesh(inner_min, inner_max),
            PLAYGROUND_FILL_COLOR,
        );
    }
}
