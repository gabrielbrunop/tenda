use std::{fmt, io, path::PathBuf, rc::Rc};
use tenda_common::{source::IdentifiedSource, span::SourceSpan};
use tenda_parser::{self};
use tenda_reporting_derive::Diagnostic;

#[derive(thiserror::Error, Debug)]
pub enum LoaderError {
    #[error("could not read entry file `{path}`: {source}")]
    EntryFileNotFound {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("scanner error in `{path}`")]
    Lexical {
        path: PathBuf,
        source_id: IdentifiedSource,
        source_code: Rc<str>,
        errors: Vec<tenda_scanner::LexicalError>,
    },

    #[error("parser error in `{path}`")]
    Parse {
        path: PathBuf,
        source_id: IdentifiedSource,
        source_code: Rc<str>,
        errors: Vec<tenda_parser::ParserError>,
    },

    #[error("resolution error in `{source_id}`")]
    Resolution {
        source_id: IdentifiedSource,
        source_code: Rc<str>,
        error: Box<ResolutionError>,
    },
}

#[derive(thiserror::Error, Debug, Diagnostic, Clone)]
#[report("erro na resolução de importação")]
pub enum ResolutionError {
    #[error("não foi possível ler `{path}`")]
    Io {
        path: PathBuf,
        kind: io::ErrorKind,
        #[span]
        span: SourceSpan,
    },

    #[error("importação circular detectada: {cycle}")]
    Circular {
        cycle: Cycle,
        #[span]
        span: SourceSpan,
    },
}

#[derive(Debug, Clone)]
pub struct Cycle(pub Vec<PathBuf>);

impl fmt::Display for Cycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();

        if let Some(first) = iter.next() {
            write!(f, "{}", first.display())?;

            for p in iter {
                write!(f, " → {}", p.display())?;
            }
        }

        Ok(())
    }
}
