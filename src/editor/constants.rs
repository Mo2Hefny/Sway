//! Editor constants for visual rendering and interaction.

use bevy::prelude::*;

// =============================================================================
// Node Visual Constants
// =============================================================================

pub const CIRCLE_THICKNESS: f32 = 3.0;
pub const CIRCLE_SEGMENTS: usize = 32;
pub const DEFAULT_NODE_RADIUS: f32 = 7.0;

// =============================================================================
// Color Visual Constants
// =============================================================================

pub const ANCHOR_NODE_COLOR: Color = Color::srgb(1.0, 0.4, 0.4);
pub const LEG_NODE_COLOR: Color = Color::srgb(0.4, 1.0, 0.4);
pub const NORMAL_NODE_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
pub const SELECTION_COLOR: Color = Color::srgb(0.3, 0.7, 1.0);
