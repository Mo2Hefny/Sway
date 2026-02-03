//! Sway - Procedural Animation Engine

pub mod ui;

use bevy::{
    diagnostic::{FrameCount, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    asset::AssetMetaCheck,
};

/// Main Sway application plugin.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(ui::theme::palette::BACKGROUND));

        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Sway | Procedural Animation Engine".to_string(),
                        fit_canvas_to_parent: true,
                        visible: false,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
        ));

        // Add Sway plugins in *dependency order*
        app.add_plugins((
            ui::UiPlugin,
        ));

        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, make_visible);
    }
}

/// Spawn the main 2D camera.
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera2d::default(),
    ));
}

/// Hide the window at startup; show it after a few frames to avoid displaying a blank screen.
fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 3 {
        window.single_mut().unwrap().visible = true;
    }
}

pub mod prelude {
    #![allow(unused_imports)]
    pub use crate::ui::prelude::*;
    pub use crate::AppPlugin;
}
