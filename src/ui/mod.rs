//! Editor UI layout implemented with bevy_egui.

pub mod icons;
pub mod messages;
pub mod panels;
pub mod state;
pub mod systems;
pub mod theme;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use icons::{UiIcons, EguiIconTextures};
use state::*;
use systems::{
    apply_editor_actions, editor_ui_system, toggle_playback_control, toggle_ui_visibility,
};

/// Bevy plugin for editor UI (bevy_egui).
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DisplaySettings>();
        app.init_resource::<FloatingPanelState>();
        app.init_resource::<InspectorState>();
        app.init_resource::<EditorToolState>();
        app.init_resource::<UiVisibility>();
        app.init_resource::<UiIcons>();
        app.init_resource::<PlaybackState>();
        app.init_resource::<InputState>();
        app.init_resource::<EguiIconTextures>();
        app.init_resource::<ImportRequested>();
        app.init_resource::<PendingConstraintActions>();

        app.add_systems(Startup, icons::load_icons);

        app.add_systems(
            Update,
            (toggle_ui_visibility, toggle_playback_control, apply_editor_actions),
        );
        // Run UI in egui pass so ctx.available_rect() is valid (after Context::run()).
        app.add_systems(EguiPrimaryContextPass, editor_ui_system);
    }
}

pub mod prelude {
    pub use super::UiPlugin;
    pub use super::icons::UiIcons;
    pub use super::state::{
        DisplaySettings, EditorToolState, FloatingPanelState, InputState, InspectorPage, InspectorState, PlaybackMode,
        PlaybackState, UiVisibility,
    };
    pub use super::theme::palette;
}
