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
//! **Evaluation order** (ADR-0144 §6): the driven bindings are TOPOLOGICALLY
//! ordered (a dependency before its dependents) and evaluated over a mutable
//! `current` map seeded from the composed snapshot — so an acyclic chain
//! `A = B.x`, `B = time*10` reads FRESH values with **no 1-frame lag**. A cycle
//! `A <-> B` has no valid order: its members fall to the end and read the value
//! still standing in `current` (the previous frame's at the back-edge), a single
//! sweep that **cannot explode**. `value` is always the PRE-expression value
//! (keyed sample or rest), never the map, so a prop never feeds back into itself.

use ph2d_anim::AnimValue;
use ph2d_ecs::{Entity, Name, World, stable_name_id};
use ph2d_expr::{Bindings, Expr, eval};
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
        driven.push(Driven {
            idx: i,
            ir,
            entity: b.entity,
            prop: b.prop,
            value,
            seed: b.target.get() as f32 * SEED_SPACING,
        });
    }

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
            time: time as f32,
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

/// A `Name.prop` prop-link identifier -> the `(entity, prop)` it names, if the name
/// resolves to an animated object and the tail is a known property. `None` for a
/// bare identifier (no dot) or an unresolved name.
fn resolve_link(name: &str, names: &BTreeMap<u64, u64>) -> Option<(u64, PropKind)> {
    let (nm, pr) = name.rsplit_once('.')?;
    Some((
        *names.get(&stable_name_id(nm))?,
        PropKind::from_expr_name(pr)?,
    ))
}

/// Collect every `(entity, prop)` a driven binding's expression reads through a
/// prop-link — the dependency edges for the topological order.
fn collect_links(e: &Expr, names: &BTreeMap<u64, u64>, out: &mut Vec<(u64, PropKind)>) {
    match e {
        Expr::Attr(name) => {
            if let Some(k) = resolve_link(name, names) {
                out.push(k);
            }
        }
        Expr::Unary(_, a) => collect_links(a, names, out),
        Expr::Binary(_, a, b) => {
            collect_links(a, names, out);
            collect_links(b, names, out);
        }
        Expr::Call(_, args) => args.iter().for_each(|a| collect_links(a, names, out)),
        Expr::Select { cond, a, b } => {
            collect_links(cond, names, out);
            collect_links(a, names, out);
            collect_links(b, names, out);
        }
        Expr::Const(_) | Expr::Param(_) => {}
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
