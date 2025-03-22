use parser::parser::Parser;
use parser::parser_error::ParserErrorKind;
use platform::Platform;
use reedline::{
    default_emacs_keybindings, DefaultPrompt, Reedline, Signal, ValidationResult, Validator,
};
use runtime::Runtime;
use scanner::scanner::Scanner;
use std::io::{IsTerminal, Read};
use std::{env, io};

struct BlockValidator;

impl Validator for BlockValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        let tokens = Scanner::new(input).scan().unwrap();
        let mut parser = Parser::new(&tokens);

        match parser.parse() {
            Ok(_) => ValidationResult::Complete,
            Err(errors) => {
                if errors
                    .iter()
                    .any(|e| e.source == ParserErrorKind::UnexpectedEoi)
                {
                    ValidationResult::Incomplete
                } else {
                    ValidationResult::Complete
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let path = &args[1];
        let code = std::fs::read_to_string(path)?;
        run_source(code);

        return Ok(());
    }

    let mut stdin = io::stdin();

    if stdin.is_terminal() {
        start_repl();

        return Ok(());
    }

    let mut buffer = String::new();
    stdin.read_to_string(&mut buffer)?;
    run_source(buffer);

    Ok(())
}

fn start_repl() {
    let keybindings = default_emacs_keybindings();
    let edit_mode = Box::new(reedline::Emacs::new(keybindings));
    let validator = Box::new(BlockValidator);

    let mut rl = Reedline::create()
        .with_edit_mode(edit_mode)
        .with_validator(validator);

    let prompt = DefaultPrompt::new(
        reedline::DefaultPromptSegment::Empty,
        reedline::DefaultPromptSegment::Empty,
    );

    let platform = Platform;
    let mut runtime = Runtime::new(platform);
    let mut exiting = false;

    loop {
        let sig = rl.read_line(&prompt);
        match sig {
            Ok(Signal::Success(line)) if line.trim() == ".sair" => break,
            Ok(Signal::Success(line)) => {
                exiting = false;

                let output = runtime.run(line);

                println!("{}", output.unwrap_or_else(|err| err));
            }
            Ok(Signal::CtrlC) if exiting => break,
            Ok(Signal::CtrlC) => {
                exiting = true;
                println!(
                    "Para sair, pressione CTRL+C novamente, ou digite .sair, ou pressione CTRL+D"
                );
            }
            Ok(Signal::CtrlD) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Erro: {:?}", err);
                break;
            }
        }
    }
}

fn run_source(source: String) {
    let platform = Platform;
    let mut runtime = Runtime::new(platform);
    let output = runtime.run(source);

    if let Err(err) = output {
        eprintln!("{}", err)
    };
}

mod platform;
mod runtime;
