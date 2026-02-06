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

// =============================================================================
// Contact Point Constants
// =============================================================================

pub const CONTACT_RADIUS: f32 = 1.5;
pub const CONTACT_COLOR: Color = Color::srgb(1.0, 0.2, 0.2);
pub const ANCHOR_CONTACT_COLOR: Color = Color::srgb(1.0, 0.85, 0.2);
pub const CONTACT_SEGMENTS: usize = 12;

// =============================================================================
// Look Vector Constants
// =============================================================================

pub const LOOK_VECTOR_LENGTH: f32 = 20.0;
pub const LOOK_VECTOR_THICKNESS: f32 = 1.5;
pub const LOOK_VECTOR_COLOR: Color = Color::srgb(1.0, 0.85, 0.3);

// =============================================================================
// Playground Color Constants
// =============================================================================

pub const PLAYGROUND_FILL_COLOR: Color = Color::srgba(0.10, 0.11, 0.12, 0.4);
pub const PLAYGROUND_OUTSIDE_COLOR: Color = Color::srgba(0.06, 0.065, 0.07, 0.8);
pub const PLAYGROUND_BORDER_COLOR: Color = Color::srgb(0.4, 0.45, 0.5);

// =============================================================================
// Constraint Color Constants
// =============================================================================

pub const CONSTRAINT_COLOR: Color = Color::srgba(0.5, 0.7, 0.9, 0.8);
pub const CONSTRAINT_PREVIEW_COLOR: Color = Color::srgba(0.5, 0.7, 0.9, 0.3);
pub const CONSTRAINT_LINE_THICKNESS: f32 = 2.0;
pub const CONSTRAINT_DASH_LENGTH: f32 = 6.0;
pub const CONSTRAINT_GAP_LENGTH: f32 = 4.0;
