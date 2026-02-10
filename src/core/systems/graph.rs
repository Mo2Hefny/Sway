use bevy::prelude::*;

use crate::core::components::DistanceConstraint;
use crate::core::resources::ConstraintGraph;
use crate::ui::state::PlaybackState;

pub fn update_constraint_graph(
    playback: Res<PlaybackState>,
    constraints: Query<&DistanceConstraint>,
    mut graph: ResMut<ConstraintGraph>,
) {
    if !playback.is_playing() {
        return;
    }

    let constraint_list: Vec<DistanceConstraint> = constraints.iter().cloned().collect();
    graph.rebuild(&constraint_list);
}
