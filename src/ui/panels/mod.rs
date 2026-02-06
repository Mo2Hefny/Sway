//! Panel spawning functions.

mod bottom_toolbar;
mod floating_panel;
mod inspector;
mod inspector_content;
mod instructions;
mod toolbar;

pub use bottom_toolbar::spawn_bottom_toolbar;
pub use floating_panel::spawn_floating_panel;
pub use inspector::spawn_right_sidebar;
pub use inspector_content::{
    update_inspector_content,
    handle_inspector_checkbox_click,
    handle_inspector_dropdown_click,
    handle_dropdown_option_click,
    handle_text_input_focus,
    handle_text_input_keyboard,
    handle_text_input_drag,
    handle_function_input_focus,
    handle_function_input_keyboard,
    update_focused_input_style,
    handle_click_outside,
    TextInputFocus,
    FunctionInputFocus,
    InspectorFieldKind,
};
pub use instructions::spawn_instruction_overlay;
pub use toolbar::spawn_tool_bar;

// Re-export widget types used by ui/mod.rs
pub use crate::ui::widgets::DropdownState;

use bevy::prelude::Val;

/// Shared helper for pixel values.
pub(crate) fn px(val: f32) -> Val {
    Val::Px(val)
}
