//! Reusable checkbox widget.

use bevy::prelude::*;
use bevy::ui::Node as UiNode;
use bevy::picking::prelude::Pickable;

use crate::ui::theme::palette::*;
use crate::ui::theme::interaction::InteractionPalette;
use super::text_input::px;

// ============================================================================
// COMPONENTS
// ============================================================================

/// Marker for checkbox buttons.
#[derive(Component, Debug)]
pub struct Checkbox;

/// Marker for the checkmark text inside a checkbox.
#[derive(Component, Debug)]
pub struct CheckboxMark;

// ============================================================================
// CONSTANTS
// ============================================================================

pub const CHECKBOX_SIZE: f32 = 18.0;

// ============================================================================
// SPAWN HELPERS
// ============================================================================

/// Spawns a checkbox with optional field component.
pub fn spawn_checkbox<T: Component + Clone>(
    parent: &mut ChildSpawnerCommands,
    checked: bool,
    field: Option<T>,
) {
    let bg = if checked { ACCENT } else { INPUT_FIELD };
    let border_color = if checked { ACCENT } else { BORDER };
    
    let mut entity = parent.spawn((
        Checkbox,
        Button,
        UiNode {
            width: px(CHECKBOX_SIZE),
            height: px(CHECKBOX_SIZE),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(3.0)),
            border: UiRect::all(px(1.0)),
            ..default()
        },
        BorderColor::all(border_color),
        BackgroundColor(bg),
        InteractionPalette {
            none: bg,
            hovered: if checked { ACCENT_HOVER } else { INPUT_FIELD_HOVER },
            pressed: if checked { ACCENT_PRESSED } else { INPUT_FIELD_FOCUS },
            active: ACCENT,
        },
    ));
    
    if let Some(f) = field {
        entity.insert(f);
    }
    
    entity.with_children(|checkbox| {
        let visibility = if checked { Visibility::Inherited } else { Visibility::Hidden };
        checkbox.spawn((
            CheckboxMark,
            Text::new("✓"),
            TextFont::from_font_size(12.0),
            TextColor(TEXT),
            visibility,
            Pickable::IGNORE,
        ));
    });
}

/// Spawns a simple checkbox without field tracking.
pub fn spawn_simple_checkbox(parent: &mut ChildSpawnerCommands, checked: bool) {
    let bg = if checked { ACCENT } else { INPUT_FIELD };
    let border_color = if checked { ACCENT } else { BORDER };
    
    parent.spawn((
        Checkbox,
        Button,
        UiNode {
            width: px(CHECKBOX_SIZE),
            height: px(CHECKBOX_SIZE),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(3.0)),
            border: UiRect::all(px(1.0)),
            ..default()
        },
        BorderColor::all(border_color),
        BackgroundColor(bg),
        InteractionPalette {
            none: bg,
            hovered: if checked { ACCENT_HOVER } else { INPUT_FIELD_HOVER },
            pressed: if checked { ACCENT_PRESSED } else { INPUT_FIELD_FOCUS },
            active: ACCENT,
        },
    )).with_children(|checkbox| {
        let visibility = if checked { Visibility::Inherited } else { Visibility::Hidden };
        checkbox.spawn((
            CheckboxMark,
            Text::new("✓"),
            TextFont::from_font_size(12.0),
            TextColor(TEXT),
            visibility,
            Pickable::IGNORE,
        ));
    });
}
