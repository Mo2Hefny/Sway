use bevy_egui::egui;
use crate::ui::theme::to_egui_color;
use crate::ui::theme::palette::*;

pub fn apply(style: &mut egui::Style) {
    use egui::{FontId, FontFamily::Proportional};

    style.text_styles = [
        (egui::TextStyle::Heading, FontId::new(14.0, Proportional)),
        (egui::TextStyle::Body, FontId::new(12.0, Proportional)),
        (egui::TextStyle::Button, FontId::new(12.0, Proportional)),
        (egui::TextStyle::Small, FontId::new(10.0, Proportional)),
    ].into();

    style.visuals.override_text_color = Some(to_egui_color(TEXT_SECONDARY));
}

pub fn heading_color() -> egui::Color32 {
    to_egui_color(TEXT)
}

pub fn body_color() -> egui::Color32 {
    to_egui_color(TEXT)
}

pub fn subinfo_color() -> egui::Color32 {
    to_egui_color(TEXT_SECONDARY)
}
