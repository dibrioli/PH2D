//! **A row says HOW it lands, and the catalog did not move when it learned to.**
//!
//! ⚠️ Red-first against a report: *"Expressões não podem ser somadas, multiplicadas,
//! etc."* Measured before the fix, **29 of 55 recipes ignored `EmitCtx::inner`** —
//! every `Link`, every `Field`, `Blink`, `Pulse`, `Ping-Pong`, `Orbit` — so a sheet of
//! `Sway` then `Blink` projected to `select(fract(time*4) < 0.5, 1, 0)` and the Sway
//! was **discarded without a word**. The other 26 composed, but with the operator
//! **baked into their own text** (`Sway` spelt `{inner} + sin(...)`), so a product was
//! not expressible at all.
//!
//! The four things these gates hold down, in the order that matters:
//!
//! 1. **the catalog did not move** — every recipe at its defaults is value-identical
//!    to the pre-`Combine` world, and byte-identical but for one named exception;
//! 2. **a source composes** — stacking two of them keeps both, under each mode;
//! 3. **the split is honest** — a source never reads `inner`, a modifier always does;
//! 4. **the parenthesisation survives a stack** — a product of a sum is the product
//!    of the whole sum.

#[path = "shared/mod.rs"]
mod shared;

use ph2d_expr::{Bindings, eval};
use ph2d_expr_parse::parse;
use ph2d_expr_recipes::{
    CATALOG, Combine, EmitCtx, KnobKind, KnobValue, RecipeStack, Row, RowKind, by_id,
};

/// A clock and a value that make nothing agree by accident: `time = 0` collapses
/// `sin`, `fract` and `floor` onto each other, and `value = 0` hides every product.
struct B {
    t: f32,
    v: f32,
}
impl Bindings for B {
    fn attr(&self, n: &str) -> f32 {
        match n {
            "time" => self.t,
            "value" => self.v,
            "Ball.x" => 3.0,
            _ => 0.0,
        }
    }
    fn param(&self, _: &str) -> f32 {
        0.0
    }
}

/// Two formulas agree everywhere a knob can put them. Both must parse.
fn agree(a: &str, b: &str) -> (bool, f32) {
    let (ia, ib) = (
        parse(a).unwrap_or_else(|e| panic!("{a} must parse: {e:?}")),
        parse(b).unwrap_or_else(|e| panic!("{b} must parse: {e:?}")),
    );
    let mut worst = 0.0f32;
    for k in 0..600 {
        let bnd = B {
            t: k as f32 * 0.017,
            v: (k % 7) as f32 * 0.5 - 1.5,
        };
        let (x, y) = (eval(&ia, &bnd), eval(&ib, &bnd));
        if x.is_finite() && y.is_finite() {
            worst = worst.max((x - y).abs());
        }
    }
    (worst < 1e-5, worst)
}

/// One row at the recipe's defaults, with links and text filled — the same fixture the
/// standalone dump used, so the two sides are comparable.
fn row_at_defaults(id: &'static str) -> Row {
    let r = by_id(id).unwrap();
    let mut row = Row::new(id).unwrap();
    for (i, k) in r.knobs.iter().enumerate() {
        match k.kind {
            KnobKind::Link => row.knobs[i] = KnobValue::Link("Ball.x".into()),
            KnobKind::Text => row.knobs[i] = KnobValue::Text("value*2".into()),
            KnobKind::Number | KnobKind::Literal => {}
        }
    }
    row
}

fn formula_of(rows: Vec<Row>) -> String {
    RecipeStack { rows }.to_formula()
}

// ⚠️ Under `tests/shared/`, not directly under `tests/`: cargo compiles every file
// directly under `tests/` as its OWN binary, so a flat `pre_combine_table.rs` shipped a
// test target with no tests in it — and a `dead_code` warning on the table, because
// nothing in that binary reads it.
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/shared/pre_combine_table.rs"
));

