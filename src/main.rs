use std::process::ExitCode;

use ansi_parser::{AnsiParser, Output};
use clap::{Parser, command};
use ts_ef::ansi_state_machine::{AnsiStateMachine, State};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    include: Vec<String>,

    #[arg(short, long, default_value_t = false)]
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
    if ansi.is_none() {
        println!("simple");
        return ExitCode::SUCCESS;
    }
    let ansi = match ansi.unwrap() {
        Output::TextBlock(_) => None,
        Output::Escape(a) => Some(a),
    };
    if ansi.is_none() {
        println!("simple");
        return ExitCode::SUCCESS;
    }
    let mut sm = AnsiStateMachine::new(ansi.unwrap(), args.include);
    let mut was_logged = false;
    for line in lines {
        let line = line.unwrap();
        let (_, should_print) = sm.run(&line);
        if sm.state == State::End && !args.show_full {
            break;
        }
        if should_print {
            println!("{}", line);
            if sm.state != State::End {
                was_logged = should_print;
            }
        }
    }

    if was_logged {
        return ExitCode::FAILURE;
    }

    return ExitCode::SUCCESS;
}
