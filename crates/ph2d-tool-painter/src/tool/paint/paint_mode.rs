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
}
