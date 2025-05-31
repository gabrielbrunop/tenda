use protocol_message::JsonProtocolMessage;
use std::io::{self, BufRead, Write};
use std::rc::Rc;
use tenda_core::common::span::SourceSpan;
use tenda_core::runtime::escape_value;
use tenda_core::{
    common::source::IdentifiedSource, parser::Parser, platform::web::*,
    prelude::setup_runtime_prelude, runtime::Runtime, scanner::Scanner,
};

const PROMPT_TERMINATOR: u8 = b'\x04';

mod protocol_message;

fn send(message: WebPlatformProtocolMessage) {
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

    send(WebPlatformProtocolMessage::Error(errs_str));
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
    let platform = WebPlatform::new(send, read_line);

    let mut runtime = Runtime::new(platform);
    setup_runtime_prelude(runtime.get_global_env_mut());

    let mut buffer = Vec::new();
    let mut source_history: Vec<(IdentifiedSource, Rc<str>)> = Vec::new();

    loop {
        let source = match read_prompt(&mut buffer)? {
            Some(source) => source,
            None => continue,
        };

        let source_id = IdentifiedSource::new();
        let source_rc = Rc::from(source.clone());
        source_history.push((source_id, source_rc));

        if source.trim().is_empty() {
            continue;
        }

        let tokens = match Scanner::new(&source, source_id).scan() {
            Ok(tokens) => tokens,
            Err(errs) => {
                let diagnostic_pairs = errs
                    .into_iter()
                    .map(|err| (err, tenda_core::reporting::sources(source_history.clone())))
                    .collect();

                send_diagnostic(diagnostic_pairs);
                continue;
            }
        };

        let ast = match Parser::new(&tokens, source_id).parse() {
            Ok(ast) => ast,
            Err(errors) => {
                let diagnostic_pairs = errors
                    .into_iter()
                    .map(|err| (err, tenda_core::reporting::sources(source_history.clone())))
                    .collect();

                send_diagnostic(diagnostic_pairs);
                continue;
            }
        };

        match runtime.eval(&ast) {
            Ok(result) => {
                send(WebPlatformProtocolMessage::Result(
                    result.kind(),
                    escape_value(&result),
                ));
            }
            Err(err) => {
                send_diagnostic(vec![(
                    *err,
                    tenda_core::reporting::sources(source_history.clone()),
                )]);
            }
        }
    }
}
