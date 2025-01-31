mod shaders;
mod camera;
mod game_state;
mod some_data;
mod macros;
mod enums_types;
mod sparse_set;

use game_state::GameState;
use sparse_set::SparseSet;

fn main() {
    let mut state = GameState::new();
    let mut sparse_set:SparseSet<String> = SparseSet::with_capacity(100);

    sparse_set.insert(0, "Poop".to_string());
    sparse_set.insert(1, "Butts".to_string());
    sparse_set.insert(2, "Pee".to_string());
    
    for i in sparse_set.into_iter() {
        println!("{}",i.value);
    }
   // while !state.window.should_close() {
   //     state.process_events();
   //     state.update();Poop
   //     state.render();
   // }
}


