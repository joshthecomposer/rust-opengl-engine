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
mod grid;
mod renderer;
mod animation;
mod debug;

use game_state::GameState;

fn main() {
   let mut state = GameState::new();
   while !state.window.should_close() {
       state.process_events();
       state.update();
       state.render();
   }
}
