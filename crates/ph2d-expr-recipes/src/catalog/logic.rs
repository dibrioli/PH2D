//! **Logic** — conditionals over the value, another object, or the clock.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Family, Neutrality, Recipe, RowKind};

pub const IF_GREATER: Recipe = Recipe {
    id: "if-greater",
    family: Family::Logic,
    label: "If Greater",
    blurb: "Pick one of two values depending on whether the value is above a mark.",
    aliases: &[
        "if",
        "greater than",
        "above",
        "over",
        "condition",
        "compare",
    ],
    knobs: &[
        Knob::num("threshold", "Threshold", 0.5, (-40.0, 40.0)),
        Knob::num("then", "Then", 1.0, (-40.0, 40.0)),
        Knob::num("else", "Else", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("select({} > {}, {}, {})", c.tight(), c.n(0), c.n(1), c.n(2)),
};

pub const IF_LESS: Recipe = Recipe {
    id: "if-less",
    family: Family::Logic,
    label: "If Less",
    blurb: "Pick one of two values depending on whether the value is below a mark.",
    aliases: &["less than", "below", "under", "condition", "compare"],
    knobs: &[
        Knob::num("threshold", "Threshold", 0.5, (-40.0, 40.0)),
        Knob::num("then", "Then", 1.0, (-40.0, 40.0)),
        Knob::num("else", "Else", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("select({} < {}, {}, {})", c.tight(), c.n(0), c.n(1), c.n(2)),
};

pub const IF_EQUAL: Recipe = Recipe {
    id: "if-equal",
    family: Family::Logic,
    label: "If Equal",
    blurb: "Pick one of two values depending on whether the value matches a mark.",
    aliases: &["equals", "is", "match", "same as", "condition"],
    knobs: &[
        Knob::num("threshold", "Equals", 0.5, (-40.0, 40.0)),
        Knob::num("then", "Then", 1.0, (-40.0, 40.0)),
        Knob::num("else", "Else", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "select({} == {}, {}, {})",
            c.tight(),
            c.n(0),
            c.n(1),
            c.n(2)
        )
    },
};

pub const GATE_AND: Recipe = Recipe {
    id: "gate-and",
    family: Family::Logic,
    label: "Gate (Both)",
    blurb: "One value only while BOTH watched objects are past their marks.",
    aliases: &["and", "both", "all", "gate", "two conditions"],
    knobs: &[
        Knob::link("a", "Watch A"),
        Knob::num("a_min", "A Over", 0.5, (-40.0, 40.0)),
        Knob::link("b", "Watch B"),
        Knob::num("b_min", "B Over", 0.5, (-40.0, 40.0)),
        Knob::num("on", "On", 1.0, (-40.0, 40.0)),
        Knob::num("off", "Off", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "select(({} > {}) && ({} > {}), {}, {})",
            c.link(0),
            c.n(1),
            c.link(2),
            c.n(3),
            c.n(4),
            c.n(5)
        )
    },
};

pub const GATE_OR: Recipe = Recipe {
    id: "gate-or",
    family: Family::Logic,
    label: "Gate (Either)",
    blurb: "One value while EITHER watched object is past its mark.",
    aliases: &["or", "either", "any", "gate", "two conditions"],
    knobs: &[
        Knob::link("a", "Watch A"),
        Knob::num("a_min", "A Over", 0.5, (-40.0, 40.0)),
        Knob::link("b", "Watch B"),
        Knob::num("b_min", "B Over", 0.5, (-40.0, 40.0)),
        Knob::num("on", "On", 1.0, (-40.0, 40.0)),
        Knob::num("off", "Off", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "select(({} > {}) || ({} > {}), {}, {})",
            c.link(0),
            c.n(1),
            c.link(2),
            c.n(3),
            c.n(4),
            c.n(5)
        )
    },
};

pub const AFTER_TIME: Recipe = Recipe {
    id: "after-time",
    family: Family::Logic,
    label: "After Time",
    blurb: "Switch from one value to another at a moment.",
    aliases: &[
        "at time",
        "after",
        "cue",
        "when",
        "trigger time",
        "step time",
    ],
    knobs: &[
        Knob::num("at", "At (s)", 1.5, (0.0, 600.0)),
        Knob::num("before", "Before", 0.0, (-40.0, 40.0)),
        Knob::num("after", "After", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("select({} > {}, {}, {})", c.clock, c.n(0), c.n(2), c.n(1)),
};
