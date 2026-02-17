use bevy_egui::egui;

use crate::ui::state::*;
use crate::ui::theme::palette::*;
use crate::ui::icons::{EguiIconTextures, to_egui_color};

pub fn draw_toolbar(
    ctx: &egui::Context,
    icons: &EguiIconTextures,
    tool_state: &mut EditorToolState,
    inspector_state: &InspectorState,
) {
    let screen = ctx.available_rect();
    let right = screen.max.x;
    let top = screen.min.y;
    let bottom = screen.max.y;

    let icon_bar_w = 48.0;
    let inspector_w = 280.0;
    let tool_bar_w = 48.0;
    let panel_height = bottom - top;

    let icon_bar_x = right - icon_bar_w;
    let inspector_x = icon_bar_x - inspector_w;
    let tool_bar_x = if inspector_state.open {
        inspector_x - tool_bar_w
    } else {
        icon_bar_x - tool_bar_w
    };

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
                        let fill = if active {
                            to_egui_color(SURFACE_HOVER)
                        } else {
                            to_egui_color(SURFACE)
                        };
                        let tool_btn = match icon {
                            Some(tid) => ui.add_sized(
                                egui::vec2(40.0, 40.0),
                                egui::Button::new(egui::Image::new(egui::load::SizedTexture::new(
                                    tid,
                                    egui::vec2(24.0, 24.0),
                                )))
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
}
