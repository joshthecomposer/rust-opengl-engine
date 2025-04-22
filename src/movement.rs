use std::collections::HashSet;

use glam::{vec3, Quat, Vec3};
use russimp_sys::built_info::PKG_AUTHORS;

use crate::{camera::Camera, entity_manager::EntityManager, enums_types::EntityType, terrain::Terrain};

pub fn handle_player_movement(pressed_keys: &HashSet<glfw::Key>, em: &mut EntityManager, player_key: usize, delta: f64, camera: &Camera, terrain: &Terrain) {
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
        new_rotation = Some(rot * em.transforms.get(player_key).unwrap().original_rotation.normalize());
        "Run"
    } else {
        new_rotation = None;
        "Idle"
    };

    let transform = em.transforms.get_mut(player_key).unwrap();
    let rotator = em.rotators.get_mut(player_key).unwrap();
    if rotator.next_rot != rotator.cur_rot {
        rotator.blend_factor += delta as f32 / rotator.blend_time;
        if rotator.blend_factor >= 1.0 {
            rotator.blend_factor = 0.0;
            rotator.cur_rot = rotator.next_rot;
        }
    }
    let animator = em.animators.get_mut(player_key).unwrap();

    animator.next_animation = new_state.to_string();

    if let Some(rot) = new_rotation {
        if rotator.blend_factor == 0.0 && rot != rotator.cur_rot {
            rotator.next_rot = rot;
        }

        transform.rotation = rotator.cur_rot.slerp(rotator.next_rot, rotator.blend_factor);
    }

    transform.position += velocity;
}

pub fn handle_npc_movement(em: &mut EntityManager, terrain: &Terrain) {
    for model in em.models.iter() {
        if let Some(ent_type) = em.entity_types.get(model.key()) {
            if ent_type != &EntityType::Terrain {
                if let Some(trans) = em.transforms.get_mut(model.key()) {
                    trans.position.y = terrain.get_height_at(trans.position.x, trans.position.z);
                }
            }
        }
    }

    for model in em.ani_models.iter() {
        if let Some(trans) = em.transforms.get_mut(model.key()) {
            trans.position.y = terrain.get_height_at(trans.position.x, trans.position.z);
        }
    }
}

pub fn revolve_around_something(object: &mut Vec3, target: &Vec3, elapsed: f32, radius: f32, speed: f32) {
    let angle = elapsed * speed;

    object.x = target.x + radius * angle.cos();
    object.z = target.z + radius * angle.sin();
    object.y = target.y + 1.0;
}
