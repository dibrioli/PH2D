//! **Shape** — operations on the value itself. No clock, no links.
//!
//! ⚠️ `Multiply / Add` is the only one here with a neutral, and it is neutral at
//! its DEFAULTS (`value*1 + 0`). The clamps are [`Neutrality::NoNeutral`] because
//! `Limit` is only the identity with an infinite range, which is not a default
//! anyone would choose — declaring a fake neutral would make the neutrality gate
//! assert something the artist can never reach.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Family, Neutrality, Recipe, RowKind};

pub const LIMIT: Recipe = Recipe {
    id: "limit",
    family: Family::Shape,
    label: "Limit",
    blurb: "Keep the value between two bounds.",
    aliases: &["clamp", "constrain", "min max", "bound", "cap", "range"],
    knobs: &[
        Knob::num("min", "Min", -1.0, (-40.0, 40.0)),
        Knob::num("max", "Max", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("min(max({}, {}), {})", c.inner, c.n(0), c.n(1)),
};

pub const FLOOR_AT: Recipe = Recipe {
    id: "floor-at",
    family: Family::Shape,
    label: "Floor At",
    blurb: "Never let the value go below this.",
    aliases: &["min", "lower bound", "at least", "ground", "clamp low"],
    knobs: &[Knob::num("min", "Min", -1.0, (-40.0, 40.0))],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("max({}, {})", c.inner, c.n(0)),
};

pub const CEILING_AT: Recipe = Recipe {
    id: "ceiling-at",
    family: Family::Shape,
    label: "Ceiling At",
    blurb: "Never let the value go above this.",
    aliases: &["max", "upper bound", "at most", "cap", "clamp high"],
    knobs: &[Knob::num("max", "Max", 1.0, (-40.0, 40.0))],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("min({}, {})", c.inner, c.n(0)),
};

pub const REMAP: Recipe = Recipe {
    id: "remap",
    family: Family::Shape,
    label: "Remap",
    blurb: "Rescale one range of values onto another.",
    aliases: &[
        "linear",
        "range mapper",
        "rescale",
        "normalize",
        "fit",
        "convert",
    ],
    knobs: &[
        Knob::num("in_lo", "In Min", 0.0, (-40.0, 40.0)),
        Knob::num("in_hi", "In Max", 1.0, (-40.0, 40.0)),
        Knob::num("out_lo", "Out Min", 0.0, (-40.0, 40.0)),
        Knob::num("out_hi", "Out Max", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "mix({}, {}, ({} - {})/({} - {}))",
            c.n(2),
            c.n(3),
            c.inner,
            c.n(0),
            c.span_hi(0, 1),
            c.n(0)
        )
    },
};

pub const REMAP_CLAMPED: Recipe = Recipe {
    id: "remap-clamped",
    family: Family::Shape,
    label: "Remap (Clamped)",
    blurb: "Remap, but never overshoot past the output range.",
    aliases: &["linear clamp", "ease range", "map clamped", "safe remap"],
    knobs: &[
        Knob::num("in_lo", "In Min", 0.0, (-40.0, 40.0)),
        Knob::num("in_hi", "In Max", 1.0, (-40.0, 40.0)),
        Knob::num("out_lo", "Out Min", 0.0, (-40.0, 40.0)),
        Knob::num("out_hi", "Out Max", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "mix({}, {}, min(max(({} - {})/({} - {}), 0), 1))",
            c.n(2),
            c.n(3),
            c.inner,
            c.n(0),
            c.span_hi(0, 1),
            c.n(0)
        )
    },
};

pub const MULTIPLY_ADD: Recipe = Recipe {
    id: "multiply-add",
    family: Family::Shape,
    label: "Multiply / Add",
    blurb: "Scale the value, then shift it.",
    aliases: &[
        "scale",
        "gain",
        "offset",
        "times plus",
        "linear transform",
        // Inherited from the retired `Negate`, which was EXACTLY this with Multiply -1.
        "negate",
        "invert sign",
        "minus",
        "opposite sign",
    ],
    knobs: &[
        Knob::num("multiply", "Multiply", 1.0, (-10.0, 10.0)),
        Knob::num("offset", "Offset", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    // Already the identity at its defaults — the one recipe whose neutral needs
    // no overrides at all.
    neutral: Neutrality::Additive(&[]),
    pair: None,
    // ⚠️ `tight()`, not `inner`: with `inner = "value + wiggle(2, 30)"` the
    // unparenthesised form means `value + (wiggle*m) + o` — a different animation
    // that parses. This is the row the pairwise-composition gate exists for.
    emit: |c| format!("{}*{} + {}", c.tight(), c.n(0), c.n(1)),
};

pub const INVERT_RANGE: Recipe = Recipe {
    id: "invert-range",
    family: Family::Shape,
    label: "Invert in Range",
    blurb: "Turn the value inside out between two bounds.",
    aliases: &["flip range", "reverse range", "mirror value", "complement"],
    knobs: &[
        Knob::num("min", "Min", 0.0, (-40.0, 40.0)),
        Knob::num("max", "Max", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("{} + {} - {}", c.n(0), c.n(1), c.tight()),
};

pub const ABSOLUTE: Recipe = Recipe {
    id: "absolute",
    family: Family::Shape,
    label: "Absolute",
    blurb: "Drop the sign — always positive.",
    aliases: &["abs", "magnitude", "positive", "unsigned"],
    knobs: &[],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("abs({})", c.inner),
};

pub const QUANTIZE: Recipe = Recipe {
    id: "quantize",
    family: Family::Shape,
    label: "Quantize",
    blurb: "Snap the value to a grid.",
    aliases: &[
        "snap",
        "step",
        "round",
        "grid",
        "posterize value",
        "discrete",
    ],
    knobs: &[Knob::num("step", "Step", 0.05, (0.001, 10.0))],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("floor({}/{} + 0.5)*{}", c.tight(), c.nz(0), c.nz(0)),
};
