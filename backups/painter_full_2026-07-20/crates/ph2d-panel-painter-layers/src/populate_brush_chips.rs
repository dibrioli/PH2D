//! The editable numeric-chip registration paired with EVERY brush slider — split from [`crate::populate`]
//! for the file-LOC cap. Called once at host boot from `populate`.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::TextInputState;

/// Register the editable numeric chip paired with EVERY brush slider (the canonical slider-with-chip
/// form, not just Size/Strength — the Stroke + Jitter rows were bare slider+text-readout before, Enio
/// 2026-06-26). `link_slider_number` ties each chip to its slider so a chip edit / drag mirrors back
/// as the slider's `ValueChanged` (the existing brush-slider forward handles it); `set_number_range
/// (0, 1, step)` makes the chip DRAG span the slider's `0..1` track proportionally — coherent with its
/// limits, like the texture number boxes. Split out of [`crate::populate`] for the 200-LOC fn cap.
pub(crate) fn register_brush_slider_chips(store: &mut WidgetStore) {
    use ph2d_editor_core::ids as core_ids;
    // Counts (Dash Length / Input Samples) step by one whole sample on the `0..1` track.
    let count_step = 1.0 / f64::from(ph2d_tool_painter::BRUSH_COUNT_SLIDER_MAX - 1);
    // The default chip stepper / drag increment on the `0..1` track (a behaviour value, not a layout
    // token — the gate's design-token rule doesn't apply).
    const STEP: f64 = 0.01; // LITERAL-PX-OK: chip 0..1 track step (non-design behaviour value)
    let slider_chips = [
        (
            core_ids::PAINTER_BRUSH_SIZE_SLIDER,
            core_ids::PAINTER_BRUSH_SIZE_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_STRENGTH_SLIDER,
            core_ids::PAINTER_BRUSH_STRENGTH_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
            core_ids::PAINTER_BRUSH_COLOR_JITTER_HUE_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_COLOR_JITTER_SAT,
            core_ids::PAINTER_BRUSH_COLOR_JITTER_SAT_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
            core_ids::PAINTER_BRUSH_COLOR_JITTER_VAL_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_RATE,
            core_ids::PAINTER_BRUSH_RATE_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_SPACING,
            core_ids::PAINTER_BRUSH_SPACING_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_OFFSET,
            core_ids::PAINTER_BRUSH_OFFSET_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_DASH_RATIO,
            core_ids::PAINTER_BRUSH_DASH_RATIO_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_DASH_LENGTH,
            core_ids::PAINTER_BRUSH_DASH_LENGTH_CHIP,
            count_step,
        ),
        (
            core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
            core_ids::PAINTER_BRUSH_INPUT_SAMPLES_CHIP,
            count_step,
        ),
        (
            core_ids::PAINTER_BRUSH_STABILIZE,
            core_ids::PAINTER_BRUSH_STABILIZE_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_JITTER,
            core_ids::PAINTER_BRUSH_JITTER_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_JITTER_SCALE,
            core_ids::PAINTER_BRUSH_JITTER_SCALE_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_JITTER_ROTATE,
            core_ids::PAINTER_BRUSH_JITTER_ROTATE_CHIP,
            STEP,
        ),
        (
            core_ids::PAINTER_BRUSH_JITTER_SPACING,
            core_ids::PAINTER_BRUSH_JITTER_SPACING_CHIP,
            STEP,
        ),
    ];
    for (slider, chip, step) in slider_chips {
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number(slider, chip);
        store.set_number_range(chip, 0.0, 1.0, step);
    }
    // Symmetry **segment count** chip — unlike the `0..1` rows above, it shows a whole-number 3..12.
    // The mapped-integer link projects the slider's `0..1` track onto `3..12` (`display = track*9 + 3`)
    // and snaps to an integer; `set_number_range` makes its drag/stepper span 3..12 by ones.
    store.register(
        core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 6.0, // LITERAL-PX-OK: default symmetry segment count
            buffer: String::new(),
            caret: 0,
            last_committed: 6.0, // LITERAL-PX-OK: default symmetry segment count
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped_integer(
        core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS,
        core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP,
        9.0, // LITERAL-PX-OK: scale — 0..1 track → a span of 9 segments (3 → 12)
        3.0, // LITERAL-PX-OK: offset — the track's 0 maps to 3 segments
    );
    store.set_number_range(
        core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP,
        3.0,  // LITERAL-PX-OK: min symmetry segment count
        12.0, // LITERAL-PX-OK: max symmetry segment count
        1.0,
    );
    // Inpaint heal chips — like the Symmetry chip, whole-number mapped-integer chips (the tool reads the
    // slider's `0..1` track and maps it to the engine value): Patch `2..6`, Quality `3..12`, Search `50..
    // 300` %. `(slider, chip, scale, offset, min, max, step, default)`.
    for (slider, chip, scale, offset, min, max, step, default) in [
        (
            core_ids::PAINTER_INPAINT_PATCH_SLIDER,
            core_ids::PAINTER_INPAINT_PATCH_CHIP,
            4.0, // LITERAL-PX-OK: scale — 0..1 track → span of 4 (2 → 6 patch radius)
            2.0, // LITERAL-PX-OK: offset — the track's 0 maps to 2
            2.0, // LITERAL-PX-OK: min patch radius
            6.0, // LITERAL-PX-OK: max patch radius
            1.0, // step by whole radius
            3.0, // LITERAL-PX-OK: default patch radius (norm 0.25)
        ),
        (
            core_ids::PAINTER_INPAINT_QUALITY_SLIDER,
            core_ids::PAINTER_INPAINT_QUALITY_CHIP,
            9.0,  // LITERAL-PX-OK: scale — 0..1 track → span of 9 (3 → 12 EM iterations)
            3.0,  // LITERAL-PX-OK: offset — the track's 0 maps to 3
            3.0,  // LITERAL-PX-OK: min EM iterations
            12.0, // LITERAL-PX-OK: max EM iterations
            1.0,  // step by whole iteration
            6.0,  // LITERAL-PX-OK: default EM iterations (norm 0.3333)
        ),
        (
            core_ids::PAINTER_INPAINT_SEARCH_SLIDER,
            core_ids::PAINTER_INPAINT_SEARCH_CHIP,
            250.0, // LITERAL-PX-OK: scale — 0..1 track → span of 250 (50 → 300 %)
            50.0,  // LITERAL-PX-OK: offset — the track's 0 maps to 50 %
            50.0,  // LITERAL-PX-OK: min search %
            300.0, // LITERAL-PX-OK: max search %
            10.0,  // LITERAL-PX-OK: search % stepper increment
            100.0, // LITERAL-PX-OK: default search % (norm 0.2 → mult 1.0)
        ),
    ] {
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: default,
                buffer: String::new(),
                caret: 0,
                last_committed: default,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped_integer(slider, chip, scale, offset);
        store.set_number_range(chip, min, max, step);
    }
}
