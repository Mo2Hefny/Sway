//! Playground boundary resource.

use bevy::prelude::*;

use crate::core::constants::*;

/// Defines the rectangular playground area in world space.
#[derive(Resource, Clone, Debug, Reflect)]
pub struct Playground {
    pub half_size: Vec2,
    pub border_margin: f32,
    pub stroke_width: f32,
    pub impact_damping: f32,
}

impl Default for Playground {
    fn default() -> Self {
        Self {
            half_size: Vec2::ZERO,
            border_margin: BORDER_MARGIN,
            stroke_width: STROKE_WIDTH,
            impact_damping: IMPACT_DAMPING,
        }
    }
}

impl Playground {
    /// Outer edge of the border stroke (inside the margin).
    pub fn stroke_outer_min(&self) -> Vec2 {
        -self.half_size + Vec2::splat(self.border_margin)
    }

    /// Outer edge of the border stroke (inside the margin).
    pub fn stroke_outer_max(&self) -> Vec2 {
        self.half_size - Vec2::splat(self.border_margin)
    }

    /// Inner edge of the border stroke — the collision surface.
    pub fn inner_min(&self) -> Vec2 {
        -self.half_size + Vec2::splat(self.border_margin + self.stroke_width)
    }

    /// Inner edge of the border stroke — the collision surface.
    pub fn inner_max(&self) -> Vec2 {
        self.half_size - Vec2::splat(self.border_margin + self.stroke_width)
    }
}
