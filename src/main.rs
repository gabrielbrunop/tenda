use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io;
use std::io::{IsTerminal, Read};
use tenda::Interpreter;

fn main() -> io::Result<()> {
    let mut stdin = io::stdin();

    if stdin.is_terminal() {
        start_repl();
    } else {
        let mut buffer = String::new();
        stdin.read_to_string(&mut buffer)?;

        run_string(buffer);
    }

    Ok(())
}

fn start_repl() {
    let mut rl = DefaultEditor::new().unwrap();
    let mut interpreter = Interpreter::new();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) if line.trim() == ".exit" => break,
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();

                let output = interpreter.interpret(line);
                println!("{}", output);
            }
            Err(ReadlineError::Interrupted) => {
                println!("To exit, type .exit or press CTRL+D");
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn run_string(string: String) {
    let mut interpreter = Interpreter::new();
    let output = interpreter.interpret(string);

    println!("{}", output);
}
