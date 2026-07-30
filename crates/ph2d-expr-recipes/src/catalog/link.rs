//! **Link** — one object reading another. The pick-whip family.
//!
//! ⚠️ Every recipe here is [`Neutrality::NoNeutral`] and ignores `inner`: a link
//! PRODUCES the value from somewhere else rather than modifying what was there.
//! That is not a defect — `Follow` means *be where that is*, and a "neutral
//! follow" would be a follow that does not follow.
//!
//! ⚠️ An UNFILLED link emits `0` (see [`crate::EmitCtx::link`]). The formula bar and the
//! preview render on every keystroke, long before the pick-whip has been used, so
//! a row that emitted `.x` would blank the modal the moment it was added.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Combine, Family, Neutrality, Recipe, RowKind};

pub const FOLLOW: Recipe = Recipe {
    id: "follow",
    family: Family::Link,
    label: "Follow",
    blurb: "Copy another object's property, scaled and offset. Multiply -1 mirrors it.",
    aliases: &[
        "link",
        "pick whip",
        "parent",
        "copy",
        "track",
        "bind",
        // Herdados do `offset-copy` APOSENTADO — o Follow já tinha os DOIS knobs
        // (Multiply e Offset), então ele contém aquele card no espaço inteiro.
        "offset copy",
        "shadow",
        "trail",
        "follow at distance",
        "clone offset",
        // ...e do `opposite`: `2*pivot - link` é Follow com Multiply −1 e Offset 2*pivot
        // (delta medido 0,000000). O blurb acima diz isso, porque um apelido que acha o
        // card e não explica como chegar lá é meio caminho.
        "opposite",
        "reflect about",
        "symmetry",
        "mirror pivot",
        "counterweight",
        "negate link",
        "flip",
        "invert follow",
        "reflect",
        "mirror",
    ],
    knobs: &[
        Knob::link("target", "Target"),
        Knob::num("multiply", "Multiply", 1.0, (-10.0, 10.0)),
        Knob::num("offset", "Offset", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("{}*{} + {}", c.link(0), c.n(1), c.n(2)),
};

pub const DISTANCE_2D: Recipe = Recipe {
    id: "distance-2d",
    family: Family::Link,
    label: "Distance",
    blurb: "How far apart two objects are.",
    aliases: &[
        "length",
        "gap",
        "proximity",
        "range",
        "how far",
        "magnitude",
    ],
    knobs: &[
        Knob::link("a_x", "A · X"),
        Knob::link("a_y", "A · Y"),
        Knob::link("b_x", "B · X"),
        Knob::link("b_y", "B · Y"),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        let (ax, ay, bx, by) = (c.link(0), c.link(1), c.link(2), c.link(3));
        format!("sqrt(({ax}-{bx})*({ax}-{bx}) + ({ay}-{by})*({ay}-{by}))")
    },
};

pub const DISTANCE_1D: Recipe = Recipe {
    id: "distance-1d",
    family: Family::Link,
    label: "Distance (1 Axis)",
    blurb: "How far apart two properties are, on one axis.",
    aliases: &["difference", "abs difference", "gap axis", "separation"],
    knobs: &[Knob::link("a", "A"), Knob::link("b", "B")],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("abs({} - {})", c.link(0), c.link(1)),
};

pub const BLEND_TWO: Recipe = Recipe {
    id: "blend-two",
    family: Family::Link,
    label: "Blend Two",
    blurb: "Slide between two objects' properties.",
    aliases: &[
        "mix",
        "lerp",
        "interpolate",
        "weighted",
        "crossfade",
        // Inherited from the retired `Midpoint`, which was EXACTLY this at Blend 0.5.
        "midpoint",
        "average",
        "centre",
        "mean",
        "between",
        "average two",
    ],
    knobs: &[
        Knob::link("a", "A"),
        Knob::link("b", "B"),
        Knob::num("blend", "Blend", 0.5, (0.0, 1.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("mix({}, {}, {})", c.link(0), c.link(1), c.n(2)),
};
