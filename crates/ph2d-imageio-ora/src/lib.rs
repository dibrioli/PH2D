#![forbid(unsafe_code)]
//! `ph2d-imageio-ora` — OpenRaster (`.ora`) decode + encode (W2.T1).
//!
//! Spec: <https://www.openraster.org/baseline/file-layout-spec-v1.html>
//!
//! ### Container layout
//!
//! - `mimetype` — STORE (uncompressed) first entry, content
//!   `image/openraster`.
//! - `stack.xml` — XML manifest describing the layer tree.
//! - `data/layer<N>.png` — RGBA8 PNG per pixel layer.
//! - `mergedimage.png` — composite preview (recommended; W2.T1
//!   emits a placeholder; proper composite lands W3 when the
//!   Painter composite pipeline is callable).
//!
//! ### What W2.T1 covers
//!
//! - Magic: ZIP signature `PK\x03\x04` (Strong) + `"ora"` extension
//!   (Weak).
//! - Decode: parse `stack.xml`, recursively build the [`Layer`](ph2d_imageio::Layer) tree
//!   (Pixel + Group), load each layer PNG into
//!   `ImageBuffer<SrgbRgba>`. Layer ordering reversed from ORA's
//!   top-first to our bottom-up convention.
//! - Encode: write ZIP with mimetype (STORE), hand-written
//!   `stack.xml`, per-pixel-layer PNG, `mergedimage.png` placeholder.
//! - Blend mode mapping: 15 ORA `svg:*` modes ↔ [`BlendMode`](ph2d_imageio::BlendMode) variants
//!   (table in `parse_blend_mode`). Unknown ORA modes default to
//!   `BlendMode::Normal` on import; non-ORA `BlendMode` variants
//!   export as `svg:src-over` (audit follow-up: extend mapping when
//!   ORA spec catches up).
//! - HR-14 [`LayerStack::version`](ph2d_imageio::LayerStack::version) / [`Layer::version`](ph2d_imageio::Layer::version) preserved.
//! - HR-15 Fluent keys on errors.
//!
//! ### Out of scope (later waves)
//!
//! - `LayerKind::Adjustment` / `Text` / `Smart` — ORA spec only
//!   models Pixel + Group. Importer accepts only those; exporter
//!   emits non-Pixel/non-Group `LayerKind`s as `Error::Unsupported`
//!   so the caller knows the document has unsupported richness.
//! - [`LayerEffect`](ph2d_imageio::LayerEffect) preservation — ORA spec has no equivalent.
//!   Layer effects are silently dropped on export.
//! - Per-layer [`ColorProfile`](ph2d_imageio::ColorProfile) — ORA spec assumes whole-document
//!   sRGB; per-layer profiles drop on export.
//! - Thumbnail (`Thumbnails/thumbnail.png`) — W2+ derives from
//!   mergedimage downscale.
//! - Proper composite for `mergedimage.png` — W3 when Painter
//!   composite is callable from imageio.

use ph2d_imageio::{ExporterRegistry, ImporterRegistry};

mod blend;
mod export;
mod import;

#[cfg(test)]
mod tests;

pub use export::OraExporter;
pub use import::OraImporter;

/// Register the ORA importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(OraImporter));
}

/// Register the ORA exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(OraExporter));
}
