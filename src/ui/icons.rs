use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle, egui};

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

/// Cached egui texture IDs for UI icons (registered on first use).
#[derive(Resource, Default)]
pub struct EguiIconTextures {
    pub hamburger: Option<egui::TextureId>,
    pub import: Option<egui::TextureId>,
    pub export: Option<egui::TextureId>,
    pub caret_right: Option<egui::TextureId>,
    pub caret_left: Option<egui::TextureId>,
    pub properties: Option<egui::TextureId>,
    pub transform: Option<egui::TextureId>,
    pub constraints: Option<egui::TextureId>,
    pub cursor_tool: Option<egui::TextureId>,
    pub add_node_tool: Option<egui::TextureId>,
    pub add_edge_tool: Option<egui::TextureId>,
    pub move_tool: Option<egui::TextureId>,
    pub play: Option<egui::TextureId>,
    pub pause: Option<egui::TextureId>,
    pub stop: Option<egui::TextureId>,
    pub checkmark: Option<egui::TextureId>,
}

impl EguiIconTextures {
    pub fn ensure_registered(&mut self, contexts: &mut EguiContexts, icons: &UiIcons) {
        if self.hamburger.is_some() {
            return;
        }
        self.hamburger = Some(contexts.add_image(EguiTextureHandle::Strong(icons.hamburger.clone())));
        self.import = Some(contexts.add_image(EguiTextureHandle::Strong(icons.import.clone())));
        self.export = Some(contexts.add_image(EguiTextureHandle::Strong(icons.export.clone())));
        self.caret_right = Some(contexts.add_image(EguiTextureHandle::Strong(icons.caret_right.clone())));
        self.caret_left = Some(contexts.add_image(EguiTextureHandle::Strong(icons.caret_left.clone())));
        self.properties = Some(contexts.add_image(EguiTextureHandle::Strong(icons.properties.clone())));
        self.transform = Some(contexts.add_image(EguiTextureHandle::Strong(icons.transform.clone())));
        self.constraints = Some(contexts.add_image(EguiTextureHandle::Strong(icons.constraints.clone())));
        self.cursor_tool = Some(contexts.add_image(EguiTextureHandle::Strong(icons.cursor_tool.clone())));
        self.add_node_tool = Some(contexts.add_image(EguiTextureHandle::Strong(icons.add_node_tool.clone())));
        self.add_edge_tool = Some(contexts.add_image(EguiTextureHandle::Strong(icons.add_edge_tool.clone())));
        self.move_tool = Some(contexts.add_image(EguiTextureHandle::Strong(icons.move_tool.clone())));
        self.play = Some(contexts.add_image(EguiTextureHandle::Strong(icons.play.clone())));
        self.pause = Some(contexts.add_image(EguiTextureHandle::Strong(icons.pause.clone())));
        self.stop = Some(contexts.add_image(EguiTextureHandle::Strong(icons.stop.clone())));
        self.checkmark = Some(contexts.add_image(EguiTextureHandle::Strong(icons.checkmark.clone())));
    }
}

/// Converts Bevy Color to egui Color32.
pub fn to_egui_color(c: Color) -> egui::Color32 {
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
