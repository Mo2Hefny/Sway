//! UI state resources.

use bevy::prelude::*;

use super::messages::*;

/// Visibility toggles for editor elements.
#[derive(Resource, Clone, Debug)]
pub struct DisplaySettings {
    pub show_skin: bool,
    pub show_edge: bool,
    pub show_nodes: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            show_skin: true,
            show_edge: true,
            show_nodes: true,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FloatingPanelState {
    pub collapsed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InspectorPage {
    #[default]
    Properties,
    Transform,
    Constraints,
}

impl InspectorPage {
    pub fn name(&self) -> &'static str {
        match self {
            InspectorPage::Properties => PAGE_PROPERTIES,
            InspectorPage::Transform => PAGE_TRANSFORM,
            InspectorPage::Constraints => PAGE_CONSTRAINTS,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct InspectorState {
    pub open: bool,
    pub active_page: InspectorPage,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            open: true,
            active_page: InspectorPage::Properties,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EditorTool {
    #[default]
    Cursor,
    AddNode,
    AddEdge,
    Move,
}

impl EditorTool {
    pub fn name(&self) -> &'static str {
        match self {
            EditorTool::Cursor => TOOL_CURSOR,
            EditorTool::AddNode => TOOL_ADD_NODE,
            EditorTool::AddEdge => TOOL_ADD_EDGE,
            EditorTool::Move => TOOL_MOVE,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct EditorToolState {
    pub active: EditorTool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum PlaybackMode {
    Playing,
    Paused,
    #[default]
    Stopped,
}

/// Shared resource that gates simulation systems.
#[derive(Resource, Clone, Debug, Default, Reflect)]
pub struct PlaybackState {
    pub mode: PlaybackMode,
}

impl PlaybackState {
    pub fn is_playing(&self) -> bool {
        self.mode == PlaybackMode::Playing
    }

    pub fn play(&mut self) {
        self.mode = PlaybackMode::Playing;
    }

    pub fn pause(&mut self) {
        self.mode = PlaybackMode::Paused;
    }

    pub fn stop(&mut self) {
        self.mode = PlaybackMode::Stopped;
    }
}

#[derive(Resource, Clone, Debug)]
pub struct UiVisibility {
    pub visible: bool,
}

impl Default for UiVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

/// Resource tracking whether the cursor is currently over UI elements.
#[derive(Resource, Clone, Debug, Default, Reflect)]
pub struct InputState {
    pub cursor_over_ui: bool,
}

impl InputState {
    pub fn can_interact_with_world(&self) -> bool {
        !self.cursor_over_ui
    }
}

pub fn update_input_state(
    mut input_state: ResMut<InputState>,
    ui_interaction: Query<&Interaction>,
) {
    input_state.cursor_over_ui = ui_interaction
        .iter()
        .any(|interaction| *interaction != Interaction::None);
}
