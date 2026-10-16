//! Where the pinned `luau-lsp` binary is: `PH2D_LUAU_LSP` → next to the running executable → the
//! cache `scripts/luau-lsp-fetch.sh` fills (`${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/luau-lsp/<ver>/`).

use std::fmt;
use std::path::PathBuf;

/// The pin (version + sha256 per OS). The fetch script reads the same file.
pub const MANIFEST: &str = include_str!("../luau-lsp.toml");

/// Environment variable that overrides every other location.
pub const ENV_BINARY: &str = "PH2D_LUAU_LSP";

/// The `version = "…"` of the manifest (top level, before the first section).
pub fn pinned_version() -> &'static str {
    for line in MANIFEST.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            break;
        }
        if let Some(rest) = line.strip_prefix("version")
            && let Some(v) = rest.trim_start().strip_prefix('=')
        {
            return v.trim().trim_matches('"');
        }
    }
    panic!("luau-lsp.toml has no top-level version");
}

pub fn binary_file_name() -> &'static str {
    if cfg!(windows) {
        "luau-lsp.exe"
    } else {
        "luau-lsp"
    }
}

/// `<cache>/ph2d/luau-lsp/<pinned version>/`, or `None` when no home directory is known.
pub fn cache_dir() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute());
    let home_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let base =
        xdg.or_else(|| std::env::var_os(home_var).map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("ph2d").join("luau-lsp").join(pinned_version()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinarySource {
    Env,
    NextToExe,
    Cache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// `PH2D_LUAU_LSP` names a file that does not exist — an explicit setting is never skipped.
    EnvPointsNowhere(PathBuf),
    NotFound {
        tried: Vec<PathBuf>,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvPointsNowhere(p) => {
                write!(f, "{ENV_BINARY}={} does not exist", p.display())
            }
            Self::NotFound { tried } => {
                write!(
                    f,
                    "luau-lsp {} not found; run `bash scripts/luau-lsp-fetch.sh` (tried:",
                    pinned_version()
                )?;
                for p in tried {
                    write!(f, " {}", p.display())?;
                }
                write!(f, ")")
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Resolves the binary in the documented order.
pub fn resolve_binary() -> Result<(PathBuf, BinarySource), ResolveError> {
    if let Some(p) = std::env::var_os(ENV_BINARY).filter(|v| !v.is_empty()) {
        let p = PathBuf::from(p);
        return if p.is_file() {
            Ok((p, BinarySource::Env))
        } else {
            Err(ResolveError::EnvPointsNowhere(p))
        };
    }
    let mut tried = Vec::new();
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(PathBuf::from))
    {
        let p = dir.join(binary_file_name());
        if p.is_file() {
            return Ok((p, BinarySource::NextToExe));
        }
        tried.push(p);
    }
    if let Some(dir) = cache_dir() {
        let p = dir.join(binary_file_name());
        if p.is_file() {
            return Ok((p, BinarySource::Cache));
        }
        tried.push(p);
    }
    Err(ResolveError::NotFound { tried })
}
