use bevy::prelude::*;
use bevy_egui::egui;
use bevy::window::PrimaryWindow;

use crate::core::{
    build_scene_data, deserialize_scene, export_to_file, import_from_file, Playground, Node as SimNode, DistanceConstraint, EXAMPLES,
};
use crate::core::components::LimbSet;
use crate::ui::state::*;
use crate::ui::theme::palette::*;
use crate::ui::messages::*;
use crate::ui::constants::*;
use crate::ui::theme::*;
use crate::ui::icons::EguiIconTextures;

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

    let panel_x = left + PANEL_INNER_MARGIN;
    let panel_y = top + PANEL_INNER_MARGIN;
    let floating_panel_w = if panel_state.collapsed { FLOATING_PANEL_WIDTH_COLLAPSED } else { FLOATING_PANEL_WIDTH };
    let surface = to_egui_color(SURFACE);

    egui::Area::new(egui::Id::new("floating_panel"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(panel_x, panel_y))
        .order(egui::Order::Foreground)
        .constrain(true)
        .show(ctx, |ui| {
            ui.allocate_ui(egui::vec2(floating_panel_w, 0.0), |ui| {
                let frame = egui::Frame::new()
                    .fill(surface)
                    .inner_margin(egui::Margin::same(FLOATING_PANEL_INNER_MARGIN as i8))
                    .corner_radius(PANEL_CORNER_RADIUS);
                frame.show(ui, |ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(PANEL_ITEM_SPACING, PANEL_ITEM_SPACING);

                    let hamburger_btn = if let Some(tid) = icons.hamburger {
                        ui.add_sized(
                            egui::vec2(HAMBURGER_BTN_SIZE, HAMBURGER_BTN_SIZE),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                tid,
                                egui::vec2(HAMBURGER_ICON_SIZE, HAMBURGER_ICON_SIZE),
                            )))
                            .fill(surface),
                        )
                    } else {
                        ui.add_sized(
                            egui::vec2(HAMBURGER_BTN_SIZE, HAMBURGER_BTN_SIZE),
                            egui::Button::new("≡").fill(surface),
                        )
                    };
                    if hamburger_btn.clicked() {
                        panel_state.collapsed = !panel_state.collapsed;
                    }

                    if !panel_state.collapsed {
                        ui.add_space(PANEL_TITLE_SPACING);
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
                        ui.add_space(PANEL_SECTION_SPACING);
                        if ui.button(BTN_IMPORT).clicked() {
                            if let Some(scene) = import_from_file() {
                                import_requested.0 = Some(scene);
                            }
                        }
                        if ui.button(BTN_EXPORT).clicked() {
                            let scene = build_scene_data(node_query, constraint_query, limb_set_query);
                            export_to_file(&scene);
                        }

                        ui.add_space(PANEL_SECTION_SPACING);
                        ui.label(egui::RichText::new("Examples")
                            .text_style(egui::TextStyle::Heading)
                            .color(typography::heading_color()));
                        
                        let mut selected = panel_state.selected_example.unwrap_or(0);
                        let mut changed = false;

                        egui::ComboBox::from_id_salt("example_selector")
                            .selected_text(panel_state.selected_example.map(|i| EXAMPLES[i].0).unwrap_or("Select Example..."))
                            .show_ui(ui, |ui| {
                                for (i, (name, _)) in EXAMPLES.iter().enumerate() {
                                    if ui.selectable_value(&mut selected, i, *name).clicked() {
                                        panel_state.selected_example = Some(i);
                                        changed = true;
                                    }
                                }
                            });

                        if changed {
                            if let Some(i) = panel_state.selected_example {
                                if let Some(scene) = deserialize_scene(EXAMPLES[i].1) {
                                    import_requested.0 = Some(scene);
                                }
                            }
                        }

                        ui.add_space(PANEL_SECTION_SPACING);
                        ui.separator();
                        ui.add_space(PANEL_TITLE_SPACING);
                        ui.label(egui::RichText::new(LABEL_PLAYGROUND_SIZE)
                            .text_style(egui::TextStyle::Heading)
                            .color(typography::heading_color()));

                        let Ok(window) = windows.single() else { return };
                        let aspect = window.width() / window.height();

                        let mut half_height = playground.half_size.y;

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(LABEL_HALF_HEIGHT).text_style(egui::TextStyle::Small).color(typography::subinfo_color()));
                            if ui
                                .add(egui::Slider::new(&mut half_height, PLAYGROUND_HALF_SIZE_RANGE))
                                .changed()
                            {
                                let half_width = half_height * aspect;
                                playground.half_size = Vec2::new(half_width, half_height);
                            }
                        });
                    }
                });
            });
        });
}
