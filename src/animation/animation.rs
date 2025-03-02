use std::collections::HashMap;

use glam::Mat4;
use russimp::{node::Node, scene::{PostProcess, Scene}};

use super::{ani_bone::AniBone, ani_model::{AniModel, BoneInfo}};

#[derive(Clone, Debug)]
pub struct AssimpNodeData {
    transformation: Mat4,
    name: String,
    children: Vec<AssimpNodeData>,
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
    duration: f64,
    ticks_per_second: f64,
    bones: Vec<AniBone>,
    root_node: AssimpNodeData,
    bone_info_map: HashMap<String, BoneInfo>,
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

        let ai_animation = scene.animations.get(0).unwrap();
        let duration = ai_animation.duration;
        let ticks_per_second = ai_animation.ticks_per_second;
        let mut dest = AssimpNodeData::new();

        Self::read_heirarchy_data(&mut dest, &scene.root.unwrap());
        // Self::read_missing_bones(model, &mut ai_animation);

        let mut anim = Self {
            duration,
            ticks_per_second,
            bones: Vec::new(),
            root_node: dest,
            bone_info_map: HashMap::new(),
        };


        anim
    }

    pub fn read_missing_bones(model: &mut AniModel, animation: &mut Animation) {
        // TODO:
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
