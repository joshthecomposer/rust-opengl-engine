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
mod grid;
mod renderer;
mod animation;
mod debug;
mod input;
mod movement_system;
mod ui;
mod sound;
mod config;
mod terrain;
// mod deprecated;
mod collision_system;
mod state_machines;
mod particles;

use std::{fs::{self, OpenOptions}, path::Path};

use game_state::GameState;
use std::io::Write;

fn main() {
    let mut state = GameState::new();
    while !state.window.should_close() {
        state.process_events();
        state.update();
        state.render();
    }

}
