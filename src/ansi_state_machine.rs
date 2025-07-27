use core::panic;

use ansi_parser::{AnsiParser, AnsiSequence, Output};

use crate::path_matches;

#[derive(PartialEq, Eq, Debug)]
pub enum State {
    ParseToPause,
    ParseToContinue,
    CheckEnd(bool),
    End,
}

#[derive(PartialEq, Eq)]
enum ParseResult {
    CheckEnd(bool),
    KeepState,
    Flip,
}

pub struct AnsiStateMachine {
    pub identifier: AnsiSequence,
    pub state: State,
    pub block: Vec<String>,
}

impl AnsiStateMachine {
    pub fn new(identifier: AnsiSequence, block: Vec<String>) -> Self {
        return Self {
            identifier,
            state: State::ParseToPause,
            block,
        };
    }

    fn parse_line(&self, line: &String, parsing_to_pause: bool) -> ParseResult {
        if line.len() == 0 {
            return ParseResult::CheckEnd(self.state == State::ParseToPause);
        }
        let mut ansi_line = line.ansi_parse();
        let ansi = ansi_line.next();
        if ansi.is_none() {
            return ParseResult::KeepState;
        }
        let should_check_path = ansi
            .map(|t| match t {
                Output::TextBlock(_text) => false,
                Output::Escape(seq) => seq == self.identifier,
            })
            .unwrap_or(false);

        if !should_check_path {
            return ParseResult::KeepState;
        }

        let path = ansi_line.next().expect("Error parsing output!");
        let path = match path {
            Output::TextBlock(p) => p,
            Output::Escape(_) => panic!("Error parsing output!"),
        };

        let should_block = path_matches(&self.block, path);
        if parsing_to_pause {}
        let should_flip = match parsing_to_pause {
            true => should_block,
            false => !should_block,
        };
        if should_flip {
            return ParseResult::Flip;
        }
        return ParseResult::KeepState;
    }

    fn process_parse_result(&mut self, line: &String, parsing_to_pause: bool) -> (&State, bool) {
        let parse_result = self.parse_line(line, parsing_to_pause);
        if let ParseResult::CheckEnd(go_back_state) = parse_result {
            self.state = State::CheckEnd(go_back_state);
        } else if parse_result == ParseResult::Flip {
            self.state = match parsing_to_pause {
                true => State::ParseToContinue,
                false => State::ParseToPause,
            };
            return (&self.state, !parsing_to_pause);
        }
        return (&self.state, parsing_to_pause);
    }

    pub fn run(&mut self, line: &String) -> (&State, bool) {
        match self.state {
            State::ParseToPause => {
                return self.process_parse_result(line, true);
            }
            State::ParseToContinue => {
                return self.process_parse_result(line, false);
            }
            State::CheckEnd(should_parse_to_pause) => {
                if line.len() == 0 {
                    self.state = State::End;
                    return (&self.state, false);
                }
                if should_parse_to_pause {
                    self.state = State::ParseToPause;
                } else {
                    self.state = State::ParseToContinue;
                }
                return self.run(line);
            }
            State::End => {
                return (&self.state, true);
            }
        }
    }
}
