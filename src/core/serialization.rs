//! Scene serialization for import/export.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::components::{DistanceConstraint, Limb, LimbSet, Node};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

mod examples {
    include!(concat!(env!("OUT_DIR"), "/examples.rs"));
}

pub use examples::EXAMPLES;

#[derive(Resource, Default)]
pub struct PendingFileOp {
    pub import_data: Option<SceneData>,
    pub export_requested: bool,
    pub export_data: Option<SceneData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConstraintData {
    pub node_a: usize,
    pub node_b: usize,
    pub rest_length: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LimbData {
    pub joints: Vec<usize>,
    pub target: Vec2,
    pub lengths: Vec<f32>,
    pub iterations: usize,
    pub tolerance: f32,
    pub flip_bend: Vec<bool>,
    pub target_node: Option<usize>,
    pub max_reach: f32,
    pub target_direction_offset: f32,
    pub step_threshold: f32,
    pub step_speed: f32,
    pub step_height: f32,
    pub is_stepping: bool,
    pub step_start: Vec2,
    pub step_dest: Vec2,
    pub step_progress: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LimbSetData {
    pub body_node: usize,
    pub limbs: Vec<LimbData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SceneData {
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub constraints: Vec<ConstraintData>,
    #[serde(default)]
    pub limb_sets: Vec<LimbSetData>,
}

static IMPORT_CHANNEL: std::sync::LazyLock<
    (
        std::sync::mpsc::Sender<SceneData>,
        std::sync::Mutex<std::sync::mpsc::Receiver<SceneData>>,
    ),
> = std::sync::LazyLock::new(|| {
    let (tx, rx) = std::sync::mpsc::channel();
    (tx, std::sync::Mutex::new(rx))
});

pub fn build_scene_data(
    nodes: &Query<(Entity, &mut Node)>,
    constraints: &Query<(Entity, &DistanceConstraint)>,
    limb_sets: &Query<(Entity, &mut LimbSet)>,
) -> SceneData {
    let (entity_list, node_list) = extract_node_data(nodes);
    let constraint_list = extract_constraint_data(constraints, &entity_list);
    let limb_set_list = extract_limb_set_data(limb_sets, &entity_list);

    SceneData {
        nodes: node_list,
        constraints: constraint_list,
        limb_sets: limb_set_list,
    }
}

pub fn spawn_scene_data(commands: &mut Commands, scene: &SceneData) -> Vec<Entity> {
    let node_entities = spawn_nodes(commands, scene);
    spawn_constraints(commands, scene, &node_entities);
    spawn_limb_sets(commands, scene, &node_entities);
    node_entities
}

pub fn export_to_file(scene: &SceneData) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Sway Scene", &["json"])
            .set_file_name("scene.json")
            .save_file()
        {
            match serialize_scene(scene) {
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
    }
    #[cfg(target_arch = "wasm32")]
    {
        let json = serialize_scene(scene).unwrap_or_default();
        spawn_local(async move {
            if let Some(file_handle) = rfd::AsyncFileDialog::new()
                .add_filter("Sway Scene", &["json"])
                .set_file_name("scene.json")
                .save_file()
                .await
            {
                let _ = file_handle.write(json.as_bytes()).await;
                info!("Scene exported to {}", file_handle.file_name());
            }
        });
    }
}

pub fn import_from_file() -> Option<SceneData> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = rfd::FileDialog::new()
            .add_filter("Sway Scene", &["json"])
            .pick_file()?;
        read_scene_from_file(&path)
    }
    #[cfg(target_arch = "wasm32")]
    {
        spawn_local(async {
            if let Some(file_handle) = rfd::AsyncFileDialog::new()
                .add_filter("Sway Scene", &["json"])
                .pick_file()
                .await
            {
                let data = file_handle.read().await;
                if let Ok(json) = std::str::from_utf8(&data) {
                    if let Some(scene) = deserialize_scene(json) {
                        let _ = IMPORT_CHANNEL.0.send(scene);
                    }
                }
            }
        });
        None
    }
}

/// System to poll for async file imports and update PendingFileOp.
pub fn sync_pending_imports(mut pending_op: ResMut<PendingFileOp>) {
    if let Ok(rx) = IMPORT_CHANNEL.1.lock() {
        while let Ok(scene) = rx.try_recv() {
            pending_op.import_data = Some(scene);
        }
    }
}

pub fn deserialize_scene(json: &str) -> Option<SceneData> {
    match serde_json::from_str::<SceneData>(json) {
        Ok(scene) => Some(scene),
        Err(e) => {
            error!("Failed to parse scene JSON: {e}");
            None
        }
    }
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

fn extract_limb_set_data(
    limb_sets: &Query<(Entity, &mut LimbSet)>,
    entity_list: &[Entity],
) -> Vec<LimbSetData> {
    limb_sets
        .iter()
        .map(|(entity, limb_set)| LimbSetData {
            body_node: find_entity_index(entity, entity_list),
            limbs: limb_set
                .limbs
                .iter()
                .map(|l| LimbData {
                    joints: l
                        .joints
                        .iter()
                        .map(|&e| find_entity_index(e, entity_list))
                        .collect(),
                    target: l.target,
                    lengths: l.lengths.clone(),
                    iterations: l.iterations,
                    tolerance: l.tolerance,
                    flip_bend: l.flip_bend.clone(),
                    target_node: l.target_node.map(|e| find_entity_index(e, entity_list)),
                    max_reach: l.max_reach,
                    target_direction_offset: l.target_direction_offset,
                    step_threshold: l.step_threshold,
                    step_speed: l.step_speed,
                    step_height: l.step_height,
                    is_stepping: l.is_stepping,
                    step_start: l.step_start,
                    step_dest: l.step_dest,
                    step_progress: l.step_progress,
                })
                .collect(),
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
        .map(|node| commands.spawn((Name::new("Node"), node.clone())).id())
        .collect()
}

fn spawn_constraints(commands: &mut Commands, scene: &SceneData, node_entities: &[Entity]) {
    for constraint in &scene.constraints {
        if constraint.node_a >= node_entities.len() || constraint.node_b >= node_entities.len() {
            continue;
        }

        let node_a = node_entities[constraint.node_a];
        let node_b = node_entities[constraint.node_b];
        commands.spawn((
            Name::new("Distance Constraint"),
            DistanceConstraint::new(node_a, node_b, constraint.rest_length),
        ));
    }
}

fn spawn_limb_sets(commands: &mut Commands, scene: &SceneData, node_entities: &[Entity]) {
    for limb_set_data in &scene.limb_sets {
        if limb_set_data.body_node >= node_entities.len() {
            continue;
        }

        let body_entity = node_entities[limb_set_data.body_node];
        let limbs = limb_set_data
            .limbs
            .iter()
            .map(|l| Limb {
                joints: l
                    .joints
                    .iter()
                    .filter_map(|&i| node_entities.get(i).copied())
                    .collect(),
                target: l.target,
                lengths: l.lengths.clone(),
                iterations: l.iterations,
                tolerance: l.tolerance,
                flip_bend: l.flip_bend.clone(),
                target_node: l.target_node.and_then(|i| node_entities.get(i).copied()),
                max_reach: l.max_reach,
                target_direction_offset: l.target_direction_offset,
                step_threshold: l.step_threshold,
                step_speed: l.step_speed,
                step_height: l.step_height,
                is_stepping: l.is_stepping,
                step_start: l.step_start,
                step_dest: l.step_dest,
                step_progress: l.step_progress,
            })
            .collect();

        commands
            .entity(body_entity)
            .insert(LimbSet { limbs });
    }
}

fn serialize_scene(scene: &SceneData) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(scene)
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
    let scene = deserialize_scene(json)?;
    info!("Scene imported from {}", path.display());
    Some(scene)
}
