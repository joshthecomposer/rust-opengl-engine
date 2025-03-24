use std::collections::HashSet;

use glam::{vec3, Quat, Vec3};

use crate::{camera::Camera, entity_manager::EntityManager};

pub fn handle_player_movement(pressed_keys: &HashSet<glfw::Key>, em: &mut EntityManager, player_key: usize, delta: f64, camera: &Camera) {
    let speed = 5.0 * delta as f32;
    let mut move_dir = vec3(0.0, 0.0, 0.0);

    let forward_flat = vec3(camera.forward.x, 0.0, camera.forward.z).normalize();
    let right_flat = vec3(camera.right.x, 0.0, camera.right.z).normalize();

    if pressed_keys.contains(&glfw::Key::W) {
        move_dir += forward_flat;
    }
    if pressed_keys.contains(&glfw::Key::S) {
        move_dir -= forward_flat;
    }
    if pressed_keys.contains(&glfw::Key::D) {
        move_dir += right_flat;
    }
    if pressed_keys.contains(&glfw::Key::A) {
        move_dir -= right_flat;
    }

    let mut velocity = vec3(0.0, 0.0, 0.0);
    let new_rotation: Option<Quat>;

    let new_state = if move_dir.length_squared() > 0.0 {
        move_dir = move_dir.normalize();
        velocity = move_dir * speed;

        let rot =Quat::from_rotation_y(f32::atan2(-move_dir.x, -move_dir.z));
        new_rotation = Some(rot * em.transforms.get(player_key).unwrap().original_rotation);
        "Run"
    } else {
        new_rotation = None;
        "Idle"
    };

    let transform = em.transforms.get_mut(player_key).unwrap();
    let animator = em.animators.get_mut(player_key).unwrap();

    animator.current_animation = new_state.to_string();

    if let Some(rot) = new_rotation {
        transform.rotation = rot;
    }

    transform.position += velocity;
}

pub fn revolve_around_something(object: &mut Vec3, target: &Vec3, elapsed: f32, radius: f32, speed: f32) {
    let angle = elapsed * speed;

    object.x = target.x + radius * angle.cos();
    object.z = target.z + radius * angle.sin();
    object.y = 1.0;
}
