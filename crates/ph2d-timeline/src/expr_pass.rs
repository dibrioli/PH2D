//! The **expression pass** (Wave C / ADR-0144). A SEPARATE post-composition pass:
//! it runs at the END of the apply, after every keyed property is composed into the
//! world. For each binding that carries a formula it reads the composed values,
//! evaluates the expression, and overwrites the driven property.
//!
//! ⚠️ **It never touches `stack_eval` or the blend.** A document with no expression
//! takes the early-out and does nothing — so it is byte-identical to the pre-feature
//! engine, and the Clips/Strips/Fade fingerprint is untouched by construction. This
//! is the whole reason ADR-0144 put the pass HERE rather than inside the evaluator.
//!
//! **Cycles** (ADR-0144 §6): a single Gauss-Jacobi sweep over a SNAPSHOT taken at
//! the start of the pass. Every expression reads the snapshot (the just-composed
//! keyed values + last frame's driven values), so `A = B.x` with B keyed is exact
//! and lag-free, and a cycle `A <-> B` reads the previous value at the edge and
//! **cannot explode**. The only lag is a driven->driven chain (a named follow-up).

use ph2d_anim::AnimValue;
use ph2d_ecs::{Entity, Name, World, stable_name_id};
use ph2d_expr::{Bindings, eval};
use std::collections::BTreeMap;

use crate::apply::{read_prop, write_prop};
use crate::doc::TimelineDoc;
use crate::prop::PropKind;

/// Spread between two bindings' wiggle seeds — large enough that the noise phases
/// of distinct properties never coincide (the hash decorrelates any distinct
/// input, so this only needs to keep the numbers apart). `// LITERAL-PX-OK`: not a
/// UI value; a seed spacing.
const SEED_SPACING: f32 = 100.0;

/// Run the expression pass at clip time `time`. No-op (byte-identical) when no
/// binding carries a formula.
pub(crate) fn run(world: &mut World, doc: &TimelineDoc, time: f64) {
    // The fade pin: nothing driven -> the pass does not exist.
    if doc.bindings().iter().all(|b| b.expr.is_none()) {
        return;
    }

    // Snapshot every live binding's composed value (the Jacobi read set) + a
    // Name -> entity map for prop-links. Read BEFORE any write, so the whole
    // sweep sees one consistent frame.
    let mut snap: BTreeMap<(u64, PropKind), f32> = BTreeMap::new();
    let mut names: BTreeMap<u64, u64> = BTreeMap::new();
    for b in doc.bindings() {
        if b.missing {
            continue;
        }
        let Some(e) = Entity::try_from_bits(b.entity) else {
            continue;
        };
        if let Some(v) = read_prop(world, e, b.prop) {
            snap.insert((b.entity, b.prop), v);
        }
        if let Some(name) = world.get::<Name>(e) {
            names
                .entry(stable_name_id(name.0.as_str()))
                .or_insert(b.entity);
        }
    }

    // Evaluate each driven binding against the snapshot. Parse per frame (the count
    // is tiny; caching the IR is a named follow-up). A parse error is a fallback:
    // the property keeps its keyed value (nothing is written for it).
    let mut writes: Vec<(usize, f32)> = Vec::new();
    for (i, b) in doc.bindings().iter().enumerate() {
        if b.missing {
            continue;
        }
        let Some(src) = &b.expr else { continue };
        let Ok(ir) = ph2d_expr_parse::parse(src) else {
            continue;
        };
        // `value` is the PRE-EXPRESSION value (AE semantics). For a KEYED prop that
        // is the composed keyed sample (the snapshot, stable across frames). For a
        // KEYLESS prop it is the static REST pose — NEVER last frame's own output,
        // which would feed back into a random walk (`value + wiggle` on a bare prop).
        let has_keys = doc
            .active_clip()
            .track(b.target)
            .is_some_and(|t| !t.is_empty());
        let value = if has_keys {
            snap.get(&(b.entity, b.prop)).copied().unwrap_or(0.0)
        } else {
            b.rest.unwrap_or(0.0)
        };
        let bindings = ExprBindings {
            snap: &snap,
            names: &names,
            value,
            time: time as f32,
            seed: b.target.get() as f32 * SEED_SPACING,
        };
        writes.push((i, eval(&ir, &bindings)));
    }

    // Write all (Jacobi: reads done, now the writes). `write_prop` takes the whole
    // binding for the Position trajectory; `doc` and `world` are distinct, so the
    // immutable binding borrow and the mutable world write do not conflict.
    for (i, v) in writes {
        let b = &doc.bindings()[i];
        if let Some(e) = Entity::try_from_bits(b.entity) {
            write_prop(world, e, b, AnimValue::Float(v), false);
        }
    }
}

/// The [`Bindings`] the pass hands each expression: `time` (the clip clock),
/// `value` (this property's PRE-EXPRESSION value — keyed sample or rest, computed
/// by the caller), `__seed` (this binding's wiggle seed), and `Name.prop`
/// prop-links resolved against the snapshot. An unknown name is `0.0` (the
/// evaluator's total contract).
struct ExprBindings<'a> {
    snap: &'a BTreeMap<(u64, PropKind), f32>,
    names: &'a BTreeMap<u64, u64>,
    value: f32,
    time: f32,
    seed: f32,
}

impl Bindings for ExprBindings<'_> {
    fn attr(&self, name: &str) -> f32 {
        match name {
            "time" => self.time,
            "__seed" => self.seed,
            "value" => self.value,
            dotted => {
                // `Name.prop` -> the snapshot value of another animated property.
                if let Some((nm, pr)) = dotted.rsplit_once('.')
                    && let (Some(&e), Some(prop)) = (
                        self.names.get(&stable_name_id(nm)),
                        PropKind::from_expr_name(pr),
                    )
                {
                    return *self.snap.get(&(e, prop)).unwrap_or(&0.0);
                }
                0.0
            }
        }
    }

    fn param(&self, _name: &str) -> f32 {
        0.0
    }
}
