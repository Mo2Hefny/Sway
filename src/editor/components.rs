//! Editor marker components shared across visual and interaction systems.

use bevy::prelude::*;

// =============================================================================
// Node Visual Components
// =============================================================================

/// Marker component for node visual entities.
#[derive(Component, Clone, Debug, Reflect)]
pub struct NodeVisual;

/// Links a visual entity back to its parent node.
#[derive(Component, Clone, Debug, Reflect)]
pub struct NodeVisualOf(pub Entity);

/// Marker for skin-contact point visuals.
#[derive(Component, Clone, Debug, Reflect)]
pub struct ContactPoint;

/// Marker for the look-direction line visual.
#[derive(Component, Clone, Debug, Reflect)]
pub struct LookVector;

// =============================================================================
// Selection Components
// =============================================================================

/// Marker for entities that are currently selected.
#[derive(Component, Clone, Debug, Reflect)]
pub struct Selected;

/// Marker to make nodes selectable via picking.
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct Selectable;

// =============================================================================
// Constraint Visual Components
// =============================================================================

/// Marker for constraint line visual entities.
#[derive(Component, Debug)]
pub struct ConstraintVisual;

/// Links a visual back to its constraint entity.
#[derive(Component, Debug)]
pub struct ConstraintVisualOf(pub Entity);

/// Marker for the dashed preview line entity.
#[derive(Component, Debug)]
pub struct ConstraintPreview;

// =============================================================================
// Playground Visual Components
// =============================================================================

/// Marker for the outside fill of the playground.
#[derive(Component, Debug)]
pub struct PlaygroundOutside;

/// Marker for the border stroke.
#[derive(Component, Debug)]
pub struct PlaygroundBorder;

/// Marker for the inside fill of the playground.
#[derive(Component, Debug)]
pub struct PlaygroundFill;
