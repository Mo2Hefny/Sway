//! Reusable text input widget for numeric values.

use bevy::prelude::*;
use bevy::ui::Node as UiNode;
use bevy::picking::prelude::Pickable;
use bevy::input::keyboard::{Key, KeyboardInput};

use crate::ui::theme::palette::*;
use crate::ui::theme::interaction::InteractionPalette;

// ============================================================================
// COMPONENTS
// ============================================================================

/// Marker for text input fields.
#[derive(Component, Debug)]
pub struct TextInput;

/// Marker for the text display inside an input field.
#[derive(Component, Debug)]
pub struct TextInputDisplay;

/// Identifies which field an input controls with min/max constraints.
#[derive(Component, Debug, Clone, Copy)]
pub struct InputField<T: Send + Sync + 'static> {
    pub kind: T,
    pub min: f32,
    pub max: f32,
}

/// Resource to track currently focused text input and drag state.
#[derive(Resource, Default)]
pub struct TextInputFocus<T: Send + Sync + Default + Clone + 'static> {
    pub entity: Option<Entity>,
    pub buffer: String,
    pub field_kind: Option<T>,
    pub min: f32,
    pub max: f32,
    /// Drag-to-adjust state: entity being dragged (before confirming click vs drag).
    pub drag_entity: Option<Entity>,
    pub drag_field_kind: Option<T>,
    pub drag_origin_x: f32,
    pub drag_origin_value: f32,
    pub drag_min: f32,
    pub drag_max: f32,
    pub is_dragging: bool,
    pub drag_accumulated: f32,
}

// ============================================================================
// CONSTANTS
// ============================================================================

pub const INPUT_HEIGHT: f32 = 20.0;
pub const INPUT_WIDTH: f32 = 65.0;
pub const AXIS_STRIP_WIDTH: f32 = 4.0;
pub const INPUT_FONT_SIZE: f32 = 11.0;

/// Minimum pixel distance before a press becomes a drag instead of a click.
pub const DRAG_THRESHOLD: f32 = 3.0;
/// How many units per pixel of horizontal mouse movement during drag.
pub const DRAG_SENSITIVITY: f32 = 0.5;

// ============================================================================
// SPAWN HELPERS
// ============================================================================

/// Spawns an input with colored axis strip on the left.
pub fn spawn_axis_input<T: Component + Clone + Send + Sync + 'static>(
    parent: &mut ChildSpawnerCommands,
    axis_color: Color,
    value: f32,
    field: InputField<T>,
    width: f32,
) {
    // Container with colored strip + input field
    parent.spawn((
        UiNode {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            height: px(INPUT_HEIGHT),
            ..default()
        },
    )).with_children(|group| {
        // Colored axis strip (left side)
        group.spawn((
            UiNode {
                width: px(AXIS_STRIP_WIDTH),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::left(px(3.0)),
                ..default()
            },
            BackgroundColor(axis_color),
            Pickable::IGNORE,
        ));
        
        // Input field
        spawn_input_body(group, value, field, width, BorderRadius::right(px(3.0)));
    });
}

/// Spawns a numeric input field (without axis strip).
pub fn spawn_numeric_input<T: Component + Clone + Send + Sync + 'static>(
    parent: &mut ChildSpawnerCommands,
    value: f32,
    field: InputField<T>,
    width: f32,
) {
    spawn_input_body(parent, value, field, width, BorderRadius::all(px(3.0)));
}

