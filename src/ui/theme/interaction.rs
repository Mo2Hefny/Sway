//! Interaction color states.

use bevy::prelude::*;
use super::palette::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub active: Color,
}

impl Default for InteractionPalette {
    fn default() -> Self {
        Self {
            none: SURFACE,
            hovered: SURFACE_HOVER,
            pressed: SURFACE_PRESSED,
            active: SURFACE_HOVER,
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct Active;
