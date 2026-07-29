//! **Field** — gradients and proximity: a value that falls off with distance.
//!
//! The nearest thing this catalog has to Cavalry's Falloff, built out of the same
//! clamped remap the Shape family uses. It stays here rather than in Shape because
//! the artist's question is different: *"fade this out as it gets far away"* is a
//! spatial idea, not an arithmetic one.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Combine, Family, Neutrality, Recipe, RowKind};

pub const FADE_BY_DISTANCE: Recipe = Recipe {
    id: "fade-by-distance",
    family: Family::Field,
    label: "Fade by Distance",
    blurb: "Full value up close, fading out as two objects separate.",
    aliases: &[
        "falloff",
        "proximity fade",
        "near far",
        "distance opacity",
        "dim",
    ],
    knobs: &[
        Knob::link("a_x", "A · X"),
        Knob::link("a_y", "A · Y"),
        Knob::link("b_x", "B · X"),
        Knob::link("b_y", "B · Y"),
        Knob::num("near_at", "Near At", 0.5, (0.0, 40.0)),
        Knob::num("far_at", "Far At", 2.0, (0.0, 40.0)),
        Knob::num("near", "Near Value", 1.0, (-40.0, 40.0)),
        Knob::num("far", "Far Value", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        let (ax, ay, bx, by) = (c.link(0), c.link(1), c.link(2), c.link(3));
        let d = format!("sqrt(({ax}-{bx})*({ax}-{bx}) + ({ay}-{by})*({ay}-{by}))");
        format!(
            "mix({}, {}, min(max(({} - {})/({} - {}), 0), 1))",
            c.n(6),
            c.n(7),
            d,
            c.n(4),
            c.span_hi(4, 5),
            c.n(4)
        )
    },
};

pub const SCALE_BY_PROXIMITY: Recipe = Recipe {
    id: "scale-by-proximity",
    family: Family::Field,
    label: "Scale by Proximity",
    blurb: "Grow when close to another object, shrink when far.",
    aliases: &[
        "react to distance",
        "magnet scale",
        "closeness",
        "grow near",
    ],
    knobs: &[
        Knob::link("a", "A"),
        Knob::link("b", "B"),
        Knob::num("near_at", "Near At", 0.5, (0.0, 40.0)),
        Knob::num("far_at", "Far At", 2.0, (0.0, 40.0)),
        Knob::num("near", "Near Value", 2.0, (-10.0, 10.0)),
        Knob::num("far", "Far Value", 0.5, (-10.0, 10.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "mix({}, {}, min(max((abs({} - {}) - {})/({} - {}), 0), 1))",
            c.n(4),
            c.n(5),
            c.link(0),
            c.link(1),
            c.n(2),
            c.span_hi(2, 3),
            c.n(2)
        )
    },
};

pub const GRADIENT_BY_VALUE: Recipe = Recipe {
    id: "gradient-by-value",
    family: Family::Field,
    label: "Driven by Another",
    blurb: "Slide between two values as another object's property moves.",
    aliases: &["remap link", "driver", "controlled by", "map from", "slave"],
    knobs: &[
        Knob::link("source", "Driver"),
        Knob::num("in_lo", "From", 0.0, (-40.0, 40.0)),
        Knob::num("in_hi", "To", 1.0, (-40.0, 40.0)),
        Knob::num("out_lo", "Gives", 0.0, (-40.0, 40.0)),
        Knob::num("out_hi", "Up To", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "mix({}, {}, min(max(({} - {})/({} - {}), 0), 1))",
            c.n(3),
            c.n(4),
            c.link(0),
            c.n(1),
            c.span_hi(1, 2),
            c.n(1)
        )
    },
};
