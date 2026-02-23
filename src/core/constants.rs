//! Core simulation constants.

// =============================================================================
// Playground Constants
// =============================================================================

pub const CELL_SIZE: f32 = 20.0;
pub const BORDER_MARGIN: f32 = 10.0;
pub const STROKE_WIDTH: f32 = 2.0;
pub const IMPACT_DAMPING: f32 = 0.5;

// =============================================================================
// Physics Constants
// =============================================================================

pub const FOLLOW_SPEED: f32 = 5.0;
pub const AIR_DAMPING: f32 = 0.98;

// =============================================================================
// Constraint Constants
// =============================================================================

pub const MIN_CONSTRAINT_DISTANCE: f32 = 10.0;
pub const MAX_CONSTRAINT_DISTANCE: f32 = 200.0;
pub const CONSTRAINT_ITERATIONS: usize = 4;

// =============================================================================
// Angle Constraint Constants
// =============================================================================

pub const ANGLE_CONSTRAINT: f32 = std::f32::consts::FRAC_PI_8;

// =============================================================================
// Steering Constants
// =============================================================================

pub const STEERING_THRESHOLD: f32 = 0.001;
pub const STEERING_STRENGTH: f32 = 0.15;
pub const TARGET_SMOOTHING: f32 = 0.08;
pub const HORIZONTAL_WANDER_BIAS: f32 = 0.05;

// =============================================================================
// Collision Avoidance Constants
// =============================================================================

pub const MIN_TARGET_DISTANCE: f32 = 0.1;
pub const MIN_COLLISION_DISTANCE: f32 = 0.01;
pub const STUCK_DETECTION_THRESHOLD: f32 = 5.0;
pub const NODE_AVOIDANCE_BUFFER: f32 = 40.0;
pub const LOOKAHEAD_WINDOW: f32 = 0.6;
pub const STEERING_RESPONSIVENESS: f32 = 4.0;
pub const STUCK_TURN_SPEED: f32 = 2.0;
pub const BOUNDARY_AVOIDANCE_RANGE: f32 = 60.0;
pub const BOUNDARY_REFLECTION_FACTOR: f32 = 1.2;
