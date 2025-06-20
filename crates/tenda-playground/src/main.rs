use protocol_message::JsonProtocolMessage;
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::rc::Rc;
use std::vec;
use tenda_core::common::span::SourceSpan;
use tenda_core::loader::{Loader, LoaderError};
use tenda_core::runtime::escape_value;
use tenda_core::{
    common::source::IdentifiedSource, prelude::get_runtime_prelude, runtime::Runtime,
};
use tenda_playground_platform::Platform;
use tenda_playground_platform::ProtocolMessage;

const PROMPT_TERMINATOR: u8 = b'\x04';

mod protocol_message;

fn send(message: ProtocolMessage) {
    let json_message = JsonProtocolMessage::from(message);
    let json_string = json_message.to_string();

    let mut stdout = io::stdout();
    stdout.write_all(json_string.as_bytes()).unwrap();
    stdout.write_all(b"\n").unwrap();
    stdout.flush().unwrap();
}

fn send_diagnostic(
    errs: Vec<(
        impl tenda_core::reporting::Diagnostic<SourceSpan>,
        impl tenda_core::reporting::Cache<IdentifiedSource>,
    )>,
) {
    let errs_str: Vec<_> = errs
        .into_iter()
        .map(|(err, cache)| {
            let mut buf = Vec::<u8>::new();

            err.to_report().write(cache, &mut buf).unwrap();

            let message = String::from_utf8_lossy(&buf).into_owned();

            message
        })
        .collect();

    send(ProtocolMessage::Error(errs_str));
}

fn read_line() -> String {
    let mut input = String::new();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    reader.read_line(&mut input).unwrap();

    input
}

fn read_prompt(buffer: &mut Vec<u8>) -> Result<Option<String>, io::Error> {
    let bytes_read = {
        let stdin = io::stdin();
        let mut reader = stdin.lock();

        buffer.clear();

        reader.read_until(PROMPT_TERMINATOR, buffer)?
    };

    if bytes_read == 0 {
        return Ok(None);
    }

    if let Some(&last) = buffer.last() {
        if last == PROMPT_TERMINATOR {
            buffer.pop();
        }
    }

    let source = String::from_utf8_lossy(buffer).into_owned();

    Ok(Some(source))
}

fn main() -> io::Result<()> {
    let platform = Platform::new(send, read_line);

    let mut runtime = Runtime::with_builtins(platform, get_runtime_prelude());
    let mut loader = Loader::new(|p: &Path| std::fs::read_to_string(p));

    let mut buffer = Vec::new();
    let mut source_history: HashMap<IdentifiedSource, Rc<str>> = HashMap::new();

    'repl: loop {
        let source = match read_prompt(&mut buffer)? {
            Some(source) => source,
            None => continue,
        };

        let source_id = IdentifiedSource::new();
        let source_rc = Rc::from(source.clone());
        source_history.insert(source_id, source_rc);

        if source.trim().is_empty() {
            continue;
        }

        let result = match loader.register_virtual(source) {
            Ok(o) => o,
            Err(e) => {
                handle_loader_error(e, &source_history);

                continue;
            }
        };

        for (p, po) in result.modules.clone() {
            if let Err(err) = runtime.load_module(p.clone(), po.unit.into()) {
                source_history.insert(po.source_id, po.text.clone());

                let err = err.as_ref().clone();

                send_diagnostic(vec![(
                    err,
                    tenda_core::reporting::sources(source_history.clone()),
                )]);

                continue 'repl;
            }
        }

        match runtime.eval(result.prompt.unit.into()) {
            Ok(result) => println!("{}", escape_value(&result)),
            Err(err) => {
                source_history.insert(result.prompt.source_id, result.prompt.text.clone());

                let err = err.as_ref().clone();

                send_diagnostic(vec![(
                    err,
                    tenda_core::reporting::sources(source_history.clone()),
                )]);
            }
        }
    }
}

fn handle_loader_error(err: LoaderError, source_history: &HashMap<IdentifiedSource, Rc<str>>) {
    use tenda_core::loader::LoaderError;

    match err {
        LoaderError::Lexical { errors, .. } => {
            let diagnostic_pairs = errors
                .into_iter()
                .map(|err| (err, tenda_core::reporting::sources(source_history.clone())))
                .collect();

            send_diagnostic(diagnostic_pairs);
        }
        LoaderError::Parse { errors, .. } => {
            let diagnostic_pairs = errors
                .into_iter()
                .map(|err| (err, tenda_core::reporting::sources(source_history.clone())))
                .collect();

            send_diagnostic(diagnostic_pairs);
        }
        LoaderError::Resolution { error, .. } => {
            let error = (
                error.as_ref().clone(),
                tenda_core::reporting::sources(source_history.clone()),
            );

            send_diagnostic(vec![error]);
        }
        LoaderError::EntryFileNotFound { path, .. } => {
            send(ProtocolMessage::Error(vec![format!(
                "erro: ao ler o arquivo `{}`",
                path.display(),
            )]));
        }
    }
}
