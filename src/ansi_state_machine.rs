use core::panic;

use ansi_parser::{AnsiParser, AnsiSequence, Output};

use crate::{State, StateMachine, path_matches};

#[derive(PartialEq, Eq)]
enum ParseResult {
    CheckEnd(bool),
    KeepState,
    Flip,
}

pub struct AnsiStateMachine {
    pub identifier: AnsiSequence,
    pub state: State,
    pub include: Vec<String>,
}

impl AnsiStateMachine {
    pub fn new(identifier: AnsiSequence, include: Vec<String>) -> Self {
        return Self {
            identifier,
            state: State::ParseToPause,
            include,
        };
    }

    fn parse_line(&self, line: &String, parsing_to_pause: bool) -> ParseResult {
        //If we get an empty line, we must check if the end is nigh
        if line.len() == 0 {
            return ParseResult::CheckEnd(self.state == State::ParseToPause);
        }

        //Next we check if we receive an ANSI sequence, if not we continue with our current state
        let mut ansi_line = line.ansi_parse();
        let ansi = ansi_line.next();
        if ansi.is_none() {
            return ParseResult::KeepState;
        }

        //But if we do receive an ANSI sequence, we check if it matches our identifier
        let should_check_path = ansi
            .map(|t| match t {
                Output::TextBlock(_text) => false,
                Output::Escape(seq) => seq == self.identifier,
            })
            .unwrap_or(false);

        if !should_check_path {
            return ParseResult::KeepState;
        }

        //And if it does, we extract the path from that line
        let path = ansi_line.next().expect("Error parsing output!");
        let path = match path {
            Output::TextBlock(p) => p,
            Output::Escape(_) => panic!("Error parsing output!"),
        };

        //The we check if we should block it or not, and based on
        // our current state we choose the next one, essentially we flip it.
        // So if we were "parsing to pause", we now "parse to continue" and vice versa.
        let should_block = !path_matches(&self.include, path);
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
            self.state = State::CheckEnd(go_back_state, 1);
        } else if parse_result == ParseResult::Flip {
            self.state = match parsing_to_pause {
                true => State::ParseToContinue,
                false => State::ParseToPause,
            };
            return (&self.state, !parsing_to_pause);
        }
        return (&self.state, parsing_to_pause);
    }
}

impl StateMachine for AnsiStateMachine {
    fn run(&mut self, line: &String) -> (&State, bool) {
        match self.state {
            State::ParseToPause => {
                return self.process_parse_result(line, true);
            }
            State::ParseToContinue => {
                return self.process_parse_result(line, false);
            }
            State::CheckEnd(should_parse_to_pause, check_count) => {
                if line.len() == 0 {
                    self.state = State::CheckEnd(should_parse_to_pause, check_count + 1);
                    return (&self.state, should_parse_to_pause);
                }

                // We know we have covered the ANSI output and now covering the summar part if:
                //   • We encountered an empty line at least twice in a row
                //   • And after all these empty lines, the next line we encounter doesn' start with an ANSI sequence
                if line
                    .as_str()
                    .ansi_parse()
                    .next()
                    .map(|i| match i {
                        Output::TextBlock(_) => true,
                        _ => false,
                    })
                    .unwrap_or(false)
                    && check_count >= 2
                {
                    self.state = State::End;
                    return self.run(line);
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

    fn is_finished(&self) -> bool {
        self.state == State::End
    }
}
