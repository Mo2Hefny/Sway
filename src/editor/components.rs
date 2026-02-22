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

/// Marker for eye visuals on anchor/head nodes.
#[derive(Component, Clone, Debug, Reflect)]
pub struct EyeVisual;

// Marker for the target position marker.
#[derive(Component, Debug)]
pub struct TargetMarker;

// Marker for the direction vector.
#[derive(Component, Debug)]
pub struct DirectionVector;

/// Links a target marker or direction vector to a specific limb in a LimbSet.
#[derive(Component, Debug)]
pub struct LimbIndex(pub usize);

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

// =============================================================================
// Skin Visual Components
// =============================================================================

/// Marker for the skin mesh entity that renders the body outline.
#[derive(Component, Debug)]
pub struct SkinMesh;

/// Marker for the skin outline stroke.
#[derive(Component, Debug)]
pub struct SkinOutline;

/// Tracks which chain group a skin entity belongs to.
#[derive(Component, Debug)]
pub struct SkinGroupIndex(pub usize);

/// Marker for limb fill mesh entities (rendered separately from body skin).
#[derive(Component, Debug)]
pub struct LimbMesh;

/// Marker for limb outline mesh entities.
#[derive(Component, Debug)]
pub struct LimbOutline;
