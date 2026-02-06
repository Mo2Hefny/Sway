//! Editor UI layout with sidebar, floating panel, and toolbar.

pub mod icons;
pub mod messages;
pub mod panels;
pub mod state;
pub mod systems;
pub mod theme;
pub mod widgets;

use bevy::prelude::*;

use icons::UiIcons;
use state::*;
use systems::*;
use widgets::{update_interaction_colors, ui_root, DropdownState};

/// Bevy plugin for editor UI.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DisplaySettings>();
        app.init_resource::<FloatingPanelState>();
        app.init_resource::<InspectorState>();
        app.init_resource::<EditorToolState>();
        app.init_resource::<UiVisibility>();
        app.init_resource::<UiIcons>();
        app.init_resource::<DropdownState>();
        app.init_resource::<PlaybackState>();
        app.init_resource::<InputState>();
        app.init_resource::<panels::TextInputFocus>();
        app.init_resource::<panels::FunctionInputFocus>();

        app.add_systems(Startup, (icons::load_icons, spawn_editor_ui).chain());

        // Update systems
        // Core UI systems
        app.add_systems(
            Update,
            (
                update_input_state,
                update_interaction_colors,
                handle_checkbox_clicks,
                update_checkbox_icons,
                handle_hamburger_click,
                update_panel_visibility,
                handle_import_click,
                handle_export_click,
            ),
        );

        // Inspector systems
        app.add_systems(
            Update,
            (
                handle_caret_click,
                update_inspector_panel_visibility,
                handle_page_icon_clicks,
                update_page_icon_styles,
                update_inspector_title,
                panels::update_inspector_content,
                panels::handle_inspector_checkbox_click,
                panels::handle_inspector_dropdown_click,
                panels::handle_dropdown_option_click,
            ),
        );

        // Text input systems
        app.add_systems(
            Update,
            (
                panels::handle_text_input_focus,
                panels::handle_text_input_drag,
                panels::handle_text_input_keyboard,
                panels::handle_function_input_focus,
                panels::handle_function_input_keyboard,
                panels::update_focused_input_style,
                panels::handle_click_outside,
            ),
        );

        // Tool bar and visibility systems
        app.add_systems(
            Update,
            (
                handle_tool_button_clicks,
                update_tool_button_styles,
                update_tool_bar_position,
                handle_ui_toggle_input,
                update_ui_visibility,
            ),
        );

        // Playback control systems
        app.add_systems(
            Update,
            (
                handle_playback_clicks,
            ),
        );
    }
}

fn spawn_editor_ui(mut commands: Commands, icons: Res<UiIcons>) {
    // Root UI container
    commands.spawn(ui_root("Editor UI")).with_children(|_parent| {});

    // Right sidebar inspector
    panels::spawn_right_sidebar(&mut commands, &icons);

    // Tool bar
    panels::spawn_tool_bar(&mut commands, &icons);

    // Left floating panel
    panels::spawn_floating_panel(&mut commands, &icons);

    // Bottom toolbar with playback controls
    panels::spawn_bottom_toolbar(&mut commands, &icons);

    // Instruction overlay
    panels::spawn_instruction_overlay(&mut commands);
}

pub mod prelude {
    pub use super::icons::UiIcons;
    pub use super::state::{
        DisplaySettings, FloatingPanelState, InspectorPage, InspectorState, 
        EditorToolState, PlaybackState, PlaybackMode, UiVisibility, InputState,
    };
    pub use super::theme::{interaction::{InteractionPalette, Active}, palette};
    pub use super::widgets::*;
    pub use super::UiPlugin;
}
