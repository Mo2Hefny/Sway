use bevy_egui::egui;
use crate::ui::theme::palette::*;
use crate::ui::theme::to_egui_color;

pub fn apply(style: &mut egui::Style) {
    let widgets = &mut style.visuals.widgets;

    widgets.noninteractive.bg_fill = to_egui_color(SURFACE);
    widgets.noninteractive.corner_radius = 4.0.into();
    widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, to_egui_color(TEXT_SECONDARY));

    widgets.inactive.bg_fill = to_egui_color(INPUT_FIELD);
    widgets.inactive.corner_radius = 4.0.into();
    widgets.inactive.fg_stroke = egui::Stroke::new(1.0, to_egui_color(TEXT_SECONDARY));

    widgets.hovered.bg_fill = to_egui_color(INPUT_FIELD_HOVER);
    widgets.hovered.corner_radius = 4.0.into();
    widgets.hovered.fg_stroke = egui::Stroke::new(1.0, to_egui_color(TEXT));

    widgets.active.bg_fill = to_egui_color(INPUT_FIELD_FOCUS);
    widgets.active.corner_radius = 4.0.into();
    widgets.active.fg_stroke = egui::Stroke::new(1.0, to_egui_color(TEXT));

    style.visuals.selection.bg_fill = to_egui_color(INPUT_FIELD_FOCUS);

    style.spacing.button_padding = egui::vec2(6.0, 3.0);
    style.spacing.slider_width = 120.0;
    style.visuals.widgets.inactive.corner_radius = 4.0.into();
    style.visuals.widgets.hovered.corner_radius = 4.0.into();
    style.visuals.widgets.active.corner_radius = 4.0.into();
}
