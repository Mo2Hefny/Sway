//! Editor UI implemented with bevy_egui, preserving the original layout.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};


use crate::core::{
    Node as SimNode, NodeType, DistanceConstraint, Playground,
    AnchorMovementMode, ProceduralPathType,
    build_scene_data, spawn_scene_data, export_to_file, import_from_file,
};
use crate::core::constants::{MIN_CONSTRAINT_DISTANCE, MAX_CONSTRAINT_DISTANCE};
use crate::editor::components::{NodeVisual, ConstraintVisual, ConstraintPreview};
use crate::editor::tools::selection::Selection;
use crate::ui::icons::UiIcons;
use crate::ui::messages::*;
use crate::ui::state::*;
use crate::ui::theme::palette::*;

/// When Some, a system in Update will clear the scene and spawn this data.
#[derive(Resource, Default)]
pub struct ImportRequested(pub Option<crate::core::SceneData>);

/// Pending constraint rest_length updates and deletions. Applied in Update after egui.
#[derive(Resource, Default)]
pub struct PendingConstraintActions {
    pub updates: Vec<(Entity, f32)>,
    pub deletes: Vec<Entity>,
    pub node_deletes: Vec<Entity>,
}


/// Cached egui texture IDs for UI icons (registered on first use).
#[derive(Resource, Default)]
pub struct EguiIconTextures {
    hamburger: Option<egui::TextureId>,
    import: Option<egui::TextureId>,
    export: Option<egui::TextureId>,
    caret_right: Option<egui::TextureId>,
    caret_left: Option<egui::TextureId>,
    properties: Option<egui::TextureId>,
    transform: Option<egui::TextureId>,
    constraints: Option<egui::TextureId>,
    cursor_tool: Option<egui::TextureId>,
    add_node_tool: Option<egui::TextureId>,
    add_edge_tool: Option<egui::TextureId>,
    move_tool: Option<egui::TextureId>,
    play: Option<egui::TextureId>,
    pause: Option<egui::TextureId>,
    stop: Option<egui::TextureId>,
    checkmark: Option<egui::TextureId>,
}

fn ensure_icons_registered(
    contexts: &mut EguiContexts,
    icons: &UiIcons,
    egui_icons: &mut EguiIconTextures,
) {
    if egui_icons.hamburger.is_some() {
        return;
    }
    egui_icons.hamburger = Some(contexts.add_image(EguiTextureHandle::Strong(icons.hamburger.clone())));
    egui_icons.import = Some(contexts.add_image(EguiTextureHandle::Strong(icons.import.clone())));
    egui_icons.export = Some(contexts.add_image(EguiTextureHandle::Strong(icons.export.clone())));
    egui_icons.caret_right = Some(contexts.add_image(EguiTextureHandle::Strong(icons.caret_right.clone())));
    egui_icons.caret_left = Some(contexts.add_image(EguiTextureHandle::Strong(icons.caret_left.clone())));
    egui_icons.properties = Some(contexts.add_image(EguiTextureHandle::Strong(icons.properties.clone())));
    egui_icons.transform = Some(contexts.add_image(EguiTextureHandle::Strong(icons.transform.clone())));
    egui_icons.constraints = Some(contexts.add_image(EguiTextureHandle::Strong(icons.constraints.clone())));
    egui_icons.cursor_tool = Some(contexts.add_image(EguiTextureHandle::Strong(icons.cursor_tool.clone())));
    egui_icons.add_node_tool = Some(contexts.add_image(EguiTextureHandle::Strong(icons.add_node_tool.clone())));
    egui_icons.add_edge_tool = Some(contexts.add_image(EguiTextureHandle::Strong(icons.add_edge_tool.clone())));
    egui_icons.move_tool = Some(contexts.add_image(EguiTextureHandle::Strong(icons.move_tool.clone())));
    egui_icons.play = Some(contexts.add_image(EguiTextureHandle::Strong(icons.play.clone())));
    egui_icons.pause = Some(contexts.add_image(EguiTextureHandle::Strong(icons.pause.clone())));
    egui_icons.stop = Some(contexts.add_image(EguiTextureHandle::Strong(icons.stop.clone())));
    egui_icons.checkmark = Some(contexts.add_image(EguiTextureHandle::Strong(icons.checkmark.clone())));
}

