//! Core simulation module.

pub mod components;
pub mod constants;
pub mod resources;
pub mod serialization;
pub mod systems;
pub mod utils;

use bevy::prelude::*;

pub use components::{AnchorMovementMode, DistanceConstraint, Limb, LimbSet, Node, NodeType, Playground, ProceduralPathType};
pub use resources::ConstraintGraph;
pub use serialization::{SceneData, build_scene_data, export_to_file, import_from_file, spawn_scene_data, sync_pending_imports, PendingFileOp};
pub use systems::{
    anchor_movement_system, collision_avoidance_system, constraint_solving_system, update_constraint_graph,
    verlet_integration_system, fabrik_solving_system, limb_builder_system
};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingFileOp>();

        app.add_systems(Update, sync_pending_imports);
    }
}
