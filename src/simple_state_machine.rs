use crate::{State, StateMachine, path_matches};

pub struct SimpleStateMachine {
    pub state: State,
    pub block: Vec<String>,
}

impl SimpleStateMachine {
    pub fn new(block: Vec<String>) -> Self {
        return Self {
            state: State::ParseToPause,
            block,
        };
    }
}

impl StateMachine for SimpleStateMachine {
    fn run(&mut self, line: &String) -> (&State, bool) {
        let path = line.split("(").next().expect("Received invalid output!");
        let should_block = path_matches(&self.block, path);
        return (&self.state, !should_block);
    }

    fn is_finished(&self) -> bool {
        false
    }
}
