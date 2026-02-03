//! Editor UI layout with sidebar, floating panel, and toolbar.

pub mod icons;
pub mod panels;
pub mod state;
pub mod systems;
pub mod theme;
pub mod widgets;

use bevy::prelude::*;

use icons::UiIcons;
use state::*;
use systems::*;
use widgets::*;

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

        app.add_systems(Startup, (icons::load_icons, spawn_editor_ui).chain());

        // Update systems
        app.add_systems(
            Update,
            (
                // Interaction colors
                update_interaction_colors,
                // Checkbox handling
                handle_checkbox_clicks,
                update_checkbox_icons,
                // Hamburger toggle handling
                handle_hamburger_click,
                update_panel_visibility,
                // Import/Export handling
                handle_import_click,
                handle_export_click,
                // Inspector systems
                handle_caret_click,
                update_inspector_panel_visibility,
                handle_page_icon_clicks,
                update_page_icon_styles,
                update_inspector_title,
                // Tool bar systems
                handle_tool_button_clicks,
                update_tool_button_styles,
                update_tool_bar_position,
                // UI visibility toggle
                handle_ui_toggle_input,
                update_ui_visibility,
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
        EditorToolState, UiVisibility,
    };
    pub use super::theme::{interaction::{InteractionPalette, Active}, palette};
    pub use super::widgets::*;
    pub use super::UiPlugin;
}
