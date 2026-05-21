#![forbid(unsafe_code)]
//! ph2d-tool-padding — Padding / Expand canvas-resize tool.
//!
//! **Condenses two legacy tools into one** (Game-Engine-Legada):
//! - *Image Padding* (`addPadding`) — signed per-edge padding/crop of all
//!   four edges at once.
//! - *Directional Expand* (`directionalExpand`) — grow/shrink from a
//!   single edge by N pixels, driven by a gizmo edge-drag.
//!
//! They are the same operation: directional expand is just editing ONE
//! field of the per-edge [`algorithm::PaddingSpec`]. So the algorithm is
//! a single function — [`algorithm::add_padding`] — that resizes the
//! canvas by a signed `{top, right, bottom, left}` spec (positive =
//! transparent expand, negative = crop), and reports the content offset
//! so the caller can keep the sprite's world position stable.
//!
//! ### Shape of this island (mirrors `ph2d-tool-make-square`)
//! - [`algorithm`] — pure `add_padding` + types + tests (no editor/ECS dep).
//! - [`icon`] — `BezPath` glyph for the chrome pill.
//! - [`manifest`] — `ToolManifest` declaration + handler stubs.
//! - [`register`] — appended (one line) to `register_all` by the Coordinator.
//!
//! SCAFFOLD (Coordinator): the algorithm signature + types + manifest are
//! pinned; the Implementer fills [`algorithm::add_padding`] (+ tests) and
//! the real [`icon::padding_bezpath`] glyph. The **stateful integration**
//! — the 4-field panel, the gizmo edge-drag (directional expand UX), the
//! live preview, the shell bake + pivot/Transform fix-up, and the
//! chrome-pill wiring — is done by the Coordinator on a clean tree (it
//! touches editor-core / shells / gizmo, NOT this crate).

pub mod algorithm;
pub mod icon;
pub mod manifest;

pub use algorithm::{PaddingResult, PaddingSpec, add_padding};
pub use icon::padding_bezpath;
pub use manifest::MANIFEST;

/// Register the Padding manifest with the runtime registry. Appended
/// (one line) to `ph2d-tool-registry-init::register_all` by the
/// Coordinator once the stateful integration lands.
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
        reg.build().expect("registry should build with padding");
        let found = reg.by_id("padding").expect("padding registered by id");
        assert_eq!(found.id, "padding");
        assert_eq!(found.label_key, "tool.padding.label");
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
