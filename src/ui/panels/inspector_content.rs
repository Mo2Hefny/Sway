//! Inspector content systems for displaying and editing selected entity properties.

use bevy::prelude::*;
use bevy::ui::Node as UiNode;
use bevy::picking::prelude::Pickable;
use bevy::input::keyboard::{Key, KeyboardInput};

use crate::core::{Node as SimNode, NodeType};
use crate::core::playground::Playground;
use crate::editor::selection::Selection;
use crate::ui::messages::PLACEHOLDER_NO_SELECTION;
use crate::ui::state::{InspectorState, InspectorPage};
use crate::ui::theme::palette::*;
use crate::ui::theme::interaction::InteractionPalette;
use crate::ui::widgets::{
    InspectorContent,
    TextInput, TextInputDisplay, InputField, TextInputFocus as GenericTextInputFocus,
    FunctionInput, FunctionInputDisplay, FunctionField, FunctionInputFocus as GenericFunctionInputFocus,
    Checkbox,
    DropdownButton, DropdownMenu, DropdownOption, DropdownState,
    spawn_axis_input, spawn_numeric_input, spawn_checkbox, spawn_dropdown_option,
    spawn_function_input,
    INPUT_HEIGHT, INPUT_FONT_SIZE,
    DRAG_THRESHOLD, DRAG_SENSITIVITY,
};

use super::px;

// ============================================================================
// INSPECTOR-SPECIFIC COMPONENTS
// ============================================================================

/// Marker for dynamically spawned inspector property rows.
#[derive(Component, Debug)]
pub struct InspectorPropertyRow;

/// Marker for section headers.
#[derive(Component, Debug)]
pub struct InspectorSection;

/// Marker for the "no selection" placeholder text.
#[derive(Component, Debug)]
pub struct NoSelectionText;

/// Field kinds specific to the node inspector.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorFieldKind {
    #[default]
    PositionX,
    PositionY,
    AccelerationX,
    AccelerationY,
    AccFnX,
    AccFnY,
    Radius,
    FollowMouse,
    NodeType,
}

pub type TextInputFocus = GenericTextInputFocus<InspectorFieldKind>;
pub type FunctionInputFocus = GenericFunctionInputFocus<InspectorFieldKind>;

// ============================================================================
// CONSTANTS
// ============================================================================

const SECTION_PADDING: f32 = 8.0;
const ROW_HEIGHT: f32 = 22.0;
const ROW_SPACING: f32 = 1.0;
const LABEL_WIDTH: f32 = 85.0;
const HEADER_FONT_SIZE: f32 = 11.0;
const LABEL_FONT_SIZE: f32 = 11.0;

const RADIUS_MIN: f32 = 4.0;
const RADIUS_MAX: f32 = 50.0;
const ACCELERATION_MIN: f32 = -10.0;
const ACCELERATION_MAX: f32 = 10.0;


// ============================================================================
// MAIN UPDATE SYSTEM
// ============================================================================

pub fn update_inspector_content(
    mut commands: Commands,
    selection: Res<Selection>,
    inspector_state: Res<InspectorState>,
    playground: Res<Playground>,
    node_query: Query<&SimNode>,
    content_query: Query<Entity, With<InspectorContent>>,
    property_rows: Query<Entity, Or<(With<InspectorPropertyRow>, With<InspectorSection>)>>,
    no_selection_query: Query<Entity, With<NoSelectionText>>,
) {
    if !selection.is_changed() && !inspector_state.is_changed() {
        return;
    }

    for entity in property_rows.iter() {
        commands.entity(entity).despawn();
    }
    for entity in no_selection_query.iter() {
        commands.entity(entity).despawn();
    }

    let Ok(content_entity) = content_query.single() else {
        return;
    };

    match selection.entity {
        Some(selected_entity) => {
            if let Ok(node) = node_query.get(selected_entity) {
                spawn_page_content(&mut commands, content_entity, node, inspector_state.active_page, &playground);
            }
        }
        None => {
            spawn_no_selection(&mut commands, content_entity);
        }
    }
}

