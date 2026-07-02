//! [`PaintMode`] — which operation the canvas pointer performs. Split from `paint.rs` for the workspace
//! file-LOC cap; a plain data enum with no `PaintState` coupling.

/// Which operation the canvas pointer performs — selected from the left rail's Painter tools and routed
/// in via `PanelEvent::SelectOption(PAINTER_PAINT_MODE, …)`. `Paint` = the normal dab-stamp (colour,
/// Shape, Grain, ramps); `Smear` drags canvas content along the stroke; `Blur` softens under each dab;
/// `Clone` copies from a sampled source at a fixed offset; `Mask` paints a TEMPORARY tool-side scratch
/// mask that hides/reveals the current layer live (no layer created — see [`super::mask`]); `Inpaint` is
/// a content-aware HEAL brush — paint over a defect to mark it (live red tint), and on pen-up the marked
/// region is reconstructed from the surrounding pixels by the `ph2d-inpaint` engine (multi-scale
/// PatchMatch), cropped to the defect neighbourhood so it stays interactive (see [`super::inpaint`]).
/// Eraser is a blend override on top of `Paint`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum PaintMode {
    #[default]
    Paint,
    Smear,
    Blur,
    Clone,
    Mask,
    Inpaint,
    /// **Fill** (Bucket) — Procreate ColorDrop: drag the colour onto the canvas to flood-fill the
    /// connected same-colour region, then drag to adjust the threshold live (see [`super::fill`]).
    Fill,
}

/// Number of [`PaintMode`] variants — the length of the per-mode brush-settings array (see
/// [`PaintMode::slot`]). Keep in lock-step with the enum.
pub(crate) const PAINT_MODE_COUNT: usize = 7;

impl PaintMode {
    /// This mode's index into the per-mode brush-settings array (`0..PAINT_MODE_COUNT`). Each tool keeps
    /// its own [`ph2d_painter_brush::BrushSpec`] here when settings are NOT linked across tools.
    pub(crate) fn slot(self) -> usize {
        match self {
            PaintMode::Paint => 0,
            PaintMode::Smear => 1,
            PaintMode::Blur => 2,
            PaintMode::Clone => 3,
            PaintMode::Mask => 4,
            PaintMode::Inpaint => 5,
            PaintMode::Fill => 6,
        }
    }
}
