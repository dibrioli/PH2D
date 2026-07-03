//! The brush/stroke-section slider forward whitelist — split from [`crate::event`]'s dispatch guard for
//! the file-LOC cap. A single membership predicate the `ValueChanged` arm matches on.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;

/// `true` when `id` is a brush / stroke-section slider whose `ValueChanged` the panel forwards to the
/// tool as a `PanelEvent::SetValue` carrying the dispatched `0..1` track (the tool maps it to its range).
/// The linked chips mirror their edit back as the slider's `ValueChanged`, so only the SLIDER ids are
/// listed here — the chip's own event is swallowed (a chip id isn't a Slider, so `store.slider` is None).
pub(crate) fn is_forwardable_brush_slider(id: NodeId) -> bool {
    id == core_ids::PAINTER_BRUSH_SIZE_SLIDER
        || id == core_ids::PAINTER_BRUSH_STRENGTH_SLIDER
        || id == core_ids::PAINTER_BRUSH_SPACING
        || id == core_ids::PAINTER_BRUSH_OFFSET
        || id == core_ids::PAINTER_BRUSH_JITTER
        || id == core_ids::PAINTER_BRUSH_DASH_RATIO
        || id == core_ids::PAINTER_BRUSH_DASH_LENGTH
        || id == core_ids::PAINTER_BRUSH_INPUT_SAMPLES
        || id == core_ids::PAINTER_BRUSH_STABILIZE
        || id == core_ids::PAINTER_BRUSH_RATE
        || id == core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS
        || id == core_ids::PAINTER_INPAINT_PATCH_SLIDER
        || id == core_ids::PAINTER_INPAINT_QUALITY_SLIDER
        || id == core_ids::PAINTER_INPAINT_SEARCH_SLIDER
        || core_ids::PAINTER_BRUSH_RANDOMIZE_SLIDERS.contains(&id)
        || core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH.contains(&id)
        // Selection section (ADR-0103): Feather + Automatic-threshold + overlay-opacity sliders.
        || id == core_ids::PAINTER_SEL_FEATHER_SLIDER
        || id == core_ids::PAINTER_SEL_THRESHOLD_SLIDER
        || id == core_ids::PAINTER_SEL_OPACITY_SLIDER
}
