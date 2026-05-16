//! Canvas zone enum — the 4-zone canvas-first layout per ADR-0023 §3.
//!
//! Lives in `ph2d-tool-registry` (not `ph2d-editor`) so [`ToolManifest`]
//! can declare its host zone without forcing tool crates to depend on
//! `ph2d-editor`. `ph2d-editor::zones` re-exports this so existing
//! callers (`zones::Zone::TopLeft`) keep working byte-for-byte.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Zone {
    /// Top-left (Editing — Gallery, Actions, Scene tree, Adjustments,
    /// Selections, Transform).
    TopLeft,
    /// Top-right (Creating — Tile painter, Sprite placer, Asset
    /// insertion, Brush, Layer, Color).
    TopRight,
    /// Sidebar (modulators — brush size/opacity, snap, undo/redo,
    /// eyedropper, Modify button).
    Sidebar,
    /// Center (canvas — 100 % of leftover).
    Center,
}
