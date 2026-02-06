//! Reusable function expression input widget.

use bevy::prelude::*;
use bevy::ui::Node as UiNode;
use bevy::picking::prelude::Pickable;

use crate::ui::theme::palette::*;
use crate::ui::theme::interaction::InteractionPalette;

use super::text_input::{INPUT_HEIGHT, INPUT_FONT_SIZE};

/// Marker for function expression input fields.
#[derive(Component, Debug)]
pub struct FunctionInput;

/// Marker for the text display inside a function input.
#[derive(Component, Debug)]
pub struct FunctionInputDisplay;

/// Identifies which function field this input controls.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionField<T: Send + Sync + 'static> {
    pub kind: T,
}

/// Resource to track currently focused function input.
#[derive(Resource, Default)]
pub struct FunctionInputFocus<T: Send + Sync + Default + Clone + 'static> {
    pub entity: Option<Entity>,
    pub buffer: String,
    pub field_kind: Option<T>,
}

fn px(val: f32) -> Val {
    Val::Px(val)
}

/// Spawns a function expression text input with a placeholder.
pub fn spawn_function_input<T: Component + Clone + Send + Sync + 'static>(
    parent: &mut ChildSpawnerCommands,
    current_expr: &str,
    field: FunctionField<T>,
    width: f32,
) {
    parent.spawn((
        FunctionInput,
        field.clone(),
        Button,
        UiNode {
            width: px(width),
            height: px(INPUT_HEIGHT),
            padding: UiRect::horizontal(px(6.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
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
    )).with_children(|input| {
        let display_text = if current_expr.is_empty() {
            "f(t)".to_string()
        } else {
            current_expr.to_string()
        };
        let text_color = if current_expr.is_empty() {
            TEXT_DISABLED
        } else {
            TEXT
        };
        input.spawn((
            FunctionInputDisplay,
            field,
            Text::new(display_text),
            TextFont::from_font_size(INPUT_FONT_SIZE),
            TextColor(text_color),
            Pickable::IGNORE,
        ));
    });
}
