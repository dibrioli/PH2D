//! The brush/stroke-section slider forward whitelist — split from [`crate::event`]'s dispatch guard for
//! the file-LOC cap. A single membership predicate the `ValueChanged` arm matches on.

use ph2d_a11y::NodeId;

/// `true` when `id` is a brush / stroke-section slider whose `ValueChanged` the panel forwards to the
/// tool as a `PanelEvent::SetValue` carrying the dispatched `0..1` track (the tool maps it to its range).
/// The linked chips mirror their edit back as the slider's `ValueChanged`, so only the SLIDER ids are
/// listed here — the chip's own event is swallowed (a chip id isn't a Slider, so `store.slider` is None).
pub(crate) fn is_forwardable_brush_slider(id: NodeId) -> bool {
    id == ph2d_tool_painter::ids::PAINTER_BRUSH_SIZE_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_STRENGTH_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_SPACING
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_OFFSET
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER
        // As barras do card Line — a MESMA tabela que as pinta e as regista.
        || crate::line_barras::e_barra(id)
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_DASH_RATIO
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_DASH_LENGTH
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_INPUT_SAMPLES
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_STABILIZE
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_RATE
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_SPRAY_COUNT
        // Grid Stamp lattice (cell size + origin offset, per axis).
        || ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_CELL.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_OFFSET.contains(&id)
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_FIT
        || id == ph2d_tool_painter::ids::PAINTER_INPAINT_PATCH_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_INPAINT_QUALITY_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_INPAINT_SEARCH_SLIDER
        || ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_SLIDERS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE.contains(&id)
        // Selection section (ADR-0103): Feather + Automatic-threshold + overlay-opacity + Offset sliders.
        || id == ph2d_tool_painter::ids::PAINTER_SEL_FEATHER_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_SEL_THRESHOLD_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_SEL_STABILIZE_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_SEL_OPACITY_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_SEL_OFFSET_SLIDER
        // Deform section (Wave 1): Size / Pressure / Distortion / Momentum / Strength sliders.
        || id == ph2d_tool_painter::ids::PAINTER_DEFORM_SIZE_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_DEFORM_PRESSURE_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_DEFORM_DISTORTION_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_DEFORM_MOMENTUM_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_DEFORM_STRENGTH_SLIDER
        // Sculpt section: the Smooth family's kernel Radius (W1) and the plane family's Offset (W2). The
        // card paints one or the other by family, but BOTH are forwarded — a row that is only reachable in
        // one mode is still a row, and the seam sweep drives them both. (Strength is the BRUSH's — the
        // sculpt does not have a second one, on purpose.)
        || id == ph2d_tool_painter::ids::PAINTER_SCULPT_RADIUS_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_SCULPT_OFFSET_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_SCULPT_DEPTH_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_SCULPT_ANGLE_SLIDER
        || id == ph2d_tool_painter::ids::PAINTER_SCULPT_SMOOTH_SLIDER
}

/// `true` when `id` is a Sculpt-panel **Click** target (the five sub-mode segments).
pub(crate) fn is_sculpt_click(id: NodeId) -> bool {
    ph2d_tool_painter::ids::PAINTER_SCULPT_CLICKS.contains(&id)
}

/// `true` when `id` is a Deform-panel **Click** target the panel forwards to the tool as
/// `PanelEvent::Click` (the mode segments, Freeze / Invert toggles, and Reset / Apply / Apply & Keep
/// actions — Deform Wave 1). Split out of `event.rs`'s dispatch guard so that (at-cap) match stays put.
pub(crate) fn is_deform_click(id: NodeId) -> bool {
    ph2d_tool_painter::ids::PAINTER_DEFORM_MODE_IDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_DEFORM_TEMPERAMENT_IDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_DEFORM_TRANSFORM_MODE_IDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_DEFORM_ACTION_IDS.contains(&id)
        // W4: the Affect Relief toggle (forwarded as a Button Click, like every checkbox row here).
        || id == ph2d_tool_painter::ids::PAINTER_DEFORM_RELIEF
}
