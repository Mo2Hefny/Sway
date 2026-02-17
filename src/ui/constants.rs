use std::ops::RangeInclusive;

// =============================================================================
// UI Layout Constants
// =============================================================================

pub const PANEL_CORNER_RADIUS: f32 = 6.0;
pub const PANEL_INNER_MARGIN: f32 = 12.0;
pub const PANEL_ITEM_SPACING: f32 = 6.0;
pub const PANEL_SECTION_SPACING: f32 = 12.0;
pub const PANEL_TITLE_SPACING: f32 = 8.0;
pub const PANEL_SEPARATOR_WIDTH: f32 = 1.0;

pub const INSPECTOR_WIDTH: f32 = 280.0;
pub const ICON_BAR_WIDTH: f32 = 48.0;
pub const ICON_BAR_INNER_MARGIN: f32 = 4.0;
pub const TOOL_BAR_WIDTH: f32 = 48.0;
pub const TOOL_BAR_INNER_MARGIN: f32 = 4.0;

pub const FLOATING_PANEL_WIDTH: f32 = 220.0;
pub const FLOATING_PANEL_WIDTH_COLLAPSED: f32 = 48.0;
pub const FLOATING_PANEL_INNER_MARGIN: f32 = 8.0;

pub const BOTTOM_TOOLBAR_MARGIN: f32 = 16.0;
pub const BOTTOM_TOOLBAR_INNER_MARGIN: f32 = 6.0;
pub const BOTTOM_TOOLBAR_ITEM_SPACING: f32 = 4.0;
pub const BOTTOM_TOOLBAR_BTN_SIZE: f32 = 28.0;
pub const BOTTOM_TOOLBAR_ICON_SIZE: f32 = 14.0;

pub const TOOL_BTN_SIZE: f32 = 36.0;
pub const TOOL_ICON_SIZE: f32 = 20.0;

pub const ICON_BAR_BTN_SIZE: f32 = 40.0;
pub const ICON_BAR_ICON_SIZE: f32 = 24.0;

pub const HAMBURGER_BTN_SIZE: f32 = 28.0;
pub const HAMBURGER_ICON_SIZE: f32 = 16.0;

pub const ACTION_BTN_HEIGHT: f32 = 32.0;

// =============================================================================
// Slider Ranges & Value Tweaks
// =============================================================================

pub const ANGLE_LIMIT_RANGE: RangeInclusive<f32> = 0.0..=180.0;

pub const LIMB_MAX_REACH_RANGE: RangeInclusive<f32> = 10.0..=500.0;
pub const LIMB_ANGLE_OFFSET_RANGE: RangeInclusive<f32> = -180.0..=180.0;
pub const LIMB_STEP_THRESHOLD_RANGE: RangeInclusive<f32> = 0.0..=200.0;
pub const LIMB_STEP_SPEED_RANGE: RangeInclusive<f32> = 0.1..=20.0;
pub const LIMB_STEP_HEIGHT_RANGE: RangeInclusive<f32> = 0.0..=100.0;

pub const MOVEMENT_SPEED_RANGE: RangeInclusive<f32> = 1.0..=50.0;
pub const PATH_AMPLITUDE_RANGE: RangeInclusive<f32> = 10.0..=200.0;

pub const COLLISION_DAMPING_RANGE: RangeInclusive<f32> = 0.0..=1.0;

pub const NODE_RADIUS_RANGE: RangeInclusive<f32> = 4.0..=50.0;

pub const PLAYGROUND_HALF_SIZE_RANGE: RangeInclusive<f32> = 400.0..=2000.0;

// =============================================================================
// Misc UI Settings
// =============================================================================

pub const HINT_ANCHOR_OFFSET: (f32, f32) = (16.0, -16.0);
pub const HINT_ITEM_SPACING: (f32, f32) = (0.0, 2.0);
pub const HINT_COLOR: (f32, f32, f32, f32) = (0.6, 0.6, 0.6, 0.5);

pub const WIDGET_DRAG_SPEED: f32 = 1.0;
pub const WIDGET_DRAG_SPEED_FINE: f32 = 0.1;
