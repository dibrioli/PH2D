//! The **expression pass** (Wave C / ADR-0144). A SEPARATE post-composition pass:
//! it runs at the END of the apply, after every keyed property is composed into the
//! world. For each binding that carries a formula it reads the composed values,
//! evaluates the expression, and overwrites the driven property.
//!
//! ⚠️ **It never enters the BLEND** (`sample_stack`/`eval_frame`) — it only READS the
//! strip layout (`sole_strip_of`/`strip_source_time`) to window a per-clip expr
//! (ADR-0145), which reports WHERE a strip plays, never a fade weight. A document with
//! no expression takes the early-out and does nothing — byte-identical to the
//! pre-feature engine, the Clips/Strips/Fade fingerprint untouched by construction.
//! This is the whole reason ADR-0144 put the pass HERE rather than inside the evaluator.
//!
//! **Evaluation order** (ADR-0144 §6): the driven bindings are TOPOLOGICALLY
//! ordered (a dependency before its dependents) and evaluated over a mutable
//! `current` map seeded from the composed snapshot — so an acyclic chain
//! `A = B.x`, `B = time*10` reads FRESH values with **no 1-frame lag**. A cycle
//! `A <-> B` has no valid order: its members fall to the end and read the value
//! still standing in `current` (the previous frame's at the back-edge), a single
//! sweep that **cannot explode**. `value` is always the PRE-expression value
//! (keyed sample or rest), never the map, so a prop never feeds back into itself.
//!
//! ⚠️ **Two clock-and-ownership rules the caller upholds, both learned from smoke
//! reports:**
//! - It runs on the **CUT clock** — the exact instant the keyed pass composed at
//!   (`clip_cut`/`container_cut`/`cut_scene`). Past a composition's authored end the
//!   keys FREEZE at the cut; the expression's `time` must freeze there too, or it
//!   extrapolates past the container/clip/scene end while everything else stands.
//! - It honours **`skip`** exactly as pass 3 does. An entity the user owns this
//!   frame (gizmo drag) or a paused pose pinned off its curve
//!   (`AutokeyState.displaced`) is NOT rewritten by the keyed pass — so if the
//!   expression drove it anyway it would read its OWN un-reset output as `value` and
//!   feed back, a monotonic drift while paused. Skipped entities still appear in the
//!   snapshot, so a prop-LINK may read their live pose.
//!
//! ⚠️ **Two sources of formula, two windows** (ADR-0144 + ADR-0145):
//! - A **GLOBAL** driver (`binding.expr`, document-wide) runs everywhere on the outer
//!   CUT clock (the Arrange scene driver). A KEYED prop's global expr rides its strip
//!   through the `composed` coverage: where the composition covers nothing it goes
//!   QUIET with the keys, and `value` is the COMPOSED value (rest when uncovered),
//!   NEVER the stale world (which kills the paused drift on an uncovered prop).
//! - A **PER-CLIP** driver (`NamedClip.expr`, ADR-0145) lives in the clip like a
//!   keyframe, so a strip that plays the clip WINDOWS it, at the strip-LOCAL time
//!   (`clip_expr_clock`). Off the strip the clip is not playing -> quiet. This is what
//!   makes a PURE expression (no keys at all) obey the strip: the clip is its home.

use ph2d_anim::AnimValue;
use ph2d_ecs::{Entity, Name, World, stable_name_id};
use ph2d_expr::{Bindings, Expr, eval};
use std::collections::BTreeMap;

use crate::apply::{read_prop, write_prop};
use crate::doc::TimelineDoc;
use crate::frame_solve::{SEED_SPACING, collect_links, resolve_link};
use crate::prop::PropKind;

