use clap::{CommandFactory, Parser as CommandParser};
use reedline::{
    default_emacs_keybindings, DefaultPrompt, Reedline, Signal, ValidationResult, Validator,
};
use std::io::{IsTerminal, Read};
use std::rc::Rc;
use std::{io, process};
use tenda_core::runtime::escape_value;
use tenda_core::{
    common::source::IdentifiedSource, parser::Parser, parser::ParserError, platform::OSPlatform,
    prelude::setup_runtime_prelude, reporting::Diagnostic, runtime::Runtime, scanner::LexicalError,
    scanner::Scanner,
};
use yansi::Paint;

#[derive(CommandParser)]
#[command(
    author,
    version,
    about = "Tenda - interpretador e REPL",
    long_about = "Execute arquivos .tnd ou inicie o REPL da linguagem Tenda.",
    disable_help_flag = true,
    disable_version_flag = true,
    help_template = "\
{name} {version}\n\n\
{about-with-newline}\n\
\x1b[1;4mUso:\x1b[0m {usage}\n\n\
\x1b[1;4mArgumentos:\x1b[0m\n{positionals}\n\n\
\x1b[1;4mOpções:\x1b[0m\n{options}\n"
)]
struct Cli {
    #[arg(value_name = "ARQUIVO")]
    file: Option<String>,

    #[arg(
        short = 'h',
        long = "help",
        alias = "ajuda",
        help = "Exibe ajuda (use '-h' para ver resumo)"
    )]
    help: bool,

    #[arg(short = 'V', long = "version", alias = "versão", help = "Exibe versão")]
    version: bool,
}

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
    let cli = Cli::parse();

    if cli.help {
        Cli::command().print_long_help().unwrap();
        println!();
        process::exit(0);
    }

    if cli.version {
        println!("tenda {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    if let Some(path) = cli.file {
        let file_content = std::fs::read_to_string(&path);

        match file_content {
            Ok(source) => run_source(&source, Box::leak(path.into_boxed_str())),
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => eprintln!("Arquivo não encontrado: {}", path),
                _ => eprintln!("Erro ao ler arquivo: {}", err),
            },
        }

        return Ok(());
    }

    let mut stdin = io::stdin();

    if !stdin.is_terminal() {
        let mut buffer = String::new();
        stdin.read_to_string(&mut buffer)?;

        run_source(&buffer, "stdin");

        return Ok(());
    }

    start_repl();

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

    let platform = OSPlatform;
    let mut runtime = Runtime::new(platform);
    let mut exiting = false;
    let mut source_history: Vec<(IdentifiedSource, Rc<str>)> = Vec::new();

    setup_runtime_prelude(runtime.get_global_env_mut());

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
                            let caches = tenda_core::reporting::sources(source_history.clone());
                            err.to_report().eprint(caches).unwrap();
                        }

                        continue;
                    }
                };

                let ast = match Parser::new(&tokens, source_id).parse() {
                    Ok(ast) => ast,
                    Err(errs) => {
                        for err in errs {
                            let caches = tenda_core::reporting::sources(source_history.clone());
                            err.to_report().eprint(caches).unwrap();
                        }

                        continue;
                    }
                };

                match runtime.eval(&ast) {
                    Ok(result) => println!("{}", escape_value(&result)),
                    Err(err) => {
                        let caches = tenda_core::reporting::sources(source_history.clone());
                        err.to_report().eprint(caches).unwrap();
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
                eprintln!("{}", err);
                break;
            }
        }
    }
}

fn run_source(source: &str, name: &'static str) {
    let platform = OSPlatform;

    let mut source_id = IdentifiedSource::new();
    source_id.set_name(name);

    let cache = (source_id, tenda_core::reporting::Source::from(source));

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

    let mut runtime = Runtime::new(platform);

    setup_runtime_prelude(runtime.get_global_env_mut());

    if let Err(err) = runtime.eval(&ast) {
        err.to_report().eprint(cache.clone()).unwrap();

        println!(
            "\n{} programa encerrado devido a um erro durante a execução",
            Paint::red("erro:").bold(),
        );
    }
}
