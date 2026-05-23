#![forbid(unsafe_code)]
//! ph2d-tool-real-size — Real Size (1:1) one-shot transform action.
//!
//! Resets the selected sprite's `Transform` scale to 1:1 — i.e. it
//! renders at its source pixel dimensions again — while preserving the
//! flip sign of each axis (a sprite flipped on X stays flipped, just at
//! `scaleX = -1`). Port of the legacy engine's `toggle_realsize` /
//! `real-size` (`Game-Engine-Legada/apps/editor/src/...`).
//!
//! ### Shape of this island (mirrors `ph2d-tool-make-square`)
//!
//! - [`algorithm`] — pure scale-reset logic + unit tests. No deps on the
//!   editor / ECS: operates on a plain `[f32; 2]` so the crate stays a
//!   leaf island (the desktop shell converts to/from `Transform.scale`).
//! - [`icon`] — Lucide-derived `BezPath` glyph for the chrome pill.
//! - [`manifest`] — `ToolManifest` declaration + handler stub.
//! - [`register`] — single entry point appended to
//!   `ph2d-tool-registry-init::register_all`.
//!
//! SCAFFOLD (Coordinator): manifest + island shape are pinned; the
//! Implementer fills [`algorithm::real_size_scale`] (+ tests) and the
//! real [`icon::real_size_bezpath`] glyph. The chrome-pill wiring
//! (NodeId, the generic `EditorAction::OneShotImageOp { tool_id:
//! "real_size", entity_bits }` push, dispatch, shell drain) is done by
//! the Coordinator on a clean tree — NOT in this crate.

pub mod algorithm;
pub mod icon;
pub mod manifest;

pub use algorithm::real_size_scale;
pub use icon::real_size_bezpath;
pub use manifest::MANIFEST;

/// Register the Real Size manifest with the runtime registry. Appended
/// (one line) to `ph2d-tool-registry-init::register_all` by the
/// Coordinator once the wiring lands.
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
        reg.build().expect("registry should build with real-size");
        let found = reg
            .by_id("real_size")
            .expect("real_size should be registered by id");
        assert_eq!(found.id, "real_size");
        assert_eq!(found.label_key, "tool.real_size.label");
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
