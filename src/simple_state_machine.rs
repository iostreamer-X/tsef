use crate::{State, StateMachine, path_matches};

pub struct SimpleStateMachine {
    pub state: State,
    pub include: Vec<String>,
}

impl SimpleStateMachine {
    pub fn new(include: Vec<String>) -> Self {
        return Self {
            state: State::ParseToPause,
            include,
        };
    }
}

impl StateMachine for SimpleStateMachine {
    fn run(&mut self, line: &String) -> (&State, bool) {
        let path = line.split("(").next().expect("Received invalid output!");
        let should_block = !path_matches(&self.include, path);
        return (&self.state, !should_block);
    }

    fn is_finished(&self) -> bool {
        false
    }
}
