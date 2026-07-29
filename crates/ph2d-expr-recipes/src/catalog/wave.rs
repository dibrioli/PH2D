//! **Wave** — rhythm. Everything here reads the clock, so a Time row above any of
//! them applies.
//!
//! ⚠️ `Orbit` is the catalog's first PAIR: it writes two properties (X and Y) and
//! half a circle is not a feature, so the two entries name each other through
//! [`Recipe::pair`] and the gallery offers them as one card.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Family, Neutrality, Recipe, RowKind};

pub const SWAY: Recipe = Recipe {
    id: "sway",
    family: Family::Wave,
    label: "Sway",
    blurb: "Smooth back-and-forth.",
    aliases: &[
        "sine",
        "sin",
        "oscillate",
        "wave",
        "rock",
        "pendulum simple",
    ],
    knobs: &[
        Knob::num("speed", "Speed", 3.0, (0.05, 40.0)),
        Knob::num("amount", "Amount", 0.5, (0.0, 40.0)),
        Knob::num("phase", "Phase", 0.0, (-10.0, 10.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| {
        format!(
            "{} + sin(({} + {})*{})*{}",
            c.inner,
            c.clock,
            c.n(2),
            c.n(0),
            c.n(1)
        )
    },
};

pub const SWAY_COSINE: Recipe = Recipe {
    id: "sway-cosine",
    family: Family::Wave,
    label: "Sway (Cosine)",
    blurb: "Sway starting at the top instead of the middle.",
    aliases: &["cos", "cosine", "quarter turn", "offset sine"],
    knobs: &[
        Knob::num("speed", "Speed", 3.0, (0.05, 40.0)),
        Knob::num("amount", "Amount", 0.5, (0.0, 40.0)),
        Knob::num("phase", "Phase", 0.0, (-10.0, 10.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| {
        format!(
            "{} + cos(({} + {})*{})*{}",
            c.inner,
            c.clock,
            c.n(2),
            c.n(0),
            c.n(1)
        )
    },
};

pub const BOUNCE: Recipe = Recipe {
    id: "bounce",
    family: Family::Wave,
    label: "Bounce",
    blurb: "Repeated hops that always come back to the value.",
    aliases: &["hop", "ball", "abs sine", "jump"],
    knobs: &[
        Knob::num("speed", "Speed", 3.0, (0.05, 40.0)),
        Knob::num("height", "Height", 0.5, (0.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("height", 0.0)]),
    pair: None,
    emit: |c| format!("{} + abs(sin({}*{}))*{}", c.inner, c.clock, c.n(0), c.n(1)),
};

pub const PING_PONG: Recipe = Recipe {
    id: "ping-pong",
    family: Family::Wave,
    label: "Ping-Pong",
    blurb: "Travels from Low to High and back, forever.",
    aliases: &[
        "triangle",
        "back and forth",
        "yoyo",
        "loop mirror",
        "shuttle",
    ],
    knobs: &[
        Knob::num("rate", "Rate", 0.5, (0.01, 20.0)),
        Knob::num("low", "Low", 0.0, (-40.0, 40.0)),
        Knob::num("high", "High", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "mix({}, {}, abs(fract({}*{})*2 - 1))",
            c.n(1),
            c.n(2),
            c.clock,
            c.n(0)
        )
    },
};

pub const RAMP_LOOP: Recipe = Recipe {
    id: "ramp-loop",
    family: Family::Wave,
    label: "Ramp Loop",
    blurb: "Climbs from Low to High, then snaps back and climbs again.",
    aliases: &["sawtooth", "repeat", "cycle", "scroll", "conveyor"],
    knobs: &[
        Knob::num("rate", "Rate", 0.5, (0.01, 20.0)),
        Knob::num("low", "Low", 0.0, (-40.0, 40.0)),
        Knob::num("high", "High", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("mix({}, {}, fract({}*{}))", c.n(1), c.n(2), c.clock, c.n(0)),
};

pub const BLINK: Recipe = Recipe {
    id: "blink",
    family: Family::Wave,
    label: "Blink",
    blurb: "Snaps between two values on a beat.",
    aliases: &["strobe", "flash", "square wave", "on off", "toggle"],
    knobs: &[
        Knob::num("rate", "Rate", 4.0, (0.05, 40.0)),
        Knob::num("duty", "Duty", 0.5, (0.0, 1.0)),
        Knob::num("on", "On", 1.0, (-40.0, 40.0)),
        Knob::num("off", "Off", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "select(fract({}*{}) < {}, {}, {})",
            c.clock,
            c.n(0),
            c.n(1),
            c.n(2),
            c.n(3)
        )
    },
};

pub const PULSE: Recipe = Recipe {
    id: "pulse",
    family: Family::Wave,
    label: "Pulse",
    blurb: "A hit on every beat that falls away before the next one.",
    aliases: &["heartbeat", "throb", "beat", "decay loop", "tick"],
    knobs: &[
        Knob::num("rate", "Rate", 2.0, (0.05, 40.0)),
        Knob::num("decay", "Decay", 4.0, (0.1, 20.0)),
        Knob::num("on", "On", 1.0, (-40.0, 40.0)),
        Knob::num("off", "Off", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "mix({}, {}, max(0, 1 - fract({}*{})*{}))",
            c.n(3),
            c.n(2),
            c.clock,
            c.n(0),
            c.n(1)
        )
    },
};

pub const ORBIT_X: Recipe = Recipe {
    id: "orbit-x",
    family: Family::Wave,
    label: "Orbit (X)",
    blurb: "Circles a centre. Pair with Orbit (Y) on the other axis.",
    aliases: &["circle", "revolve", "spin around", "rotate about"],
    knobs: &[
        Knob::num("centre", "Centre", 0.0, (-40.0, 40.0)),
        Knob::num("radius", "Radius", 1.0, (0.0, 40.0)),
        Knob::num("speed", "Speed", 2.0, (0.05, 20.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: Some("orbit-y"),
    emit: |c| format!("{} + cos({}*{})*{}", c.n(0), c.clock, c.n(2), c.n(1)),
};

pub const ORBIT_Y: Recipe = Recipe {
    id: "orbit-y",
    family: Family::Wave,
    label: "Orbit (Y)",
    blurb: "Circles a centre. Pair with Orbit (X) on the other axis.",
    aliases: &["circle", "revolve", "spin around", "rotate about"],
    knobs: &[
        Knob::num("centre", "Centre", 0.0, (-40.0, 40.0)),
        Knob::num("radius", "Radius", 1.0, (0.0, 40.0)),
        Knob::num("speed", "Speed", 2.0, (0.05, 20.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: Some("orbit-x"),
    emit: |c| format!("{} + sin({}*{})*{}", c.n(0), c.clock, c.n(2), c.n(1)),
};
