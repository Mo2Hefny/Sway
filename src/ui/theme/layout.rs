use bevy_egui::egui;
use crate::ui::constants::*;

pub fn apply(style: &mut egui::Style) {
    style.spacing.item_spacing = egui::vec2(PANEL_ITEM_SPACING, PANEL_ITEM_SPACING);
    style.spacing.window_margin = egui::Margin::same(PANEL_INNER_MARGIN as i8);
}