/// **The catalog is value-identical to the world before `Combine`.**
///
/// ⚠️ Exactly ONE text changes, and it is named here rather than tolerated by a loose
/// comparison: `Free Fall` used to spell its own minus (`{inner} - 0.5*g*t*t`), and a
/// source contributes a term, so it now contributes `-0.5*g*t*t` and the fold writes
/// the `+`. `value + -0.5*9.8*time*time` parses (the grammar's `mul` takes `unary` on
/// its right) and is the same number.
#[test]
fn the_catalog_is_value_identical_to_the_pre_combine_world() {
    const TEXT_MOVED: &[&str] = &["free-fall"];
    // ⚠️ Retired as MEASURED duplicates of a survivor, each named with the setting that
    // reproduces it. Listed rather than deleted from the table so that removing a recipe
    // stays a DECISION with a reason attached, and so a re-added id is caught.
    const RETIRED: &[&str] = &["sway-cosine", "ramp-loop", "mirror", "midpoint", "negate"];
    // ⚠️ `Jitter` is the one recipe whose VALUE moved on purpose: it emitted a constant
    // (`noise(7)` = +0.197 forever, excursion 0.0000) and now reads the per-binding
    // `__seed`, so a group of objects each gets a different offset — which is what its own
    // blurb promised and what the report said it did not do.
    const VALUE_MOVED: &[&str] = &["jitter"];
    assert_eq!(
        PRE_COMBINE.len(),
        CATALOG.len() + RETIRED.len(),
        "the frozen table must cover the catalog plus the retired — a recipe added without \
         a line here is a recipe whose default nobody pinned"
    );
    for r in RETIRED {
        assert!(
            ph2d_expr_recipes::by_id(r).is_none(),
            "{r} is listed as retired but is still in the catalog"
        );
    }
    for (id, before) in PRE_COMBINE {
        if RETIRED.contains(id) || VALUE_MOVED.contains(id) {
            continue;
        }
        let now = formula_of(vec![row_at_defaults(id)]);
        let (ok, worst) = agree(before, &now);
        assert!(
            ok,
            "{id} changed VALUE at its defaults (worst {worst})\n  before: {before}\n  now:    {now}"
        );
        if !TEXT_MOVED.contains(id) {
            assert_eq!(
                &now, before,
                "{id} changed TEXT at its defaults; only {TEXT_MOVED:?} may"
            );
        }
    }
    // …and the exception really is one, so nobody "tidies" the list by adding to it.
    for id in TEXT_MOVED.iter().chain(VALUE_MOVED) {
        let now = formula_of(vec![row_at_defaults(id)]);
        let before = PRE_COMBINE.iter().find(|(i, _)| i == id).unwrap().1;
        assert_ne!(
            &now, before,
            "{id} is listed as text-moved but did not move"
        );
    }
}

/// **A source composes: stacking two of them keeps both.**
///
/// This is the report. Sway over Blink used to leave `select(...)` alone; under Add the
/// sway is in the answer, and it is checked by VALUE, not by looking for a `+`.
#[test]
fn stacking_two_sources_keeps_both_under_add_and_multiply() {
    let sway = formula_of(vec![row_at_defaults("sway")]);
    let blink_only = formula_of(vec![row_at_defaults("blink")]);

    let mut blink = row_at_defaults("blink");
    blink.combine = Combine::Add;
    let sum = formula_of(vec![row_at_defaults("sway"), blink.clone()]);
    let (same_as_blink, _) = agree(&sum, &blink_only);
    assert!(
        !same_as_blink,
        "an Add row must not swallow the rows above it — this is the reported bug:\n  {sum}"
    );
    let (matches_sum, worst) = agree(&sum, &format!("({sway}) + ({blink_only})"));
    assert!(
        matches_sum,
        "Add must be the sum of the two (worst {worst})\n  {sum}"
    );

    blink.combine = Combine::Multiply;
    let product = formula_of(vec![row_at_defaults("sway"), blink]);
    let (matches_product, worst) = agree(&product, &format!("({sway})*({blink_only})"));
    assert!(
        matches_product,
        "Multiply must be the product of the two (worst {worst})\n  {product}"
    );

    // Replace is the old behaviour, and it is still reachable — the artist can ask for
    // it, they just no longer get it without asking.
    let mut r = row_at_defaults("blink");
    r.combine = Combine::Replace;
    let (replaced, _) = agree(&formula_of(vec![row_at_defaults("sway"), r]), &blink_only);
    assert!(replaced, "Replace must drop the rows above");
}

