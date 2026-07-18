//! The emitter's **param UI metadata** — labels, ranges, widgets. Split from
//! `lib.rs` at the HR-18 LOC cap; it is a clean seam, because none of this is
//! behaviour: the node computes the same particles whatever a slider looks like.
//!
//! The one thing here that is NOT free-standing is the `max` row's ceiling: it
//! and [`super::MAX_ALIVE`] answer the same question, so it is DERIVED, never
//! re-typed ([[feedback_two_doors_to_the_same_question_diverge]]).

use super::MAX_ALIVE;
use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

/// Params whose typed entry reaches past their slider (Blender's hard limits).
///
/// A `rate` in the millions is not a mis-click: paired with a millisecond `life`
/// it is a one-frame burst, and `MAX_ALIVE` bounds what actually gets BUILT
/// regardless — `rate` drives no allocation at all, only how fast spawn indices
/// advance.
///
/// ⚠️ **This number does not protect anything, and the thing that needs
/// protecting is not a rate.** The emitter's real ceiling is the product
/// `rate × playhead < 2²⁴`, because ids are `f32`: past 16.777.216 two particles
/// share an id and both the CPU pairing and the GPU gather hand them the same
/// state, silently (gated: `the_id_space_is_exact_only_below_two_to_the_24`).
/// At 4.000.000/s that is **4,2 seconds** of playback; at the slider's 12.000/s
/// it is 23 minutes. No STATIC cap on `rate` can express a bound on a product,
/// which is why this one is an ergonomic stop and not a safety one. The fix is
/// exact ids (wrapped indices, or a `u32`/`f64` id column) — a change to the
/// stream data model, not to this table.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "rate",
    max: 4_000_000.0,
}];

/// Param UI hints (M1.P1).
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rate",
        label: "Rate",
        min: 0.0,
        // The SLIDER's range: 12.000/s is a dense fountain at a 1 s life, and the
        // whole authoring range lives below it. Typing reaches much further —
        // see `PARAM_HARD_MAX`.
        max: 12_000.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "life",
        label: "Life",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "spread",
        label: "Spread",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "x",
        label: "Origin X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "y",
        label: "Origin Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 100.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "max",
        label: "Max Particles",
        min: 1.0,
        // DERIVED, never re-typed: this row and `MAX_ALIVE` answer the same
        // question ("how many particles may be alive?"), and when the constant
        // went 4096 → 16384 this literal stayed behind and silently became the
        // real ceiling the artist could reach
        // ([[feedback_two_doors_to_the_same_question_diverge]]).
        max: MAX_ALIVE as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "size",
        label: "Size",
        min: 0.01,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];
