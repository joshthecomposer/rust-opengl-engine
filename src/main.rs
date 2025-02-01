mod shaders;
mod camera;
mod game_state;
mod some_data;
mod macros;
mod enums_types;
mod sparse_set;
mod uniforms;

use game_state::GameState;
use sparse_set::SparseSet;

fn main() {
    // let mut sparse_set:SparseSet<String> = SparseSet::with_capacity(100);
    // sparse_set.insert(0, "Poop".to_string());
    // sparse_set.insert(1, "Butts".to_string());
    // sparse_set.insert(2, "Pee".to_string());
    // sparse_set.remove(1);
    // sparse_set.insert(1, "Asses".to_string());
    // sparse_set.insert(3, "Ronald".to_string());
    // sparse_set.insert(4, "McDonald".to_string());
    // sparse_set.remove(0);
    // sparse_set.insert(0, "Barney".to_string());
    // println!("{:?}", sparse_set.sparse);
    // println!("{:?}", sparse_set.dense);

    // println!("{}", sparse_set.get(0).unwrap());
    
    let mut state = GameState::new();
       while !state.window.should_close() {
           state.process_events();
           state.update();
           state.render();
       }
}


