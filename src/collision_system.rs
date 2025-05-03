use glam::{Mat3, Vec3};

use crate::{entity_manager::EntityManager, enums_types::{Size3, Transform}};

pub fn update(em: &mut EntityManager) {
    handle_entity_collisions(em);
}

fn handle_entity_collisions(em: &mut EntityManager) {
    let mut collisions: Vec<(usize, usize)> = vec![];

    for c1 in em.cylinders.iter() {
        for c2 in em.cylinders.iter() {
            if c1.key() >= c2.key() {
                continue;
            }

            let id1 = c1.key();
            let id2 = c2.key();

            if let (Some(t1), Some(t2)) = (em.transforms.get(id1), em.transforms.get(id2)) {
                let cyl1 = c1.value();
                let cyl2 = c2.value();

                let dx = t1.position.x - t2.position.x;
                let dz = t1.position.z - t2.position.z;
                let dist_sq = dx * dx + dz * dz;
                let radius_sum = cyl1.r + cyl2.r;
                let overlap_horizontal = dist_sq <= radius_sum * radius_sum;

                let y1_min = t1.position.y;
                let y1_max = y1_min + cyl1.h;
                let y2_min = t2.position.y;
                let y2_max = y2_min + cyl2.h;
                let overlap_vertical = y1_min < y2_max && y1_max > y2_min;

                if overlap_horizontal && overlap_vertical {
                    collisions.push((id1, id2));
                }
            }
        }
    }

    if !collisions.is_empty() {
        println!("COLLISION DETECTED!!!!!!");
    }
}
