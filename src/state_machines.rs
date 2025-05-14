use glam::Vec3;

use crate::{entity_manager::EntityManager, enums_types::{Faction, SimState}};

pub fn update(em: &mut EntityManager) {
    entity_sim_state_machine(em);
}

fn entity_sim_state_machine(em: &mut EntityManager) {
    for fac in em.factions.iter() {
        if *fac.value() == Faction::Enemy {
            let state = em.sim_states.get_mut(fac.key()).unwrap();
            let player_key = em.factions.iter().find(|e| *e.value() == Faction::Player).unwrap().key();
            let player_pos = em.transforms.get(player_key).unwrap().position;
            let entity_pos = em.transforms.get(fac.key()).unwrap().position;
            let animator = em.animators.get_mut(fac.key()).unwrap();
            let destination = em.destinations.get_mut(fac.key()).unwrap();

            let trans = em.transforms.get(fac.key()).unwrap();

            let next_state = (|| match state {
                SimState::Dancing => {
                    *destination = entity_pos;
                    SimState::Dancing
                },
                SimState::Waiting => {
                    animator.set_next_animation("Idle");
                    *destination = entity_pos;

                    let to_player = (player_pos - entity_pos).with_y(0.0).normalize();
                    let forward = (trans.rotation * trans.original_rotation.inverse() * -Vec3::Z).with_y(0.0).normalize();
                    let alignment = forward.dot(to_player);
                    let fov_threshold = 0.5; // cos(30 degrees);

                    let view_distance = 12.0;

                    let player_in_range = entity_pos.distance(player_pos) <= view_distance;

                    if  alignment >= fov_threshold && player_in_range {
                        return SimState::Aggro
                    }

                    SimState::Waiting
                },
                SimState::Aggro => {
                    animator.set_next_animation("Run");
                    *destination = player_pos;

                    if entity_pos.distance(player_pos) > 12.0 {
                        return SimState::Waiting
                    } 

                    SimState::Aggro
                },
            })();

            *state = next_state;
        }
    }
}
