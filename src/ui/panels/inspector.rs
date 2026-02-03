//! Right sidebar inspector panel spawning.

use bevy::prelude::*;
use bevy::picking::prelude::Pickable;

use crate::ui::icons::UiIcons;
use crate::ui::messages::*;
use crate::ui::state::*;
use crate::ui::theme::palette::*;
use crate::ui::theme::interaction::InteractionPalette;
use crate::ui::widgets::{
    RightSidebarRoot, InspectorPanel, IconBar, CaretButton, CaretIcon,
    PageIconButton, PageIconImage, PageIconFor, InspectorTitle, InspectorContent,
};

use super::px;

/// Spawns the right sidebar inspector with collapsible panel and page tabs.
pub fn spawn_right_sidebar(commands: &mut Commands, icons: &UiIcons) {
    let caret_right_icon = icons.caret_right.clone();
    let properties_icon = icons.properties.clone();
    let transform_icon = icons.transform.clone();
    let physics_icon = icons.physics.clone();
    let constraints_icon = icons.constraints.clone();
    
    commands.spawn((
        Name::new("Right Sidebar Inspector"),
        RightSidebarRoot,
        Node {
            position_type: PositionType::Absolute,
            right: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        Pickable::IGNORE,
    )).with_children(|parent| {
        parent.spawn((
            Name::new("Inspector Panel"),
            InspectorPanel,
            Node {
                width: px(280.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(16.0)),
                ..default()
            },
            BackgroundColor(SURFACE),
        )).with_children(|panel| {
            panel.spawn((
                Name::new("Inspector Title"),
                InspectorTitle,
                Text::new(PAGE_PROPERTIES),
                TextFont::from_font_size(16.0),
                TextColor(TEXT),
                Node {
                    margin: UiRect::bottom(px(16.0)),
                    ..default()
                },
            ));
            
            panel.spawn((
                Name::new("Inspector Content"),
                InspectorContent,
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                },
            )).with_children(|content| {
                content.spawn((
                    Text::new(PLACEHOLDER_NO_SELECTION),
                    TextFont::from_font_size(14.0),
                    TextColor(TEXT_SECONDARY),
                ));
            });
        });

        parent.spawn((
            Name::new("Vertical Divider"),
            Node {
                width: px(1.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(SURFACE_HOVER),
        ));

        parent.spawn((
            Name::new("Icon Bar"),
            IconBar,
            Node {
                width: px(48.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(4.0)),
                row_gap: px(4.0),
                ..default()
            },
            BackgroundColor(SURFACE),
        )).with_children(|bar| {
            bar.spawn((
                Name::new("Caret Button"),
                CaretButton,
                Button,
                Node {
                    width: px(40.0),
                    height: px(40.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(4.0)),
                    ..default()
                },
                BackgroundColor(SURFACE),
                InteractionPalette::default(),
            )).with_children(|btn| {
                btn.spawn((
                    CaretIcon,
                    ImageNode::new(caret_right_icon.clone()),
                    Node {
                        width: px(24.0),
                        height: px(24.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            });

            bar.spawn((
                Name::new("Horizontal Divider"),
                Node {
                    width: Val::Percent(100.0),
                    height: px(1.0),
                    margin: UiRect::axes(px(0.0), px(4.0)),
                    ..default()
                },
                BackgroundColor(SURFACE_HOVER),
            ));

            spawn_page_icon_button(bar, InspectorPage::Properties, properties_icon.clone(), true);
            spawn_page_icon_button(bar, InspectorPage::Transform, transform_icon.clone(), false);
            spawn_page_icon_button(bar, InspectorPage::Physics, physics_icon.clone(), false);
            spawn_page_icon_button(bar, InspectorPage::Constraints, constraints_icon.clone(), false);
        });
    });
}

/// Spawns an inspector page tab button with icon.
fn spawn_page_icon_button(
    parent: &mut ChildSpawnerCommands,
    page: InspectorPage,
    icon: Handle<Image>,
    is_active: bool,
) {
    let bg_color = if is_active { SURFACE_HOVER } else { SURFACE };
    
    parent.spawn((
        Name::new(format!("Page Icon: {}", page.name())),
        PageIconButton,
        PageIconFor(page),
        Button,
        Node {
            width: px(40.0),
            height: px(40.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BackgroundColor(bg_color),
        InteractionPalette {
            none: bg_color,
            hovered: SURFACE_HOVER,
            pressed: SURFACE_PRESSED,
            active: SURFACE_HOVER,
        },
    )).with_children(|btn| {
        btn.spawn((
            PageIconImage,
            PageIconFor(page),
            ImageNode::new(icon),
            Node {
                width: px(24.0),
                height: px(24.0),
                ..default()
            },
            Pickable::IGNORE,
        ));
    });
}
