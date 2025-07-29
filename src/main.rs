use std::process::ExitCode;

use ansi_parser::{AnsiParser, Output};
use clap::{Parser, command};
use ts_ef::{
    StateMachine, ansi_state_machine::AnsiStateMachine, simple_state_machine::SimpleStateMachine,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Glob pattern to match. For example: src/features/**/*"
    )]
    include: Vec<String>,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Show the summary output as well when using --pretty with tsc"
    )]
    show_full: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let mut lines = std::io::stdin().lines().peekable();
    let first_line = lines.peek();
    if first_line.is_none() {
        return ExitCode::SUCCESS;
    }
    let first_line = first_line.unwrap().as_ref().unwrap();
    let ansi = first_line.ansi_parse().next();

    let mut sm: Box<dyn StateMachine> = match ansi {
        None => Box::new(SimpleStateMachine::new(args.include)),
        Some(ansi) => match ansi {
            Output::TextBlock(_) => Box::new(SimpleStateMachine::new(args.include)),
            Output::Escape(a) => Box::new(AnsiStateMachine::new(a, args.include)),
        },
    };

    let mut was_logged = false;
    for line in lines {
        let line = line.unwrap();
        let (_, should_print) = sm.run(&line);
        if sm.is_finished() && !args.show_full {
            break;
        }
        if should_print {
            println!("{}", line);
            if !sm.is_finished() {
                was_logged = should_print;
            }
        }
    }

    if was_logged {
        return ExitCode::FAILURE;
    }

    return ExitCode::SUCCESS;
}
