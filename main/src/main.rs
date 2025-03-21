use platform::Platform;
use runtime::Runtime;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io;
use std::io::{IsTerminal, Read};

fn main() -> io::Result<()> {
    let mut stdin = io::stdin();

    if stdin.is_terminal() {
        start_repl();
    } else {
        let mut buffer = String::new();
        stdin.read_to_string(&mut buffer)?;

        run_source(buffer);
    }

    Ok(())
}

fn start_repl() {
    let mut rl = DefaultEditor::new().unwrap();
    let platform = Platform;
    let mut runtime = Runtime::new(platform);
    let mut exiting = false;

    loop {
        let readline = rl.readline("> ");

        match readline {
            Ok(line) if line.trim() == ".sair" => break,
            Ok(line) => {
                exiting = false;

                rl.add_history_entry(line.as_str()).unwrap();

                let output = runtime.run(line);
                println!("{}", output.unwrap_or_else(|err| err));
            }
            Err(ReadlineError::Interrupted) if exiting => break,
            Err(ReadlineError::Interrupted) => {
                exiting = true;
                println!(
                    "Para sair, pressione CTRL+C novamente, ou digite .sair, ou pressione CTRL+D"
                );
            }
            Err(ReadlineError::Eof) => {
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

fn run_source(string: String) {
    let platform = Platform;
    let mut runtime = Runtime::new(platform);
    let output = runtime.run(string);

    if let Err(err) = output {
        eprintln!("{}", err)
    };
}

mod platform;
mod runtime;
