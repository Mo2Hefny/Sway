use bevy::prelude::*;
use bevy_egui::egui;
use bevy::window::PrimaryWindow;

use crate::core::{
    build_scene_data, export_to_file, import_from_file, Playground, Node as SimNode, DistanceConstraint,
};
use crate::core::components::LimbSet;
use crate::ui::state::*;
use crate::ui::theme::palette::*;
use crate::ui::messages::*;
use crate::ui::icons::{EguiIconTextures, to_egui_color};

pub fn draw_floating_panel(
    ctx: &egui::Context,
    icons: &EguiIconTextures,
    panel_state: &mut FloatingPanelState,
    display_settings: &mut DisplaySettings,
    import_requested: &mut ImportRequested,
    playground: &mut Playground,
    node_query: &Query<(Entity, &mut SimNode)>,
    constraint_query: &Query<(Entity, &DistanceConstraint)>,
    limb_set_query: &mut Query<(Entity, &mut LimbSet)>,
    windows: &Query<&Window, With<PrimaryWindow>>,
) {
    let screen = ctx.available_rect();
    let left = screen.min.x;
    let top = screen.min.y;

    let panel_x = left + 16.0;
    let panel_y = top + 16.0;
    let floating_panel_w = if panel_state.collapsed { 56.0 } else { 240.0 };
    let surface = to_egui_color(SURFACE);

    egui::Area::new(egui::Id::new("floating_panel"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(panel_x, panel_y))
        .order(egui::Order::Foreground)
        .constrain(true)
        .show(ctx, |ui| {
            ui.set_min_width(floating_panel_w);
            let frame = egui::Frame::new()
                .fill(surface)
                .inner_margin(egui::Margin::same(12))
                .corner_radius(4.0);
            frame.show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);

                // Header: hamburger
                let hamburger_btn = if let Some(tid) = icons.hamburger {
                    ui.add_sized(
                        egui::vec2(32.0, 32.0),
                        egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                            tid,
                            egui::vec2(18.0, 18.0),
                        )))
                        .fill(surface),
                    )
                } else {
                    ui.add_sized(egui::vec2(32.0, 32.0), egui::Button::new("≡").fill(surface))
                };
                if hamburger_btn.clicked() {
                    panel_state.collapsed = !panel_state.collapsed;
                }

                if !panel_state.collapsed {
                    ui.add_space(8.0);
                    let mut skin = display_settings.show_skin;
                    if ui.checkbox(&mut skin, LABEL_SHOW_SKIN).changed() {
                        display_settings.show_skin = skin;
                    }
                    let mut edge = display_settings.show_edge;
                    if ui.checkbox(&mut edge, LABEL_SHOW_EDGE).changed() {
                        display_settings.show_edge = edge;
                    }
                    let mut nodes = display_settings.show_nodes;
                    if ui.checkbox(&mut nodes, LABEL_SHOW_NODES).changed() {
                        display_settings.show_nodes = nodes;
                    }
                    let mut debug = display_settings.show_debug;
                    if ui.checkbox(&mut debug, LABEL_SHOW_DEBUG).changed() {
                        display_settings.show_debug = debug;
                    }
                    ui.add_space(16.0);
                    if ui.button(BTN_IMPORT).clicked() {
                        if let Some(scene) = import_from_file() {
                            import_requested.0 = Some(scene);
                        }
                    }
                    if ui.button(BTN_EXPORT).clicked() {
                        let scene = build_scene_data(node_query, constraint_query, limb_set_query);
                        export_to_file(&scene);
                    }

                    // Playground size slider
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(LABEL_PLAYGROUND_SIZE);

                    // Aspect ratio logic
                    let Ok(window) = windows.single() else { return };
                    let aspect = window.width() / window.height();

                    // Control half-height, derive half-width from aspect ratio
                    let mut half_height = playground.half_size.y;

                    if ui
                        .add(egui::Slider::new(&mut half_height, 400.0..=2000.0).text("Half Size"))
                        .changed()
                    {
                        let half_width = half_height * aspect;
                        playground.half_size = Vec2::new(half_width, half_height);
                    }
                }
            });
        });
}
