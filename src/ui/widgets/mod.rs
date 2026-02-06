//! Reusable UI widgets and marker components.

mod text_input;
mod checkbox;
mod dropdown;
mod function_input;
mod markers;

pub use text_input::*;
pub use checkbox::*;
pub use dropdown::*;
pub use function_input::*;
pub use markers::*;

use bevy::prelude::*;

use super::theme::interaction::InteractionPalette;

/// Updates button colors based on both interaction state (hover/press) and active state.
pub fn update_interaction_colors(
    mut query: Query<
        (
            &Interaction,
            &InteractionPalette,
            &mut BackgroundColor,
            Option<&super::theme::interaction::Active>,
        ),
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
