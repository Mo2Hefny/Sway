//! Editor systems for node visualization and interaction.

pub mod components;
pub mod constants;
pub mod resources;
pub mod tools;
pub mod visuals;
pub mod mesh;

pub use resources::SkinChains;

use bevy::prelude::*;

use crate::core::{
    ConstraintGraph, Playground, anchor_movement_system, collision_avoidance_system, constraint_solving_system,
    fabrik_solving_system, limb_builder_system, update_constraint_graph, verlet_integration_system,
};
use tools::*;
use visuals::*;
use mesh::*;

/// Plugin for editor visualization and interaction systems.
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Selection>();
        app.init_resource::<ConstraintGraph>();
        app.init_resource::<SkinChains>();
        app.init_resource::<Playground>();
        app.init_resource::<EdgeCreationState>();
        app.init_resource::<CameraState>();

        app.add_systems(Startup, (spawn_playground_visual, spawn_skin_visual, spawn_limb_visual));
        app.add_systems(
            PostUpdate,
            (
                spawn_node_visuals,
                sync_node_visuals,
                spawn_constraint_visuals,
                sync_constraint_visuals,
                sync_playground_visual,
                update_skin_chains,
                sync_skin_visual,
                sync_limb_visual,
                update_node_visibility,
                update_debug_visibility,
                update_edge_visibility,
                update_eye_visibility,
            ),
        );
        app.add_systems(
            Update,
            (handle_camera_zoom, handle_camera_pan, handle_node_selection).chain(),
        );
        app.add_systems(
            Update,
            (
                handle_delete_selected,
                handle_add_node_tool,
                handle_add_edge_tool,
                cancel_edge_creation,
                render_constraint_preview,
            )
                .chain()
                .after(handle_node_selection),
        );
        app.add_systems(
            Update,
            (
                update_constraint_graph,
                limb_builder_system,
                anchor_movement_system,
                verlet_integration_system,
                constraint_solving_system,
                fabrik_solving_system,
                collision_avoidance_system,
                update_selection_visuals,
            )
                .chain()
                .after(render_constraint_preview),
        );
    }
}
