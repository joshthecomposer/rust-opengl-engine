use glam::Vec3;

use crate::{entity_manager::EntityManager, enums_types::{AnimationType, Faction, SimState}};

pub fn update(em: &mut EntityManager, dt: f32) {
    entity_sim_state_machine(em, dt);
}

fn entity_sim_state_machine(em: &mut EntityManager, dt: f32) {
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
                    animator.set_next_animation(AnimationType::Dance);
                    SimState::Dancing
                },
                SimState::Waiting => {
                    animator.set_next_animation(AnimationType::Idle);
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
                    animator.set_next_animation(AnimationType::Run);
                    *destination = player_pos;

                    if entity_pos.distance(player_pos) > 12.0 {
                        return SimState::Waiting
                    } 

                    SimState::Aggro
                },
                SimState::Dying => {
                    animator.set_next_animation(AnimationType::Death);
                    *destination = entity_pos;

                    let anim = animator.animations.get(&AnimationType::Death).unwrap();
                    if anim.current_time >= anim.duration - 0.001 {
                        return SimState::Dead { time: 0.0, target_time: 5.0 }
                    }
                    
                    SimState::Dying
                },
                SimState::Dead { time, target_time } => {
                    animator.set_next_animation(AnimationType::Death);

                    let new_time = *time + dt;

                    if new_time >= *target_time {
                        em.entity_trashcan.push(fac.key());
                    }

                    SimState::Dead { time: new_time, target_time: *target_time }
                },
            })();

            *state = next_state;
        }
    }
}
