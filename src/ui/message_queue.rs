use std::collections::VecDeque;

#[derive(PartialEq)]
pub enum UiMessage {
    LeftMouseClicked,
    WindowShouldClose,
    PauseToggle,
}

pub struct MessageQueue {
    pub queue: VecDeque<UiMessage>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn send(&mut self, msg: UiMessage) {
        self.queue.push_back(msg);
    }

    pub fn drain(&mut self) {
        self.queue.drain(..);
    }
}