/// Internal: spawns the actual input body with button and text.
fn spawn_input_body<T: Component + Clone + Send + Sync + 'static>(
    parent: &mut ChildSpawnerCommands,
    value: f32,
    field: InputField<T>,
    width: f32,
    border_radius: BorderRadius,
) {
    parent.spawn((
        TextInput,
        field.clone(),
        Button,
        UiNode {
            width: px(width),
            height: px(INPUT_HEIGHT),
            padding: UiRect::horizontal(px(6.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            border_radius,
            ..default()
        },
        BackgroundColor(INPUT_FIELD),
        InteractionPalette {
            none: INPUT_FIELD,
            hovered: INPUT_FIELD_HOVER,
            pressed: INPUT_FIELD_FOCUS,
            active: INPUT_FIELD_FOCUS,
        },
    )).with_children(|input| {
        input.spawn((
            TextInputDisplay,
            field,
            Text::new(format!("{:.2}", value)),
            TextFont::from_font_size(INPUT_FONT_SIZE),
            TextColor(TEXT),
            Pickable::IGNORE,
        ));
    });
}

// ============================================================================
// SYSTEMS (Generic)
// ============================================================================

/// Handle clicking on text input to focus it.
/// This is a generic system factory - call with your field type.
pub fn handle_text_input_focus_system<T: Component + Clone + Copy + Send + Sync + Default + PartialEq + 'static>(
    mut focus: ResMut<TextInputFocus<T>>,
    input_query: Query<(Entity, &Interaction, &InputField<T>), (Changed<Interaction>, With<TextInput>)>,
    text_query: Query<(&Text, &InputField<T>), With<TextInputDisplay>>,
) {
    for (entity, interaction, field) in input_query.iter() {
        if *interaction == Interaction::Pressed {
            // Find the current value from the field's text (match by field kind)
            let mut current_value = String::new();
            for (text, text_field) in text_query.iter() {
                if text_field.kind == field.kind {
                    current_value = text.0.clone();
                    break;
                }
            }
            
            focus.entity = Some(entity);
            focus.buffer = current_value;
            focus.field_kind = Some(field.kind);
            focus.min = field.min;
            focus.max = field.max;
        }
    }
}

/// Handle keyboard input for focused text field.
/// Returns Some((field_kind, clamped_value)) when Enter/Tab is pressed.
pub fn handle_text_input_keyboard_system<T: Component + Clone + Copy + Send + Sync + Default + PartialEq + 'static>(
    mut focus: ResMut<TextInputFocus<T>>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut text_query: Query<(&mut Text, &InputField<T>), With<TextInputDisplay>>,
) -> Option<(T, f32)> {
    let Some(_focused_entity) = focus.entity else { return None };
    let Some(field_kind) = focus.field_kind else { return None };
    
    let mut result = None;
    
    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        
        match &event.logical_key {
            Key::Character(c) => {
                let ch = c.as_str();
                // Allow digits, minus, period
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
                    let clamped = value.clamp(focus.min, focus.max);
                    focus.buffer = format!("{:.2}", clamped);
                    update_input_text(&mut text_query, field_kind, &focus.buffer);
                    result = Some((field_kind, clamped));
                }
            }
            Key::Escape => {
                focus.entity = None;
                focus.buffer.clear();
                focus.field_kind = None;
            }
            Key::Tab => {
                if let Ok(value) = focus.buffer.parse::<f32>() {
                    let clamped = value.clamp(focus.min, focus.max);
                    result = Some((field_kind, clamped));
                }
                focus.entity = None;
                focus.buffer.clear();
                focus.field_kind = None;
            }
            _ => {}
        }
    }
    
    result
}

/// Update text display by field kind.
fn update_input_text<T: Component + Clone + Copy + Send + Sync + PartialEq + 'static>(
    text_query: &mut Query<(&mut Text, &InputField<T>), With<TextInputDisplay>>,
    field_kind: T,
    new_text: &str,
) {
    for (mut text, field) in text_query.iter_mut() {
        if field.kind == field_kind {
            text.0 = if new_text.is_empty() { "0".to_string() } else { new_text.to_string() };
            break;
        }
    }
}

/// Visual feedback for focused input.
pub fn update_focused_input_style_system<T: Send + Sync + Default + Clone + 'static>(
    focus: Res<TextInputFocus<T>>,
    mut input_query: Query<(Entity, &mut BackgroundColor), With<TextInput>>,
) {
    for (entity, mut bg) in input_query.iter_mut() {
        if Some(entity) == focus.entity {
            *bg = BackgroundColor(INPUT_FIELD_FOCUS);
        } else {
            *bg = BackgroundColor(INPUT_FIELD);
        }
    }
}

/// Clear focus when clicking outside inputs.
/// Returns Some((field_kind, clamped_value)) if a value should be applied.
pub fn handle_click_outside_system<T: Clone + Copy + Send + Sync + Default + 'static>(
    mut focus: ResMut<TextInputFocus<T>>,
    mouse: Res<ButtonInput<MouseButton>>,
    input_query: Query<&Interaction, With<TextInput>>,
) -> Option<(T, f32)> {
    if !mouse.just_pressed(MouseButton::Left) {
        return None;
    }
    
    let clicking_input = input_query.iter().any(|i| *i != Interaction::None);
    
    if !clicking_input && focus.entity.is_some() {
        let result = if let Some(field_kind) = focus.field_kind {
            if let Ok(value) = focus.buffer.parse::<f32>() {
                let clamped = value.clamp(focus.min, focus.max);
                Some((field_kind, clamped))
            } else {
                None
            }
        } else {
            None
        };
        
        focus.entity = None;
        focus.buffer.clear();
        focus.field_kind = None;
        
        return result;
    }
    
    None
}

// ============================================================================
// HELPER
// ============================================================================

/// Shared helper for pixel values.
pub(crate) fn px(val: f32) -> Val {
    Val::Px(val)
}
