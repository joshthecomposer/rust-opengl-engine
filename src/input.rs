use std::collections::HashSet;

pub fn handle_keyboard_input(key: glfw::Key, action: glfw::Action, pressed_keys: &mut HashSet<glfw::Key>) {
    match action {
        glfw::Action::Press => { pressed_keys.insert(key); }
        glfw::Action::Release => { pressed_keys.remove(&key); }
        _=> ()
    }
}

pub fn handle_mouse_motion() {
}
