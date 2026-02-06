//! Reusable dropdown widget.

use bevy::prelude::*;
use bevy::ui::Node as UiNode;
use bevy::picking::prelude::Pickable;

use crate::ui::theme::palette::*;
use crate::ui::theme::interaction::InteractionPalette;
use super::text_input::{px, INPUT_HEIGHT, INPUT_FONT_SIZE};

// ============================================================================
// COMPONENTS
// ============================================================================

/// Marker for dropdown buttons.
#[derive(Component, Debug)]
pub struct DropdownButton;

/// Marker for dropdown menu container.
#[derive(Component, Debug)]
pub struct DropdownMenu;

/// Marker for dropdown option.
#[derive(Component, Debug)]
pub struct DropdownOption<T: Send + Sync + 'static>(pub T);

/// Resource to track if dropdown is open.
#[derive(Resource, Default)]
pub struct DropdownState {
    pub open: bool,
}

// ============================================================================
// SPAWN HELPERS
// ============================================================================

/// Configuration for spawning a dropdown.
pub struct DropdownConfig<T: Send + Sync + 'static> {
    pub width: f32,
    pub current: T,
    pub options: Vec<(T, String)>,
}

/// Spawns a dropdown with the given configuration.
/// The field parameter is optional for attaching a field component.
pub fn spawn_dropdown<T, F>(
    parent: &mut ChildSpawnerCommands,
    current_text: &str,
    field: Option<F>,
    width: f32,
) -> Entity 
where 
    T: Send + Sync + 'static,
    F: Component + Clone,
{
    let wrapper_entity = Entity::PLACEHOLDER;
    
    parent.spawn((
        Name::new("Dropdown Wrapper"),
        UiNode {
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )).with_children(|wrapper| {
        // Dropdown button
        let mut btn = wrapper.spawn((
            Name::new("Dropdown Button"),
            DropdownButton,
            Button,
            UiNode {
                width: px(width),
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
        ));
        
        if let Some(f) = field {
            btn.insert(f);
        }
        
        btn.with_children(|btn_content| {
            btn_content.spawn((
                Text::new(current_text),
                TextFont::from_font_size(INPUT_FONT_SIZE),
                TextColor(TEXT),
                Pickable::IGNORE,
            ));
            btn_content.spawn((
                Text::new("▼"),
                TextFont::from_font_size(8.0),
                TextColor(TEXT_SECONDARY),
                Pickable::IGNORE,
            ));
        });
        
        // Dropdown menu (hidden by default)
        wrapper.spawn((
            Name::new("Dropdown Menu"),
            DropdownMenu,
            UiNode {
                position_type: PositionType::Absolute,
                top: px(INPUT_HEIGHT + 2.0),
                right: px(0.0),
                width: px(width),
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
        ));
    });
    
    wrapper_entity
}

/// Spawns a dropdown option.
pub fn spawn_dropdown_option<T: Component + Clone>(
    parent: &mut ChildSpawnerCommands,
    option_value: T,
    display_text: &str,
    is_selected: bool,
) {
    let bg = if is_selected { SURFACE_HOVER } else { SURFACE };
    parent.spawn((
        Name::new(format!("Option: {}", display_text)),
        DropdownOption(option_value),
        Button,
        UiNode {
            width: Val::Percent(100.0),
            height: px(22.0),
            padding: UiRect::horizontal(px(8.0)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(bg),
        InteractionPalette {
            none: bg,
            hovered: SURFACE_HOVER,
            pressed: SURFACE_PRESSED,
            active: ACCENT,
        },
    )).with_children(|opt| {
        opt.spawn((
            Text::new(display_text),
            TextFont::from_font_size(INPUT_FONT_SIZE),
            TextColor(if is_selected { ACCENT } else { TEXT }),
            Pickable::IGNORE,
        ));
    });
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// Toggles dropdown menu visibility when button is clicked.
pub fn handle_dropdown_toggle(
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

/// Closes dropdown menu.
pub fn close_dropdown(
    dropdown_state: &mut ResMut<DropdownState>,
    menu_query: &mut Query<&mut UiNode, With<DropdownMenu>>,
) {
    dropdown_state.open = false;
    for mut node in menu_query.iter_mut() {
        node.display = Display::None;
    }
}
