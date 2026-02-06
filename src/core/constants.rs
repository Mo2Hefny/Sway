//! Core simulation constants.

use bevy::prelude::*;

// =============================================================================
// Playground Constants
// =============================================================================

pub const BORDER_MARGIN: f32 = 10.0;
pub const STROKE_WIDTH: f32 = 2.0;
pub const IMPACT_DAMPING: f32 = 0.5;
pub const PLAYGROUND_FILL_COLOR: Color = Color::srgba(0.10, 0.11, 0.12, 0.4);
pub const PLAYGROUND_OUTSIDE_COLOR: Color = Color::srgba(0.06, 0.065, 0.07, 0.8);
pub const PLAYGROUND_BORDER_COLOR: Color = Color::srgb(0.4, 0.45, 0.5);

// =============================================================================
// Physics Constants
// =============================================================================

pub const FOLLOW_SPEED: f32 = 5.0;
