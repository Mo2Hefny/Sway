//! UI marker components for layout and panel elements.

use bevy::picking::prelude::Pickable;
use bevy::prelude::*;

use crate::ui::state::{EditorTool, InspectorPage};

// ============================================================================
// PANEL MARKERS
// ============================================================================

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
    ShowDebug,
}

#[derive(Component, Debug)]
pub struct CheckboxIcon;

#[derive(Component, Debug)]
pub struct ImportButton;

#[derive(Component, Debug)]
pub struct ExportButton;

// ============================================================================
// INSPECTOR MARKERS
// ============================================================================

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

// ============================================================================
// TOOLBAR MARKERS
// ============================================================================

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

// ============================================================================
// OVERLAY MARKERS
// ============================================================================

#[derive(Component, Debug)]
pub struct InstructionOverlayRoot;

#[derive(Component, Debug)]
pub struct InstructionColumn;

#[derive(Component, Debug)]
pub struct InstructionLine;

// ============================================================================
// ROOT
// ============================================================================

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
