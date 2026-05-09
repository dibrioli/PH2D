//! Errors emitted by the asset pipeline.
//!
//! Surface kept narrow: callers either have a usable [`crate::Asset`]
//! or a structured reason why not. We do NOT wrap arbitrary
//! `Box<dyn Error>` — every variant is enumerable so the renderer +
//! editor can branch on the failure mode (e.g. show a "missing texture"
//! placeholder vs. a "corrupt PNG" warning).

use std::path::PathBuf;

#[derive(Debug)]
pub enum AssetError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Decode {
        path: Option<PathBuf>,
        message: String,
    },
    Watch(String),
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "i/o error reading {}: {}", path.display(), source)
            }
            Self::Decode {
                path: Some(p),
                message,
            } => write!(f, "decode error for {}: {}", p.display(), message),
            Self::Decode {
                path: None,
                message,
            } => write!(f, "decode error: {message}"),
            Self::Watch(s) => write!(f, "watcher error: {s}"),
        }
    }
}

impl std::error::Error for AssetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
