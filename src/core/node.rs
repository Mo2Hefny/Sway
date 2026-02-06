//! Node component definition.

use bevy::prelude::*;

/// Identifies the type of a simulation node.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum NodeType {
    Anchor,
    Leg,
    #[default]
    Normal,
}

impl NodeType {
    pub fn name(&self) -> &'static str {
        match self {
            NodeType::Anchor => "Anchor",
            NodeType::Leg => "Leg",
            NodeType::Normal => "Normal",
        }
    }
}

/// Simulation node representing a point in spine.
#[derive(Component, Clone, Debug, Reflect)]
#[require(Transform)]
pub struct Node {
    pub position: Vec2,
    pub acceleration: Vec2,
    pub radius: f32,
    pub follow_mouse: bool,
    pub node_type: NodeType,
    pub prev_position: Vec2,
    pub acc_fn_x: String,
    pub acc_fn_y: String,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            radius: 5.0,
            follow_mouse: false,
            node_type: NodeType::Normal,
            prev_position: Vec2::ZERO,
            acc_fn_x: String::new(),
            acc_fn_y: String::new(),
        }
    }
}

impl Node {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            prev_position: position,
            ..default()
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    pub fn with_follow_mouse(mut self, follow_mouse: bool) -> Self {
        self.follow_mouse = follow_mouse;
        self
    }

    pub fn verlet_step(&mut self, dt: f32) {
        let new_position = 2.0 * self.position - self.prev_position + self.acceleration * dt;
        self.prev_position = self.position;
        self.position = new_position;
    }
}
