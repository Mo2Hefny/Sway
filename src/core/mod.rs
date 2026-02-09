//! Core simulation module.

pub mod components;
pub mod constants;
pub mod serialization;
pub mod systems;
pub mod utils;

pub use components::{DistanceConstraint, Node, NodeType, Playground, AnchorMovementMode, ProceduralPathType};
pub use serialization::{SceneData, build_scene_data, spawn_scene_data, export_to_file, import_from_file};
pub use systems::{
    anchor_movement_system, collision_avoidance_system, constraint_solving_system,
    boundary_collision_system, verlet_integration_system,
};
