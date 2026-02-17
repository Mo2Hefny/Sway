use bevy::prelude::*;
use bevy_egui::egui;

use crate::ui::messages::*;
use crate::ui::icons::to_egui_color;

pub fn draw_instruction_hints(ctx: &egui::Context) {
    // Re-check original code:
    // .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(left + 16.0, bottom - 16.0))
    
    egui::Area::new(egui::Id::new("instructions"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(16.0, -16.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 2.0);
            let hint = to_egui_color(Color::srgba(0.6, 0.6, 0.6, 0.5));
            ui.label(egui::RichText::new(HINT_SELECT).color(hint));
            ui.label(egui::RichText::new(HINT_ADD_NODE).color(hint));
            ui.label(egui::RichText::new(HINT_ADD_EDGE).color(hint));
            ui.label(egui::RichText::new(HINT_MOVE).color(hint));
            ui.label(egui::RichText::new(HINT_TOGGLE_UI).color(hint));
            ui.label(egui::RichText::new(HINT_PLAY).color(hint));
            ui.label(egui::RichText::new(HINT_STOP).color(hint));
        });
}
