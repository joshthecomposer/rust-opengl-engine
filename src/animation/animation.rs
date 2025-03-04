use std::collections::HashMap;

use gl::PointSize;
use glam::Mat4;
use russimp::{animation::Animation as RAnimation, bone, node::Node, scene::{PostProcess, Scene}};

use crate::debug::write::write_data;

use super::{ani_bone::AniBone, ani_model::{AniModel, BoneInfo}};

#[derive(Clone, Debug)]
pub struct AssimpNodeData {
    pub transformation: Mat4,
    pub name: String,
    pub children: Vec<AssimpNodeData>,
}

impl AssimpNodeData {
    pub fn new() -> Self {
        Self {
            transformation: Mat4::IDENTITY,
            name: "".to_string(),
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub duration: f64,
    pub ticks_per_second: f64,
    pub bones: Vec<AniBone>,
    pub root_node: AssimpNodeData,
    pub bone_info_map: HashMap<String, BoneInfo>,
    pub global_inverse_transformation: Mat4,
}

impl Animation {
    pub fn new(animation_path: String, model: &mut AniModel) -> Animation {
        println!("=============================================================");
        println!("BEGIN LOADING OF SCENE FOR ANIMATION: {}", animation_path);
        println!("=============================================================");

        let scene = Scene::from_file(
            animation_path.as_str(),
            vec![
                PostProcess::Triangulate,
                PostProcess::FlipUVs,
            ],
        ).unwrap();

        assert!(scene.root.is_some());

        write_data(&scene, "scene.txt");

        let ai_animation = scene.animations.get(0).unwrap();
        let duration = ai_animation.duration;
        let ticks_per_second = ai_animation.ticks_per_second;
        let mut dest = AssimpNodeData::new();

        let root_node = scene.root.as_ref().unwrap();

        Self::read_heirarchy_data(&mut dest, root_node);

        let root_transform = AniModel::russimp_mat4_to_glam(root_node.transformation.clone());
        let global_inverse_transformation = root_transform.inverse();

        let mut anim = Self {
            duration,
            ticks_per_second,
            bones: Vec::new(),
            root_node: dest,
            bone_info_map: HashMap::new(),
            global_inverse_transformation,
        };

        anim.read_missing_bones(model, &ai_animation);

        anim
    }

    pub fn find_bone(&mut self, input: &String) -> Option<&mut AniBone> {
        self.bones.iter_mut().find(|b| b.name == *input)
    }

    pub fn read_missing_bones(&mut self, model: &mut AniModel, ai_animation: &RAnimation) {
        let bone_count = &mut model.bone_counter;
        let bone_info_map = &mut model.bone_info_map;

        for channel in ai_animation.channels.iter() { 
            let bone_name = channel.name.clone();
            if !bone_info_map.contains_key(&bone_name) {
                bone_info_map.insert(bone_name.clone(), BoneInfo {
                    id: *bone_count,
                    offset: Mat4::IDENTITY,
                });
                *bone_count += 1;
            }

            self.bones.push(AniBone::new(
                bone_name.clone(),
                bone_info_map.get(&bone_name).unwrap().id as usize,
                channel
            ));
        }

        self.bone_info_map = bone_info_map.clone();
    }

    pub fn read_heirarchy_data(dest: &mut AssimpNodeData, src: &Node) {
        dest.name = src.name.clone();
        dest.transformation = AniModel::russimp_mat4_to_glam(src.transformation);
        println!("=============================================================");
        println!("READING HEIRARCHY DATA");
        println!("=============================================================");

        for child in src.children.borrow().iter() {
            let mut new_data = AssimpNodeData::new();
            Self::read_heirarchy_data(&mut new_data, &child);
            dest.children.push(new_data);
        }
    }
}
