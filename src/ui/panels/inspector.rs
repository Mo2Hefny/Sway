use bevy::prelude::*;
use bevy_egui::egui;

use crate::core::constants::{MAX_CONSTRAINT_DISTANCE, MIN_CONSTRAINT_DISTANCE};
use crate::core::components::LimbSet;
use crate::core::{
    DistanceConstraint, Node as SimNode, NodeType, Playground, AnchorMovementMode, ProceduralPathType,
};
use crate::editor::tools::selection::Selection;
use crate::ui::state::*;
use crate::ui::theme::palette::*;
use crate::ui::messages::*;
use crate::ui::icons::{EguiIconTextures, to_egui_color};

pub fn draw_inspector_panel(
    ctx: &egui::Context,
    icons: &EguiIconTextures,
    inspector_state: &mut InspectorState,
    selection: &Selection,
    playground: &Playground,
    node_query: &mut Query<(Entity, &mut SimNode)>,
    limb_set_query: &mut Query<(Entity, &mut LimbSet)>,
    constraint_query: &Query<(Entity, &DistanceConstraint)>,
    pending_actions: &mut PendingConstraintActions,
) {
    let screen = ctx.available_rect();
    let right = screen.max.x;
    let top = screen.min.y;
    let bottom = screen.max.y;

    let icon_bar_w = 48.0;
    let inspector_w = 280.0;
    let panel_height = bottom - top;

    let icon_bar_x = right - icon_bar_w;
    let inspector_x = icon_bar_x - inspector_w;

    let mut constraint_updates: Vec<(Entity, f32)> = Vec::new();
    let mut constraint_deletes: Vec<Entity> = Vec::new();
    let mut node_deletes: Vec<Entity> = Vec::new();

    if inspector_state.open {
        egui::Area::new(egui::Id::new("inspector_panel"))
            .fixed_pos(egui::pos2(inspector_x, top))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.allocate_ui(egui::vec2(inspector_w, panel_height), |ui| {
                    let frame = egui::Frame::new()
                        .fill(to_egui_color(SURFACE))
                        .inner_margin(egui::Margin::same(16))
                        .corner_radius(4.0);
                    frame.show(ui, |ui| {
                        ui.set_min_size(egui::vec2(inspector_w - 32.0, panel_height - 32.0));
                        ui.heading(inspector_state.active_page.name());
                        ui.add_space(16.0);
                        inspector_content_ui(
                            ui,
                            selection,
                            playground,
                            inspector_state.active_page,
                            node_query,
                            limb_set_query,
                            constraint_query,
                            &mut constraint_updates,
                            &mut constraint_deletes,
                            &mut node_deletes,
                        );
                    });
                });
            });
    }

    pending_actions.updates.extend(constraint_updates);
    pending_actions.deletes.extend(constraint_deletes);
    pending_actions.node_deletes.extend(node_deletes);

    // Inspector icon bar
    egui::Area::new(egui::Id::new("inspector_icon_bar"))
        .fixed_pos(egui::pos2(icon_bar_x, top))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.allocate_ui(egui::vec2(icon_bar_w, panel_height), |ui| {
                let frame = egui::Frame::new()
                    .fill(to_egui_color(SURFACE))
                    .inner_margin(egui::Margin::same(4))
                    .corner_radius(4.0);
                frame.show(ui, |ui| {
                    ui.set_min_size(egui::vec2(icon_bar_w - 8.0, panel_height - 8.0));
                    ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 4.0);
                    let caret_id = if inspector_state.open {
                        icons.caret_right
                    } else {
                        icons.caret_left
                    };
                    let caret_btn = match caret_id {
                        Some(tid) => ui.add_sized(
                            egui::vec2(40.0, 40.0),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                tid,
                                egui::vec2(24.0, 24.0),
                            )))
                            .fill(to_egui_color(SURFACE)),
                        ),
                        None => ui.add_sized(
                            egui::vec2(40.0, 40.0),
                            egui::Button::new("◀").fill(to_egui_color(SURFACE)),
                        ),
                    };
                    if caret_btn.clicked() {
                        inspector_state.open = !inspector_state.open;
                    }
                    ui.separator();
                    for page in [
                        InspectorPage::Properties,
                        InspectorPage::Transform,
                        InspectorPage::Constraints,
                    ] {
                        let active = inspector_state.active_page == page;
                        let fill = if active {
                            to_egui_color(SURFACE_HOVER)
                        } else {
                            to_egui_color(SURFACE)
                        };
                        let icon = match page {
                            InspectorPage::Properties => icons.properties,
                            InspectorPage::Transform => icons.transform,
                            InspectorPage::Constraints => icons.constraints,
                        };
                        let page_btn = match icon {
                            Some(tid) => ui.add_sized(
                                egui::vec2(40.0, 40.0),
                                egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                    tid,
                                    egui::vec2(24.0, 24.0),
                                )))
                                .fill(fill),
                            ),
                            None => ui.add_sized(egui::vec2(40.0, 40.0), egui::Button::new("").fill(fill)),
                        };
                        if page_btn.clicked() {
                            inspector_state.active_page = page;
                        }
                    }
                });
            });
        });
}

