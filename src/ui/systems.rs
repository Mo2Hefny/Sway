//! UI interaction systems.

use bevy::prelude::*;

use super::icons::UiIcons;
use super::state::*;
use super::theme::interaction::Active;
use super::widgets::{
    CheckboxButton, CheckboxSetting, CheckboxIcon, HamburgerButton, PanelBody,
    ImportButton, ExportButton, CaretButton, CaretIcon, InspectorPanel,
    PageIconButton, PageIconFor, InspectorTitle, ToolButton, ToolButtonFor,
    ToolBarRoot, FloatingPanel, RightSidebarRoot, InstructionOverlayRoot, BottomToolbar,
    PlaybackButton, PlaybackAction,
};

/// Handles checkbox button clicks and updates display settings.
pub fn handle_checkbox_clicks(
    mut display_settings: ResMut<DisplaySettings>,
    query: Query<(&Interaction, &CheckboxSetting), (Changed<Interaction>, With<CheckboxButton>)>,
) {
    for (interaction, setting) in &query {
        if *interaction == Interaction::Pressed {
            match setting {
                CheckboxSetting::ShowSkin => {
                    display_settings.show_skin = !display_settings.show_skin;
                    info!("Show Skin: {}", display_settings.show_skin);
                }
                CheckboxSetting::ShowEdge => {
                    display_settings.show_edge = !display_settings.show_edge;
                    info!("Show Edge: {}", display_settings.show_edge);
                }
                CheckboxSetting::ShowNodes => {
                    display_settings.show_nodes = !display_settings.show_nodes;
                    info!("Show Nodes: {}", display_settings.show_nodes);
                }
            }
        }
    }
}

/// Updates checkbox icon visibility based on display settings.
pub fn update_checkbox_icons(
    display_settings: Res<DisplaySettings>,
    mut query: Query<(&CheckboxSetting, &mut Visibility), With<CheckboxIcon>>,
) {
    if !display_settings.is_changed() {
        return;
    }

    for (setting, mut visibility) in &mut query {
        let is_checked = match setting {
            CheckboxSetting::ShowSkin => display_settings.show_skin,
            CheckboxSetting::ShowEdge => display_settings.show_edge,
            CheckboxSetting::ShowNodes => display_settings.show_nodes,
        };
        *visibility = if is_checked {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Handles hamburger menu toggle to collapse/expand floating panel.
pub fn handle_hamburger_click(
    mut panel_state: ResMut<FloatingPanelState>,
    query: Query<&Interaction, (Changed<Interaction>, With<HamburgerButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            panel_state.collapsed = !panel_state.collapsed;
            info!("Panel collapsed: {}", panel_state.collapsed);
        }
    }
}

/// Updates floating panel body display based on collapsed state.
pub fn update_panel_visibility(
    panel_state: Res<FloatingPanelState>,
    mut panel_body_query: Query<&mut Node, With<PanelBody>>,
) {
    if !panel_state.is_changed() {
        return;
    }

    let display = if panel_state.collapsed {
        Display::None
    } else {
        Display::Flex
    };

    for mut node in &mut panel_body_query {
        node.display = display;
    }
}

/// Handles import button clicks.
pub fn handle_import_click(
    query: Query<&Interaction, (Changed<Interaction>, With<ImportButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            info!("Import clicked");
        }
    }
}

/// Handles export button clicks.
pub fn handle_export_click(
    query: Query<&Interaction, (Changed<Interaction>, With<ExportButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            info!("Export clicked");
        }
    }
}

/// Handles inspector caret button clicks to toggle panel visibility.
pub fn handle_caret_click(
    mut inspector_state: ResMut<InspectorState>,
    query: Query<&Interaction, (Changed<Interaction>, With<CaretButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            inspector_state.open = !inspector_state.open;
            info!("Inspector panel open: {}", inspector_state.open);
        }
    }
}

/// Updates inspector panel visibility and caret icon based on state.
pub fn update_inspector_panel_visibility(
    inspector_state: Res<InspectorState>,
    icons: Res<UiIcons>,
    mut panel_query: Query<&mut Visibility, With<InspectorPanel>>,
    mut caret_query: Query<&mut ImageNode, With<CaretIcon>>,
) {
    if !inspector_state.is_changed() {
        return;
    }

    let visibility = if inspector_state.open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut vis in &mut panel_query {
        *vis = visibility;
    }

    let caret_icon = if inspector_state.open {
        icons.caret_right.clone()
    } else {
        icons.caret_left.clone()
    };
    for mut image in &mut caret_query {
        image.image = caret_icon.clone();
    }
}

/// Handles inspector page tab clicks to switch active page.
pub fn handle_page_icon_clicks(
    mut inspector_state: ResMut<InspectorState>,
    query: Query<(&Interaction, &PageIconFor), (Changed<Interaction>, With<PageIconButton>)>,
) {
    for (interaction, page_for) in &query {
        if *interaction == Interaction::Pressed {
            inspector_state.active_page = page_for.0;
            info!("Inspector page changed to: {:?}", inspector_state.active_page);
        }
    }
}

