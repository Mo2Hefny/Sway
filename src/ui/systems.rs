//! General UI systems for the editor.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::core::constants::{MAX_CONSTRAINT_DISTANCE, MIN_CONSTRAINT_DISTANCE};
use crate::core::components::LimbSet;
use crate::core::{
    DistanceConstraint, Node as SimNode, Playground, spawn_scene_data, PendingFileOp,
};
use crate::editor::components::{ConstraintPreview, ConstraintVisual, NodeVisual};
use crate::editor::tools::selection::Selection;
use crate::ui::icons::{UiIcons, EguiIconTextures};
use crate::ui::panels::*;
use crate::ui::state::*;
use crate::ui::theme::*;

/// Main editor UI system: orchestrates all egui panels and updates InputState.
pub fn editor_ui_system(
    mut contexts: EguiContexts,
    (mut display_settings, mut panel_state, mut inspector_state): (
        ResMut<DisplaySettings>,
        ResMut<FloatingPanelState>,
        ResMut<InspectorState>,
    ),
    mut tool_state: ResMut<EditorToolState>,
    mut playback: ResMut<PlaybackState>,
    ui_visibility: Res<UiVisibility>,
    mut input_state: ResMut<InputState>,
    icons: Res<UiIcons>,
    mut egui_icons: ResMut<EguiIconTextures>,
    mut import_requested: ResMut<ImportRequested>,
    selection: Res<Selection>,
    mut playground: ResMut<Playground>,
    mut node_query: Query<(Entity, &mut SimNode)>,
    mut limb_set_query: Query<(Entity, &mut LimbSet)>,
    constraint_query: Query<(Entity, &DistanceConstraint)>,
    mut pending_actions: ResMut<PendingConstraintActions>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    egui_icons.ensure_registered(&mut contexts, &icons);
    let Ok(ctx) = contexts.ctx_mut() else { return };
    apply_theme(ctx);

    if !ui_visibility.visible {
        input_state.cursor_over_ui = false;
        return;
    }

    // DRAW PANELS
    draw_floating_panel(
        ctx,
        &egui_icons,
        &mut panel_state,
        &mut display_settings,
        &mut import_requested,
        &mut playground,
        &node_query,
        &constraint_query,
        &mut limb_set_query,
        &windows,
    );

    draw_toolbar(
        ctx,
        &egui_icons,
        &mut tool_state,
        &inspector_state,
    );

    draw_inspector_panel(
        ctx,
        &egui_icons,
        &mut inspector_state,
        &selection,
        &playground,
        &mut node_query,
        &mut limb_set_query,
        &constraint_query,
        &mut pending_actions,
    );

    draw_playback_toolbar(
        ctx,
        &egui_icons,
        &mut playback,
    );

    draw_instruction_hints(ctx);

    input_state.cursor_over_ui = ctx.wants_pointer_input();
}

/// Toggle UI visibility with H key.
pub fn toggle_ui_visibility(keyboard: Res<ButtonInput<KeyCode>>, mut ui_visibility: ResMut<UiVisibility>) {
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

/// Applies deferred import, constraint updates, and constraint deletes.
pub fn apply_editor_actions(
    mut commands: Commands,
    mut import_requested: ResMut<ImportRequested>,
    mut pending_file_op: ResMut<PendingFileOp>,
    mut pending_actions: ResMut<PendingConstraintActions>,
    mut selection: ResMut<Selection>,
    node_query: Query<(Entity, &mut SimNode)>,
    mut constraint_query: Query<(Entity, &mut DistanceConstraint)>,
    visual_entities: Query<Entity, Or<(With<NodeVisual>, With<ConstraintVisual>, With<ConstraintPreview>)>>,
) {
    let scene_to_import = import_requested.0.take().or(pending_file_op.import_data.take());
    if let Some(scene) = scene_to_import {
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
        for (c_entity, constraint) in constraint_query.iter() {
            if constraint.involves(entity) {
                commands.entity(c_entity).despawn();
            }
        }
        commands.entity(entity).despawn();
        selection.deselect();
    }
}
