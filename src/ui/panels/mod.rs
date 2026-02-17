//! UI panel modules for egui implementation.

mod floating_panel;
mod toolbar;
mod inspector;
mod playback;
mod hints;

pub use floating_panel::draw_floating_panel;
pub use toolbar::draw_toolbar;
pub use inspector::draw_inspector_panel;
pub use playback::draw_playback_toolbar;
pub use hints::draw_instruction_hints;