fn to_egui_color(c: Color) -> egui::Color32 {
    use bevy::color::Srgba;
    let srgba: Srgba = c.into();
    let [r, g, b, a] = srgba.to_f32_array();
    egui::Color32::from_rgba_unmultiplied(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    )
}

/// Main editor UI system: draws all panels in the original layout and updates InputState.
pub fn editor_ui_system(
    mut contexts: EguiContexts,
    mut display_settings: ResMut<DisplaySettings>,
    mut panel_state: ResMut<FloatingPanelState>,
    mut inspector_state: ResMut<InspectorState>,
    mut tool_state: ResMut<EditorToolState>,
    mut playback: ResMut<PlaybackState>,
    ui_visibility: Res<UiVisibility>,
    mut input_state: ResMut<InputState>,
    icons: Res<UiIcons>,
    mut egui_icons: ResMut<EguiIconTextures>,
    mut import_requested: ResMut<ImportRequested>,
    selection: Res<Selection>,
    playground: Res<Playground>,
    mut node_query: Query<(Entity, &mut SimNode)>,
    constraint_query: Query<(Entity, &DistanceConstraint)>,
    mut pending_actions: ResMut<PendingConstraintActions>,
) {
    let mut node_deletes: Vec<Entity> = Vec::new();
    ensure_icons_registered(&mut contexts, &icons, &mut egui_icons);
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if !ui_visibility.visible {
        input_state.cursor_over_ui = false;
        return;
    }

    let icons = &egui_icons;

    let screen = ctx.available_rect();
    let left = screen.min.x;
    let right = screen.max.x;
    let top = screen.min.y;
    let bottom = screen.max.y;

    // --- Floating panel (top-left, 16px margin); fixed width so collapse/expand doesn't break layout ---
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
                        egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(tid, egui::vec2(18.0, 18.0))))
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
                        let scene = build_scene_data(&node_query, &constraint_query);
                        export_to_file(&scene);
                    }
                }
            });
        });

    // --- Right side layout (from right to left): icon bar → inspector panel → toolbar ---
    // All panels span full height. Use fixed_pos for precise positioning.
    let icon_bar_w = 48.0;
    let inspector_w = 280.0;
    let tool_bar_w = 48.0;
    let panel_height = bottom - top;  // Full screen height

    // Calculate LEFT EDGE X positions for each panel (from right edge of screen)
    // Icon bar: at the far right
    let icon_bar_x = right - icon_bar_w;
    // Inspector panel: to the left of icon bar  
    let inspector_x = icon_bar_x - inspector_w;
    // Toolbar: to the left of inspector panel (or icon bar when inspector closed)
    let tool_bar_x = if inspector_state.open {
        inspector_x - tool_bar_w
    } else {
        icon_bar_x - tool_bar_w
    };

    // 1) Toolbar (drawn first so it sits behind inspector elements)
    egui::Area::new(egui::Id::new("tool_bar"))
        .fixed_pos(egui::pos2(tool_bar_x, top))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.allocate_ui(egui::vec2(tool_bar_w, panel_height), |ui| {
                let frame = egui::Frame::new()
                    .fill(to_egui_color(SURFACE))
                    .inner_margin(egui::Margin::same(4))
                    .corner_radius(4.0);
                frame.show(ui, |ui| {
                    ui.set_min_size(egui::vec2(tool_bar_w - 8.0, panel_height - 8.0));
                    ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 4.0);
                    ui.add_space(4.0);
                    for (tool, icon) in [
                        (EditorTool::Cursor, icons.cursor_tool),
                        (EditorTool::AddNode, icons.add_node_tool),
                        (EditorTool::AddEdge, icons.add_edge_tool),
                        (EditorTool::Move, icons.move_tool),
                    ] {
                        let active = tool_state.active == tool;
                        let fill = if active { to_egui_color(SURFACE_HOVER) } else { to_egui_color(SURFACE) };
                        let tool_btn = match icon {
                            Some(tid) => ui.add_sized(
                                egui::vec2(40.0, 40.0),
                                egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(tid, egui::vec2(24.0, 24.0))))
                                    .fill(fill),
                            ),
                            None => ui.add_sized(egui::vec2(40.0, 40.0), egui::Button::new(tool.name()).fill(fill)),
                        };
                        if tool_btn.clicked() {
                            tool_state.active = tool;
                        }
                    }
                });
            });
        });

    // 2) Inspector panel (when open; directly beside icon bar)
    let mut constraint_updates: Vec<(Entity, f32)> = Vec::new();
    let mut constraint_deletes: Vec<Entity> = Vec::new();
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
                            &selection,
                            &playground,
                            inspector_state.active_page,
                            &mut node_query,
                            &constraint_query,
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

    // 3) Inspector icon bar (far right; inspector open/close + tabs)
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
                    let caret_id = if inspector_state.open { icons.caret_right } else { icons.caret_left };
                    let caret_btn = match caret_id {
                        Some(tid) => ui.add_sized(
                            egui::vec2(40.0, 40.0),
                            egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(tid, egui::vec2(24.0, 24.0))))
                                .fill(to_egui_color(SURFACE)),
                        ),
                        None => ui.add_sized(egui::vec2(40.0, 40.0), egui::Button::new("◀").fill(to_egui_color(SURFACE))),
                    };
                    if caret_btn.clicked() {
                        inspector_state.open = !inspector_state.open;
                    }
                    ui.separator();
                    for page in [InspectorPage::Properties, InspectorPage::Transform, InspectorPage::Constraints] {
                        let active = inspector_state.active_page == page;
                        let fill = if active { to_egui_color(SURFACE_HOVER) } else { to_egui_color(SURFACE) };
                        let icon = match page {
                            InspectorPage::Properties => icons.properties,
                            InspectorPage::Transform => icons.transform,
                            InspectorPage::Constraints => icons.constraints,
                        };
                        let page_btn = match icon {
                            Some(tid) => ui.add_sized(
                                egui::vec2(40.0, 40.0),
                                egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(tid, egui::vec2(24.0, 24.0))))
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

    // --- Bottom toolbar (playback, centered with bottom margin) ---
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
                        egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(tid, egui::vec2(16.0, 16.0))))
                            .fill(to_egui_color(SURFACE)),
                    ),
                    None => ui.add_sized(egui::vec2(32.0, 32.0), egui::Button::new("▶").fill(to_egui_color(SURFACE))),
                };
                if play_btn.clicked() {
                    playback.play();
                }
                let pause_btn = match icons.pause {
                    Some(tid) => ui.add_sized(
                        egui::vec2(32.0, 32.0),
                        egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(tid, egui::vec2(16.0, 16.0))))
                            .fill(to_egui_color(SURFACE)),
                    ),
                    None => ui.add_sized(egui::vec2(32.0, 32.0), egui::Button::new("⏸").fill(to_egui_color(SURFACE))),
                };
                if pause_btn.clicked() {
                    playback.pause();
                }
                let stop_btn = match icons.stop {
                    Some(tid) => ui.add_sized(
                        egui::vec2(32.0, 32.0),
                        egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(tid, egui::vec2(16.0, 16.0))))
                            .fill(to_egui_color(SURFACE)),
                    ),
                    None => ui.add_sized(egui::vec2(32.0, 32.0), egui::Button::new("⏹").fill(to_egui_color(SURFACE))),
                };
                if stop_btn.clicked() {
                    playback.stop();
                }
                });
            });
        });

    // --- Instructions (bottom-left) ---
    egui::Area::new(egui::Id::new("instructions"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(left + 16.0, bottom - 16.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 2.0);
            let hint = to_egui_color(Color::srgba(0.6, 0.6, 0.6, 0.5));
            ui.label(egui::RichText::new(HINT_SELECT).color(hint));
            ui.label(egui::RichText::new(HINT_ADD_NODE).color(hint));
            ui.label(egui::RichText::new(HINT_ADD_EDGE).color(hint));
            ui.label(egui::RichText::new(HINT_MOVE).color(hint));
            ui.label(egui::RichText::new(HINT_TOGGLE_UI).color(hint));
            ui.label(egui::RichText::new(HINT_PLAY).color(hint));
            ui.label(egui::RichText::new(HINT_STOP).color(hint));
        });

    input_state.cursor_over_ui = ctx.wants_pointer_input();
}

