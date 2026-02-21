use bevy::prelude::*;
use bevy_egui::egui;

use crate::ui::messages::*;
use crate::ui::constants::*;
use crate::ui::theme::*;

pub fn draw_instruction_hints(ctx: &egui::Context) {
    
    egui::Area::new(egui::Id::new("instructions"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(HINT_ANCHOR_OFFSET.0, HINT_ANCHOR_OFFSET.1))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(HINT_ITEM_SPACING.0, HINT_ITEM_SPACING.1);
            let hint = to_egui_color(Color::srgba(HINT_COLOR.0, HINT_COLOR.1, HINT_COLOR.2, HINT_COLOR.3));
            ui.label(egui::RichText::new(HINT_SELECT).color(hint).text_style(egui::TextStyle::Small));
            ui.label(egui::RichText::new(HINT_FOLLOW_NODE).color(hint).text_style(egui::TextStyle::Small));
            ui.label(egui::RichText::new(HINT_MOVE).color(hint).text_style(egui::TextStyle::Small));
            ui.label(egui::RichText::new(HINT_TOGGLE_UI).color(hint).text_style(egui::TextStyle::Small));
            ui.label(egui::RichText::new(HINT_PLAY).color(hint).text_style(egui::TextStyle::Small));
        });
}
