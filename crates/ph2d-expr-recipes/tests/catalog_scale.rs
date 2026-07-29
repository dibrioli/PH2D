//! **The gates for the catalog's SCALE** (plano 10 §8, the smoke of 2026-07-29).
//!
//! Split from `catalog.rs` by subject when that file crossed the 700-LOC cap, and
//! the line of the cut is the meaning: that file asks *does the catalog emit text
//! the one parser accepts and compose correctly?*, this one asks *are its numbers
//! the right SIZE for a canvas?* — a question the first smoke of the modal put on
//! the table and nothing else in the suite was asking.

use ph2d_expr::{Bindings, eval};
use ph2d_expr_parse::parse;
use ph2d_expr_recipes::{CATALOG, Family, KnobKind, KnobValue, RecipeStack, RowKind};

mod shared;
use shared::{Knobs, row_with, set_knobs};

// ------------------------------------------------------------- the scale

/// The window a default is JUDGED over.
///
/// ⚠️ Not an arbitrary number and not the modal's — it is *how long the artist
/// looks before deciding the default is wrong*, and it is the same two seconds the
/// preview loops over for the same reason. Judge over ten and `Free Fall` fails for
/// being gravity; judge over a tenth and nothing fails at all.
const JUDGE_SECONDS: f32 = 2.0;

/// Samples across that window.
const JUDGE_SAMPLES: usize = 240;

/// The whole curve a formula draws across the judging window.
///
/// `value` and any link are non-zero and different from each other: at `value = 0`
/// every MULTIPLICATIVE recipe (`Flicker`, `Multiply / Add`) is flat zero whatever
/// its knobs say, and a gate reading that would call them broken.
fn trajectory(src: &str, value: f32) -> Option<Vec<f32>> {
    struct Clock(f32, f32);
    impl Bindings for Clock {
        fn attr(&self, name: &str) -> f32 {
            match name {
                "time" => self.0,
                "value" => self.1,
                // Any `Name.prop` a link knob points at — DIFFERENT per name, so a
                // recipe that reads two links (`Blend Two`, `Distance`) is not
                // handed the same number twice and quietly collapsed.
                other => 0.3 + other.len() as f32 * 0.11,
            }
        }
        fn param(&self, _: &str) -> f32 {
            0.0
        }
    }
    let e = parse(src).ok()?;
    Some(
        (0..JUDGE_SAMPLES)
            .map(|i| {
                let t = i as f32 / JUDGE_SAMPLES as f32 * JUDGE_SECONDS;
                let v = eval(&e, &Clock(t, value));
                if v.is_finite() { v } else { 0.0 }
            })
            .collect(),
    )
}

/// How far a formula takes a property that is resting at zero.
fn peak_excursion(src: &str) -> Option<f32> {
    Some(
        trajectory(src, 0.0)?
            .into_iter()
            .fold(0.0_f32, |p, v| p.max(v.abs())),
    )
}

/// **No recipe flings the object off a 4K canvas at its own defaults.**
///
/// ⚠️ Born RED, and this is the gate for the first smoke of the modal: *"Shake —
/// changing the parameters did not change the animation"*. The formula responded
/// (there is a gate for that below); the SCREEN did not, because `amount: 30` meant
/// **30 metres = 3000 px** at the project's 100 px/m and the object had left the
/// frame — so every knob value looked identical, which is to say looked like
/// nothing. Measured on the pre-change catalog: **14 of 47 value recipes** did this,
/// the worst (`Free Fall`) by **777 480 px**.
///
/// The bar is HALF a canvas because the object starts at the centre: a default that
/// reaches the edge is already at the limit of useful, and one that passes it is
/// invisible. Mutation: put `amount` back to `30.0` in `life.rs` ⇒ RED, naming Shake
/// and its number.
#[test]
fn no_recipe_flings_the_object_off_a_4k_canvas() {
    let half_canvas = ph2d_expr_recipes::CANVAS_M * 0.5;
    let mut off = Vec::new();
    for r in CATALOG {
        if r.kind != RowKind::Value {
            continue;
        }
        let mut stack = RecipeStack::new();
        stack.push(row_with(r.id, Knobs::Default));
        let src = stack.to_formula();
        let Some(peak) = peak_excursion(&src) else {
            continue;
        };
        if peak > half_canvas {
            off.push(format!(
                "{} reaches {peak:.2} m ({:.0} px)",
                r.id,
                peak * 100.0
            ));
        }
    }
    assert!(
        off.is_empty(),
        "these defaults put the object off a 4K canvas ({half_canvas} m from centre):\n  {}",
        off.join("\n  ")
    );
}