// ============================================================================
// INSPECTOR LAYOUT
// ============================================================================

fn spawn_no_selection(commands: &mut Commands, content_entity: Entity) {
    commands.entity(content_entity).with_children(|parent| {
        parent.spawn((
            NoSelectionText,
            UiNode {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        )).with_children(|container| {
            container.spawn((
                Text::new(PLACEHOLDER_NO_SELECTION),
                TextFont::from_font_size(13.0),
                TextColor(TEXT_DISABLED),
            ));
        });
    });
}

fn spawn_page_content(commands: &mut Commands, content_entity: Entity, node: &SimNode, page: InspectorPage, playground: &Playground) {
    match page {
        InspectorPage::Properties => spawn_properties_page(commands, content_entity, node),
        InspectorPage::Transform => spawn_transform_page(commands, content_entity, node, playground),
        InspectorPage::Physics => spawn_physics_page(commands, content_entity, node),
        InspectorPage::Constraints => spawn_constraints_page(commands, content_entity),
    }
}

fn spawn_properties_page(commands: &mut Commands, content_entity: Entity, node: &SimNode) {
    commands.entity(content_entity).with_children(|parent| {
        spawn_section_header(parent, "Node Settings");
        spawn_dropdown_field(parent, "Node Type", node.node_type, InspectorFieldKind::NodeType);
        spawn_checkbox_field(parent, "Follow Mouse", node.follow_mouse, InspectorFieldKind::FollowMouse);
    });
}

fn spawn_transform_page(commands: &mut Commands, content_entity: Entity, node: &SimNode, playground: &Playground) {
    let inner_min = playground.inner_min();
    let inner_max = playground.inner_max();
    let pos_min_x = inner_min.x + node.radius;
    let pos_max_x = inner_max.x - node.radius;
    let pos_min_y = inner_min.y + node.radius;
    let pos_max_y = inner_max.y - node.radius;

    commands.entity(content_entity).with_children(|parent| {
        spawn_section_header(parent, "Transform");
        spawn_vec2_field_asymmetric(parent, "Position", node.position,
            InspectorFieldKind::PositionX, InspectorFieldKind::PositionY,
            pos_min_x, pos_max_x, pos_min_y, pos_max_y);
        spawn_number_field(parent, "Radius", node.radius,
            InspectorFieldKind::Radius, RADIUS_MIN, RADIUS_MAX);
    });
}

fn spawn_physics_page(commands: &mut Commands, content_entity: Entity, node: &SimNode) {
    commands.entity(content_entity).with_children(|parent| {
        spawn_section_header(parent, "Acceleration");
        spawn_acceleration_axis(parent, "X", AXIS_X,
            node.acceleration.x, &node.acc_fn_x,
            InspectorFieldKind::AccelerationX, InspectorFieldKind::AccFnX);
        spawn_acceleration_axis(parent, "Y", AXIS_Y,
            node.acceleration.y, &node.acc_fn_y,
            InspectorFieldKind::AccelerationY, InspectorFieldKind::AccFnY);
    });
}

fn spawn_constraints_page(commands: &mut Commands, content_entity: Entity) {
    commands.entity(content_entity).with_children(|parent| {
        spawn_section_header(parent, "Constraints");
        parent.spawn((
            InspectorPropertyRow,
            UiNode {
                width: Val::Percent(100.0),
                padding: UiRect::all(px(SECTION_PADDING)),
                ..default()
            },
        )).with_children(|row| {
            row.spawn((
                Text::new("No constraints yet"),
                TextFont::from_font_size(LABEL_FONT_SIZE),
                TextColor(TEXT_DISABLED),
                Pickable::IGNORE,
            ));
        });
    });
}

/// Spawns two rows for one acceleration axis
fn spawn_acceleration_axis(
    parent: &mut ChildSpawnerCommands,
    axis_name: &str,
    axis_color: Color,
    value: f32,
    expr: &str,
    value_field: InspectorFieldKind,
    fn_field: InspectorFieldKind,
) {
    // Row 1: axis label + numeric value
    parent.spawn((
        Name::new(format!("Accel {} Value", axis_name)),
        InspectorPropertyRow,
        UiNode {
            width: Val::Percent(100.0),
            height: px(ROW_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(SECTION_PADDING)),
            margin: UiRect::vertical(px(ROW_SPACING)),
            ..default()
        },
    )).with_children(|row| {
        spawn_property_label(row, axis_name);

        row.spawn((
            UiNode {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
        )).with_children(|container| {
            spawn_axis_input(container, axis_color, value,
                InputField { kind: value_field, min: ACCELERATION_MIN, max: ACCELERATION_MAX }, 120.0);
        });
    });

    // Row 2: function expression input
    parent.spawn((
        Name::new(format!("Accel {} Function", axis_name)),
        InspectorPropertyRow,
        UiNode {
            width: Val::Percent(100.0),
            height: px(ROW_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(SECTION_PADDING)),
            margin: UiRect::bottom(px(4.0)),
            ..default()
        },
    )).with_children(|row| {
        spawn_property_label(row, "");

        row.spawn((
            UiNode {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
        )).with_children(|container| {
            spawn_function_input(container, expr,
                FunctionField { kind: fn_field }, 120.0);
        });
    });
}

// ============================================================================
// SECTION HELPERS
// ============================================================================

fn spawn_section_header(parent: &mut ChildSpawnerCommands, title: &str) {
    parent.spawn((
        Name::new(format!("Section: {}", title)),
        InspectorSection,
        UiNode {
            width: Val::Percent(100.0),
            height: px(28.0),
            padding: UiRect::new(px(SECTION_PADDING), px(SECTION_PADDING), px(8.0), px(4.0)),
            align_items: AlignItems::Center,
            margin: UiRect::top(px(4.0)),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BackgroundColor(SECTION_HEADER),
    )).with_children(|header| {
        header.spawn((
            Text::new(title.to_uppercase()),
            TextFont {
                font_size: HEADER_FONT_SIZE,
                ..default()
            },
            TextColor(TEXT),
            Pickable::IGNORE,
        ));
    });
}

// ============================================================================
// FIELD HELPERS
// ============================================================================

/// Spawns a Vec2 field with separate min/max per axis.
fn spawn_vec2_field_asymmetric(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: Vec2,
    field_x: InspectorFieldKind,
    field_y: InspectorFieldKind,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
) {
    parent.spawn((
        Name::new(format!("Property: {}", label)),
        InspectorPropertyRow,
        UiNode {
            width: Val::Percent(100.0),
            height: px(ROW_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(SECTION_PADDING)),
            margin: UiRect::vertical(px(ROW_SPACING)),
            ..default()
        },
    )).with_children(|row| {
        spawn_property_label(row, label);
        
        row.spawn((
            UiNode {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: px(6.0),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
        )).with_children(|inputs| {
            spawn_axis_input(inputs, AXIS_X, value.x, InputField { kind: field_x, min: min_x, max: max_x }, 65.0);
            spawn_axis_input(inputs, AXIS_Y, value.y, InputField { kind: field_y, min: min_y, max: max_y }, 65.0);
        });
    });
}

/// Spawns a single numeric field
fn spawn_number_field(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: f32,
    field: InspectorFieldKind,
    min: f32,
    max: f32,
) {
    parent.spawn((
        Name::new(format!("Property: {}", label)),
        InspectorPropertyRow,
        UiNode {
            width: Val::Percent(100.0),
            height: px(ROW_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(SECTION_PADDING)),
            margin: UiRect::vertical(px(ROW_SPACING)),
            ..default()
        },
    )).with_children(|row| {
        spawn_property_label(row, label);
        
        row.spawn((
            UiNode {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
        )).with_children(|container| {
            spawn_numeric_input(container, value, InputField { kind: field, min, max }, 80.0);
        });
    });
}

/// Spawns a checkbox field
fn spawn_checkbox_field(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    checked: bool,
    field: InspectorFieldKind,
) {
    parent.spawn((
        Name::new(format!("Property: {}", label)),
        InspectorPropertyRow,
        UiNode {
            width: Val::Percent(100.0),
            height: px(ROW_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(SECTION_PADDING)),
            margin: UiRect::vertical(px(ROW_SPACING)),
            ..default()
        },
    )).with_children(|row| {
        spawn_property_label(row, label);
        
        row.spawn((
            UiNode {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
        )).with_children(|container| {
            spawn_checkbox(container, checked, Some(InputField { kind: field, min: 0.0, max: 1.0 }));
        });
    });
}

/// Spawns a dropdown field for enum selection
fn spawn_dropdown_field(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    current: NodeType,
    field: InspectorFieldKind,
) {
    parent.spawn((
        Name::new(format!("Property: {}", label)),
        InspectorPropertyRow,
        UiNode {
            width: Val::Percent(100.0),
            height: px(ROW_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(SECTION_PADDING)),
            margin: UiRect::vertical(px(ROW_SPACING)),
            ..default()
        },
    )).with_children(|row| {
        spawn_property_label(row, label);
        
        row.spawn((
            UiNode {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
        )).with_children(|container| {
            container.spawn((
                Name::new("Dropdown Wrapper"),
                UiNode {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            )).with_children(|wrapper| {
                wrapper.spawn((
                    Name::new("Dropdown Button"),
                    DropdownButton,
                    InputField { kind: field, min: 0.0, max: 2.0 },
                    Button,
                    UiNode {
                        width: px(90.0),
                        height: px(INPUT_HEIGHT),
                        padding: UiRect::horizontal(px(8.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        border_radius: BorderRadius::all(px(3.0)),
                        border: UiRect::all(px(1.0)),
                        ..default()
                    },
                    BorderColor::all(BORDER),
                    BackgroundColor(INPUT_FIELD),
                    InteractionPalette {
                        none: INPUT_FIELD,
                        hovered: INPUT_FIELD_HOVER,
                        pressed: INPUT_FIELD_FOCUS,
                        active: INPUT_FIELD_FOCUS,
                    },
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new(current.name()),
                        TextFont::from_font_size(INPUT_FONT_SIZE),
                        TextColor(TEXT),
                        Pickable::IGNORE,
                    ));
                    btn.spawn((
                        Text::new("▼"),
                        TextFont::from_font_size(8.0),
                        TextColor(TEXT_SECONDARY),
                        Pickable::IGNORE,
                    ));
                });
                
                wrapper.spawn((
                    Name::new("Dropdown Menu"),
                    DropdownMenu,
                    UiNode {
                        position_type: PositionType::Absolute,
                        top: px(INPUT_HEIGHT + 2.0),
                        right: px(0.0),
                        width: px(90.0),
                        flex_direction: FlexDirection::Column,
                        border_radius: BorderRadius::all(px(3.0)),
                        border: UiRect::all(px(1.0)),
                        display: Display::None,
                        overflow: Overflow::visible(),
                        ..default()
                    },
                    BorderColor::all(BORDER),
                    BackgroundColor(SURFACE),
                    ZIndex(100),
                )).with_children(|menu| {
                    spawn_dropdown_option(menu, NodeType::Normal, NodeType::Normal.name(), current == NodeType::Normal);
                    spawn_dropdown_option(menu, NodeType::Anchor, NodeType::Anchor.name(), current == NodeType::Anchor);
                    spawn_dropdown_option(menu, NodeType::Leg, NodeType::Leg.name(), current == NodeType::Leg);
                });
            });
        });
    });
}

// ============================================================================
// HELPER
// ============================================================================

fn spawn_property_label(parent: &mut ChildSpawnerCommands, text: &str) {
    parent.spawn((
        UiNode {
            width: px(LABEL_WIDTH),
            ..default()
        },
    )).with_children(|container| {
        container.spawn((
            Text::new(text),
            TextFont::from_font_size(LABEL_FONT_SIZE),
            TextColor(TEXT_SECONDARY),
            Pickable::IGNORE,
        ));
    });
}

// ============================================================================
// INTERACTION HANDLERS
// ============================================================================

/// Handle mouse-down on text input: begin drag tracking (click vs drag determined later).
pub fn handle_text_input_focus(
    mut focus: ResMut<TextInputFocus>,
    input_query: Query<(Entity, &Interaction, &InputField<InspectorFieldKind>), (Changed<Interaction>, With<TextInput>)>,
    text_query: Query<(&Text, &InputField<InspectorFieldKind>), With<TextInputDisplay>>,
    windows: Query<&Window>,
) {
    for (entity, interaction, field) in input_query.iter() {
        if *interaction == Interaction::Pressed && focus.entity.is_none() {
            let mut current_value = 0.0_f32;
            for (text, text_field) in text_query.iter() {
                if text_field.kind == field.kind {
                    current_value = text.0.parse::<f32>().unwrap_or(0.0);
                    break;
                }
            }

            let cursor_x = windows.iter().next()
                .and_then(|w| w.cursor_position())
                .map(|p| p.x)
                .unwrap_or(0.0);

            focus.drag_entity = Some(entity);
            focus.drag_field_kind = Some(field.kind);
            focus.drag_origin_x = cursor_x;
            focus.drag_origin_value = current_value;
            focus.drag_min = field.min;
            focus.drag_max = field.max;
            focus.is_dragging = false;
            focus.drag_accumulated = 0.0;
        }
    }
}

/// Handle drag-to-adjust: track horizontal mouse movement while holding, scrub value.
/// On release: if it was a short click (no drag), enter text-edit mode instead.
pub fn handle_text_input_drag(
    mut focus: ResMut<TextInputFocus>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut text_query: Query<(&mut Text, &InputField<InspectorFieldKind>), With<TextInputDisplay>>,
    selection: Res<Selection>,
    mut node_query: Query<&mut SimNode>,
) {
    let Some(drag_entity) = focus.drag_entity else { return };
    let Some(drag_field) = focus.drag_field_kind else { return };

    let cursor_x = windows.iter().next()
        .and_then(|w| w.cursor_position())
        .map(|p| p.x)
        .unwrap_or(focus.drag_origin_x);

    let delta_x = cursor_x - focus.drag_origin_x;

    if !focus.is_dragging && delta_x.abs() > DRAG_THRESHOLD {
        focus.is_dragging = true;
    }

    if focus.is_dragging {
        let new_value = (focus.drag_origin_value + delta_x * DRAG_SENSITIVITY)
            .clamp(focus.drag_min, focus.drag_max);
        apply_field_value(&selection, &mut node_query, drag_field, new_value);
        update_input_text(&mut text_query, drag_field, &format!("{:.2}", new_value));
        focus.drag_accumulated = new_value;
    }

    if mouse.just_released(MouseButton::Left) {
        if focus.is_dragging {
            focus.drag_entity = None;
            focus.drag_field_kind = None;
            focus.is_dragging = false;
        } else {
            let current_text = format!("{:.2}", focus.drag_origin_value);
            focus.entity = Some(drag_entity);
            focus.buffer = current_text;
            focus.field_kind = Some(drag_field);
            focus.min = focus.drag_min;
            focus.max = focus.drag_max;
            focus.drag_entity = None;
            focus.drag_field_kind = None;
        }
    }
}

/// Handle keyboard input for focused text field
pub fn handle_text_input_keyboard(
    mut focus: ResMut<TextInputFocus>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut text_query: Query<(&mut Text, &InputField<InspectorFieldKind>), With<TextInputDisplay>>,
    selection: Res<Selection>,
    mut node_query: Query<&mut SimNode>,
) {
    let Some(_focused_entity) = focus.entity else { return };
    let Some(field_kind) = focus.field_kind else { return };
    
    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        
        match &event.logical_key {
            Key::Character(c) => {
                let ch = c.as_str();
                if ch.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '.') {
                    focus.buffer.push_str(ch);
                    update_input_text(&mut text_query, field_kind, &focus.buffer);
                }
            }
            Key::Backspace => {
                focus.buffer.pop();
                update_input_text(&mut text_query, field_kind, &focus.buffer);
            }
            Key::Enter => {
                if let Ok(value) = focus.buffer.parse::<f32>() {
                    let clamped: f32 = value.clamp(focus.min, focus.max);
                    apply_field_value(&selection, &mut node_query, field_kind, clamped);
                    focus.buffer = format!("{:.2}", clamped);
                    update_input_text(&mut text_query, field_kind, &focus.buffer);
                }
            }
            Key::Escape => {
                focus.entity = None;
                focus.buffer.clear();
                focus.field_kind = None;
            }
            Key::Tab => {
                if let Ok(value) = focus.buffer.parse::<f32>() {
                    let clamped: f32 = value.clamp(focus.min, focus.max);
                    apply_field_value(&selection, &mut node_query, field_kind, clamped);
                }
                focus.entity = None;
                focus.buffer.clear();
                focus.field_kind = None;
            }
            _ => {}
        }
    }
}

/// Update text display by field kind
fn update_input_text(
    text_query: &mut Query<(&mut Text, &InputField<InspectorFieldKind>), With<TextInputDisplay>>,
    field_kind: InspectorFieldKind,
    new_text: &str,
) {
    for (mut text, field) in text_query.iter_mut() {
        if field.kind == field_kind {
            text.0 = if new_text.is_empty() { "0".to_string() } else { new_text.to_string() };
            break;
        }
    }
}

/// Apply a field value to the selected node
fn apply_field_value(
    selection: &Res<Selection>,
    node_query: &mut Query<&mut SimNode>,
    kind: InspectorFieldKind,
    value: f32,
) {
    let Some(selected_entity) = selection.entity else { return };
    let Ok(mut node) = node_query.get_mut(selected_entity) else { return };
    
    match kind {
        InspectorFieldKind::PositionX => node.position.x = value,
        InspectorFieldKind::PositionY => node.position.y = value,
        InspectorFieldKind::AccelerationX => node.acceleration.x = value,
        InspectorFieldKind::AccelerationY => node.acceleration.y = value,
        InspectorFieldKind::Radius => node.radius = value,
        _ => {}
    }
}

/// Visual feedback for focused input
pub fn update_focused_input_style(
    focus: Res<TextInputFocus>,
    fn_focus: Res<FunctionInputFocus>,
    mut input_query: Query<(Entity, &mut BackgroundColor), With<TextInput>>,
    mut fn_input_query: Query<(Entity, &mut BackgroundColor), (With<FunctionInput>, Without<TextInput>)>,
) {
    for (entity, mut bg) in input_query.iter_mut() {
        if Some(entity) == focus.entity {
            *bg = BackgroundColor(INPUT_FIELD_FOCUS);
        } else {
            *bg = BackgroundColor(INPUT_FIELD);
        }
    }
    for (entity, mut bg) in fn_input_query.iter_mut() {
        if Some(entity) == fn_focus.entity {
            *bg = BackgroundColor(INPUT_FIELD_FOCUS);
        } else {
            *bg = BackgroundColor(INPUT_FIELD);
        }
    }
}

/// Clear focus when clicking outside inputs
pub fn handle_click_outside(
    mut focus: ResMut<TextInputFocus>,
    mut fn_focus: ResMut<FunctionInputFocus>,
    mouse: Res<ButtonInput<MouseButton>>,
    input_query: Query<&Interaction, With<TextInput>>,
    fn_input_query: Query<&Interaction, With<FunctionInput>>,
    selection: Res<Selection>,
    mut node_query: Query<&mut SimNode>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if focus.drag_entity.is_some() {
        return;
    }
    
    let clicking_input = input_query.iter().any(|i| *i != Interaction::None)
        || fn_input_query.iter().any(|i| *i != Interaction::None);
    
    if !clicking_input && focus.entity.is_some() {
        if let (Some(field_kind), Ok(value)) = (focus.field_kind, focus.buffer.parse::<f32>()) {
            let clamped: f32 = value.clamp(focus.min, focus.max);
            apply_field_value(&selection, &mut node_query, field_kind, clamped);
        }
        focus.entity = None;
        focus.buffer.clear();
        focus.field_kind = None;
    }

    if !clicking_input && fn_focus.entity.is_some() {
        if let Some(field_kind) = fn_focus.field_kind {
            apply_function_value(&selection, &mut node_query, field_kind, &fn_focus.buffer);
        }
        fn_focus.entity = None;
        fn_focus.buffer.clear();
        fn_focus.field_kind = None;
    }
}

pub fn handle_inspector_checkbox_click(
    mut commands: Commands,
    selection: Res<Selection>,
    inspector_state: Res<InspectorState>,
    playground: Res<Playground>,
    mut node_query: Query<&mut SimNode>,
    checkbox_query: Query<(&Interaction, &InputField<InspectorFieldKind>), (Changed<Interaction>, With<Checkbox>)>,
    content_query: Query<Entity, With<InspectorContent>>,
    property_rows: Query<Entity, Or<(With<InspectorPropertyRow>, With<InspectorSection>)>>,
) {
    for (interaction, field) in checkbox_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(selected_entity) = selection.entity else { continue };
        let Ok(mut node) = node_query.get_mut(selected_entity) else { continue };

        if field.kind == InspectorFieldKind::FollowMouse {
            node.follow_mouse = !node.follow_mouse;
            rebuild_inspector(&mut commands, &content_query, &property_rows, &node, inspector_state.active_page, &playground);
        }
    }
}

pub fn handle_inspector_dropdown_click(
    mut dropdown_state: ResMut<DropdownState>,
    dropdown_query: Query<&Interaction, (Changed<Interaction>, With<DropdownButton>)>,
    mut menu_query: Query<&mut UiNode, With<DropdownMenu>>,
) {
    for interaction in dropdown_query.iter() {
        if *interaction == Interaction::Pressed {
            dropdown_state.open = !dropdown_state.open;
            
            for mut node in menu_query.iter_mut() {
                node.display = if dropdown_state.open { Display::Flex } else { Display::None };
            }
        }
    }
}

pub fn handle_dropdown_option_click(
    mut commands: Commands,
    mut dropdown_state: ResMut<DropdownState>,
    selection: Res<Selection>,
    inspector_state: Res<InspectorState>,
    playground: Res<Playground>,
    mut node_query: Query<&mut SimNode>,
    option_query: Query<(&Interaction, &DropdownOption<NodeType>), Changed<Interaction>>,
    mut menu_query: Query<&mut UiNode, With<DropdownMenu>>,
    content_query: Query<Entity, With<InspectorContent>>,
    property_rows: Query<Entity, Or<(With<InspectorPropertyRow>, With<InspectorSection>)>>,
) {
    for (interaction, option) in option_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(selected_entity) = selection.entity else { continue };
        let Ok(mut node) = node_query.get_mut(selected_entity) else { continue };

        node.node_type = option.0;
        
        dropdown_state.open = false;
        for mut menu_node in menu_query.iter_mut() {
            menu_node.display = Display::None;
        }
        
        rebuild_inspector(&mut commands, &content_query, &property_rows, &node, inspector_state.active_page, &playground);
    }
}

fn rebuild_inspector(
    commands: &mut Commands,
    content_query: &Query<Entity, With<InspectorContent>>,
    property_rows: &Query<Entity, Or<(With<InspectorPropertyRow>, With<InspectorSection>)>>,
    node: &SimNode,
    page: InspectorPage,
    playground: &Playground,
) {
    for entity in property_rows.iter() {
        commands.entity(entity).despawn();
    }
    
    let Ok(content_entity) = content_query.single() else { return };
    spawn_page_content(commands, content_entity, node, page, playground);
}

// ============================================================================
// FUNCTION INPUT HANDLERS
// ============================================================================

/// Handle clicking on function input to focus it
pub fn handle_function_input_focus(
    mut fn_focus: ResMut<FunctionInputFocus>,
    input_query: Query<(Entity, &Interaction, &FunctionField<InspectorFieldKind>), (Changed<Interaction>, With<FunctionInput>)>,
    text_query: Query<(&Text, &FunctionField<InspectorFieldKind>), With<FunctionInputDisplay>>,
) {
    for (entity, interaction, field) in input_query.iter() {
        if *interaction == Interaction::Pressed {
            let mut current_value = String::new();
            for (text, text_field) in text_query.iter() {
                if text_field.kind == field.kind {
                    let val = text.0.clone();
                    if val != "f(t)" {
                        current_value = val;
                    }
                    break;
                }
            }
            
            fn_focus.entity = Some(entity);
            fn_focus.buffer = current_value;
            fn_focus.field_kind = Some(field.kind);
        }
    }
}

/// Handle keyboard input for focused function expression field
pub fn handle_function_input_keyboard(
    mut fn_focus: ResMut<FunctionInputFocus>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut text_query: Query<(&mut Text, &mut TextColor, &FunctionField<InspectorFieldKind>), With<FunctionInputDisplay>>,
    selection: Res<Selection>,
    mut node_query: Query<&mut SimNode>,
) {
    let Some(_focused_entity) = fn_focus.entity else { return };
    let Some(field_kind) = fn_focus.field_kind else { return };
    
    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        
        match &event.logical_key {
            Key::Character(c) => {
                fn_focus.buffer.push_str(c.as_str());
                update_function_text(&mut text_query, field_kind, &fn_focus.buffer);
            }
            Key::Space => {
                fn_focus.buffer.push(' ');
                update_function_text(&mut text_query, field_kind, &fn_focus.buffer);
            }
            Key::Backspace => {
                fn_focus.buffer.pop();
                update_function_text(&mut text_query, field_kind, &fn_focus.buffer);
            }
            Key::Enter => {
                apply_function_value(&selection, &mut node_query, field_kind, &fn_focus.buffer);
                update_function_text(&mut text_query, field_kind, &fn_focus.buffer);
            }
            Key::Escape => {
                fn_focus.entity = None;
                fn_focus.buffer.clear();
                fn_focus.field_kind = None;
            }
            _ => {}
        }
    }
}

fn update_function_text(
    text_query: &mut Query<(&mut Text, &mut TextColor, &FunctionField<InspectorFieldKind>), With<FunctionInputDisplay>>,
    field_kind: InspectorFieldKind,
    new_text: &str,
) {
    for (mut text, mut color, field) in text_query.iter_mut() {
        if field.kind == field_kind {
            if new_text.is_empty() {
                text.0 = "f(t)".to_string();
                color.0 = TEXT_DISABLED;
            } else {
                text.0 = new_text.to_string();
                color.0 = TEXT;
            }
            break;
        }
    }
}

fn apply_function_value(
    selection: &Res<Selection>,
    node_query: &mut Query<&mut SimNode>,
    kind: InspectorFieldKind,
    expr: &str,
) {
    let Some(selected_entity) = selection.entity else { return };
    let Ok(mut node) = node_query.get_mut(selected_entity) else { return };
    
    match kind {
        InspectorFieldKind::AccFnX => node.acc_fn_x = expr.to_string(),
        InspectorFieldKind::AccFnY => node.acc_fn_y = expr.to_string(),
        _ => {}
    }
}

