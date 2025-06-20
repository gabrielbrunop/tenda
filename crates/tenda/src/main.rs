use clap::{CommandFactory, Parser as CommandParser};
use reedline::{
    default_emacs_keybindings, DefaultPrompt, Reedline, Signal, ValidationResult, Validator,
};
use std::collections::HashMap;
use std::io;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tenda_core::loader::LoaderError;
use tenda_core::runtime::escape_value;
use tenda_core::{
    common::source::IdentifiedSource, loader::Loader, parser::Parser, parser::ParserError,
    platform::OSPlatform, prelude::get_runtime_prelude, reporting::Diagnostic, runtime::Runtime,
    scanner::LexicalError, scanner::Scanner,
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

        return Ok(());
    }

    if cli.version {
        println!("tenda {}", env!("CARGO_PKG_VERSION"));

        return Ok(());
    }

    if let Some(path) = cli.file {
        run_file(PathBuf::from(path));

        return Ok(());
    }

    let mut stdin = io::stdin();

    if !stdin.is_terminal() {
        let mut buffer = String::new();
        stdin.read_to_string(&mut buffer)?;

        run_source(&buffer);

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
    let mut loader = Loader::new(|p: &Path| std::fs::read_to_string(p));
    let mut runtime = Runtime::with_builtins(platform, get_runtime_prelude());
    let mut exiting = false;
    let mut source_history: HashMap<IdentifiedSource, Rc<str>> = HashMap::new();

    'repl: loop {
        let sig = rl.read_line(&prompt);

        match sig {
            Ok(Signal::Success(line)) if line.trim() == ".sair" => break,
            Ok(Signal::Success(line)) => {
                exiting = false;

                let result = match loader.register_virtual(line) {
                    Ok(o) => o,
                    Err(e) => {
                        handle_loader_error(e, false);

                        continue;
                    }
                };

                source_history.insert(result.prompt.source_id, result.prompt.text.clone());

                for (p, po) in result.modules.clone() {
                    if let Err(err) = runtime.load_module(p.clone(), po.unit.into()) {
                        source_history.insert(po.source_id, po.text.clone());

                        let caches = tenda_core::reporting::sources(source_history.clone());
                        err.to_report().eprint(caches).unwrap();

                        continue 'repl;
                    }
                }

                match runtime.eval(result.prompt.unit.into()) {
                    Ok(result) => println!("{}", escape_value(&result)),
                    Err(err) => {
                        let sources = source_history.clone();
                        let caches = tenda_core::reporting::sources(sources);

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

fn run_file(path: PathBuf) {
    let mut loader = Loader::new(|p: &Path| std::fs::read_to_string(p));

    let result = match loader.load_entry(path.clone()) {
        Ok(o) => o,
        Err(e) => return handle_loader_error(e, true),
    };

    let mut runtime = Runtime::with_builtins(OSPlatform, get_runtime_prelude());
    let mut source_history: HashMap<IdentifiedSource, Rc<str>> = HashMap::new();

    let last = result
        .modules
        .last()
        .map(|(_, po)| po.source_id)
        .expect("there should be at least one module");

    for (p, po) in result.modules {
        source_history.insert(po.source_id, po.text.clone());

        let res = if po.source_id != last {
            runtime.load_module(p.clone(), po.unit.into()).err()
        } else {
            runtime.eval(po.unit.into()).err()
        };

        if let Some(err) = res {
            let caches = tenda_core::reporting::sources(source_history.clone());
            err.to_report().eprint(caches).unwrap();

            println!(
                "\n{} programa encerrado devido a um erro durante a execução",
                Paint::red("erro:").bold(),
            );

            return;
        }
    }
}

fn run_source(source: &str) {
    let platform = OSPlatform;
    let mut loader = Loader::new(|p: &Path| std::fs::read_to_string(p));

    let mut runtime = Runtime::with_builtins(platform, get_runtime_prelude());

    let result = match loader.register_virtual(source.to_string()) {
        Ok(o) => o,
        Err(e) => return handle_loader_error(e, false),
    };

    for (p, po) in result.modules.clone() {
        if let Err(err) = runtime.load_module(p.clone(), po.unit.into()) {
            let cache = (po.source_id, tenda_core::reporting::Source::from(po.text));
            err.to_report().eprint(cache).unwrap();

            println!(
                "\n{} programa encerrado devido a um erro durante a execução",
                Paint::red("erro:").bold(),
            );

            continue;
        }
    }

    if let Err(err) = runtime.eval(result.prompt.unit.into()) {
        let cache = (
            result.prompt.source_id,
            tenda_core::reporting::Source::from(result.prompt.text),
        );

        err.to_report().eprint(cache.clone()).unwrap();

        println!(
            "\n{} programa encerrado devido a um erro durante a execução",
            Paint::red("erro:").bold(),
        );
    }
}

fn handle_loader_error(err: LoaderError, end: bool) {
    use tenda_core::loader::LoaderError;

    match err {
        LoaderError::Lexical {
            source_id,
            source_code,
            errors,
            ..
        } => {
            let len = errors.len();
            let cache = (source_id, tenda_core::reporting::Source::from(source_code));

            for err in errors {
                err.to_report().eprint(cache.clone()).unwrap();
            }

            if end {
                println!(
                    "\n{} programa não pôde ser executado devido a {} erro(s) léxico(s) encontrado(s)",
                    Paint::red("erro:").bold(),
                    len,
                );
            }
        }
        LoaderError::Parse {
            source_id,
            source_code,
            errors,
            ..
        } => {
            let len = errors.len();
            let cache = (source_id, tenda_core::reporting::Source::from(source_code));

            for err in errors {
                err.to_report().eprint(cache.clone()).unwrap();
            }

            if end {
                println!(
                    "\n{} programa não pôde ser executado devido a {} erro(s) de sintaxe encontrado(s)",
                    Paint::red("erro:").bold(),
                    len,
                );
            }
        }
        LoaderError::Resolution {
            source_id,
            source_code,
            error,
        } => {
            let cache = (source_id, tenda_core::reporting::Source::from(source_code));

            error.to_report().eprint(cache).unwrap();

            if end {
                println!(
                    "\n{} programa não pôde ser executado devido a erro(s) de resolução",
                    Paint::red("erro:").bold(),
                );
            }
        }
        LoaderError::EntryFileNotFound { path, .. } => {
            eprintln!(
                "{} ao ler o arquivo `{}`",
                Paint::red("erro:").bold(),
                path.display(),
            );

            if end {
                println!(
                    "\n{} programa não pôde ser executado devido a erro(s) de leitura de arquivo",
                    Paint::red("erro:").bold(),
                );
            }
        }
    }
}
