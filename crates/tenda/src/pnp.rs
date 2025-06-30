use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use tenda_core::scanner::is_identifier;

pub fn load_plain(path: &Path) -> Result<HashMap<String, PathBuf>, PnPError> {
    let mut map = HashMap::new();

    for (idx, line) in std::fs::read_to_string(path)
        .map_err(|e| PnPError::FileAccessError {
            path: path.into(),
            source: e,
        })?
        .lines()
        .enumerate()
    {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (virt, real) = line
            .split_once('=')
            .ok_or(PnPError::LineMissingSeparator { linha: idx + 1 })?;

        if !is_identifier(virt.trim()) {
            return Err(PnPError::InvalidIdentifier(virt.to_owned()));
        }

        map.insert(virt.trim().to_owned(), PathBuf::from(real.trim()));
    }

    Ok(map)
}

pub fn parse_inline(pairs: &[String]) -> Result<HashMap<String, PathBuf>, PnPError> {
    let mut map = HashMap::new();

    for pair in pairs {
        let (virt, real) = pair
            .split_once('=')
            .ok_or_else(|| PnPError::InvalidMapping(pair.clone()))?;

        if !is_identifier(virt.trim()) {
            return Err(PnPError::InvalidIdentifier(virt.to_owned()));
        }

        map.insert(virt.trim().to_owned(), PathBuf::from(real.trim()));
    }

    Ok(map)
}

#[derive(Debug, thiserror::Error)]
pub enum PnPError {
    #[error("linha {linha}: esperava separador `=` entre chave e valor")]
    LineMissingSeparator { linha: usize },

    #[error("mapeamento inválido: `{0}`, use formato VIRTUAL=ARQUIVO")]
    InvalidMapping(String),

    #[error("não foi possível ler arquivo `{path}`: {source}")]
    FileAccessError {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("chave inválida: `{0}`")]
    InvalidIdentifier(String),
}
