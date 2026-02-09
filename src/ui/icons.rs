//! Icon loading and management.

use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::prelude::*;

/// Handles to rasterized SVG icons.
#[derive(Resource, Clone, Debug, Default)]
pub struct UiIcons {
    pub hamburger: Handle<Image>,
    pub import: Handle<Image>,
    pub export: Handle<Image>,
    pub caret_right: Handle<Image>,
    pub caret_left: Handle<Image>,
    pub properties: Handle<Image>,
    pub transform: Handle<Image>,
    pub constraints: Handle<Image>,
    pub cursor_tool: Handle<Image>,
    pub add_node_tool: Handle<Image>,
    pub add_edge_tool: Handle<Image>,
    pub move_tool: Handle<Image>,
    pub play: Handle<Image>,
    pub pause: Handle<Image>,
    pub stop: Handle<Image>,
    pub checkmark: Handle<Image>,
}

impl UiIcons {
    /// Converts SVG bytes to rasterized image handle at specified size.
    fn rasterize_svg(images: &mut Assets<Image>, svg_bytes: &[u8], size: u32) -> Handle<Image> {
        let options = resvg::usvg::Options::default();
        let tree = match resvg::usvg::Tree::from_data(svg_bytes, &options) {
            Ok(tree) => tree,
            Err(_) => return images.add(Image::default()),
        };

        let mut pixmap = match resvg::tiny_skia::Pixmap::new(size, size) {
            Some(p) => p,
            None => return images.add(Image::default()),
        };

        let svg_size = tree.size();
        let scale_x = size as f32 / svg_size.width();
        let scale_y = size as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);
        let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);

        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let image = Image::new(
            bevy::render::render_resource::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            pixmap.data().to_vec(),
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        images.add(image)
    }

    /// Loads and rasterizes all UI icons from embedded SVG assets.
    pub fn load_all(images: &mut Assets<Image>) -> Self {
        const ICON_SIZE: u32 = 24;

        Self {
            hamburger: Self::rasterize_svg(images, include_bytes!("../assets/icons/hamburger-menu.svg"), ICON_SIZE),
            import: Self::rasterize_svg(images, include_bytes!("../assets/icons/import.svg"), ICON_SIZE),
            export: Self::rasterize_svg(images, include_bytes!("../assets/icons/export.svg"), ICON_SIZE),
            caret_right: Self::rasterize_svg(images, include_bytes!("../assets/icons/caret-right.svg"), ICON_SIZE),
            caret_left: Self::rasterize_svg(images, include_bytes!("../assets/icons/caret-left.svg"), ICON_SIZE),
            properties: Self::rasterize_svg(
                images,
                include_bytes!("../assets/icons/options-vertical.svg"),
                ICON_SIZE,
            ),
            transform: Self::rasterize_svg(images, include_bytes!("../assets/icons/transform.svg"), ICON_SIZE),
            constraints: Self::rasterize_svg(images, include_bytes!("../assets/icons/vector.svg"), ICON_SIZE),
            cursor_tool: Self::rasterize_svg(images, include_bytes!("../assets/icons/cursor.svg"), ICON_SIZE),
            add_node_tool: Self::rasterize_svg(images, include_bytes!("../assets/icons/add-node.svg"), ICON_SIZE),
            add_edge_tool: Self::rasterize_svg(images, include_bytes!("../assets/icons/vector.svg"), ICON_SIZE),
            move_tool: Self::rasterize_svg(images, include_bytes!("../assets/icons/move-cursor.svg"), ICON_SIZE),
            play: Self::rasterize_svg(images, include_bytes!("../assets/icons/play.svg"), ICON_SIZE),
            pause: Self::rasterize_svg(images, include_bytes!("../assets/icons/pause.svg"), ICON_SIZE),
            stop: Self::rasterize_svg(images, include_bytes!("../assets/icons/stop.svg"), ICON_SIZE),
            checkmark: Self::rasterize_svg(images, include_bytes!("../assets/icons/x.svg"), ICON_SIZE),
        }
    }
}

/// Startup system that loads all UI icons into the UiIcons resource.
pub fn load_icons(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icons = UiIcons::load_all(&mut images);
    commands.insert_resource(icons);
}
