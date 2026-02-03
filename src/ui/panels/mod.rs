//! Panel spawning functions.

mod bottom_toolbar;
mod floating_panel;
mod inspector;
mod instructions;
mod toolbar;

pub use bottom_toolbar::spawn_bottom_toolbar;
pub use floating_panel::spawn_floating_panel;
pub use inspector::spawn_right_sidebar;
pub use instructions::spawn_instruction_overlay;
pub use toolbar::spawn_tool_bar;

use bevy::prelude::Val;

/// Shared helper for pixel values.
pub(crate) fn px(val: f32) -> Val {
    Val::Px(val)
}
