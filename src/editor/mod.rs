//! Editor systems for node visualization and interaction.

pub mod components;
pub mod constants;
pub mod tools;
pub mod visuals;

use bevy::prelude::*;

use crate::core::{
    anchor_movement_system, constraint_solving_system,
    verlet_integration_system, boundary_collision_system,
    save_frame_start_system, Playground,
};
use tools::*;
use visuals::*;

/// Plugin for editor visualization and interaction systems.
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Selection>();
        app.init_resource::<Playground>();
        app.init_resource::<EdgeCreationState>();
        
        app.add_systems(Startup, (spawn_playground_visual, spawn_skin_visual));
        app.add_systems(PostUpdate, (
            spawn_node_visuals,
            sync_node_visuals,
            spawn_constraint_visuals,
            sync_constraint_visuals,
            sync_playground_visual,
            sync_skin_visual,
            update_node_visibility,
            update_debug_visibility,
            update_edge_visibility,
            update_eye_visibility,
        ));
        app.add_systems(
            Update,
            (
                sync_playground_to_window,
                (handle_node_selection, handle_delete_selected, handle_add_node_tool, handle_add_edge_tool),
                cancel_edge_creation,
                render_constraint_preview,
                save_frame_start_system,
                anchor_movement_system,
                boundary_collision_system,
                verlet_integration_system,
                constraint_solving_system,
                update_selection_visuals,
            )
                .chain(),
        );
    }
}
