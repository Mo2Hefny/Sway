//! Editor systems for node visualization and interaction.

pub mod constants;
pub mod node_visual;
pub mod playground_visual;
pub mod selection;
pub mod tools;

use bevy::prelude::*;

use crate::core::{
    follow_mouse_system, verlet_integration_system, boundary_collision_system, Playground,
};
use node_visual::*;
use playground_visual::*;
use selection::*;
use tools::*;

/// Plugin for editor visualization and interaction systems.
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Selection>();
        app.init_resource::<Playground>();
        
        app.add_systems(Startup, spawn_playground_visual);
        app.add_systems(PostUpdate, (
            spawn_node_visuals,
            sync_node_visuals,
            sync_playground_visual,
        ));
        app.add_systems(
            Update,
            (
                sync_playground_to_window,
                (handle_node_selection, handle_add_node_tool),
                follow_mouse_system,
                boundary_collision_system,
                verlet_integration_system,
                update_selection_visuals,
            )
                .chain(),
        );
    }
}
