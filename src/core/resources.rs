use bevy::prelude::*;
use std::collections::HashMap;

use crate::core::components::DistanceConstraint;

#[derive(Resource, Default)]
pub struct ConstraintGraph {
    pub adjacency: HashMap<Entity, Vec<(Entity, f32)>>,
    pub node_groups: HashMap<Entity, u32>,
    pub group_min_nodes: HashMap<u32, Entity>,
}

impl ConstraintGraph {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            node_groups: HashMap::new(),
            group_min_nodes: HashMap::new(),
        }
    }

    pub fn rebuild(&mut self, constraints: &[&DistanceConstraint]) {
        self.adjacency.clear();
        self.node_groups.clear();
        self.group_min_nodes.clear();

        for constraint in constraints {
            self.adjacency
                .entry(constraint.node_a)
                .or_default()
                .push((constraint.node_b, constraint.rest_length));
            self.adjacency
                .entry(constraint.node_b)
                .or_default()
                .push((constraint.node_a, constraint.rest_length));
        }

        let mut stack = Vec::new();
        let mut group_id = 0;

        let mut sorted_nodes: Vec<_> = self.adjacency.keys().cloned().collect();
        sorted_nodes.sort();

        for node in sorted_nodes {
            if !self.node_groups.contains_key(&node) {
                stack.push(node);
                self.node_groups.insert(node, group_id);
                self.group_min_nodes.insert(group_id, node);

                while let Some(current_node) = stack.pop() {
                    if let Some(min_node) = self.group_min_nodes.get_mut(&group_id) {
                        if current_node.index().index() < min_node.index().index() {
                            *min_node = current_node;
                        }
                    }

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

    pub fn get_group_min(&self, group_id: u32) -> Option<Entity> {
        self.group_min_nodes.get(&group_id).copied()
    }

    pub fn get_group(&self, node: Entity) -> Option<u32> {
        self.node_groups.get(&node).copied()
    }
}
