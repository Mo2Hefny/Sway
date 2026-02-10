use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SkinChains {
    pub chains: Vec<Vec<(Entity, f32)>>,
}
