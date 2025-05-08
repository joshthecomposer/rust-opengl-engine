use crate::{entity_manager::EntityManager, enums_types::{Faction, SimState}};

pub fn entity_sim_state_machine(em: &mut EntityManager) {
    for fac in em.factions.iter() {
        if *fac.value() == Faction::Enemy {
            let state = em.sim_states.get_mut(fac.key()).unwrap();
            let player_key = em.factions.iter().find(|e| *e.value() == Faction::Player).unwrap().key();
            let player_pos = em.transforms.get(player_key).unwrap().position;
            let entity_pos = em.transforms.get(fac.key()).unwrap().position;
            let animator = em.animators.get_mut(fac.key()).unwrap();
            let destination = em.destinations.get_mut(fac.key()).unwrap();

            let next_state = match state {
                SimState::Dancing => {
                    SimState::Dancing
                },
                SimState::Waiting => {
                    if entity_pos.distance(player_pos) <= 12.0 {
                        *destination = player_pos;
                        animator.set_next_animation("Run");
                        SimState::Aggro
                    } else {
                        animator.set_next_animation("Idle");
                        SimState::Waiting
                    }
                },
                SimState::Aggro => {
                    if entity_pos.distance(player_pos) > 12.0 {
                        *destination = entity_pos;
                        animator.set_next_animation("Idle");
                        SimState::Waiting
                    } else {
                        *destination = player_pos;
                        animator.set_next_animation("Run");
                        SimState::Aggro
                    }
                },
            };

            *state = next_state;
        }
    }
}
