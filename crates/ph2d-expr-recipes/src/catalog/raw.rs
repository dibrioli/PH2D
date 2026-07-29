//! **Raw** — the escape hatch, and it is an ITEM OF THE CATALOG.
//!
//! Cinema 4D (`Formula`), Cavalry (`JavaScript Deformer`) and Rive (`Formula`) all
//! put code in the catalog rather than behind a mode. Hiding it is what makes
//! people believe the tool cannot do the thing; listing it makes the ceiling
//! visible and gives the experienced artist a row instead of a fight.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Family, Neutrality, Recipe, RowKind};

pub const CUSTOM: Recipe = Recipe {
    id: "custom",
    family: Family::Raw,
    label: "Custom Formula",
    blurb: "Write it yourself. `value`, `time`, `Name.x` and the maths functions.",
    aliases: &[
        "formula",
        "expression",
        "code",
        "script",
        "manual",
        "raw",
        "js",
    ],
    knobs: &[Knob::text("formula", "Formula")],
    kind: RowKind::Raw,
    // ⚠️ `None`, and it is a lie we accept: the artist's text may well read the
    // clock. Declaring `Explicit` would promise that a Time row above reaches it,
    // and it does not — we substitute the clock into OUR emits, never into text we
    // did not write. Saying "no clock" understates; saying "explicit" would be
    // wrong in the direction that surprises.
    clock: ClockUse::None,
    // An EMPTY formula passes the value through unchanged, which is the identity —
    // but that is the unfinished state, not a tuning, so it is not a neutral.
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        let t = c.text(0);
        if t.is_empty() {
            c.inner.to_string()
        } else {
            t.to_string()
        }
    },
};
