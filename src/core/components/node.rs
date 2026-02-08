//! Node component definition.

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
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

#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
#[serde(default)]
#[require(Transform)]
pub struct Node {
    pub position: Vec2,
    pub acceleration: Vec2,
    pub radius: f32,
    pub node_type: NodeType,
    pub prev_position: Vec2,
    pub frame_start_position: Vec2,
    pub chain_angle: f32,
    pub follow_target: bool,
    pub movement_speed: f32,
    pub angle_constraint: f32,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            radius: 5.0,
            node_type: NodeType::Normal,
            prev_position: Vec2::ZERO,
            frame_start_position: Vec2::ZERO,
            chain_angle: std::f32::consts::PI,
            follow_target: false,
            movement_speed: 12.0,
            angle_constraint: std::f32::consts::FRAC_PI_4,
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

    pub fn verlet_step(&mut self, dt: f32) {
        let new_position = 2.0 * self.position - self.prev_position + self.acceleration * dt;
        self.prev_position = self.position;
        self.position = new_position;
    }

    pub fn save_frame_start(&mut self) {
        self.frame_start_position = self.position;
    }
}
