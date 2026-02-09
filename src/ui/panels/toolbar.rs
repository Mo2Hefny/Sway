//! Tool bar spawning.

use bevy::picking::prelude::Pickable;
use bevy::prelude::*;

use crate::ui::icons::UiIcons;
use crate::ui::state::*;
use crate::ui::theme::interaction::Active;
use crate::ui::theme::interaction::InteractionPalette;
use crate::ui::theme::palette::*;
use crate::ui::widgets::{ToolBarRoot, ToolButton, ToolButtonFor, ToolIconColumn, ToolIconImage};

use super::px;

/// Spawns the vertical tool bar with editor tool buttons.
pub fn spawn_tool_bar(commands: &mut Commands, icons: &UiIcons) {
    commands
        .spawn((
            Name::new("Tool Bar Root"),
            ToolBarRoot,
            Node {
                position_type: PositionType::Absolute,
                right: px(328.0),
                top: px(0.0),
                bottom: px(0.0),
                width: px(48.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(4.0)),
                row_gap: px(4.0),
                ..default()
            },
            BackgroundColor(SURFACE),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Tool Icon Column"),
                    ToolIconColumn,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4.0),
                        ..default()
                    },
                ))
                .with_children(|column| {
                    spawn_tool_button(column, EditorTool::Cursor, icons.cursor_tool.clone(), true);
                    spawn_tool_button(column, EditorTool::AddNode, icons.add_node_tool.clone(), false);
                    spawn_tool_button(column, EditorTool::AddEdge, icons.add_edge_tool.clone(), false);
                    spawn_tool_button(column, EditorTool::Move, icons.move_tool.clone(), false);
                });
        });
}

/// Spawns a tool button with icon and active state styling.
fn spawn_tool_button(parent: &mut ChildSpawnerCommands, tool: EditorTool, icon: Handle<Image>, is_active: bool) {
    let bg_color = if is_active { SURFACE_HOVER } else { SURFACE };

    let mut entity = parent.spawn((
        Name::new(format!("Tool Button: {}", tool.name())),
        ToolButton,
        ToolButtonFor(tool),
        Button,
        Node {
            width: px(40.0),
            height: px(40.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BackgroundColor(bg_color),
        InteractionPalette {
            none: SURFACE,
            hovered: SURFACE_HOVER,
            pressed: SURFACE_PRESSED,
            active: SURFACE_HOVER,
        },
    ));

    if is_active {
        entity.insert(Active);
    }

    entity.with_children(|btn| {
        btn.spawn((
            ToolIconImage,
            ToolButtonFor(tool),
            ImageNode::new(icon),
            Node {
                width: px(24.0),
                height: px(24.0),
                ..default()
            },
            Pickable::IGNORE,
        ));
    });
}
