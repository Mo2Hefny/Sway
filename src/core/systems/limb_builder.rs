//! System to automatically construct and update LimbSet components based on graph topology.

use bevy::prelude::*;

use crate::core::components::{Limb, LimbSet, Node, NodeType};
use crate::core::resources::ConstraintGraph;

pub fn limb_builder_system(
    graph: Res<ConstraintGraph>,
    mut commands: Commands,
    nodes: Query<(Entity, &Node)>,
    existing_limb_sets: Query<(Entity, &LimbSet)>,
) {
    if !graph.is_changed() {
        return;
    }

    let mut body_limbs: Vec<(Entity, Vec<Limb>)> = Vec::new();

    for (body_entity, body_node) in nodes.iter() {
        if body_node.node_type == NodeType::Limb {
            continue;
        }

        let Some(neighbors) = graph.adjacency.get(&body_entity) else {
            continue;
        };

        let mut limbs_for_body: Vec<Limb> = Vec::new();

        for &(neighbor_entity, _rest_len) in neighbors.iter() {
            let Ok((_, neighbor_node)) = nodes.get(neighbor_entity) else {
                continue;
            };
            if neighbor_node.node_type != NodeType::Limb {
                continue;
            }

            let chain = trace_limb_chain(neighbor_entity, body_entity, &graph, &nodes);
            if !chain.is_empty() {
                let chain_len = chain.len();
                limbs_for_body.push(Limb {
                    joints: chain,
                    flip_bend: vec![false; chain_len],
                    ..Default::default()
                });
            }
        }

        if !limbs_for_body.is_empty() {
            body_limbs.push((body_entity, limbs_for_body));
        }
    }

    let active_bodies: std::collections::HashSet<Entity> =
        body_limbs.iter().map(|(e, _)| *e).collect();

    for (entity, _) in existing_limb_sets.iter() {
        if !active_bodies.contains(&entity) {
            commands.entity(entity).remove::<LimbSet>();
        }
    }

    for (body_entity, new_limbs) in body_limbs {
        if let Ok((_, existing)) = existing_limb_sets.get(body_entity) {
            let existing_chains: Vec<&[Entity]> =
                existing.limbs.iter().map(|l| l.joints.as_slice()).collect();
            let new_chains: Vec<&[Entity]> =
                new_limbs.iter().map(|l| l.joints.as_slice()).collect();

            if existing_chains != new_chains {
                let mut merged: Vec<Limb> = Vec::with_capacity(new_limbs.len());
                for new_limb in new_limbs {
                    let preserved = existing.limbs.iter().find(|old| old.joints == new_limb.joints);
                    if let Some(old) = preserved {
                        merged.push(Limb {
                            joints: new_limb.joints,
                            lengths: Vec::new(),
                            ..old.clone()
                        });
                    } else {
                        merged.push(new_limb);
                    }
                }
                commands.entity(body_entity).insert(LimbSet { limbs: merged });
            }
        } else {
            commands
                .entity(body_entity)
                .insert(LimbSet { limbs: new_limbs });
        }
    }
}

// =============================================================================
// Private Methods
// =============================================================================

fn trace_limb_chain(
    start: Entity,
    body: Entity,
    graph: &ConstraintGraph,
    nodes: &Query<(Entity, &Node)>,
) -> Vec<Entity> {
    let mut chain = Vec::new();
    let mut current = start;
    let mut prev = body;
    let mut depth = 0;

    loop {
        if depth > 100 {
            warn!("Limb chain too long (>100 nodes), terminating trace.");
            break;
        }
        depth += 1;

        chain.push(current);

        let neighbors = match graph.adjacency.get(&current) {
            Some(n) => n,
            None => break,
        };

        let next_node = neighbors
            .iter()
            .map(|(e, _)| *e)
            .find(|&e| {
                if e == prev {
                    return false;
                }
                if let Ok((_, node)) = nodes.get(e) {
                    node.node_type == NodeType::Limb
                } else {
                    false
                }
            });

        if let Some(next) = next_node {
            if chain.contains(&next) {
                warn!("Limb cycle detected at {:?}, cutting chain.", next);
                break;
            }
            prev = current;
            current = next;
        } else {
            break;
        }
    }

    chain
}
