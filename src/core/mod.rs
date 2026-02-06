//! Core simulation module.

pub mod components;
pub mod constants;
pub mod systems;

pub use components::{DistanceConstraint, Node, NodeType, Playground};
pub use systems::{
    constraint_solving_system, follow_mouse_system,
    boundary_collision_system, verlet_integration_system,
};
