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
        let height = 1000.0;
        let aspect = 16.0 / 9.0;
        let width = height * aspect;
        Self {
            half_size: Vec2::new(width, height),
            border_margin: BORDER_MARGIN,
            stroke_width: STROKE_WIDTH,
            impact_damping: IMPACT_DAMPING,
        }
    }
}

impl Playground {
    pub fn stroke_outer_min(&self) -> Vec2 {
        Self::calculate_outer_bound(-self.half_size, self.border_margin)
    }

    pub fn stroke_outer_max(&self) -> Vec2 {
        Self::calculate_outer_bound(self.half_size, -self.border_margin)
    }

    pub fn inner_min(&self) -> Vec2 {
        Self::calculate_inner_bound(-self.half_size, self.border_margin, self.stroke_width)
    }

    pub fn inner_max(&self) -> Vec2 {
        Self::calculate_inner_bound(self.half_size, self.border_margin, self.stroke_width)
    }

    fn calculate_outer_bound(half_size: Vec2, margin_offset: f32) -> Vec2 {
        half_size + Vec2::splat(margin_offset)
    }

    fn calculate_inner_bound(half_size: Vec2, margin: f32, stroke: f32) -> Vec2 {
        let offset = margin + stroke;
        if half_size.x < 0.0 || half_size.y < 0.0 {
            half_size + Vec2::splat(offset)
        } else {
            half_size - Vec2::splat(offset)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CellEntry {
    pub cell_x: i32,
    pub cell_y: i32,
    pub collider_index: usize,
}

impl PartialEq for CellEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cell_x == other.cell_x && self.cell_y == other.cell_y
    }
}

impl Eq for CellEntry {}

impl PartialOrd for CellEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CellEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cell_x
            .cmp(&other.cell_x)
            .then_with(|| self.cell_y.cmp(&other.cell_y))
    }
}
