//! Core simulation systems (physics, constraints, input).

pub mod anchor_movement;
pub mod collision_avoidance;
pub mod constraint_solver;
pub mod fabrik;
pub mod graph;
pub mod limb_builder;
pub mod physics;

pub use anchor_movement::anchor_movement_system;
pub use collision_avoidance::collision_avoidance_system;
pub use constraint_solver::constraint_solving_system;
pub use fabrik::fabrik_solving_system;
pub use graph::update_constraint_graph;
pub use limb_builder::limb_builder_system;
pub use physics::{boundary_collision_system, verlet_integration_system};
