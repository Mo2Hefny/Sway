//! Instruction overlay spawning.

use bevy::picking::prelude::Pickable;
use bevy::prelude::*;

use crate::ui::messages::*;
use crate::ui::widgets::{InstructionColumn, InstructionLine, InstructionOverlayRoot};

use super::px;

/// Spawns the keyboard shortcuts instruction overlay.
pub fn spawn_instruction_overlay(commands: &mut Commands) {
    let hint_color = Color::srgba(0.6, 0.6, 0.6, 0.5);

    commands
        .spawn((
            Name::new("Instruction Overlay Root"),
            InstructionOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(16.0),
                bottom: px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(2.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Instruction Column"),
                    InstructionColumn,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(2.0),
                        ..default()
                    },
                ))
                .with_children(|column| {
                    spawn_instruction_line(column, HINT_SELECT, hint_color);
                    spawn_instruction_line(column, HINT_ADD_NODE, hint_color);
                    spawn_instruction_line(column, HINT_ADD_EDGE, hint_color);
                    spawn_instruction_line(column, HINT_MOVE, hint_color);
                    spawn_instruction_line(column, HINT_TOGGLE_UI, hint_color);
                    spawn_instruction_line(column, HINT_PLAY, hint_color);
                    spawn_instruction_line(column, HINT_STOP, hint_color);
                });
        });
}

/// Spawns a single instruction text line.
fn spawn_instruction_line(parent: &mut ChildSpawnerCommands, text: &str, color: Color) {
    parent.spawn((
        Name::new(format!("Instruction: {}", text)),
        InstructionLine,
        Text::new(text),
        TextFont::from_font_size(12.0),
        TextColor(color),
        Pickable::IGNORE,
    ));
}