/// Toggle UI visibility with H key. Runs in Update so editor_ui_system can stay in EguiPrimaryContextPass.
pub fn toggle_ui_visibility(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_visibility: ResMut<UiVisibility>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        ui_visibility.visible = !ui_visibility.visible;
    }
}

/// Toggles playback state when Space is pressed, unless typing in UI.
pub fn toggle_playback_control(
    mut contexts: EguiContexts,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut playback: ResMut<PlaybackState>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        if ctx.wants_keyboard_input() {
            return;
        }
    }
    if keyboard.just_pressed(KeyCode::Space) {
        playback.toggle();
    }
}

/// Applies deferred import, constraint updates, and constraint deletes. Runs in Update after egui.
pub fn apply_editor_actions(
    mut commands: Commands,
    mut import_requested: ResMut<ImportRequested>,
    mut pending_actions: ResMut<PendingConstraintActions>,
    mut selection: ResMut<Selection>,
    node_query: Query<(Entity, &mut SimNode)>,
    mut constraint_query: Query<(Entity, &mut DistanceConstraint)>,
    visual_entities: Query<Entity, Or<(With<NodeVisual>, With<ConstraintVisual>, With<ConstraintPreview>)>>,
) {
    if let Some(scene) = import_requested.0.take() {
        selection.deselect();
        for e in visual_entities.iter() {
            commands.entity(e).despawn();
        }
        let constraint_list: Vec<Entity> = constraint_query.iter().map(|(e, _)| e).collect();
        for e in constraint_list {
            commands.entity(e).despawn();
        }
        let node_list: Vec<Entity> = node_query.iter().map(|(e, _)| e).collect();
        for e in node_list {
            commands.entity(e).despawn();
        }
        spawn_scene_data(&mut commands, &scene);
    }
    for (entity, len) in pending_actions.updates.drain(..) {
        if let Ok((_, mut c)) = constraint_query.get_mut(entity) {
            c.rest_length = len.clamp(MIN_CONSTRAINT_DISTANCE, MAX_CONSTRAINT_DISTANCE);
        }
    }
    for entity in pending_actions.deletes.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in pending_actions.node_deletes.drain(..) {
        // Despawn all constraints involving this node
        for (c_entity, constraint) in constraint_query.iter() {
            if constraint.involves(entity) {
                commands.entity(c_entity).despawn();
            }
        }
        commands.entity(entity).despawn();
        selection.deselect();
    }
}

