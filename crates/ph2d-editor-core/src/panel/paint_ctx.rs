//! `PaintCtx` — per-frame context handed to every panel's
//! `Panel::paint` impl.
//!
//! ADR-0029 §4.5. Differs from the pre-ADR `PaintCtx` (which lived
//! in `ph2d-editor::panel_registry` and held `&mut HeroScreen`
//! directly): now holds `&mut dyn PanelHostInternal` so panel
//! crates have zero static dep on the concrete host type.

use super::PanelHostInternal;
use crate::screen::HeroLayout;
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Per-frame context for panel paint thunks.
///
/// Borrow note: `host`, `scene`, and `text_system` are independent
/// `&mut`s; Rust's field-level borrow splitting handles disjoint
/// use within a single paint pass.
pub struct PaintCtx<'a> {
    /// Host (typically `HeroScreen`). Panels read/write store +
    /// theme + project + per-panel-state through this trait object.
    pub host: &'a mut dyn PanelHostInternal,
    /// Pre-computed sub-region rects. Floating panels (Widget
    /// Gallery, Grid Snap) compute their own rects locally; this
    /// gives docked panels (Inspector, Hierarchy) their slot.
    pub layout: &'a HeroLayout,
    /// Outer viewport — needed by floating-panel clamping math.
    pub viewport: Rect,
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
}
