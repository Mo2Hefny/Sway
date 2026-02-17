use bevy_egui::egui;

use crate::ui::state::*;
use crate::ui::theme::palette::*;
use crate::ui::icons::{EguiIconTextures, to_egui_color};

pub fn draw_playback_toolbar(
    ctx: &egui::Context,
    icons: &EguiIconTextures,
    playback: &mut PlaybackState,
) {
    let bottom_margin = 24.0;
    egui::Area::new(egui::Id::new("bottom_toolbar"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -bottom_margin))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let frame = egui::Frame::new()
                .fill(to_egui_color(SURFACE))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(4.0);
            frame.show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(6.0, 6.0);
                ui.horizontal(|ui| {
                    let play_btn = match icons.play {
                        Some(tid) => ui.add_sized(
                            egui::vec2(32.0, 32.0),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                tid,
                                egui::vec2(16.0, 16.0),
                            )))
                            .fill(to_egui_color(SURFACE)),
                        ),
                        None => ui.add_sized(
                            egui::vec2(32.0, 32.0),
                            egui::Button::new("▶").fill(to_egui_color(SURFACE)),
                        ),
                    };
                    if play_btn.clicked() {
                        playback.play();
                    }
                    let pause_btn = match icons.pause {
                        Some(tid) => ui.add_sized(
                            egui::vec2(32.0, 32.0),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                tid,
                                egui::vec2(16.0, 16.0),
                            )))
                            .fill(to_egui_color(SURFACE)),
                        ),
                        None => ui.add_sized(
                            egui::vec2(32.0, 32.0),
                            egui::Button::new("⏸").fill(to_egui_color(SURFACE)),
                        ),
                    };
                    if pause_btn.clicked() {
                        playback.pause();
                    }
                    let stop_btn = match icons.stop {
                        Some(tid) => ui.add_sized(
                            egui::vec2(32.0, 32.0),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                tid,
                                egui::vec2(16.0, 16.0),
                            )))
                            .fill(to_egui_color(SURFACE)),
                        ),
                        None => ui.add_sized(
                            egui::vec2(32.0, 32.0),
                            egui::Button::new("⏹").fill(to_egui_color(SURFACE)),
                        ),
                    };
                    if stop_btn.clicked() {
                        playback.stop();
                    }
                });
            });
        });
}
