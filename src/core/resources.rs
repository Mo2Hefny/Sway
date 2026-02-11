use bevy::prelude::*;
use std::collections::HashMap;

use crate::core::components::DistanceConstraint;

#[derive(Resource, Default)]
pub struct ConstraintGraph {
    pub adjacency: HashMap<Entity, Vec<(Entity, f32)>>,
    pub node_groups: HashMap<Entity, u32>,
}

impl ConstraintGraph {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            node_groups: HashMap::new(),
        }
    }

    pub fn rebuild(&mut self, constraints: &Vec<DistanceConstraint>) {
        self.adjacency.clear();
        self.node_groups.clear();

        for constraint in constraints {
            self.adjacency
                .entry(constraint.node_a)
                .or_insert_with(Vec::new)
                .push((constraint.node_b, constraint.rest_length));
            self.adjacency
                .entry(constraint.node_b)
                .or_insert_with(Vec::new)
                .push((constraint.node_a, constraint.rest_length));
        }

        let mut stack = Vec::new();
        let mut group_id = 0;

        for node in self.adjacency.keys() {
            if !self.node_groups.contains_key(node) {
                stack.push(*node);
                self.node_groups.insert(*node, group_id);

                while let Some(current_node) = stack.pop() {
                    if let Some(neighbors) = self.adjacency.get(&current_node) {
                        for &(neighbor, _) in neighbors {
                            if !self.node_groups.contains_key(&neighbor) {
                                self.node_groups.insert(neighbor, group_id);
                                stack.push(neighbor);
                            }
                        }
                    }
                }
                group_id += 1;
            }
        }
    }

    pub fn get_group(&self, node: Entity) -> Option<u32> {
        self.node_groups.get(&node).copied()
    }
}
