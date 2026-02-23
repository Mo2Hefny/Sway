//! Bevy SystemParam definitions for node syncing.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::components::LimbSet;
use crate::core::Node;
use crate::editor::components::*;
use crate::ui::state::DisplaySettings;

#[derive(SystemParam)]
pub struct NodeIterationParams<'w, 's> {
    pub query: Query<'w, 's, (Entity, &'static Node, &'static mut Transform, &'static Children, Option<&'static LimbSet>, &'static mut NodeVisualCache), (Changed<Node>, Without<ContactPoint>, Without<LookVector>, Without<EyeVisual>, Without<TargetMarker>, Without<DirectionVector>, Without<AngleArc>)>,
}

#[derive(SystemParam)]
pub struct NodeSyncParams<'w, 's> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<ColorMaterial>>,
    pub visual_query: Query<'w, 's, &'static mut Mesh2d, With<NodeVisual>>,
    pub contact_query: Query<'w, 's, &'static mut Transform, (With<ContactPoint>, Without<Node>, Without<LookVector>, Without<EyeVisual>, Without<TargetMarker>, Without<DirectionVector>, Without<AngleArc>)>,
    pub contact_children: Query<'w, 's, Entity, With<ContactPoint>>,
    pub look_query: Query<'w, 's, &'static mut Transform, (With<LookVector>, Without<Node>, Without<ContactPoint>, Without<EyeVisual>, Without<TargetMarker>, Without<DirectionVector>, Without<AngleArc>)>,
    pub look_children: Query<'w, 's, Entity, With<LookVector>>,
    pub eye_query: Query<'w, 's, &'static mut Transform, (With<EyeVisual>, Without<Node>, Without<ContactPoint>, Without<LookVector>, Without<TargetMarker>, Without<DirectionVector>, Without<AngleArc>)>,
    pub eye_children: Query<'w, 's, Entity, With<EyeVisual>>,
    pub target_query: Query<'w, 's, &'static mut Transform, (With<TargetMarker>, Without<Node>, Without<ContactPoint>, Without<LookVector>, Without<EyeVisual>, Without<DirectionVector>, Without<AngleArc>)>,
    pub target_children: Query<'w, 's, Entity, With<TargetMarker>>,
    pub dir_query: Query<'w, 's, &'static mut Transform, (With<DirectionVector>, Without<Node>, Without<ContactPoint>, Without<LookVector>, Without<EyeVisual>, Without<TargetMarker>, Without<AngleArc>)>,
    pub dir_children: Query<'w, 's, Entity, With<DirectionVector>>,
    pub arc_query: Query<'w, 's, &'static mut Mesh2d, (With<AngleArc>, Without<NodeVisual>)>,
    pub arc_children: Query<'w, 's, Entity, With<AngleArc>>,
    pub material_query: Query<'w, 's, &'static MeshMaterial2d<ColorMaterial>>,
    pub target_parent_query: Query<'w, 's, (Entity, &'static ChildOf), With<TargetMarker>>,
}

#[derive(SystemParam)]
pub struct NodeVisibilityParams<'w, 's> {
    pub display_settings: Res<'w, DisplaySettings>,
    pub node_visuals: Query<'w, 's, &'static mut Visibility, (With<NodeVisual>, Without<ContactPoint>, Without<LookVector>, Without<EyeVisual>, Without<TargetMarker>, Without<DirectionVector>, Without<AngleArc>)>,
    pub contacts: Query<'w, 's, &'static mut Visibility, (With<ContactPoint>, Without<NodeVisual>, Without<LookVector>, Without<EyeVisual>, Without<TargetMarker>, Without<DirectionVector>, Without<AngleArc>)>,
    pub looks: Query<'w, 's, &'static mut Visibility, (With<LookVector>, Without<NodeVisual>, Without<ContactPoint>, Without<EyeVisual>, Without<TargetMarker>, Without<DirectionVector>, Without<AngleArc>)>,
    pub targets: Query<'w, 's, &'static mut Visibility, (With<TargetMarker>, Without<NodeVisual>, Without<ContactPoint>, Without<EyeVisual>, Without<DirectionVector>, Without<AngleArc>)>,
    pub dirs: Query<'w, 's, &'static mut Visibility, (With<DirectionVector>, Without<NodeVisual>, Without<ContactPoint>, Without<LookVector>, Without<EyeVisual>, Without<TargetMarker>, Without<AngleArc>)>,
    pub eyes: Query<'w, 's, &'static mut Visibility, (With<EyeVisual>, Without<NodeVisual>, Without<ContactPoint>, Without<LookVector>, Without<TargetMarker>, Without<DirectionVector>, Without<AngleArc>)>,
    pub arcs: Query<'w, 's, &'static mut Visibility, (With<AngleArc>, Without<NodeVisual>, Without<ContactPoint>, Without<LookVector>, Without<EyeVisual>, Without<TargetMarker>, Without<DirectionVector>)>,
}
