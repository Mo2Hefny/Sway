use bevy_egui::egui;

use crate::ui::state::*;
use crate::ui::theme::palette::*;
use crate::ui::constants::*;
use crate::ui::theme::*;
use crate::ui::icons::EguiIconTextures;

pub fn draw_playback_toolbar(
    ctx: &egui::Context,
    icons: &EguiIconTextures,
    playback: &mut PlaybackState,
) {
    let bottom_margin = BOTTOM_TOOLBAR_MARGIN;
    egui::Area::new(egui::Id::new("bottom_toolbar"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -bottom_margin))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let frame = egui::Frame::new()
                .fill(to_egui_color(SURFACE))
                .inner_margin(egui::Margin::same(BOTTOM_TOOLBAR_INNER_MARGIN as i8))
                .corner_radius(PANEL_CORNER_RADIUS);
            frame.show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(BOTTOM_TOOLBAR_ITEM_SPACING, BOTTOM_TOOLBAR_ITEM_SPACING);
                ui.horizontal(|ui| {
                    let play_btn = match icons.play {
                        Some(tid) => ui.add_sized(
                            egui::vec2(BOTTOM_TOOLBAR_BTN_SIZE, BOTTOM_TOOLBAR_BTN_SIZE),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                tid,
                                egui::vec2(BOTTOM_TOOLBAR_ICON_SIZE, BOTTOM_TOOLBAR_ICON_SIZE),
                            )))
                            .fill(to_egui_color(SURFACE)),
                        ),
                        None => ui.add_sized(
                            egui::vec2(BOTTOM_TOOLBAR_BTN_SIZE, BOTTOM_TOOLBAR_BTN_SIZE),
                            egui::Button::new("▶").fill(to_egui_color(SURFACE)),
                        ),
                    };
                    if play_btn.clicked() {
                        playback.play();
                    }
                    let pause_btn = match icons.pause {
                        Some(tid) => ui.add_sized(
                            egui::vec2(BOTTOM_TOOLBAR_BTN_SIZE, BOTTOM_TOOLBAR_BTN_SIZE),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                tid,
                                egui::vec2(BOTTOM_TOOLBAR_ICON_SIZE, BOTTOM_TOOLBAR_ICON_SIZE),
                            )))
                            .fill(to_egui_color(SURFACE)),
                        ),
                        None => ui.add_sized(
                            egui::vec2(BOTTOM_TOOLBAR_BTN_SIZE, BOTTOM_TOOLBAR_BTN_SIZE),
                            egui::Button::new("⏸").fill(to_egui_color(SURFACE)),
                        ),
                    };
                    if pause_btn.clicked() {
                        playback.pause();
                    }
                });
            });
        });
}
