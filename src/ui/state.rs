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
    Physics,
    Constraints,
}

impl InspectorPage {
    pub fn name(&self) -> &'static str {
        match self {
            InspectorPage::Properties => PAGE_PROPERTIES,
            InspectorPage::Transform => PAGE_TRANSFORM,
            InspectorPage::Physics => PAGE_PHYSICS,
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

#[derive(Resource, Clone, Debug)]
pub struct UiVisibility {
    pub visible: bool,
}

impl Default for UiVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}
