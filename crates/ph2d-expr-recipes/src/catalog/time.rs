//! **Time rows** — the family that rewrites the clock for the rows BELOW it.
//!
//! This is Cinema 4D's `Time` effector and Houdini's `shift`, and it costs nothing
//! extra because we own the emitted text: a Value row is handed the current clock
//! as [`crate::EmitCtx::clock`], so a Time row is just a row that changes that string.
//!
//! ⚠️ Every emit here **parenthesises** its result. The clock is substituted into
//! positions like `sin({clock}*3)`, so an unwrapped `-time` would become
//! `sin(-time*3)` — which happens to be the same, and `time - 0.2` would become
//! `sin(time - 0.2*3)`, which is NOT. One rule, applied to all seven.
//!
//! ⚠️ A Time row does not reach `Shake`/`Turbulence`: `wiggle` carries its own
//! clock inside the parser ([`ClockUse::Own`]). That is declared per-recipe so the
//! UI can say so rather than let the artist discover it.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Family, Neutrality, Recipe, RowKind};

pub const STEPPED_TIME: Recipe = Recipe {
    id: "stepped-time",
    family: Family::Time,
    label: "Stepped Time",
    blurb: "Hold each moment, for a stop-motion feel.",
    aliases: &[
        "posterize time",
        "stop motion",
        "on twos",
        "hold frames",
        "choppy",
    ],
    knobs: &[Knob::num("rate", "Steps / s", 6.0, (0.5, 60.0))],
    kind: RowKind::Time,
    combine: None,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("floor({}*{})/{}", c.clock, c.nz(0), c.nz(0)),
};

pub const DELAY: Recipe = Recipe {
    id: "delay",
    family: Family::Time,
    label: "Delay",
    blurb: "Run the rows below as if the clock were behind.",
    aliases: &["offset time", "lag", "shift", "wait", "later", "behind"],
    knobs: &[Knob::num("seconds", "Seconds", 0.2, (-10.0, 10.0))],
    kind: RowKind::Time,
    combine: None,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("({} - {})", c.clock, c.n(0)),
};

pub const SPEED: Recipe = Recipe {
    id: "speed",
    family: Family::Time,
    label: "Speed",
    blurb: "Run the rows below faster or slower.",
    aliases: &["time scale", "fast", "slow motion", "rate", "tempo"],
    knobs: &[Knob::num("factor", "Factor", 2.0, (-10.0, 10.0))],
    kind: RowKind::Time,
    combine: None,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("({}*{})", c.clock, c.n(0)),
};

pub const REVERSE_TIME: Recipe = Recipe {
    id: "reverse-time",
    family: Family::Time,
    label: "Reverse Time",
    blurb: "Run the rows below backwards.",
    aliases: &["backwards", "rewind", "negate time", "invert time"],
    knobs: &[],
    kind: RowKind::Time,
    combine: None,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("(-{})", c.clock),
};

pub const FREEZE_AFTER: Recipe = Recipe {
    id: "freeze-after",
    family: Family::Time,
    label: "Freeze After",
    blurb: "Let the rows below stop at a moment and hold there.",
    aliases: &["hold", "stop", "freeze frame", "end at", "clamp time"],
    knobs: &[Knob::num("at", "At (s)", 2.0, (0.0, 600.0))],
    kind: RowKind::Time,
    combine: None,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("min({}, {})", c.clock, c.n(0)),
};

pub const START_AT: Recipe = Recipe {
    id: "start-at",
    family: Family::Time,
    label: "Start At",
    blurb: "Keep the rows below at their opening pose until this moment.",
    aliases: &["begin at", "wait until", "hold start", "clamp time low"],
    knobs: &[Knob::num("at", "At (s)", 1.0, (0.0, 600.0))],
    kind: RowKind::Time,
    combine: None,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("max({}, {})", c.clock, c.n(0)),
};

pub const PING_PONG_TIME: Recipe = Recipe {
    id: "ping-pong-time",
    family: Family::Time,
    label: "Ping-Pong Time",
    blurb: "Play the rows below forwards, then backwards, forever.",
    aliases: &["boomerang", "yoyo time", "loop mirror time", "shuttle time"],
    knobs: &[Knob::num("rate", "Rate", 0.5, (0.01, 20.0))],
    kind: RowKind::Time,
    combine: None,
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("(abs(fract({}*{})*2 - 1)/{})", c.clock, c.nz(0), c.nz(0)),
};
