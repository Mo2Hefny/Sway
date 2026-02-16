use bevy::prelude::*;

use crate::core::components::DistanceConstraint;
use crate::core::resources::ConstraintGraph;
use crate::ui::state::PlaybackState;

pub fn update_constraint_graph(
    playback: Res<PlaybackState>,
    constraints: Query<&DistanceConstraint, Changed<DistanceConstraint>>,
    all_constraints: Query<&DistanceConstraint>,
    mut graph: ResMut<ConstraintGraph>,
    mut has_built: Local<bool>,
) {
    if !playback.is_playing() {
        *has_built = false;
        return;
    }

    let should_rebuild = !*has_built || !constraints.is_empty();
    if !should_rebuild {
        return;
    }

    *has_built = true;
    let constraint_list: Vec<DistanceConstraint> = all_constraints.iter().cloned().collect();
    graph.rebuild(&constraint_list);
}
