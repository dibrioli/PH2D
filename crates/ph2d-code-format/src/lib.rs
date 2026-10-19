//! `ph2d-code-format` — formatting a Luau script with the PINNED StyLua (owner decision 6; CodeEditor
//! Onda 2 I13). The program is a child process, never modified; the pin is `stylua-pin.toml`, the
//! download `scripts/stylua-fetch.sh`, the formatting rules `ph2d-format.toml` (always passed with
//! `--config-path`, so no config of the user's home, cwd or `.editorconfig` is ever read).
//!
//! StyLua does not fit a 60 Hz frame above ~800 lines (I0d), so it runs on a [`Worker`] thread and the
//! answer comes back as the minimal [`Edit`]s against the text it was given ([`changes`]).
//! Measurements: `docs/CodeEditor/ferramentas/medicoes_onda2/I0d_stylua.md`.

mod diff;
mod run;
mod worker;

pub use diff::{Edit, changes};
pub use ph2d_code_lsp::{BinarySource, Pin, ResolveError};
pub use run::{CONFIG, FormatFn, Program, Refusal, classify};
pub use worker::{Done, Outcome, Worker};

/// The StyLua pin.
pub const STYLUA: Pin = Pin {
    manifest: include_str!("../stylua-pin.toml"),
    env: "PH2D_STYLUA",
    tool: "stylua",
    fetch: "scripts/stylua-fetch.sh",
};
