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
    /// Creates a constraint with `rest_length` clamped to valid range.
    pub fn new(node_a: Entity, node_b: Entity, rest_length: f32) -> Self {
        Self {
            node_a,
            node_b,
            rest_length: rest_length.clamp(MIN_CONSTRAINT_DISTANCE, MAX_CONSTRAINT_DISTANCE),
        }
    }

    /// Returns true if this constraint connects the given entity.
    pub fn involves(&self, entity: Entity) -> bool {
        self.node_a == entity || self.node_b == entity
    }

    /// Returns the other entity in this constraint, if `entity` is one end.
    pub fn other(&self, entity: Entity) -> Option<Entity> {
        if self.node_a == entity {
            Some(self.node_b)
        } else if self.node_b == entity {
            Some(self.node_a)
        } else {
            None
        }
    }
}
