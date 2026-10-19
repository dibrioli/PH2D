//! One test binary for the crate (repo convention: a new test = a file here + a `mod` line).
//!
//! The gates that drive luau-lsp FAIL (never skip) when the pinned binary is missing:
//! run `bash scripts/luau-lsp-fetch.sh`.

mod support;

mod controlo_de_cegueira;
mod definicoes;
mod framing;
mod ganchos;
mod latencia;
mod position_utf16;
mod script_canonico;
mod solver_scripts_reais;
mod supervisao;
