//! Editor constants for visual rendering and interaction.

use bevy::prelude::*;

// =============================================================================
// Node Visual Constants
// =============================================================================

pub const CIRCLE_THICKNESS: f32 = 4.0;
pub const CIRCLE_SEGMENTS: usize = 32;
pub const DEFAULT_NODE_RADIUS: f32 = 10.0;

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
// Eye Constants
// =============================================================================

pub const EYE_RADIUS: f32 = 3.5;
pub const EYE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
pub const EYE_PUPIL_RADIUS: f32 = 1.5;
pub const EYE_PUPIL_COLOR: Color = Color::srgb(0.05, 0.05, 0.05);
pub const EYE_DISTANCE_RATIO: f32 = 0.5;

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

// =============================================================================
// Skin Constants
// =============================================================================

pub const SKIN_COLOR: Color = Color::srgba(0.45, 0.65, 0.85, 0.5);
pub const SKIN_COLOR_OPAQUE: Color = Color::srgba(0.45, 0.65, 0.85, 1.0);
pub const OUTLINE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
pub const OUTLINE_THICKNESS: f32 = 2.0;
pub const SPLINE_SAMPLES: usize = 8;

// =============================================================================
// Motion Constants
// =============================================================================

pub const TARGET_MARKER_SIZE: f32 = 10.0;
pub const TARGET_MARKER_THICKNESS: f32 = 1.0;
pub const TARGET_MARKER_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.4);
pub const DIRECTION_VECTOR_LENGTH: f32 = 40.0;
pub const DIRECTION_VECTOR_THICKNESS: f32 = 2.0;
pub const DIRECTION_VECTOR_COLOR: Color = Color::srgba(0.3, 0.8, 1.0, 0.6);
