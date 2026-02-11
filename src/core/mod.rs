//! Core simulation module.

pub mod components;
pub mod constants;
pub mod resources;
pub mod serialization;
pub mod systems;
pub mod utils;

pub use components::{AnchorMovementMode, DistanceConstraint, Node, NodeType, Playground, ProceduralPathType};
pub use resources::ConstraintGraph;
pub use serialization::{SceneData, build_scene_data, export_to_file, import_from_file, spawn_scene_data};
pub use systems::{
    anchor_movement_system, collision_avoidance_system, constraint_solving_system, update_constraint_graph,
    verlet_integration_system,
};
