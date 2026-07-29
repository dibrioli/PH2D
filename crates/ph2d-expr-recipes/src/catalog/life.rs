//! **Life** — the organic imperfection that makes a still object look alive.
//!
//! ⚠️ Two clock families live here on purpose. `Shake`/`Turbulence` are the
//! `wiggle` sugar, which builds `time + __seed` **inside the parser** — no clock we
//! choose reaches them, so they declare [`ClockUse::Own`] and a `Stepped Time` row
//! above them is honestly reported as not applying. `Drift` is the same idea spelt
//! over the current clock, so it IS steppable — which is why both ship instead of one.
//!
//! ⚠️ **Two NOISES live here too, and the difference is the whole family.**
//! `smoothnoise(x)` is a value noise — one random number per unit of `x`,
//! interpolated — so multiplying `x` by a Speed genuinely makes it wobble faster.
//! `noise(x)` is the raw HASH, where adjacent inputs are uncorrelated: sampled over
//! a clock it is white noise whose character does not respond to Speed at all
//! (measured: 494-509 crossings/s across a 32x sweep). `Jitter` is the ONE recipe
//! that wants the hash — it asks for a single fixed random offset per Seed, never a
//! wobble — and it is the reason `noise` stays in the grammar.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Combine, Family, Neutrality, Recipe, RowKind};

pub const SHAKE: Recipe = Recipe {
    id: "shake",
    family: Family::Life,
    label: "Shake",
    blurb: "Random wobble around the current value.",
    aliases: &["wiggle", "jitter", "noise", "camera shake", "handheld"],
    knobs: &[
        Knob::num("speed", "Speed", 2.0, (0.05, 20.0)),
        Knob::num("amount", "Amount", 0.3, (0.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Add),
    clock: ClockUse::Own,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| format!("wiggle({}, {})", c.n(0), c.n(1)),
};

pub const TURBULENCE: Recipe = Recipe {
    id: "turbulence",
    family: Family::Life,
    label: "Turbulence",
    blurb: "Shake with fine detail layered on top.",
    aliases: &[
        "wiggle octaves",
        "fractal noise",
        "rough shake",
        "turbulent",
    ],
    knobs: &[
        Knob::num("speed", "Speed", 2.0, (0.05, 20.0)),
        Knob::num("amount", "Amount", 0.3, (0.0, 40.0)),
        // ⚠️ Literal by contract: `wiggle` unrolls one `noise` per octave at PARSE
        // time, so these two size the expression tree and cannot be driven.
        Knob::lit("detail", "Detail", 3.0, (1.0, 4.0)).step(1.0),
        Knob::lit("roughness", "Roughness", 0.5, (0.0, 1.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Add),
    clock: ClockUse::Own,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| format!("wiggle({}, {}, {}, {})", c.n(0), c.n(1), c.lit(2), c.lit(3)),
};

pub const DRIFT: Recipe = Recipe {
    id: "drift",
    family: Family::Life,
    label: "Drift",
    blurb: "Slow smooth wander. Follows a Time row, unlike Shake.",
    aliases: &["smooth noise", "wander", "float", "perlin", "sway random"],
    knobs: &[
        Knob::num("speed", "Speed", 0.5, (0.01, 10.0)),
        Knob::num("amount", "Amount", 0.2, (0.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Add),
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| format!("smoothnoise({}*{})*{}", c.clock, c.n(0), c.n(1)),
};

pub const JITTER: Recipe = Recipe {
    id: "jitter",
    family: Family::Life,
    label: "Jitter",
    blurb: "A fixed random offset. Same every frame; change Seed to reroll.",
    aliases: &["random offset", "seed", "vary", "randomize", "stagger"],
    knobs: &[
        Knob::num("seed", "Seed", 7.0, (0.0, 1000.0)),
        Knob::num("amount", "Amount", 0.2, (0.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Add),
    clock: ClockUse::None,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| format!("noise({})*{}", c.n(0), c.n(1)),
};

pub const BREATHE: Recipe = Recipe {
    id: "breathe",
    family: Family::Life,
    label: "Breathe",
    blurb: "A gentle rise and fall that never goes below the value.",
    aliases: &["pulse soft", "idle", "inhale", "swell", "one sided sine"],
    knobs: &[
        Knob::num("speed", "Speed", 2.0, (0.05, 20.0)),
        Knob::num("amount", "Amount", 0.15, (0.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Add),
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| format!("(sin({}*{})*0.5 + 0.5)*{}", c.clock, c.n(0), c.n(1)),
};

pub const FLICKER: Recipe = Recipe {
    id: "flicker",
    family: Family::Life,
    label: "Flicker",
    blurb: "Multiplies the value by a wandering brightness. Good for opacity.",
    aliases: &["flame", "candle", "blink soft", "fluctuate", "lamp"],
    knobs: &[
        Knob::num("speed", "Speed", 8.0, (0.1, 40.0)),
        // Neutral is 1.0: `mix(1, 1, x)` is exactly 1, so the row multiplies by one.
        Knob::num("min", "Min", 0.3, (0.0, 1.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Multiply),
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("min", 1.0)]),
    pair: None,
    emit: |c| {
        format!(
            "mix({}, 1, smoothnoise({}*{})*0.5 + 0.5)",
            c.n(1),
            c.clock,
            c.n(0)
        )
    },
};