fn inspector_content_ui(
    ui: &mut egui::Ui,
    selection: &Selection,
    playground: &Playground,
    page: InspectorPage,
    node_query: &mut Query<(Entity, &mut SimNode)>,
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
                            if ui.selectable_label(node.node_type == NodeType::Normal, NodeType::Normal.name()).clicked() {
                                node.node_type = NodeType::Normal;
                            }
                            if ui.selectable_label(node.node_type == NodeType::Anchor, NodeType::Anchor.name()).clicked() {
                                node.node_type = NodeType::Anchor;
                            }
                            if ui.selectable_label(node.node_type == NodeType::Leg, NodeType::Leg.name()).clicked() {
                                node.node_type = NodeType::Leg;
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
                                if ui.selectable_label(node.movement_mode == AnchorMovementMode::None, AnchorMovementMode::None.name()).clicked() {
                                    node.movement_mode = AnchorMovementMode::None;
                                }
                                if ui.selectable_label(node.movement_mode == AnchorMovementMode::FollowTarget, AnchorMovementMode::FollowTarget.name()).clicked() {
                                    node.movement_mode = AnchorMovementMode::FollowTarget;
                                }
                                if ui.selectable_label(node.movement_mode == AnchorMovementMode::Procedural, AnchorMovementMode::Procedural.name()).clicked() {
                                    node.movement_mode = AnchorMovementMode::Procedural;
                                    // Initialize path center to current position when switching to procedural
                                    if node.path_center == bevy::math::Vec2::ZERO {
                                        node.path_center = node.position;
                                    }
                                }
                            });
                    });
                    
                    match node.movement_mode {
                        AnchorMovementMode::None => {
                            // Static anchor - no additional controls
                        }
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
                                        if ui.selectable_label(node.path_type == ProceduralPathType::Circle, "Circle").clicked() {
                                            node.path_type = ProceduralPathType::Circle;
                                        }
                                        if ui.selectable_label(node.path_type == ProceduralPathType::Wave, "Wave").clicked() {
                                            node.path_type = ProceduralPathType::Wave;
                                        }
                                        if ui.selectable_label(node.path_type == ProceduralPathType::Wander, "Wander").clicked() {
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

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let delete_btn = ui.add_sized(
                [ui.available_width(), 32.0],
                egui::Button::new(egui::RichText::new("Delete Node").color(to_egui_color(Color::srgba(1.0, 0.4, 0.4, 1.0))))
                    .fill(to_egui_color(Color::srgba(0.8, 0.2, 0.2, 0.1))),
            );
            if delete_btn.clicked() {
                node_deletes.push(selected_entity);
            }
        }
        InspectorPage::Transform => {
            let inner_min = playground.inner_min();
            let inner_max = playground.inner_max();
            let pos_min_x = inner_min.x + node.radius;
            let pos_max_x = inner_max.x - node.radius;
            let pos_min_y = inner_min.y + node.radius;
            let pos_max_y = inner_max.y - node.radius;
            ui.collapsing("Transform", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position");
                    let mut pos = node.position;
                    let changed_x = ui.add(egui::DragValue::new(&mut pos.x).range(pos_min_x..=pos_max_x)).changed();
                    let changed_y = ui.add(egui::DragValue::new(&mut pos.y).range(pos_min_y..=pos_max_y)).changed();
                    if changed_x || changed_y {
                        node.position = pos;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Radius");
                    let mut r = node.radius;
                    if ui.add(egui::DragValue::new(&mut r).range(4.0..=50.0)).changed() {
                        node.radius = r;
                    }
                });
            });
            ui.collapsing("Acceleration", |ui| {
                ui.horizontal(|ui| {
                    ui.label("X");
                    let mut ax = node.acceleration.x;
                    if ui.add(egui::DragValue::new(&mut ax).range(-10.0..=10.0)).changed() {
                        node.acceleration.x = ax;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Y");
                    let mut ay = node.acceleration.y;
                    if ui.add(egui::DragValue::new(&mut ay).range(-10.0..=10.0)).changed() {
                        node.acceleration.y = ay;
                    }
                });
            });
        }
        InspectorPage::Constraints => {
            let connected: Vec<_> = constraint_query
                .iter()
                .filter(|(_, c)| c.involves(selected_entity))
                .collect();
            ui.collapsing("Constraints", |ui| {
                if connected.is_empty() {
                    ui.colored_label(to_egui_color(TEXT_DISABLED), "No constraints");
                } else {
                    for (constraint_entity, constraint) in &connected {
                        let other = constraint.other(selected_entity).unwrap();
                        let label = format!("→ {:?}", other);
                        ui.horizontal(|ui| {
                            if ui.small_button("×").clicked() {
                                constraint_deletes.push(*constraint_entity);
                            }
                            ui.label(&label);
                            let mut len = constraint.rest_length;
                            if ui.add(egui::DragValue::new(&mut len).range(MIN_CONSTRAINT_DISTANCE..=MAX_CONSTRAINT_DISTANCE)).changed() {
                                constraint_updates.push((*constraint_entity, len));
                            }
                        });
                    }
                }
            });
        }
    }
}