/// **Turning a knob turns what the formula PRODUCES** — the half of the smoke
/// report that was never broken, pinned so nobody "fixes" it.
///
/// ⚠️ The oracle is the produced CURVE, not the emitted text and not its peak: a
/// recipe that interpolated its knob into a comment would change the string and
/// animate identically, and a peak is blind to frequency — `Sway` at 2 Hz and at
/// 30 Hz reach the same height, so the first version of this gate called eleven
/// working knobs dead.
///
/// ⚠️ **Scoped to the three MOVING families, and the narrowing is honest rather
/// than convenient.** Logic, Link, Field and Shape SELECT between values, and no
/// single fixture can make both sides of a branch live at once: with the condition
/// false the `Then` knob is genuinely unreachable, and a gate that called that
/// "inert" would need a per-recipe table of conditions to keep in sync with the
/// emitters — the drift-prone shape this catalog exists to avoid. Wave, Life and
/// PhysicsLite are exactly where the smoke report came from, and there every number
/// knob is continuous in the output, so the sweep can be total.
#[test]
fn turning_a_knob_turns_what_the_formula_produces() {
    let mut inert = Vec::new();
    for r in CATALOG {
        if r.kind != RowKind::Value
            || !matches!(r.family, Family::Wave | Family::Life | Family::PhysicsLite)
        {
            continue;
        }
        for (i, k) in r.knobs.iter().enumerate() {
            if k.kind != KnobKind::Number {
                continue;
            }
            let sample = |v: f32| {
                // Links FILLED: a knob that only matters once the pick-whip has
                // been used (`Switch`'s Above, `Gate`'s thresholds) is live code,
                // and an empty link would report it dead.
                let mut row = row_with(r.id, Knobs::Perturbed);
                set_knobs(&mut row, Knobs::Default);
                for (j, kk) in r.knobs.iter().enumerate() {
                    if kk.kind == KnobKind::Link {
                        row.knobs[j] = KnobValue::Link("Ball.x".into());
                    }
                }
                row.knobs[i] = KnobValue::Num(v);
                let mut stack = RecipeStack::new();
                stack.push(row);
                trajectory(&stack.to_formula(), 0.7)
            };
            let (lo, hi) = k.range;
            // Two thirds apart inside the range, so neither probe is the default
            // (a knob compared against itself never moves).
            let (a, b) = (lo + (hi - lo) / 6.0, lo + (hi - lo) * 5.0 / 6.0);
            if let (Some(x), Some(y)) = (sample(a), sample(b))
                && x.iter().zip(&y).all(|(p, q)| (p - q).abs() < f32::EPSILON)
            {
                inert.push(format!("{}.{}", r.id, k.key));
            }
        }
    }
    assert!(
        inert.is_empty(),
        "these knobs do not reach the arithmetic: {inert:?}"
    );
}

/// **Every knob steps in a number a person would type.**
///
/// The number box's arrows and its drag both read the registered step; without one
/// the dispatch falls back to a buffer heuristic (`1.0` for a value with no dot),
/// and one click on an Amount whose default is `0.3` moves three canvases. Mutation:
/// return the raw `span / 200` without snapping ⇒ steps like `0.3987` appear, RED.
#[test]
fn every_knob_steps_in_a_number_a_person_would_type() {
    const NICE: [f32; 9] = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0];
    for r in CATALOG {
        for k in r.knobs {
            if !matches!(k.kind, KnobKind::Number | KnobKind::Literal) {
                continue;
            }
            let s = k.step_value();
            assert!(
                NICE.contains(&s),
                "{}.{} steps by {s}, which is not a number anyone types",
                r.id,
                k.key
            );
            let span = (k.range.1 - k.range.0).abs();
            assert!(
                s <= span || span == 0.0,
                "{}.{} steps past its own range in one click",
                r.id,
                k.key
            );
        }
    }
    // The one override, named: octaves size an unrolled noise tree, so they are
    // integers and a derived 0.01 would be a control that does nothing four times
    // out of five.
    let detail = ph2d_expr_recipes::by_id("turbulence")
        .unwrap()
        .knobs
        .iter()
        .find(|k| k.key == "detail")
        .unwrap();
    assert!(
        (detail.step_value() - 1.0).abs() < f32::EPSILON,
        "wiggle octaves step by whole numbers"
    );
}