/// Run the expression pass at the composition's CUT clock `time` (see the module
/// docs). `skip` mirrors the keyed pass: a driven entity it claims is left alone.
/// `composed` is what the keyed pass just wrote, per `(entity, prop)` — the coverage
/// mask and the pre-expression `value`. No-op (byte-identical) when no formula.
pub(crate) fn run(
    world: &mut World,
    doc: &TimelineDoc,
    time: f64,
    skip: &dyn Fn(u64) -> bool,
    composed: &BTreeMap<(u64, PropKind), f32>,
) {
    // ADR-0146 W2 — this pass now handles ONLY GLOBAL drivers (`binding.expr`, the
    // document-wide channel transform of ADR-0144). Per-clip expressions are first-class
    // lane SOURCES at the two sample sites (the blend `eval_frame` / `solo_source_value`),
    // so they fade; nothing to do here if no global driver exists.
    if !doc.bindings().iter().any(|b| b.expr.is_some()) {
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

    // Parse the driven bindings every frame. A parse error is a fallback: the
    // property keeps its keyed value (nothing is collected, so nothing is written).
    //
    // ⚠️ Caching the parsed IR was MEASURED and REJECTED (not a premature-opt, §0):
    // a representative expression parses in **335 ns**, so 10 driven props cost
    // **3.35 µs/frame** — 0.02 % of a 60 fps frame (100 props = 0.2 %). A cache would
    // add a keyed side-table + invalidation for no measurable gain; the string on the
    // binding stays the single source of truth. Re-measure only if a scene ever runs
    // hundreds of expression-driven properties.
    let mut driven: Vec<Driven> = Vec::new();
    for (i, b) in doc.bindings().iter().enumerate() {
        // Honour `skip` like the keyed pass: a gizmo-owned or displaced-pinned entity
        // is not driven, or it reads its own un-reset output back (module docs).
        if b.missing || skip(b.entity) {
            continue;
        }
        let Some(src) = &b.expr else { continue };
        let Ok(ir) = ph2d_expr_parse::parse(src) else {
            continue;
        };
        // A KEYED prop's expression rides its strip: outside the window the
        // composition covers nothing, so it goes quiet WITH the keys (Report B)
        // rather than playing forever. A pure expression (no keys anywhere) has no
        // window and always runs. `value` is the COMPOSED pre-expression value (rest
        // when uncovered), NEVER the world — which could be our own last output.
        let keyed = doc
            .clips()
            .iter()
            .any(|c| c.clip.track(b.target).is_some_and(|t| !t.is_empty()));
        let composed_v = composed.get(&(b.entity, b.prop)).copied();
        if keyed && composed_v.is_none() {
            continue;
        }
        let value = composed_v.unwrap_or(b.rest.unwrap_or(0.0));
        driven.push(Driven {
            idx: i,
            ir,
            entity: b.entity,
            prop: b.prop,
            value,
            seed: b.target.get() as f32 * SEED_SPACING,
            // The GLOBAL driver runs on the composition's cut clock, everywhere (the
            // Arrange scene driver — ADR-0145).
            time: time as f32,
        });
    }

    // ADR-0146 W2 — the per-clip loop that used to live here is GONE. A per-clip
    // expression is now a lane source at the two sample sites (`eval_frame` under a stack,
    // `solo_source_value` without one), so it fades with its strip instead of overwriting
    // the composed value. This pass keeps only the GLOBAL drivers collected above.

    // Evaluate in dependency order over a MUTABLE `current` (seeded from the
    // snapshot): a binding reads `current` and writes its result back, so an acyclic
    // chain reads fresh values with no lag; cycle members (ordered last) read the
    // value still standing, non-exploding.
    let order = topo_order(&driven, &names);
    let mut current = snap;
    let mut writes: Vec<(usize, f32)> = Vec::new();
    for &p in &order {
        let d = &driven[p];
        let bindings = ExprBindings {
            cur: &current,
            names: &names,
            value: d.value,
            time: d.time,
            seed: d.seed,
        };
        let v = eval(&d.ir, &bindings);
        current.insert((d.entity, d.prop), v);
        writes.push((d.idx, v));
    }

    // Write all. `write_prop` takes the whole binding for the Position trajectory;
    // `doc` and `world` are distinct, so the immutable binding borrow and the
    // mutable world write do not conflict.
    for (i, v) in writes {
        let b = &doc.bindings()[i];
        if let Some(e) = Entity::try_from_bits(b.entity) {
            write_prop(world, e, b, AnimValue::Float(v), false);
        }
    }
}

/// One driven binding, parsed and ready to evaluate.
struct Driven {
    /// Index into `doc.bindings()` (where the write lands).
    idx: usize,
    ir: Expr,
    entity: u64,
    prop: PropKind,
    /// The pre-expression value `value` resolves to (keyed sample or rest).
    value: f32,
    /// This binding's wiggle seed.
    seed: f32,
    /// The clock this expr evaluates at: the composition cut clock (a GLOBAL driver)
    /// or the strip-LOCAL time (a per-clip driver) — ADR-0145.
    time: f32,
}

/// The [`Bindings`] the pass hands each expression: `time` (the clip clock),
/// `value` (this property's PRE-EXPRESSION value — keyed sample or rest, computed
/// by the caller), `__seed` (this binding's wiggle seed), and `Name.prop`
/// prop-links resolved against `cur` (the value standing this sweep). An unknown
/// name is `0.0` (the evaluator's total contract).
struct ExprBindings<'a> {
    cur: &'a BTreeMap<(u64, PropKind), f32>,
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
            dotted => resolve_link(dotted, self.names)
                .and_then(|k| self.cur.get(&k).copied())
                .unwrap_or(0.0),
        }
    }

    fn param(&self, _name: &str) -> f32 {
        0.0
    }
}

/// Kahn topological order of the driven bindings — a dependency before its
/// dependents, so an acyclic chain evaluates fresh. A cycle has no valid order;
/// its members never reach in-degree 0 and are appended in original order (they
/// read the value still standing in `current`, non-exploding — ADR-0144 §6).
fn topo_order(driven: &[Driven], names: &BTreeMap<u64, u64>) -> Vec<usize> {
    let n = driven.len();
    let pos_of: BTreeMap<(u64, PropKind), usize> = driven
        .iter()
        .enumerate()
        .map(|(p, d)| ((d.entity, d.prop), p))
        .collect();
    let mut indeg = vec![0usize; n];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut links = Vec::new();
    for (p, d) in driven.iter().enumerate() {
        links.clear();
        collect_links(&d.ir, names, &mut links);
        let mut seen: Vec<usize> = Vec::new();
        for k in &links {
            if let Some(&q) = pos_of.get(k)
                && q != p
                && !seen.contains(&q)
            {
                seen.push(q);
                indeg[p] += 1;
                dependents[q].push(p);
            }
        }
    }
    let mut queue: Vec<usize> = (0..n).filter(|&p| indeg[p] == 0).collect();
    let mut order = Vec::with_capacity(n);
    let mut qi = 0;
    while qi < queue.len() {
        let p = queue[qi];
        qi += 1;
        order.push(p);
        for &r in &dependents[p] {
            indeg[r] -= 1;
            if indeg[r] == 0 {
                queue.push(r);
            }
        }
    }
    // Cycle members (in-degree never hit 0) — append in original order.
    for p in 0..n {
        if !order.contains(&p) {
            order.push(p);
        }
    }
    order
}
