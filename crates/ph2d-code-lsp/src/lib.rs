//! `ph2d-code-lsp` — the pinned `luau-lsp` (ADR-0188) and a minimal stdio LSP client for it.
//!
//! No UI here: the script editor drives [`LspClient`] with the buffer text and PH2D's
//! `runtime/luau/ph2d.d.luau` as `@ph2d` definitions. Pin + checksums: `luau-lsp.toml`; download:
//! `scripts/luau-lsp-fetch.sh`.

pub mod client;
pub mod framing;
pub mod position;
pub mod resolve;

pub use client::{
    CompletionItem, Diagnostic, LspClient, hooks_in_definitions, is_unused_engine_hook, LspError, Notification, PublishedDiagnostics,
    ServerConfig, ServerMessage, Solver, ph2d_settings,
};
pub use position::{Position, Range, byte_to_position, end_position, position_to_byte};
pub use resolve::{BinarySource, ResolveError, pinned_version, resolve_binary};
