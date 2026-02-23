use bevy::prelude::*;
use bevy_egui::egui;

use crate::ui::messages::*;
use crate::ui::constants::*;
use crate::ui::theme::*;

pub fn draw_instruction_hints(ctx: &egui::Context) {
    
    egui::Area::new(egui::Id::new("instructions"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(HINT_ANCHOR_OFFSET.0, HINT_ANCHOR_OFFSET.1))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(HINT_ITEM_SPACING.0, HINT_ITEM_SPACING.1);
            let hint_color = to_egui_color(Color::srgba(HINT_COLOR.0, HINT_COLOR.1, HINT_COLOR.2, HINT_COLOR.3));
            
            let label = |text: &str| {
                egui::Label::new(
                    egui::RichText::new(text)
                        .color(hint_color)
                        .size(16.0)
                )
                .selectable(false)
            };

            ui.add(label(HINT_FOLLOW_NODE));
            ui.add(label(HINT_TOGGLE_UI));
            ui.add(label(HINT_PLAY));
        });
}
