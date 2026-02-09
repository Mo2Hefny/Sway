//! Distance constraint linking two simulation nodes.

use bevy::prelude::*;

use crate::core::constants::*;

/// A distance constraint entity linking two nodes.
#[derive(Component, Clone, Debug, Reflect)]
pub struct DistanceConstraint {
    pub node_a: Entity,
    pub node_b: Entity,
    pub rest_length: f32,
}

impl DistanceConstraint {
    pub fn new(node_a: Entity, node_b: Entity, rest_length: f32) -> Self {
        Self {
            node_a,
            node_b,
            rest_length: Self::clamp_rest_length(rest_length),
        }
    }

    pub fn involves(&self, entity: Entity) -> bool {
        self.node_a == entity || self.node_b == entity
    }

    pub fn other(&self, entity: Entity) -> Option<Entity> {
        if self.node_a == entity {
            Some(self.node_b)
        } else if self.node_b == entity {
            Some(self.node_a)
        } else {
            None
        }
    }
    
    fn clamp_rest_length(length: f32) -> f32 {
        length.clamp(MIN_CONSTRAINT_DISTANCE, MAX_CONSTRAINT_DISTANCE)
    }
}