/// Updates inspector page icon active states.
pub fn update_page_icon_styles(
    inspector_state: Res<InspectorState>,
    mut commands: Commands,
    button_query: Query<(Entity, &PageIconFor), With<PageIconButton>>,
) {
    if !inspector_state.is_changed() {
        return;
    }

    for (entity, page_for) in &button_query {
        let is_active = page_for.0 == inspector_state.active_page;
        if is_active {
            commands.entity(entity).insert(Active);
            info!("Page button {:?} set to active", page_for.0);
        } else {
            commands.entity(entity).remove::<Active>();
        }
    }
}

/// Updates inspector title text based on active page.
pub fn update_inspector_title(
    inspector_state: Res<InspectorState>,
    mut title_query: Query<&mut Text, With<InspectorTitle>>,
) {
    if !inspector_state.is_changed() {
        return;
    }

    for mut text in &mut title_query {
        **text = inspector_state.active_page.name().to_string();
    }
}

/// Handles tool button clicks to change active editor tool.
pub fn handle_tool_button_clicks(
    mut tool_state: ResMut<EditorToolState>,
    query: Query<(&Interaction, &ToolButtonFor), (Changed<Interaction>, With<ToolButton>)>,
) {
    for (interaction, tool_for) in &query {
        if *interaction == Interaction::Pressed {
            tool_state.active = tool_for.0;
            info!("Tool changed to: {:?}", tool_state.active);
        }
    }
}

/// Updates tool button active states.
pub fn update_tool_button_styles(
    tool_state: Res<EditorToolState>,
    mut commands: Commands,
    button_query: Query<(Entity, &ToolButtonFor), With<ToolButton>>,
) {
    if !tool_state.is_changed() {
        return;
    }

    for (entity, tool_for) in &button_query {
        let is_active = tool_for.0 == tool_state.active;
        if is_active {
            commands.entity(entity).insert(Active);
            info!("Tool button {:?} set to active", tool_for.0);
        } else {
            commands.entity(entity).remove::<Active>();
        }
    }
}

/// Updates tool bar position based on inspector panel state.
pub fn update_tool_bar_position(
    inspector_state: Res<InspectorState>,
    mut tool_bar_query: Query<&mut Node, With<ToolBarRoot>>,
) {
    if !inspector_state.is_changed() {
        return;
    }

    let right_offset = if inspector_state.open {
        px(328.0)
    } else {
        px(48.0)
    };

    for mut node in &mut tool_bar_query {
        node.right = right_offset;
    }
}

/// Handles 'H' key press to toggle overall UI visibility.
pub fn handle_ui_toggle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_visibility: ResMut<UiVisibility>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        ui_visibility.visible = !ui_visibility.visible;
        info!("UI visibility: {}", ui_visibility.visible);
    }
}

/// Updates visibility of all UI panels based on global UI visibility state.
pub fn update_ui_visibility(
    ui_visibility: Res<UiVisibility>,
    mut floating_panel_query: Query<&mut Visibility, (With<FloatingPanel>, Without<RightSidebarRoot>, Without<ToolBarRoot>, Without<InstructionOverlayRoot>, Without<BottomToolbar>)>,
    mut sidebar_query: Query<&mut Visibility, (With<RightSidebarRoot>, Without<FloatingPanel>, Without<ToolBarRoot>, Without<InstructionOverlayRoot>, Without<BottomToolbar>)>,
    mut tool_bar_query: Query<&mut Visibility, (With<ToolBarRoot>, Without<FloatingPanel>, Without<RightSidebarRoot>, Without<InstructionOverlayRoot>, Without<BottomToolbar>)>,
    mut overlay_query: Query<&mut Visibility, (With<InstructionOverlayRoot>, Without<FloatingPanel>, Without<RightSidebarRoot>, Without<ToolBarRoot>, Without<BottomToolbar>)>,
    mut bottom_bar_query: Query<&mut Visibility, (With<BottomToolbar>, Without<FloatingPanel>, Without<RightSidebarRoot>, Without<ToolBarRoot>, Without<InstructionOverlayRoot>)>,
) {
    if !ui_visibility.is_changed() {
        return;
    }

    let visibility = if ui_visibility.visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut vis in &mut floating_panel_query {
        *vis = visibility;
    }
    for mut vis in &mut sidebar_query {
        *vis = visibility;
    }
    for mut vis in &mut tool_bar_query {
        *vis = visibility;
    }
    for mut vis in &mut overlay_query {
        *vis = visibility;
    }
    for mut vis in &mut bottom_bar_query {
        *vis = visibility;
    }
}

pub fn handle_playback_clicks(
    mut playback: ResMut<PlaybackState>,
    query: Query<(&Interaction, &PlaybackAction), (Changed<Interaction>, With<PlaybackButton>)>,
) {
    for (interaction, action) in &query {
        if *interaction == Interaction::Pressed {
            match action {
                PlaybackAction::Play => {
                    playback.play();
                    info!("Playback: Playing");
                }
                PlaybackAction::Pause => {
                    playback.pause();
                    info!("Playback: Paused");
                }
                PlaybackAction::Stop => {
                    playback.stop();
                    info!("Playback: Stopped");
                }
            }
        }
    }
}
