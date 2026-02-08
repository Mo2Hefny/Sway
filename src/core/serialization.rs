//! Scene serialization for import/export.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::components::{DistanceConstraint, Node, NodeType};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConstraintData {
    pub node_a: usize,
    pub node_b: usize,
    pub rest_length: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SceneData {
    pub nodes: Vec<Node>,
    pub constraints: Vec<ConstraintData>,
}

pub fn build_scene_data(
    nodes: &Query<(Entity, &mut Node)>,
    constraints: &Query<(Entity, &DistanceConstraint)>,
) -> SceneData {
    let mut entity_list: Vec<Entity> = Vec::new();
    let mut node_list: Vec<Node> = Vec::new();

    for (entity, node) in nodes.iter() {
        entity_list.push(entity);
        node_list.push(node.clone());
    }

    let entity_to_index = |e: Entity| -> usize {
        entity_list.iter().position(|&ent| ent == e).unwrap_or(0)
    };

    let constraint_list: Vec<ConstraintData> = constraints
        .iter()
        .map(|(_, c)| ConstraintData {
            node_a: entity_to_index(c.node_a),
            node_b: entity_to_index(c.node_b),
            rest_length: c.rest_length,
        })
        .collect();

    SceneData {
        nodes: node_list,
        constraints: constraint_list,
    }
}

pub fn spawn_scene_data(commands: &mut Commands, scene: &SceneData) -> Vec<Entity> {
    let mut node_entities: Vec<Entity> = Vec::new();

    for node in &scene.nodes {
        let entity = commands
            .spawn((
                Name::new("Node"),
                node.clone(),
            ))
            .id();
        node_entities.push(entity);
    }

    for constraint in &scene.constraints {
        let node_a = node_entities[constraint.node_a];
        let node_b = node_entities[constraint.node_b];
        commands.spawn((
            Name::new("Distance Constraint"),
            DistanceConstraint::new(node_a, node_b, constraint.rest_length),
        ));
    }

    node_entities
}

pub fn export_to_file(scene: &SceneData) {
    let path = rfd::FileDialog::new()
        .add_filter("Sway Scene", &["json"])
        .set_file_name("scene.json")
        .save_file();

    let Some(path) = path else { return };

    match serde_json::to_string_pretty(scene) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                error!("Failed to write scene file: {e}");
            } else {
                info!("Scene exported to {}", path.display());
            }
        }
        Err(e) => error!("Failed to serialize scene: {e}"),
    }
}

pub fn import_from_file() -> Option<SceneData> {
    let path = rfd::FileDialog::new()
        .add_filter("Sway Scene", &["json"])
        .pick_file()?;

    match std::fs::read_to_string(&path) {
        Ok(json) => match serde_json::from_str::<SceneData>(&json) {
            Ok(scene) => {
                info!("Scene imported from {}", path.display());
                Some(scene)
            }
            Err(e) => {
                error!("Failed to parse scene file: {e}");
                None
            }
        },
        Err(e) => {
            error!("Failed to read scene file: {e}");
            None
        }
    }
}
