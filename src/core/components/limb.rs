//! Limb component for FABRIK Inverse Kinematics.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Default, Reflect, Serialize, Deserialize)]
pub struct LimbSet {
    pub limbs: Vec<Limb>,
}

#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct Limb {
    pub joints: Vec<Entity>,
    pub target: Vec2,
    pub lengths: Vec<f32>,
    pub iterations: usize,
    pub tolerance: f32,
    pub flip_bend: Vec<bool>,
    pub target_node: Option<Entity>,
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

impl Default for Limb {
    fn default() -> Self {
        Self {
            joints: Vec::new(),
            target: Vec2::ZERO,
            lengths: Vec::new(),
            iterations: 10,
            tolerance: 0.1,
            flip_bend: Vec::new(),
            target_node: None,
            max_reach: 100.0,
            target_direction_offset: 0.0,
            step_threshold: 20.0,
            step_speed: 5.0,
            step_height: 10.0,
            is_stepping: false,
            step_start: Vec2::ZERO,
            step_dest: Vec2::ZERO,
            step_progress: 0.0,
        }
    }
}
