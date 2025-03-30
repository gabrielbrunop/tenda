use common::report::Report;
use common::source::IdentifiedSource;
use interpreter::interpreter::Interpreter;
use parser::parser::Parser;
use parser::parser_error::ParserError;
use platform::Platform;
use reedline::{
    default_emacs_keybindings, DefaultPrompt, Reedline, Signal, ValidationResult, Validator,
};
use scanner::scanner::Scanner;
use scanner::scanner_error::LexicalError;
use std::io::{IsTerminal, Read};
use std::rc::Rc;
use std::{env, io};
use yansi::Paint;

struct BlockValidator;

impl Validator for BlockValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        let dummy_source_id = IdentifiedSource::dummy();

        let tokens = match Scanner::new(input, dummy_source_id).scan() {
            Ok(tokens) => tokens,
            Err(errors) => {
                if errors
                    .iter()
                    .any(|e| matches!(e, LexicalError::UnexpectedEoi { .. }))
                {
                    return ValidationResult::Incomplete;
                } else {
                    return ValidationResult::Complete;
                }
            }
        };

        let mut parser = Parser::new(&tokens, dummy_source_id);

        match parser.parse() {
            Ok(_) => ValidationResult::Complete,
            Err(errors) => {
                let has_eoi = errors
                    .iter()
                    .any(|e| matches!(e, ParserError::UnexpectedEoi { .. }));

                if has_eoi {
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
        let path: &'static str = Box::leak(args[1].clone().into_boxed_str());
        let file_content = std::fs::read_to_string(path);

        match file_content {
            Ok(source) => run_source(&source, path),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => eprintln!("Arquivo não encontrado: {}", path),
                _ => eprintln!("Erro ao ler arquivo: {}", err),
            },
        }

        return Ok(());
    }

    let mut stdin = io::stdin();

    if stdin.is_terminal() {
        start_repl();

        return Ok(());
    }

    let mut buffer = String::new();
    stdin.read_to_string(&mut buffer)?;
    run_source(&buffer, "stdin");

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
    let mut runtime = Interpreter::new(platform);
    let mut exiting = false;
    let mut source_history: Vec<(IdentifiedSource, Rc<str>)> = Vec::new();

    loop {
        let sig = rl.read_line(&prompt);

        match sig {
            Ok(Signal::Success(line)) if line.trim() == ".sair" => break,
            Ok(Signal::Success(line)) => {
                exiting = false;

                let source_id = IdentifiedSource::new();
                let source_rc = Rc::from(line.clone());
                source_history.push((source_id, source_rc));

                let tokens = match Scanner::new(&line, source_id).scan() {
                    Ok(tokens) => tokens,
                    Err(errs) => {
                        for err in errs {
                            let caches = ariadne::sources(source_history.clone());
                            err.to_report().eprint(caches).unwrap();
                        }

                        continue;
                    }
                };

                let ast = match Parser::new(&tokens, source_id).parse() {
                    Ok(ast) => ast,
                    Err(errs) => {
                        for err in errs {
                            let source_caches = ariadne::sources(source_history.clone());
                            err.to_report().eprint(source_caches).unwrap();
                        }

                        continue;
                    }
                };

                match runtime.eval(&ast) {
                    Ok(result) => {
                        println!("{}", result);
                    }
                    Err(err) => {
                        let source_caches = ariadne::sources(source_history.clone());
                        err.to_report().eprint(source_caches).unwrap();
                    }
                }
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

fn run_source(source: &str, name: &'static str) {
    let platform = Platform;

    let mut source_id = IdentifiedSource::new();
    source_id.set_name(name);

    let cache = (source_id, ariadne::Source::from(source));

    let tokens = match Scanner::new(source, source_id).scan() {
        Ok(tokens) => tokens,
        Err(errs) => {
            let len = errs.len();

            for err in errs {
                err.to_report().eprint(cache.clone()).unwrap();
            }

            println!(
                "\n{} programa não pôde ser executado devido a {} erro(s) léxico(s) encontrado(s)",
                Paint::red("erro:").bold(),
                len,
            );

            return;
        }
    };

    let ast = match Parser::new(&tokens, source_id).parse() {
        Ok(ast) => ast,
        Err(errs) => {
            let len = errs.len();

            for err in errs {
                err.to_report().eprint(cache.clone()).unwrap();
            }

            println!(
                "\n{} programa não pôde ser executado devido a {} erro(s) de sintaxe encontrado(s)",
                Paint::red("erro:").bold(),
                len,
            );

            return;
        }
    };

    let mut runtime = Interpreter::new(platform);

    if let Err(err) = runtime.eval(&ast) {
        err.to_report().eprint(cache.clone()).unwrap();

        println!(
            "\n{} programa encerrado devido a um erro durante a execução",
            Paint::red("erro:").bold(),
        );
    }
}

mod platform;
