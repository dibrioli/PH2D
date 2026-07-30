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

/// **Shake — e o Turbulence mora DENTRO dele** (FASE A, plano 12).
///
/// ⚠️ `turbulence ~> shake` com delta **0,000000**: turbulence é shake com octaves, e
/// `Detail`/`Roughness` eram knobs de IMPLEMENTAÇÃO num card próprio. Absorvidos aqui, e o
/// default do Shake **não se moveu**: o contrato do parser diz que `octaves = 1` é
/// byte-idêntico ao lowering de uma oitava, então `wiggle(s, a, 1, 0.5)` é o
/// `wiggle(s, a)` de antes ao bit (gate `absorbing_turbulence_left_shakes_default_alone`).
pub const SHAKE: Recipe = Recipe {
    id: "shake",
    family: Family::Life,
    label: "Shake",
    blurb: "Random wobble around the current value. Detail layers finer wobble on top.",
    aliases: &[
        "wiggle",
        "jitter",
        "noise",
        "camera shake",
        "handheld",
        // Herdados do `turbulence` APOSENTADO (ver `retired.rs`): cortar sem herdar é
        // esconder capacidade, e a auditoria mediu os RÓTULOS sendo esquecidos.
        "turbulence",
        "wiggle octaves",
        "fractal noise",
        "rough shake",
        "turbulent",
    ],
    knobs: &[
        Knob::num("speed", "Speed", 2.0, (0.05, 20.0)),
        Knob::num("amount", "Amount", 0.3, (0.0, 40.0)),
        // ⚠️ Literal por contrato: o `wiggle` desenrola um `noise` por oitava em tempo de
        // PARSE, então estes dois dimensionam a árvore da expressão e não podem ser
        // dirigidos. Default 1 / 0.5 = os do parser (os do AE).
        Knob::lit("detail", "Detail", 1.0, (1.0, 4.0)).step(1.0),
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

/// **A fixed random offset that differs PER OBJECT.**
///
/// ⚠️ Red-first against a report (*"Jitter não funciona"*), and the measurement was
/// brutal: at its defaults it emitted `value + noise(7)*0.2` — a **CONSTANT** `+0.197`,
/// excursion `0.0000` forever. It did not animate, it drew a flat curve in the preview
/// strip, and worst of all it was a WORSE version of typing an Offset: the number it
/// produced was unpredictable and the same on every object, so the one thing its own
/// blurb promised — *"stagger"* — was the one thing it could not do.
///
/// The fix is the `__seed` the evaluator already binds. It is derived from the BINDING
/// (`SEED_SPACING` apart per target), so it is a different number for every object and
/// every property, and `noise(__seed + seed)` is what *"a random offset per object,
/// reroll with Seed"* actually means. It is the same channel `wiggle` uses internally,
/// so this is not a new capability — it is the one Jitter was written to have.
///
/// ⚠️ Still constant IN TIME, and that is the whole point: this is the one recipe in
/// `Life` that must NOT move. A group of objects each nudged a different fixed amount is
/// what an animator reaches for before anything is animated at all.
pub const JITTER: Recipe = Recipe {
    id: "jitter",
    family: Family::Life,
    label: "Jitter",
    blurb: "A fixed random offset, different on every object. Seed rerolls it.",
    aliases: &[
        "random offset",
        "seed",
        "vary",
        "randomize",
        "stagger",
        "per object",
    ],
    knobs: &[
        Knob::num("seed", "Seed", 7.0, (0.0, 1000.0)),
        Knob::num("amount", "Amount", 0.2, (0.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Add),
    clock: ClockUse::None,
    neutral: Neutrality::Additive(&[("amount", 0.0)]),
    pair: None,
    emit: |c| format!("noise(__seed + {})*{}", c.n(0), c.n(1)),
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
