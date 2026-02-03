//! Bottom toolbar (playback controls) spawning.

use bevy::prelude::*;
use bevy::picking::prelude::Pickable;

use crate::ui::icons::UiIcons;
use crate::ui::theme::palette::*;
use crate::ui::theme::interaction::InteractionPalette;
use crate::ui::widgets::{BottomToolbar, PlaybackButton};

use super::px;

/// Spawns the bottom playback control toolbar.
pub fn spawn_bottom_toolbar(commands: &mut Commands, icons: &UiIcons) {
    commands.spawn((
        Name::new("Bottom Toolbar"),
        BottomToolbar,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(px(-60.0)),
            flex_direction: FlexDirection::Row,
            column_gap: px(6.0),
            padding: UiRect::all(px(6.0)),
            border_radius: BorderRadius::all(px(6.0)),
            ..default()
        },
        BackgroundColor(SURFACE),
    )).with_children(|parent| {
        spawn_playback_button(parent, "Play", icons.play.clone());
        spawn_playback_button(parent, "Pause", icons.pause.clone());
        spawn_playback_button(parent, "Stop", icons.stop.clone());
    });
}

/// Spawns a playback control button with icon.
fn spawn_playback_button(parent: &mut ChildSpawnerCommands, name: &str, icon: Handle<Image>) {
    parent.spawn((
        Name::new(format!("{} Button", name)),
        PlaybackButton,
        Button,
        Node {
            width: px(32.0),
            height: px(32.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BackgroundColor(SURFACE),
        InteractionPalette {
            none: SURFACE,
            hovered: SURFACE_HOVER,
            pressed: SURFACE_PRESSED,
            active: SURFACE_HOVER,
        },
    )).with_children(|btn| {
        btn.spawn((
            ImageNode::new(icon),
            Node {
                width: px(16.0),
                height: px(16.0),
                ..default()
            },
            Pickable::IGNORE,
        ));
    });
}
