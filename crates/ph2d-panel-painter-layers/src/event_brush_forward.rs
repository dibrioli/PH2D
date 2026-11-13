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
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_FLOW_SLIDER
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
        || ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS.contains(&id)
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

/// `true` when `id` is a Brush-section toggle / momentary button the panel forwards to the tool as a plain
/// `PanelEvent::Click` (the tool flips the matching bool / acts). Split out of `event.rs`'s dispatch so
/// `try_apply_brush_event` stays under the function cap.
pub(crate) fn is_forwarded_click(id: NodeId) -> bool {
    id == ph2d_tool_painter::ids::PAINTER_BRUSH_ERASER
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE
                || ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_SCOPES.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_BRUSH_TILING.contains(&id)
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_REPEAT_IMAGE
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_SPACE_ATTEN
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_CONTINUOUS_SPACING
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_ACCUMULATE
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_WASH
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_SMEAR_FIO
                // Color mixing: as opções Off · Wet blend e as do Sample (`cards_mistura`).
                || ph2d_tool_painter::ids::PAINTER_BRUSH_MIX_MODES.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_BRUSH_MIX_SAMPLES.contains(&id)
                || id == ph2d_tool_painter::ids::PAINTER_LINE_SOLID
                || id == ph2d_tool_painter::ids::PAINTER_LINE_SKETCHY_MAGNETIFY
                || id == ph2d_tool_painter::ids::PAINTER_LINE_WIRE_CONNECTION
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_SYNC
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_LINE_DIMENSIONS
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_EDGE_TO_EDGE
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_SHOW
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_RAKE
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_GRAIN_SAME_AS_PAPER
                || id == ph2d_tool_painter::ids::PAINTER_SHAPE_WATERCOLOR_AUTO
                || id == ph2d_tool_painter::ids::PAINTER_SHAPE_RESET
                || id == ph2d_tool_painter::ids::PAINTER_SHAPE_USE_LAYERS
                || id == ph2d_tool_painter::ids::PAINTER_SHAPE_HOSE_ADD
                || id == ph2d_tool_painter::ids::PAINTER_SHAPE_HOSE_CLEAR
                || id == ph2d_tool_painter::ids::PAINTER_DUAL_IMAGE
                || id == ph2d_tool_painter::ids::PAINTER_SHAPE_PER_LAYER_COLOR
                || id == ph2d_tool_painter::ids::PAINTER_SHAPE_ALPHA_FROM_IMAGE
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_PONTAS_RECTAS
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_FIXO
                || ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_RAMP_BUTTONS.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_BUTTONS.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_BUTTONS.contains(&id)
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_FALLOFF_ADD // Custom-falloff "+" point button
                // Stroke shape-editor buttons (Apply/Apply&Keep/Delete/Convert/Simplify/Merge) — MUST be
                // forwarded or the Click is dropped. Plus the Offset-card Trim + the Operation segments.
                || ph2d_tool_painter::ids::PAINTER_BRUSH_STROKE_BUTTONS.contains(&id)
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_OFFSET_TRIM
                || ph2d_tool_painter::ids::PAINTER_STROKE_OP_IDS.contains(&id)
                // Symmetry: Use/Circular checkboxes, X/Y/Custom axis segments, Draw-Line/Pick-Center, reset.
                || ph2d_tool_painter::ids::PAINTER_BRUSH_SYMMETRY_CLICKABLE.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_BRUSH_SECTION_RESETS.contains(&id)
                // Wet Paint (see PAINTER_WETPAINT_CLICKS: absent here = dead under the mouse).
                || ph2d_tool_painter::ids::PAINTER_WETPAINT_CLICKS.contains(&id)
                || id == ph2d_tool_painter::ids::PAINTER_PERFORMANCE_PALPITE
                // Watercolor section: Wet-edges + Pigment toggles + the section reset.
                || ph2d_tool_painter::ids::PAINTER_WATERCOLOR_CLICKS.contains(&id)
                // Impasto: Enable + the section reset + Depth-Source / Draw-To segments + Show Impasto.
                || ph2d_tool_painter::ids::PAINTER_IMPASTO_CLICKS.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_BUTTONS.contains(&id)
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_CLONE_SET_SOURCE
                || id == ph2d_tool_painter::ids::PAINTER_BRUSH_CLONE_ALIGNED
                // Mask: sub-brush segments, canvas op buttons, overlay-colour swatches, Apply.
                || ph2d_tool_painter::ids::PAINTER_MASK_BRUSH.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_MASK_OP.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_MASK_COLOR.contains(&id)
                || id == ph2d_tool_painter::ids::PAINTER_MASK_APPLY
                // Selection (ADR-0103): mode/op/action segments + Edit/Convert + Wave-5 content actions.
                || ph2d_tool_painter::ids::PAINTER_SEL_MODE_IDS.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_SEL_OP_IDS.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_SEL_ACTION_IDS.contains(&id)
                || ph2d_tool_painter::ids::PAINTER_SEL_WAVE5_IDS.contains(&id)
                || id == ph2d_tool_painter::ids::PAINTER_SEL_EDIT
                || id == ph2d_tool_painter::ids::PAINTER_SEL_CONVERT || id == ph2d_tool_painter::ids::PAINTER_SEL_SIMPLIFY
                || id == ph2d_tool_painter::ids::PAINTER_SEL_MERGE
                || ph2d_tool_painter::ids::PAINTER_SEL_OFFSET_APPLY_IDS.contains(&id)
                || is_deform_click(id)
                // Sculpt (`docs/Painter/18…`): the Smooth / Sharpen sub-mode segments.
                || is_sculpt_click(id)
}
