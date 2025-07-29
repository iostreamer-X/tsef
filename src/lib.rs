use std::path::PathBuf;

use path_matchers::{PathMatcher, glob};

/*
 * State Machine Diagram:
 *
 *     ┌─────────────────┐         ┌───────────────────┐
 *     │   ParseToPause  │◄───────►│  ParseToContinue  │
 *     └─────────┬───────┘         └─────────┬─────────┘
 *               │                           │
 *               │                           │
 *               ▼                           ▼
 *     ┌─────────────────────────────────────────────────┐
 *     │            CheckEnd(bool, u8)                   │
 *     └─────┬───────────────────┬───────────────────┬───┘
 *           │                   │                   │
 *           ▼                   ▼                   ▼
 *     ┌─────────┐     ┌─────────────────┐     ┌─────────────────┐
 *     │   End   │     │   ParseToPause  │     │ ParseToContinue │
 *     └─────────┘     └─────────────────┘     └─────────────────┘
 *
 * State Transitions:
 * • ParseToPause ⟷ ParseToContinue (bidirectional)
 * • ParseToPause → CheckEnd
 * • ParseToContinue → CheckEnd
 * • CheckEnd → ParseToPause
 * • CheckEnd → ParseToContinue
 * • CheckEnd → End
 */
#[derive(PartialEq, Eq, Debug)]
pub enum State {
    ParseToPause,
    ParseToContinue,
    CheckEnd(bool, u8),
    End,
}

pub trait StateMachine {
    fn is_finished(&self) -> bool;
    fn run(&mut self, line: &String) -> (&State, bool);
}

pub mod ansi_state_machine;
pub mod simple_state_machine;

pub fn path_matches(list: &Vec<String>, path: &str) -> bool {
    if list.len() == 0 {
        return true;
    }
    for item in list {
        let matcher =
            glob(item.as_str()).expect("Invalid path string! Please provide a correct one.");
        if matcher.matches(&PathBuf::from(&path)) {
            return true;
        }
    }

    return false;
}
