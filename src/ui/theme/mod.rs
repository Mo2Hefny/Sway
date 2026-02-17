pub mod palette;
pub mod layout;
pub mod widgets;
pub mod typography;

pub use palette::*;
use bevy_egui::egui;

pub fn apply_theme(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.visuals.dark_mode = true;
        layout::apply(style);
        widgets::apply(style);
        typography::apply(style);
    });
}

pub fn to_egui_color(c: bevy::prelude::Color) -> egui::Color32 {
    let srgba = c.to_srgba();
    egui::Color32::from_rgba_unmultiplied(
        (srgba.red * 255.0) as u8,
        (srgba.green * 255.0) as u8,
        (srgba.blue * 255.0) as u8,
        (srgba.alpha * 255.0) as u8,
    )
}
