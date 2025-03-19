use std::collections::HashSet;

use glam::{vec3, Quat, Vec3};

use crate::entity_manager::EntityManager;

pub fn handle_player_movement(pressed_keys: &HashSet<glfw::Key>, em: &mut EntityManager, player_key: usize, delta: f64) {
    use glfw::Key::*;

    let speed = 5.0 * delta as f32;
    let diagonal_speed = speed * 0.707;
    let transform = em.transforms.get_mut(player_key).unwrap();
    let animator = em.animators.get_mut(player_key).unwrap();

    let mut direction_flags = 0;
    if pressed_keys.contains(&W) { direction_flags |= 8 }
    if pressed_keys.contains(&A) { direction_flags |= 4 }
    if pressed_keys.contains(&S) { direction_flags |= 2 }
    if pressed_keys.contains(&D) { direction_flags |= 1 }

    let (new_state, new_rotation, velocity) = match direction_flags {
        8 => ("Run", Some(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)), vec3(-speed, 0.0, 0.0)),
        4 => ("Run", Some(Quat::from_rotation_y(std::f32::consts::PI)), vec3(0.0, 0.0, speed)),
        2 => ("Run", Some(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)), vec3(speed, 0.0, 0.0)),
        1 => ("Run", Some(Quat::from_rotation_y(0.0)), vec3(0.0, 0.0, -speed)),

        // Diagonals 
        9  => ("Run", Some(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)), vec3(-diagonal_speed, 0.0, -diagonal_speed)),
        3 => ("Run", Some(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4)), vec3(diagonal_speed, 0.0, -diagonal_speed)),
        6  => ("Run", Some(Quat::from_rotation_y(-3.0 * std::f32::consts::FRAC_PI_4)), vec3(diagonal_speed, 0.0, diagonal_speed)),
        12 => ("Run", Some(Quat::from_rotation_y(3.0 * std::f32::consts::FRAC_PI_4)), vec3(-diagonal_speed, 0.0, diagonal_speed)),

        _ => ("Idle", None, Vec3::splat(0.0)),
    };

    animator.current_animation = new_state.to_string();

    if let Some(rot) = new_rotation {
        transform.rotation = rot * transform.original_rotation;
    }

    transform.position += velocity;
}
