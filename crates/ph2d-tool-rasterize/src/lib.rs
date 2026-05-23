#![forbid(unsafe_code)]
//! ph2d-tool-rasterize — Rasterize one-shot image action.
//!
//! Bakes the active Transform of the selected sprite — scale (with sign
//! / flip), rotation — directly into its source RGBA8 buffer, then
//! resets the sprite Transform to identity (scale = 1.0, rotation = 0).
//! The resulting bitmap reflects the on-screen pixels exactly while
//! restoring a clean 1:1 baseline for future edits.
//!
//! Sabor 1 (one-shot stateless) per DIRETRIZ §3.8.3: only [`register`]
//! is exposed; the Tool trait is not implemented (no panel, no modal
//! state). The chrome pill broadcasts `OneShotImageOp { tool_id =
//! "rasterize", entity_bits }` per selected sprite (Fase 0 broadcast in
//! `image_actions::apply`); the desktop shell drain reads the source
//! bitmap + Transform, calls [`rasterize`], commits the result via
//! `texture_edit::commit_edited_texture`, and writes back
//! `Transform { scale = (1.0, 1.0), rotation = 0.0, .. }`.
//!
//! ### Island shape (mirrors `ph2d-tool-make-square` / `-real-size`)
//!
//! - [`algorithm`] — pure-Rust `std`-only Mitchell-Netravali resample,
//!   flip, and rotation + unit tests.
//! - [`icon`] — Lucide-derived `BezPath` glyph for the Image Tools pill.
//! - [`manifest`] — `ToolManifest` declaration + handler stub.
//! - [`register`] — single entry point appended to
//!   `ph2d-tool-registry-init::register_all` by `ph2d-tool-sync`.

pub mod algorithm;
pub mod icon;
pub mod manifest;

pub use algorithm::{RasterizeResult, rasterize};
pub use icon::rasterize_bezpath;
pub use manifest::MANIFEST;

/// Register the Rasterize manifest with the runtime registry. Wired
/// into `ph2d-tool-registry-init::register_all` by `ph2d-tool-sync`.
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
        reg.build().expect("registry should build with rasterize");
        let found = reg
            .by_id("rasterize")
            .expect("rasterize should be registered by id");
        assert_eq!(found.id, "rasterize");
        assert_eq!(found.label_key, "tool.rasterize.label");
    }

    #[test]
    fn register_is_idempotent() {
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        register(&mut reg);
        reg.build().expect("idempotent registration should build");
        assert_eq!(reg.manifests().len(), 1);
    }
}
