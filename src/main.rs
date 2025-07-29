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

    // If receive no input, then we treat it as compilation being successful
    // and return a success exit code
    if first_line.is_none() {
        return ExitCode::SUCCESS;
    }

    // We use two state machines, one to parse ANSI output and one for non-ANSI
    // We check the first line of output to detect if we are getting ANSI input or not
    let first_line = first_line.unwrap().as_ref().unwrap();
    let ansi = first_line.ansi_parse().next();

    // If we do get ANSI input, we extract the sequence and create our AnsiStateMachine
    // from it, else SimpleStateMachine.
    //
    // We extract the ANSI sequence because we use it as a marker to figure out on which
    // lines do we need to extract the path and check if the ouput should be logged or now.
    // For example, when you run `tsc --pretty` and there are some errors, the output would look like:
    //
    // \x[[<some ansi sequence>src/features/orders/index.ts\x[[some other ansi sequence ...and the rest of the error
    //
    // This is the structure we base our state transitions on.
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

        //Our state machine parses a line and tells us whether we should print or not
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