/// **A product of a sum brackets the whole sum.**
///
/// ⚠️ The failure this catches parses and animates: `value + sin(t)*k` means
/// `value + (sin(t)*k)`, which is not the product of the two. It is the same defect
/// `EmitCtx::tight` exists for, one level up.
#[test]
fn multiplying_onto_a_sum_brackets_the_sum() {
    let mut flicker = row_at_defaults("flicker");
    flicker.combine = Combine::Multiply;
    let f = formula_of(vec![row_at_defaults("shake"), flicker]);
    let shake = formula_of(vec![row_at_defaults("shake")]);
    let flick_only = {
        let r = by_id("flicker").unwrap();
        let row = row_at_defaults("flicker");
        (r.emit)(&EmitCtx {
            knobs: &row.knobs,
            defs: r.knobs,
            inner: "value",
            clock: "time",
        })
    };
    let (ok, worst) = agree(&f, &format!("({shake})*({flick_only})"));
    assert!(ok, "worst {worst}\n  got: {f}");
}

/// **The split is honest in both directions.**
///
/// A source that read `inner` would have the rows above counted TWICE under Add; a
/// modifier that ignored it would drop them, which is the bug this wave exists to fix.
/// Neither is visible in a formula anyone reads — hence a gate over the whole catalog.
#[test]
fn a_source_never_reads_inner_and_a_modifier_always_does() {
    const SENTINEL: &str = "SENTINEL";
    for r in CATALOG {
        let row = row_at_defaults(r.id);
        let out = (r.emit)(&EmitCtx {
            knobs: &row.knobs,
            defs: r.knobs,
            inner: SENTINEL,
            clock: "time",
        });
        match (r.kind, r.combine) {
            (RowKind::Time, c) => assert!(
                c.is_none(),
                "{}: a Time row rewrites the clock; it has no mode to choose",
                r.id
            ),
            (_, Some(_)) => assert!(
                !out.contains(SENTINEL),
                "{}: a SOURCE must contribute ALONE — the fold adds it. Reading `inner` \
                 counts the rows above twice:\n  {out}",
                r.id
            ),
            (RowKind::Raw, None) => {} // the artist's own text; it may or may not read `value`
            (RowKind::Value, None) => assert!(
                out.contains(SENTINEL),
                "{}: a MODIFIER must fold `inner`. Ignoring it silently discards every \
                 row above:\n  {out}",
                r.id
            ),
        }
    }
}

/// A row's mode is offered exactly when the recipe declares one — the ONE question the
/// card and the fold both ask, so the chip and the picture cannot disagree.
#[test]
fn a_row_combines_exactly_when_its_recipe_is_a_source() {
    for r in CATALOG {
        let row = Row::new(r.id).unwrap();
        assert_eq!(
            row.combines(),
            r.combine.is_some(),
            "{}: `Row::combines` must mirror the recipe",
            r.id
        );
        if let Some(c) = r.combine {
            assert_eq!(
                row.combine, c,
                "{}: a new row starts at the recipe's mode",
                r.id
            );
        }
    }
}

/// The chip cycles through all three and comes home — a two-state toggle would make one
/// mode unreachable by pointer, which is how a mode nobody can select ships.
#[test]
fn the_mode_cycle_reaches_every_mode() {
    let mut seen = vec![];
    let mut c = Combine::Add;
    for _ in 0..Combine::ALL.len() {
        seen.push(c);
        c = c.next();
    }
    assert_eq!(c, Combine::Add, "the cycle must close");
    for m in Combine::ALL {
        assert!(seen.contains(&m), "{m:?} is unreachable by cycling");
    }
}
