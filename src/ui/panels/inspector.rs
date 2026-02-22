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
use crate::ui::constants::*;
use crate::ui::theme::*;
use crate::ui::icons::EguiIconTextures;

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

    let icon_bar_w = ICON_BAR_WIDTH;
    let inspector_w = INSPECTOR_WIDTH;
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
                        .inner_margin(egui::Margin::same(PANEL_INNER_MARGIN as i8))
                        .corner_radius(PANEL_CORNER_RADIUS);
                    
                    frame.show(ui, |ui| {
                        ui.set_min_size(egui::vec2(inspector_w - PANEL_INNER_MARGIN * 2.0, panel_height - PANEL_INNER_MARGIN * 2.0));
                        ui.label(egui::RichText::new(inspector_state.active_page.name())
                            .text_style(egui::TextStyle::Heading)
                            .color(typography::heading_color()));
                        ui.add_space(PANEL_INNER_MARGIN);
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

                    let rect = ui.max_rect();
                    let stroke = egui::Stroke::new(PANEL_SEPARATOR_WIDTH, to_egui_color(SEPARATOR));
                    ui.painter().vline(rect.right(), rect.y_range(), stroke);
                    ui.painter().vline(rect.left(), rect.y_range(), stroke);
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
                    .inner_margin(egui::Margin::same(ICON_BAR_INNER_MARGIN as i8))
                    .corner_radius(PANEL_CORNER_RADIUS);
                
                frame.show(ui, |ui| {
                    ui.set_min_size(egui::vec2(icon_bar_w - ICON_BAR_INNER_MARGIN * 2.0, panel_height - ICON_BAR_INNER_MARGIN * 2.0));
                    ui.style_mut().spacing.item_spacing = egui::vec2(PANEL_ITEM_SPACING, PANEL_ITEM_SPACING);
                    let caret_id = if inspector_state.open {
                        icons.caret_right
                    } else {
                        icons.caret_left
                    };
                    let caret_btn = match caret_id {
                        Some(tid) => ui.add_sized(
                            egui::vec2(ICON_BAR_BTN_SIZE, ICON_BAR_BTN_SIZE),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                tid,
                                egui::vec2(ICON_BAR_ICON_SIZE, ICON_BAR_ICON_SIZE),
                            )))
                            .fill(to_egui_color(SURFACE)),
                        ),
                        None => ui.add_sized(
                            egui::vec2(ICON_BAR_BTN_SIZE, ICON_BAR_BTN_SIZE),
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
                                egui::vec2(ICON_BAR_BTN_SIZE, ICON_BAR_BTN_SIZE),
                                egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                    tid,
                                    egui::vec2(ICON_BAR_ICON_SIZE, ICON_BAR_ICON_SIZE),
                                )))
                                .fill(fill),
                            ),
                            None => ui.add_sized(egui::vec2(ICON_BAR_BTN_SIZE, ICON_BAR_BTN_SIZE), egui::Button::new("").fill(fill)),
                        };
                        if page_btn.clicked() {
                            inspector_state.active_page = page;
                        }
                    }
                });

                let rect = ui.max_rect();
                let stroke = egui::Stroke::new(PANEL_SEPARATOR_WIDTH, to_egui_color(SEPARATOR));
                ui.painter().vline(rect.left(), rect.y_range(), stroke);
                ui.painter().vline(rect.right(), rect.y_range(), stroke);
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
            ui.collapsing(egui::RichText::new(SECTION_NODE_SETTINGS)
                    .text_style(egui::TextStyle::Heading)
                    .color(typography::heading_color()), |ui| {
                ui.horizontal(|ui| {
                    ui.label(PROP_NODE_TYPE);
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
                ui.checkbox(&mut node.is_head, PROP_IS_HEAD);
                ui.add_space(PANEL_ITEM_SPACING);
                ui.vertical(|ui| {
                    ui.label(PROP_ANGLE_MIN);
                    let mut angle_min_deg = node.angle_min.to_degrees();
                    if ui.add(egui::Slider::new(&mut angle_min_deg, ANGLE_LIMIT_RANGE).suffix("°")).changed() {
                        node.angle_min = angle_min_deg.to_radians();
                        node.angle_max = node.angle_max.max(node.angle_min);
                    }
                    
                    ui.label(PROP_ANGLE_MAX);
                    let mut angle_max_deg = node.angle_max.to_degrees();
                    if ui.add(egui::Slider::new(&mut angle_max_deg, ANGLE_LIMIT_RANGE).suffix("°")).changed() {
                        node.angle_max = angle_max_deg.to_radians();
                        node.angle_min = node.angle_min.min(node.angle_max);
                    }
                });

                if node.node_type == NodeType::Anchor {
                    ui.add_space(PANEL_TITLE_SPACING);
                    ui.separator();
                    ui.add_space(PANEL_TITLE_SPACING);

                    // Movement Mode selector
                    ui.horizontal(|ui| {
                        ui.label(PROP_MOVEMENT_MODE);
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
                            ui.vertical(|ui| {
                                ui.label(PROP_MOVEMENT_SPEED);
                                ui.add(egui::Slider::new(&mut node.movement_speed, MOVEMENT_SPEED_RANGE));
                            });
                        }
                        AnchorMovementMode::Procedural => {
                            // Path Type selector
                            ui.horizontal(|ui| {
                                ui.label(PROP_PATH_TYPE);
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

                            ui.vertical(|ui| {
                                ui.label(PROP_AMPLITUDE_X);
                                ui.add(egui::Slider::new(&mut node.path_amplitude.x, PATH_AMPLITUDE_RANGE));
                            });

                            ui.vertical(|ui| {
                                ui.label(PROP_AMPLITUDE_Y);
                                ui.add(egui::Slider::new(&mut node.path_amplitude.y, PATH_AMPLITUDE_RANGE));
                            });

                            ui.vertical(|ui| {
                                ui.label(PROP_MOVEMENT_SPEED);
                                ui.add(egui::Slider::new(&mut node.movement_speed, MOVEMENT_SPEED_RANGE));
                            });

                            if ui.button(BTN_SET_CENTER).clicked() {
                                node.path_center = node.position;
                            }
                        }
                    }
                }
            });
            
            if let Ok((_e, mut limb_set)) = limb_set_query.get_mut(selected_entity) {
                ui.collapsing(egui::RichText::new(SECTION_LIMB_IK)
                    .text_style(egui::TextStyle::Heading)
                    .color(typography::heading_color()), |ui| {
                    let limb_count = limb_set.limbs.len();
                    for (i, limb) in limb_set.limbs.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{} #{}  |  {} {}", LABEL_LIMB, i + 1, LABEL_JOINTS, limb.joints.len()))
                                .text_style(egui::TextStyle::Heading)
                                .color(typography::heading_color()));
                        });
                        ui.vertical(|ui| {
                            ui.label(PROP_MAX_REACH);
                            ui.add(egui::Slider::new(&mut limb.max_reach, LIMB_MAX_REACH_RANGE));
                        });
                        ui.vertical(|ui| {
                            ui.label(PROP_ANGLE_OFFSET);
                            let mut deg = limb.target_direction_offset.to_degrees();
                            if ui.add(egui::Slider::new(&mut deg, LIMB_ANGLE_OFFSET_RANGE).suffix("°")).changed() {
                                limb.target_direction_offset = deg.to_radians();
                            }
                        });

                        ui.label(egui::RichText::new(SECTION_STEPPING).text_style(egui::TextStyle::Body).strong());
                        ui.horizontal(|ui| {
                            ui.label(PROP_THRESHOLD);
                            ui.add(egui::DragValue::new(&mut limb.step_threshold).speed(WIDGET_DRAG_SPEED_FINE).range(LIMB_STEP_THRESHOLD_RANGE));
                        });
                        ui.horizontal(|ui| {
                            ui.label(PROP_SPEED);
                            ui.add(egui::DragValue::new(&mut limb.step_speed).speed(WIDGET_DRAG_SPEED_FINE).range(LIMB_STEP_SPEED_RANGE));
                        });
                        ui.horizontal(|ui| {
                            ui.label(PROP_HEIGHT);
                            ui.add(egui::DragValue::new(&mut limb.step_height).speed(WIDGET_DRAG_SPEED_FINE).range(LIMB_STEP_HEIGHT_RANGE));
                        });
                        
                        ui.add_space(PANEL_ITEM_SPACING);
                        ui.label(PROP_JOINT_FLIP);
                        for (j, flipped) in limb.flip_bend.iter_mut().enumerate() {
                             ui.checkbox(flipped, format!("{} {}", LABEL_JOINT, j + 1));
                        }
                        
                        if i < limb_count - 1 {
                            ui.add_space(PANEL_TITLE_SPACING);
                            ui.separator();
                            ui.add_space(PANEL_TITLE_SPACING);
                        }
                    }
                });
            }

            ui.collapsing(egui::RichText::new(SECTION_PHYSICS)
                .text_style(egui::TextStyle::Heading)
                .color(typography::heading_color()), |ui| {
                ui.vertical(|ui| {
                    ui.label(PROP_COLLISION_DAMP);
                    ui.add(egui::Slider::new(&mut node.collision_damping, COLLISION_DAMPING_RANGE));
                });
            });

            ui.collapsing(egui::RichText::new(SECTION_ACCELERATION)
                .text_style(egui::TextStyle::Heading)
                .color(typography::heading_color()), |ui| {
                ui.label(LABEL_ACCEL_CONSTANT);
                ui.horizontal(|ui| {
                    ui.colored_label(to_egui_color(AXIS_X), "X");
                    ui.add(egui::DragValue::new(&mut node.constant_acceleration.x).speed(WIDGET_DRAG_SPEED));
                    ui.add_space(PANEL_TITLE_SPACING);
                    ui.colored_label(to_egui_color(AXIS_Y), "Y");
                    ui.add(egui::DragValue::new(&mut node.constant_acceleration.y).speed(WIDGET_DRAG_SPEED));
                });

                ui.add_space(PANEL_ITEM_SPACING);
                ui.label(LABEL_ACCEL_LIVE);
                ui.horizontal(|ui| {
                    ui.colored_label(to_egui_color(AXIS_X), "X");
                    ui.label(format!("{:.2}", node.acceleration.x));
                    ui.add_space(PANEL_SECTION_SPACING);
                    ui.colored_label(to_egui_color(AXIS_Y), "Y");
                    ui.label(format!("{:.2}", node.acceleration.y));
                });
            });

            ui.add_space(PANEL_TITLE_SPACING);
            ui.separator();
            ui.add_space(PANEL_TITLE_SPACING);

            let delete_btn = ui.add_sized(
                [ui.available_width(), ACTION_BTN_HEIGHT],
                egui::Button::new(
                    egui::RichText::new(BTN_DELETE_NODE).color(to_egui_color(Color::srgba(1.0, 0.4, 0.4, 1.0))),
                )
                .fill(to_egui_color(Color::srgba(0.8, 0.2, 0.2, 0.1))),
            );
            if delete_btn.clicked() {
                node_deletes.push(selected_entity);
            }
        }
        InspectorPage::Transform => {
            ui.collapsing(egui::RichText::new(PROP_POSITION)
                .text_style(egui::TextStyle::Heading)
                .color(typography::heading_color()), |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(to_egui_color(AXIS_X), "X");
                    ui.add(egui::DragValue::new(&mut node.position.x).speed(WIDGET_DRAG_SPEED));
                    ui.add_space(PANEL_SECTION_SPACING);
                    ui.colored_label(to_egui_color(AXIS_Y), "Y");
                    ui.add(egui::DragValue::new(&mut node.position.y).speed(WIDGET_DRAG_SPEED));
                });
            });
            ui.collapsing(egui::RichText::new(PROP_RADIUS)
                .text_style(egui::TextStyle::Heading)
                .color(typography::heading_color()), |ui| {
                ui.add(egui::Slider::new(&mut node.radius, NODE_RADIUS_RANGE));
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
