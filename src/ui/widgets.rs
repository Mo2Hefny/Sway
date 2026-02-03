//! UI marker components.

use bevy::prelude::*;

use super::theme::interaction::InteractionPalette;
use super::state::{InspectorPage, EditorTool};

#[derive(Component, Debug)]
pub struct FloatingPanel;

#[derive(Component, Debug)]
pub struct PanelContainer;

#[derive(Component, Debug)]
pub struct PanelBody;

#[derive(Component, Debug)]
pub struct HeaderRow;

#[derive(Component, Debug)]
pub struct HamburgerButton;

#[derive(Component, Debug)]
pub struct CheckboxRow;

#[derive(Component, Debug)]
pub struct CheckboxButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxSetting {
    ShowSkin,
    ShowEdge,
    ShowNodes,
}

#[derive(Component, Debug)]
pub struct CheckboxIcon;

#[derive(Component, Debug)]
pub struct ImportButton;

#[derive(Component, Debug)]
pub struct ExportButton;

#[derive(Component, Debug)]
pub struct RightSidebarRoot;

#[derive(Component, Debug)]
pub struct InspectorPanel;

#[derive(Component, Debug)]
pub struct IconBar;

#[derive(Component, Debug)]
pub struct CaretButton;

#[derive(Component, Debug)]
pub struct CaretIcon;

#[derive(Component, Debug)]
pub struct PageIconButton;

#[derive(Component, Debug)]
pub struct PageIconImage;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageIconFor(pub InspectorPage);

#[derive(Component, Debug)]
pub struct InspectorTitle;

#[derive(Component, Debug)]
pub struct InspectorContent;

#[derive(Component, Debug)]
pub struct ToolBarRoot;

#[derive(Component, Debug)]
pub struct ToolIconColumn;

#[derive(Component, Debug)]
pub struct ToolButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolButtonFor(pub EditorTool);

#[derive(Component, Debug)]
pub struct ToolIconImage;

#[derive(Component, Debug)]
pub struct BottomToolbar;

#[derive(Component, Debug)]
pub struct PlaybackButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackAction {
    Play,
    Pause,
    Stop,
}

#[derive(Component, Debug)]
pub struct InstructionOverlayRoot;

#[derive(Component, Debug)]
pub struct InstructionColumn;

#[derive(Component, Debug)]
pub struct InstructionLine;

#[derive(Component, Debug)]
pub struct UiRoot;

pub fn ui_root(name: &str) -> impl Bundle {
    (
        Name::new(name.to_string()),
        UiRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// Updates button colors based on both interaction state (hover/press) and active state.
pub fn update_interaction_colors(
    mut query: Query<
        (
            &Interaction,
            &InteractionPalette,
            &mut BackgroundColor,
            Option<&super::theme::interaction::Active>,
        ),
        Or<(Changed<Interaction>, Changed<super::theme::interaction::Active>)>,
    >,
) {
    for (interaction, palette, mut background, active) in &mut query {
        let color = if active.is_some() {
            match interaction {
                Interaction::None => palette.active,
                Interaction::Hovered => palette.hovered,
                Interaction::Pressed => palette.pressed,
            }
        } else {
            match interaction {
                Interaction::None => palette.none,
                Interaction::Hovered => palette.hovered,
                Interaction::Pressed => palette.pressed,
            }
        };
        *background = BackgroundColor(color);
    }
}
