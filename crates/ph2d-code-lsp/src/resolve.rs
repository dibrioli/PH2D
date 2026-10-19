//! Where a pinned helper program is: its env var → next to the running executable → the cache its
//! fetch script fills (`${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/<tool>/<ver>/`). ONE law for every helper
//! ([`Pin`]): `luau-lsp` here, StyLua in `ph2d-code-format`.

use std::fmt;
use std::path::PathBuf;

/// A pinned helper program: the manifest (version + sha256 per OS, read by its fetch script too), the
/// variable that overrides every other location, and the name of its cache folder and executable.
#[derive(Debug, Clone, Copy)]
pub struct Pin {
    pub manifest: &'static str,
    pub env: &'static str,
    pub tool: &'static str,
    pub fetch: &'static str,
}

/// The `luau-lsp` pin.
pub const LUAU_LSP: Pin = Pin {
    manifest: include_str!("../luau-lsp.toml"),
    env: "PH2D_LUAU_LSP",
    tool: "luau-lsp",
    fetch: "scripts/luau-lsp-fetch.sh",
};

/// The pin (version + sha256 per OS). The fetch script reads the same file.
pub const MANIFEST: &str = LUAU_LSP.manifest;

/// Environment variable that overrides every other location.
pub const ENV_BINARY: &str = LUAU_LSP.env;

impl Pin {
    /// The `version = "…"` of the manifest (top level, before the first section).
    pub fn version(&self) -> &'static str {
        for line in self.manifest.lines() {
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
        panic!("the {} manifest has no top-level version", self.tool);
    }

    pub fn binary_file_name(&self) -> String {
        if cfg!(windows) {
            format!("{}.exe", self.tool)
        } else {
            self.tool.to_owned()
        }
    }

    /// `<cache>/ph2d/<tool>/<pinned version>/`, or `None` when no home directory is known.
    pub fn cache_dir(&self) -> Option<PathBuf> {
        let xdg = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute());
        let home_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        let base =
            xdg.or_else(|| std::env::var_os(home_var).map(|h| PathBuf::from(h).join(".cache")))?;
        Some(base.join("ph2d").join(self.tool).join(self.version()))
    }

    /// Resolves the binary in the documented order.
    pub fn resolve(&self) -> Result<(PathBuf, BinarySource), ResolveError> {
        if let Some(p) = std::env::var_os(self.env).filter(|v| !v.is_empty()) {
            let p = PathBuf::from(p);
            return if p.is_file() {
                Ok((p, BinarySource::Env))
            } else {
                Err(ResolveError::EnvPointsNowhere {
                    pin: *self,
                    path: p,
                })
            };
        }
        let mut tried = Vec::new();
        if let Some(dir) = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(PathBuf::from))
        {
            let p = dir.join(self.binary_file_name());
            if p.is_file() {
                return Ok((p, BinarySource::NextToExe));
            }
            tried.push(p);
        }
        if let Some(dir) = self.cache_dir() {
            let p = dir.join(self.binary_file_name());
            if p.is_file() {
                return Ok((p, BinarySource::Cache));
            }
            tried.push(p);
        }
        Err(ResolveError::NotFound { pin: *self, tried })
    }
}

/// The `version = "…"` of the `luau-lsp` manifest.
pub fn pinned_version() -> &'static str {
    LUAU_LSP.version()
}

pub fn binary_file_name() -> String {
    LUAU_LSP.binary_file_name()
}

/// `<cache>/ph2d/luau-lsp/<pinned version>/`, or `None` when no home directory is known.
pub fn cache_dir() -> Option<PathBuf> {
    LUAU_LSP.cache_dir()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinarySource {
    Env,
    NextToExe,
    Cache,
}

#[derive(Debug, Clone)]
pub enum ResolveError {
    /// The env var names a file that does not exist — an explicit setting is never skipped.
    EnvPointsNowhere {
        pin: Pin,
        path: PathBuf,
    },
    NotFound {
        pin: Pin,
        tried: Vec<PathBuf>,
    },
}

impl PartialEq for ResolveError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::EnvPointsNowhere { pin: a, path: p },
                Self::EnvPointsNowhere { pin: b, path: q },
            ) => a.tool == b.tool && p == q,
            (Self::NotFound { pin: a, tried: p }, Self::NotFound { pin: b, tried: q }) => {
                a.tool == b.tool && p == q
            }
            _ => false,
        }
    }
}

impl Eq for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvPointsNowhere { pin, path } => {
                write!(f, "{}={} does not exist", pin.env, path.display())
            }
            Self::NotFound { pin, tried } => {
                write!(
                    f,
                    "{} {} not found; run `bash {}` (tried:",
                    pin.tool,
                    pin.version(),
                    pin.fetch
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

/// Resolves the `luau-lsp` binary in the documented order.
pub fn resolve_binary() -> Result<(PathBuf, BinarySource), ResolveError> {
    LUAU_LSP.resolve()
}
