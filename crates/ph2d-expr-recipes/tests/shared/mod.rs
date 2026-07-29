//! **The fixture both catalog gate files drive** — one answer to *"what does a row
//! look like for a probe?"*, shared rather than copied.
//!
//! ⚠️ A directory module, not a `tests/*.rs`: cargo compiles every file directly
//! under `tests/` as its own binary, so a flat `catalog_fixture.rs` would ship a
//! third test target containing no tests.

#![allow(dead_code)]

use ph2d_expr::Bindings;
use ph2d_expr_recipes::{KnobKind, KnobValue, Row};

/// The bindings a gate evaluates under. `time` is deliberately NOT a round number
/// — a clock of 0 makes `sin`, `fract` and `floor` agree with each other by
/// accident, and half these recipes would compose correctly by coincidence.
pub struct B(pub f32);
impl Bindings for B {
    fn attr(&self, name: &str) -> f32 {
        match name {
            "value" => self.0,
            "time" => 0.37,
            _ => 0.0,
        }
    }
    fn param(&self, _: &str) -> f32 {
        0.0
    }
}

/// How a row's knobs are set for a given probe.
#[derive(Clone, Copy)]
pub enum Knobs {
    Default,
    /// Both ends of every slider.
    End(bool),
    /// **Every number typed as zero.**
    ///
    /// ⚠️ Not a slider position — a knob range is a DRAG range, not a validity
    /// claim, and the artist can select the field and press Delete. This is the
    /// probe that reaches the denominators (`Quantize` step, `Stepped Time` rate,
    /// a zero-width `Remap`) and the parser's own floor on `wiggle` octaves. The
    /// first version of this gate only walked the range ends, where `step` bottoms
    /// out at 0.001 — so it passed with the divide-by-zero guards deleted.
    Zero,
    /// Away from the defaults, so a recipe that is the identity at its defaults
    /// (`Multiply / Add`) actually does something.
    Perturbed,
}

pub fn set_knobs(row: &mut Row, how: Knobs) {
    let r = ph2d_expr_recipes::by_id(row.recipe).unwrap();
    for (i, k) in r.knobs.iter().enumerate() {
        let (lo, hi) = k.range;
        row.knobs[i] = match (k.kind, how) {
            (_, Knobs::Default) => k.default_value(),
            (KnobKind::Number | KnobKind::Literal, Knobs::End(high)) => {
                KnobValue::Num(if high { hi } else { lo })
            }
            (KnobKind::Number | KnobKind::Literal, Knobs::Zero) => KnobValue::Num(0.0),
            (KnobKind::Number | KnobKind::Literal, Knobs::Perturbed) => {
                KnobValue::Num(lo + 0.6 * (hi - lo))
            }
            (KnobKind::Link, Knobs::End(false) | Knobs::Zero) => KnobValue::Link(String::new()),
            (KnobKind::Link, _) => KnobValue::Link("Ball.x".into()),
            (KnobKind::Text, Knobs::End(false) | Knobs::Zero) => KnobValue::Text(String::new()),
            (KnobKind::Text, _) => KnobValue::Text("value*2".into()),
        };
    }
}

pub fn row_with(id: &'static str, how: Knobs) -> Row {
    let mut row = Row::new(id).unwrap();
    set_knobs(&mut row, how);
    row
}
