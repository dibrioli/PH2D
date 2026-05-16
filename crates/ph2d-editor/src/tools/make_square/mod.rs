//! Make Square — thin facade re-exporting the standalone tool crate.
//!
//! Before PR 4 piloto, `algorithm.rs` and `icon.rs` lived here. After
//! PR 4 they moved to `crates/ph2d-tool-make-square/` (the first
//! convention-by-discovery tool crate per
//! `docs/Migracao/2026-05-convention-by-discovery.md`). This module
//! re-exports the public surface so legacy callers
//! (`ph2d_editor::make_square`, `ph2d_editor::square_bezpath`,
//! `ph2d_editor::MakeSquareResult`) keep working byte-for-byte.
//!
//! Removed in PR 10 cleanup once the call-site sweep confirms zero
//! consumers reach for `ph2d_editor::tools::make_square` (vs the
//! flat `ph2d_editor::make_square` re-export). The shell already uses
//! the flat path; the path-prefixed module is here only to avoid
//! breaking any indirect consumer during the transition.

pub use ph2d_tool_make_square::{MakeSquareResult, make_square, square_bezpath};