fn inspector_content_ui(
    ui: &mut egui::Ui,
    selection: &Selection,
    _playground: &Playground,
    page: InspectorPage,
    node_query: &mut Query<(Entity, &mut SimNode)>,
    limb_set_query: &mut Query<(Entity, &mut LimbSet)>,
    constraint_query: &Query<(Entity, &DistanceConstraint)>,
    constraint_updates: &mut Vec<(Entity, f32)>,
    constraint_deletes: &mut Vec<Entity>,
    node_deletes: &mut Vec<Entity>,
) {
    let Some(selected_entity) = selection.entity else {
        ui.colored_label(to_egui_color(TEXT_SECONDARY), PLACEHOLDER_NO_SELECTION);
        return;
    };
    let Ok((_, mut node)) = node_query.get_mut(selected_entity) else {
        ui.colored_label(to_egui_color(TEXT_SECONDARY), PLACEHOLDER_NO_SELECTION);
        return;
    };

    match page {
        InspectorPage::Properties => {
            ui.collapsing("Node Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Node Type");
                    egui::ComboBox::from_id_salt("node_type")
                        .selected_text(node.node_type.name())
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(node.node_type == NodeType::Normal, NodeType::Normal.name())
                                .clicked()
                            {
                                node.node_type = NodeType::Normal;
                            }
                            if ui
                                .selectable_label(node.node_type == NodeType::Anchor, NodeType::Anchor.name())
                                .clicked()
                            {
                                node.node_type = NodeType::Anchor;
                            }
                            if ui
                                .selectable_label(node.node_type == NodeType::Limb, NodeType::Limb.name())
                                .clicked()
                            {
                                node.node_type = NodeType::Limb;
                            }
                        });
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Angle Limit");
                    let mut angle_deg = node.angle_constraint.to_degrees();
                    if ui.add(egui::Slider::new(&mut angle_deg, 0.0..=180.0)).changed() {
                        node.angle_constraint = angle_deg.to_radians();
                    }
                    ui.label("°");
                });

                if let Ok((_e, mut limb_set)) = limb_set_query.get_mut(selected_entity) {
                    let limb_count = limb_set.limbs.len();
                    for (i, limb) in limb_set.limbs.iter_mut().enumerate() {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        let label = if limb_count == 1 {
                            "Limb IK".to_string()
                        } else {
                            format!("Limb {} IK", i + 1)
                        };
                        ui.label(&label);
                        ui.label(format!("Joints: {}", limb.joints.len()));
                        ui.horizontal(|ui| {
                            ui.label("Max Reach");
                            ui.add(egui::Slider::new(&mut limb.max_reach, 10.0..=1000.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Angle Offset:");
                            let mut deg = limb.target_direction_offset.to_degrees();
                            if ui.add(egui::DragValue::new(&mut deg).speed(1.0).suffix("°")).changed() {
                                limb.target_direction_offset = deg.to_radians();
                            }
                        });

                        ui.heading("Stepping");
                        ui.horizontal(|ui| {
                            ui.label("Threshold:");
                            ui.add(egui::DragValue::new(&mut limb.step_threshold).speed(0.1).range(0.0..=200.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Speed:");
                            ui.add(egui::DragValue::new(&mut limb.step_speed).speed(0.1).range(0.1..=20.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Height:");
                            ui.add(egui::DragValue::new(&mut limb.step_height).speed(0.1).range(0.0..=100.0));
                        });
                        
                        ui.add_space(4.0);
                        ui.label("Joint Flip:");
                        for (j, flipped) in limb.flip_bend.iter_mut().enumerate() {
                             ui.checkbox(flipped, format!("Joint {}", j + 1));
                        }
                    }
                }

                if node.node_type == NodeType::Anchor {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Movement Mode selector
                    ui.horizontal(|ui| {
                        ui.label("Movement Mode");
                        egui::ComboBox::from_id_salt("movement_mode")
                            .selected_text(node.movement_mode.name())
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(
                                        node.movement_mode == AnchorMovementMode::None,
                                        AnchorMovementMode::None.name(),
                                    )
                                    .clicked()
                                {
                                    node.movement_mode = AnchorMovementMode::None;
                                }
                                if ui
                                    .selectable_label(
                                        node.movement_mode == AnchorMovementMode::FollowTarget,
                                        AnchorMovementMode::FollowTarget.name(),
                                    )
                                    .clicked()
                                {
                                    node.movement_mode = AnchorMovementMode::FollowTarget;
                                }
                                if ui
                                    .selectable_label(
                                        node.movement_mode == AnchorMovementMode::Procedural,
                                        AnchorMovementMode::Procedural.name(),
                                    )
                                    .clicked()
                                {
                                    node.movement_mode = AnchorMovementMode::Procedural;
                                    // Initialize path center to current position when switching to procedural
                                    if node.path_center == bevy::math::Vec2::ZERO {
                                        node.path_center = node.position;
                                    }
                                }
                            });
                    });

                    match node.movement_mode {
                        AnchorMovementMode::None => { }
                        AnchorMovementMode::FollowTarget => {
                            ui.horizontal(|ui| {
                                ui.label("Movement Speed");
                                ui.add(egui::Slider::new(&mut node.movement_speed, 1.0..=50.0));
                            });
                        }
                        AnchorMovementMode::Procedural => {
                            // Path Type selector
                            ui.horizontal(|ui| {
                                ui.label("Path Type");
                                egui::ComboBox::from_id_salt("path_type")
                                    .selected_text(match node.path_type {
                                        ProceduralPathType::Circle => "Circle",
                                        ProceduralPathType::Wave => "Wave",
                                        ProceduralPathType::Wander => "Wander",
                                    })
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(node.path_type == ProceduralPathType::Circle, "Circle")
                                            .clicked()
                                        {
                                            node.path_type = ProceduralPathType::Circle;
                                        }
                                        if ui
                                            .selectable_label(node.path_type == ProceduralPathType::Wave, "Wave")
                                            .clicked()
                                        {
                                            node.path_type = ProceduralPathType::Wave;
                                        }
                                        if ui
                                            .selectable_label(node.path_type == ProceduralPathType::Wander, "Wander")
                                            .clicked()
                                        {
                                            node.path_type = ProceduralPathType::Wander;
                                        }
                                    });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Amplitude X");
                                ui.add(egui::Slider::new(&mut node.path_amplitude.x, 10.0..=200.0));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Amplitude Y");
                                ui.add(egui::Slider::new(&mut node.path_amplitude.y, 10.0..=200.0));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Movement Speed");
                                ui.add(egui::Slider::new(&mut node.movement_speed, 1.0..=50.0));
                            });

                            if ui.button("Set Center to Current Position").clicked() {
                                node.path_center = node.position;
                            }
                        }
                    }
                }
            });

            ui.collapsing("Physics", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Collision Damp");
                    ui.add(egui::Slider::new(&mut node.collision_damping, 0.0..=1.0));
                });
            });

            ui.collapsing("Acceleration", |ui| {
                ui.label("Constant");
                ui.horizontal(|ui| {
                    ui.colored_label(to_egui_color(AXIS_X), "X");
                    ui.add(egui::DragValue::new(&mut node.constant_acceleration.x).speed(1.0));
                    ui.add_space(8.0);
                    ui.colored_label(to_egui_color(AXIS_Y), "Y");
                    ui.add(egui::DragValue::new(&mut node.constant_acceleration.y).speed(1.0));
                });

                ui.add_space(4.0);
                ui.label("Live (Accumulated)");
                ui.horizontal(|ui| {
                    ui.colored_label(to_egui_color(AXIS_X), "X");
                    ui.label(format!("{:.2}", node.acceleration.x));
                    ui.add_space(16.0);
                    ui.colored_label(to_egui_color(AXIS_Y), "Y");
                    ui.label(format!("{:.2}", node.acceleration.y));
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let delete_btn = ui.add_sized(
                [ui.available_width(), 32.0],
                egui::Button::new(
                    egui::RichText::new("Delete Node").color(to_egui_color(Color::srgba(1.0, 0.4, 0.4, 1.0))),
                )
                .fill(to_egui_color(Color::srgba(0.8, 0.2, 0.2, 0.1))),
            );
            if delete_btn.clicked() {
                node_deletes.push(selected_entity);
            }
        }
        InspectorPage::Transform => {
            ui.collapsing("Position", |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(to_egui_color(AXIS_X), "X");
                    ui.add(egui::DragValue::new(&mut node.position.x).speed(1.0));
                    ui.add_space(16.0);
                    ui.colored_label(to_egui_color(AXIS_Y), "Y");
                    ui.add(egui::DragValue::new(&mut node.position.y).speed(1.0));
                });
            });
            ui.collapsing("Radius", |ui| {
                ui.add(egui::Slider::new(&mut node.radius, 4.0..=50.0));
            });
        }
        InspectorPage::Constraints => {
            let connected: Vec<(Entity, &DistanceConstraint)> = constraint_query
                .iter()
                .filter(|(_, c)| c.involves(selected_entity))
                .collect();

            if connected.is_empty() {
                ui.colored_label(to_egui_color(TEXT_DISABLED), "No constraints");
            } else {
                for (entity, constraint) in connected {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let other = constraint.other(selected_entity).unwrap();
                            ui.label(format!("→ {:?}", other));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("×").clicked() {
                                    constraint_deletes.push(entity);
                                }
                            });
                        });
                        let mut len = constraint.rest_length;
                        if ui
                            .add(
                                egui::Slider::new(&mut len, MIN_CONSTRAINT_DISTANCE..=MAX_CONSTRAINT_DISTANCE)
                                    .text("Length"),
                            )
                            .changed()
                        {
                            constraint_updates.push((entity, len));
                        }
                    });
                }
            }
        }
    }
}
