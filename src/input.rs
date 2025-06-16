use std::collections::HashSet;

use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec3Swizzles, Vec4Swizzles};
use glfw::MouseButton;

use crate::{camera::{self, Camera}, entity_manager::EntityManager, enums_types::{AnimationType, Faction}};

pub fn handle_keyboard_input(key: glfw::Key, action: glfw::Action, pressed_keys: &mut HashSet<glfw::Key>) {
    match action {
        glfw::Action::Press => { pressed_keys.insert(key); }
        glfw::Action::Release => { pressed_keys.remove(&key); }
        _=> ()
    }
}

pub fn handle_mouse_motion() {
}

pub fn handle_mouse_input(button: MouseButton, action: glfw::Action, cursor_pos: Vec2, screen_size: Vec2, camera: &Camera, em: &mut EntityManager, pressed_keys: &HashSet<glfw::Key>) {
    match action {
        glfw::Action::Press => { 
            if button == glfw::MouseButtonLeft {

                if !pressed_keys.contains(&glfw::Key::LeftShift) {
                    em.selected.clear();


                    let player_id = em.factions.iter().filter(|f| *f.value() == Faction::Player).last().unwrap().key();
                    let animator = em.animators.get_mut(player_id).unwrap();

                    animator.set_next_animation(AnimationType::Slash);
                }

                let (ray_origin, ray_dir) = mouse_ray_from_screen(cursor_pos, screen_size, camera);

                let mut closest = None;
                let mut min_t = f32::MAX;

                for cyl in em.cylinders.iter() {
                    let trans = em.transforms.get(cyl.key()).unwrap();
                    let cyl_base = trans.position;

                    let height = cyl.value.h;
                    let radius = cyl.value.r;

                    if let Some(t) = ray_hits_cylinder(ray_origin, ray_dir, cyl_base, height, radius) {
                        if t < min_t {
                            min_t = t;
                            closest = Some(cyl.key());
                        }
                    }
                }

                if let Some(id) = closest {
                    let parent_id = em.parents.get(id).unwrap().parent_id;

                    em.selected.push(parent_id);
                    em.selected.push(id);
                }
            }
        },
        glfw::Action::Release => (),
        _ => ()
   }
}

fn mouse_ray_from_screen(
    mouse_pos: Vec2,
    screen_size: Vec2,
    camera: &Camera,
) -> (Vec3, Vec3) {
    let (mouse_x, mouse_y) = (mouse_pos.x, mouse_pos.y);
    let (screen_w, screen_h) = (screen_size.x, screen_size.y);
    
    // Calculate NDC
    // transform x to match opengl left-to-right convention
    let x = (2.0 * mouse_x) / screen_w - 1.0;
    // invert y. Screen space Y is top-down whereas opengl is bottom-up
    let y = 1.0 - (2.0 * mouse_y) / screen_h;
    // the ray goes INTO the screen (negative z)
    let z = -1.0;
    let ray_ndc = vec4(x, y, z, 1.0);

    // we want to reverse the transform pipeline. clip -> view -> world
    let inv_proj = camera.projection.inverse();
    let inv_view = camera.view.inverse();

    let ray_eye = inv_proj * ray_ndc;
    let ray_eye = vec4(ray_eye.x, ray_eye.y, -1.0, 0.0);

    let ray_world = (inv_view * ray_eye).xyz().normalize();
    let camera_pos = camera.position;

    (camera_pos, ray_world)
}

fn ray_hits_cylinder(
    ray_origin: Vec3,
    ray_dir: Vec3,
    cyl_base: Vec3,
    height: f32,
    radius: f32,
) -> Option<f32> {
    // Project onto XZ plane
    let d = vec2(ray_dir.x, ray_dir.z);
    let o = vec2(ray_origin.x - cyl_base.x, ray_origin.z - cyl_base.z);

    let a = d.dot(d);
    let b = 2.0 * o.dot(d);
    let c = o.dot(o) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_disc = discriminant.sqrt();
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);

    for &t in &[t1, t2] {
        if t < 0.0 { continue; }

        let y = ray_origin.y + t * ray_dir.y;
        if y >= cyl_base.y && y <= cyl_base.y + height {
            return Some(t);
        }
    }

    None
}











