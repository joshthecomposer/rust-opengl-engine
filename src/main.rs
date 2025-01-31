mod shaders;
mod camera;
mod game_state;
mod some_data;
mod macros;

use game_state::GameState;

fn main() {
    let mut state = GameState::new();

    while !state.window.should_close() {
        state.update();
        state.render();
    }
}


