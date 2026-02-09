//! Floating panel spawning.

use bevy::picking::prelude::Pickable;
use bevy::prelude::*;

use crate::ui::icons::UiIcons;
use crate::ui::messages::*;
use crate::ui::theme::interaction::InteractionPalette;
use crate::ui::theme::palette::*;
use crate::ui::widgets::{
    CheckboxButton, CheckboxIcon, CheckboxRow, CheckboxSetting, ExportButton, FloatingPanel, HamburgerButton,
    HeaderRow, ImportButton, PanelBody, PanelContainer, PlaygroundSizeSlider, SliderHandle, SliderRow, SliderTrack,
};

use super::px;

/// Spawns the collapsible floating panel with display settings and import/export controls.
pub fn spawn_floating_panel(commands: &mut Commands, icons: &UiIcons) {
    let hamburger_icon = icons.hamburger.clone();
    let import_icon = icons.import.clone();
    let export_icon = icons.export.clone();
    let checkmark_icon = icons.checkmark.clone();

    commands
        .spawn((
            Name::new("Floating Panel"),
            FloatingPanel,
            Node {
                position_type: PositionType::Absolute,
                left: px(16.0),
                top: px(16.0),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            BackgroundColor(SURFACE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Panel Container"),
                    PanelContainer,
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(px(12.0)),
                        row_gap: px(8.0),
                        ..default()
                    },
                ))
                .with_children(|container| {
                    container
                        .spawn((
                            Name::new("Header Row"),
                            HeaderRow,
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|header| {
                            header
                                .spawn((
                                    Name::new("Hamburger Button"),
                                    HamburgerButton,
                                    Button,
                                    Node {
                                        width: px(32.0),
                                        height: px(32.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        border_radius: BorderRadius::all(px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(SURFACE),
                                    InteractionPalette::default(),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        ImageNode::new(hamburger_icon.clone()),
                                        Node {
                                            width: px(18.0),
                                            height: px(18.0),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ));
                                });
                        });

                    container
                        .spawn((
                            Name::new("Panel Body"),
                            PanelBody,
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: px(8.0),
                                ..default()
                            },
                        ))
                        .with_children(|body| {
                            spawn_checkbox_row(
                                body,
                                LABEL_SHOW_SKIN,
                                CheckboxSetting::ShowSkin,
                                checkmark_icon.clone(),
                            );
                            spawn_checkbox_row(
                                body,
                                LABEL_SHOW_EDGE,
                                CheckboxSetting::ShowEdge,
                                checkmark_icon.clone(),
                            );
                            spawn_checkbox_row(
                                body,
                                LABEL_SHOW_NODES,
                                CheckboxSetting::ShowNodes,
                                checkmark_icon.clone(),
                            );
                            spawn_checkbox_row(
                                body,
                                LABEL_SHOW_DEBUG,
                                CheckboxSetting::ShowDebug,
                                checkmark_icon.clone(),
                            );

                            body.spawn(Node {
                                flex_grow: 1.0,
                                min_height: px(16.0),
                                ..default()
                            });

                            spawn_icon_text_button(body, import_icon.clone(), BTN_IMPORT, ImportButton);
                            spawn_icon_text_button(body, export_icon.clone(), BTN_EXPORT, ExportButton);

                            body.spawn(Node {
                                flex_grow: 1.0,
                                min_height: px(8.0),
                                ..default()
                            });

                            spawn_slider_row(body, LABEL_PLAYGROUND_SIZE);
                        });
                });
        });
}

/// Spawns a checkbox row with label and setting toggle.
fn spawn_checkbox_row(
    parent: &mut ChildSpawnerCommands,
    label_text: &str,
    setting: CheckboxSetting,
    checkmark_icon: Handle<Image>,
) {
    parent
        .spawn((
            Name::new(format!("Checkbox Row: {}", label_text)),
            CheckboxRow,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8.0),
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Name::new("Checkbox"),
                CheckboxButton,
                setting,
                Button,
                Node {
                    width: px(20.0),
                    height: px(20.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(3.0)),
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BorderColor::from(TEXT_SECONDARY),
                BackgroundColor(SURFACE),
                InteractionPalette {
                    none: SURFACE,
                    hovered: SURFACE_HOVER,
                    pressed: SURFACE_PRESSED,
                    active: SURFACE_HOVER,
                },
            ))
            .with_children(|checkbox| {
                checkbox.spawn((
                    CheckboxIcon,
                    setting,
                    ImageNode::new(checkmark_icon),
                    Node {
                        width: px(14.0),
                        height: px(14.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            });

            row.spawn((
                Text::new(label_text),
                TextFont::from_font_size(14.0),
                TextColor(TEXT),
                Pickable::IGNORE,
            ));
        });
}

/// Spawns a button with icon and text label.
fn spawn_icon_text_button<C: Component>(parent: &mut ChildSpawnerCommands, icon: Handle<Image>, text: &str, marker: C) {
    parent
        .spawn((
            Name::new(format!("{} Button", text)),
            marker,
            Button,
            Node {
                width: px(176.0),
                height: px(36.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(8.0),
                border_radius: BorderRadius::all(px(4.0)),
                ..default()
            },
            BackgroundColor(SURFACE),
            InteractionPalette {
                none: SURFACE,
                hovered: SURFACE_HOVER,
                pressed: SURFACE_PRESSED,
                active: SURFACE_HOVER,
            },
        ))
        .with_children(|btn| {
            btn.spawn((
                ImageNode::new(icon),
                Node {
                    width: px(16.0),
                    height: px(16.0),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            btn.spawn((
                Text::new(text),
                TextFont::from_font_size(14.0),
                TextColor(TEXT),
                Pickable::IGNORE,
            ));
        });
}

/// Spawns a slider row with label for playground size control.
fn spawn_slider_row(parent: &mut ChildSpawnerCommands, label_text: &str) {
    parent
        .spawn((
            Name::new(format!("Slider Row: {}", label_text)),
            SliderRow,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label_text),
                TextFont::from_font_size(12.0),
                TextColor(TEXT_SECONDARY),
                Pickable::IGNORE,
            ));

            row.spawn((
                Name::new("Slider Track"),
                SliderTrack,
                PlaygroundSizeSlider,
                Button,
                Node {
                    width: px(176.0),
                    height: px(8.0),
                    border_radius: BorderRadius::all(px(4.0)),
                    ..default()
                },
                BackgroundColor(SURFACE_PRESSED),
            ))
            .with_children(|track| {
                track.spawn((
                    Name::new("Slider Handle"),
                    SliderHandle,
                    PlaygroundSizeSlider,
                    Node {
                        width: px(16.0),
                        height: px(16.0),
                        position_type: PositionType::Absolute,
                        top: px(-4.0),
                        left: Val::Percent(50.0),
                        border_radius: BorderRadius::all(px(8.0)),
                        ..default()
                    },
                    BackgroundColor(ACCENT),
                ));
            });
        });
}
