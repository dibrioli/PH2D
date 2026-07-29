//! **Physics-lite** — motion that reads as physical without a solver.
//!
//! ⚠️ These are NOT the physics module. Nothing here collides, and the decay is
//! LINEAR because the language has no `exp` — a true spring settle, inertial
//! bounce and overshoot all wait on that one primitive, which lives in the FROZEN
//! `ph2d-expr` (ADR-0039) and costs an ADR. Six recipes unlock together the day it
//! lands; until then these four are honest about being approximations.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Family, Neutrality, Recipe, RowKind};

pub const PENDULUM: Recipe = Recipe {
    id: "pendulum",
    family: Family::PhysicsLite,
    label: "Pendulum",
    blurb: "Swings and gradually settles.",
    aliases: &["swing", "damped", "settle", "wobble down", "decay sine"],
    knobs: &[
        Knob::num("speed", "Speed", 4.0, (0.05, 40.0)),
        Knob::num("amount", "Amount", 0.5, (0.0, 40.0)),
        Knob::num("decay", "Decay", 0.3, (0.0, 5.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| {
        format!(
            "{} + sin({}*{})*max(0, 1 - {}*{})*{}",
            c.inner,
            c.clock,
            c.n(0),
            c.clock,
            c.n(2),
            c.n(1)
        )
    },
};

pub const FREE_FALL: Recipe = Recipe {
    id: "free-fall",
    family: Family::PhysicsLite,
    label: "Free Fall",
    blurb: "Accelerates downward like gravity.",
    aliases: &["gravity", "drop", "fall", "accelerate", "parabola"],
    knobs: &[Knob::num("gravity", "Gravity", 9.8, (0.0, 50.0))],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("gravity", 0.0)]),
    pair: None,
    emit: |c| format!("{} - 0.5*{}*{}*{}", c.inner, c.n(0), c.clock, c.clock),
};

pub const THROW: Recipe = Recipe {
    id: "throw",
    family: Family::PhysicsLite,
    label: "Throw",
    blurb: "Launches, arcs over, and falls.",
    aliases: &["toss", "launch", "arc", "projectile", "lob", "jump arc"],
    knobs: &[
        Knob::num("speed", "Launch Speed", 3.0, (-40.0, 40.0)),
        Knob::num("gravity", "Gravity", 9.8, (0.0, 50.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("speed", 0.0), ("gravity", 0.0)]),
    pair: None,
    emit: |c| {
        format!(
            "{} + {}*{} - 0.5*{}*{}*{}",
            c.inner,
            c.n(0),
            c.clock,
            c.n(1),
            c.clock,
            c.clock
        )
    },
};

pub const WAVE_ALONG_CHAIN: Recipe = Recipe {
    id: "wave-along-chain",
    family: Family::PhysicsLite,
    label: "Wave Along Chain",
    blurb: "Sways behind a leader, so a row of objects ripples.",
    aliases: &[
        "follow through",
        "overlap",
        "whip",
        "tail",
        "ripple",
        "cascade",
    ],
    knobs: &[
        Knob::link("lead", "Leader"),
        Knob::num("delay", "Delay", 0.01, (-1.0, 1.0)),
        Knob::num("speed", "Speed", 3.0, (0.05, 40.0)),
        Knob::num("amount", "Amount", 0.5, (0.0, 40.0)),
    ],
    kind: RowKind::Value,
    clock: ClockUse::Explicit,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| {
        format!(
            "{} + sin(({} - {}*{})*{})*{}",
            c.inner,
            c.clock,
            c.link(0),
            c.n(1),
            c.n(2),
            c.n(3)
        )
    },
};
