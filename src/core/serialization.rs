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
    let (entity_list, node_list) = extract_node_data(nodes);
    let constraint_list = extract_constraint_data(constraints, &entity_list);

    SceneData {
        nodes: node_list,
        constraints: constraint_list,
    }
}

pub fn spawn_scene_data(commands: &mut Commands, scene: &SceneData) -> Vec<Entity> {
    let node_entities = spawn_nodes(commands, scene);
    spawn_constraints(commands, scene, &node_entities);
    node_entities
}

pub fn export_to_file(scene: &SceneData) {
    if let Some(path) = request_save_path() {
        match serialize_scene(scene) {
            Ok(json) => write_scene_to_file(&path, &json),
            Err(e) => error!("Failed to serialize scene: {e}"),
        }
    }
}

pub fn import_from_file() -> Option<SceneData> {
    let path = request_load_path()?;
    read_scene_from_file(&path)
}

// =============================================================================
// Private Methods
// =============================================================================

fn extract_node_data(nodes: &Query<(Entity, &mut Node)>) -> (Vec<Entity>, Vec<Node>) {
    let mut entity_list = Vec::new();
    let mut node_list = Vec::new();

    for (entity, node) in nodes.iter() {
        entity_list.push(entity);
        node_list.push(node.clone());
    }

    (entity_list, node_list)
}

fn extract_constraint_data(
    constraints: &Query<(Entity, &DistanceConstraint)>,
    entity_list: &[Entity],
) -> Vec<ConstraintData> {
    constraints
        .iter()
        .map(|(_, c)| ConstraintData {
            node_a: find_entity_index(c.node_a, entity_list),
            node_b: find_entity_index(c.node_b, entity_list),
            rest_length: c.rest_length,
        })
        .collect()
}

fn find_entity_index(entity: Entity, entity_list: &[Entity]) -> usize {
    entity_list.iter().position(|&e| e == entity).unwrap_or(0)
}

fn spawn_nodes(commands: &mut Commands, scene: &SceneData) -> Vec<Entity> {
    scene
        .nodes
        .iter()
        .map(|node| {
            commands
                .spawn((Name::new("Node"), node.clone()))
                .id()
        })
        .collect()
}

fn spawn_constraints(commands: &mut Commands, scene: &SceneData, node_entities: &[Entity]) {
    for constraint in &scene.constraints {
        let node_a = node_entities[constraint.node_a];
        let node_b = node_entities[constraint.node_b];
        commands.spawn((
            Name::new("Distance Constraint"),
            DistanceConstraint::new(node_a, node_b, constraint.rest_length),
        ));
    }
}

fn request_save_path() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Sway Scene", &["json"])
        .set_file_name("scene.json")
        .save_file()
}

fn request_load_path() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Sway Scene", &["json"])
        .pick_file()
}

fn serialize_scene(scene: &SceneData) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(scene)
}

fn write_scene_to_file(path: &std::path::Path, json: &str) {
    if let Err(e) = std::fs::write(path, json) {
        error!("Failed to write scene file: {e}");
    } else {
        info!("Scene exported to {}", path.display());
    }
}

fn read_scene_from_file(path: &std::path::Path) -> Option<SceneData> {
    match std::fs::read_to_string(path) {
        Ok(json) => parse_scene(&json, path),
        Err(e) => {
            error!("Failed to read scene file: {e}");
            None
        }
    }
}

fn parse_scene(json: &str, path: &std::path::Path) -> Option<SceneData> {
    match serde_json::from_str::<SceneData>(json) {
        Ok(scene) => {
            info!("Scene imported from {}", path.display());
            Some(scene)
        }
        Err(e) => {
            error!("Failed to parse scene file: {e}");
            None
        }
    }
}
