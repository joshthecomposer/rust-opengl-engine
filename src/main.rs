mod shaders;
mod camera;
mod game_state;
mod some_data;
mod macros;
mod enums_types;
mod sparse_set;
mod uniforms;
mod entity_manager;
mod lights;
mod math_utils;
mod model;
mod mesh;
mod level;
mod grid;
mod renderer;
mod animation;

use std::fs::OpenOptions;
use std::io::Write;

use game_state::GameState;
use model::Model;
use russimp::scene::{Scene, PostProcess};
use russimp::Russult;
use sparse_set::SparseSet;

use crate::grid::Grid;

fn main() {
    let mut scene = Scene::from_file("resources/models/my_obj/TestBone/arm.fbx", vec![]);
    
    let debug_output = format!("{:#?}", scene);

    // Write to a file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug_output.txt")
        .expect("Failed to open debug file");

    writeln!(file, "{}", debug_output).expect("Failed to write to file");



   let mut state = GameState::new();
   while !state.window.should_close() {
       state.process_events();
       state.update();
       state.render();
   }
}


