#![forbid(unsafe_code)]
//! ph2d-tool-make-square — Make Square one-shot image action.
//!
//! Pads an RGBA8 image with fully-transparent pixels on its shorter
//! axis so the result is a perfect square (`width == height`) without
//! distorting or cropping any source pixel. The original is centered
//! inside the new canvas; the new transparent band is split as evenly
//! as possible between the two opposite edges.
//!
//! ### Shape of this island
//!
//! First convention-by-discovery tool crate (piloto PR 4 of
//! `docs/Migracao/2026-05-convention-by-discovery.md`). Follows the
//! template in Appendix A of the migration plan:
//!
//! - [`algorithm`] — pure-Rust `std`-only padding logic + tests.
//! - [`icon`] — Lucide-derived `BezPath` glyph for chrome rendering.
//! - [`manifest`] — `ToolManifest` declaration + handler stub.
//! - [`register`] — single entry point called by
//!   `crates/ph2d-editor/src/tools/registry_init.rs`.
//!
//! Public re-exports preserve the legacy `ph2d_editor::make_square` /
//! `ph2d_editor::square_bezpath` / `ph2d_editor::MakeSquareResult`
//! API surface — `ph2d-editor` keeps those `pub use`s pointing here
//! via the thin facade in `crates/ph2d-editor/src/tools/make_square/`
//! (removed in PR 10 cleanup).

pub mod algorithm;
pub mod icon;
pub mod manifest;

pub use algorithm::{MakeSquareResult, make_square};
pub use icon::square_bezpath;
pub use manifest::MANIFEST;

/// Register the Make Square manifest with the runtime registry.
/// Invoked from `crates/ph2d-editor/src/tools/registry_init.rs` —
/// **the only line in that file that mentions this tool**.
///
/// Idempotent on identical `&'static` references per
/// [`ph2d_tool_registry::Registry::register`].
pub fn register(reg: &mut ph2d_tool_registry::Registry) {
    reg.register(&MANIFEST);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        reg.build().expect("registry should build with make-square");
        let found = reg
            .by_id("make_square")
            .expect("make_square should be registered by id");
        assert_eq!(found.id, "make_square");
        assert_eq!(found.label_key, "tool.make_square.label");
    }

    #[test]
    fn register_is_idempotent() {
        // Registering twice is a no-op per the manifest dedup
        // contract — protects against accidental double-call in the
        // `register_all` append-only list.
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        register(&mut reg);
        reg.build().expect("idempotent registration should build");
        assert_eq!(reg.manifests().len(), 1);
    }
}
