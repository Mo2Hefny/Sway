#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

//! Application entry point.

use bevy::prelude::*;
use sway::AppPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}
