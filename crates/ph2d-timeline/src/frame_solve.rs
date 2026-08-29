//! The frame solver's shared state — what the blend knows about *other* channels
//! this frame, so a prop-link (`Name.prop`) can read a source that is itself faded
//! (ADR-0152). It is the `snap`+`names` the ADR-0144 post-pass built, lifted to a
//! type the blend can be handed.
//!
//! **Through W2 this is threaded EMPTY.** In W0 it is never read (the codegen of
//! `eval_frame`/`sample_stack` gains a parameter while the IEEE-754 arithmetic is
//! untouched — the fade fingerprint `tests/fade_fingerprint.rs` stays byte-for-byte); in
//! W1 a per-clip expression reads it (via [`eval_expr`]) but every `Name.prop` link
//! resolves to 0 because the map is empty. W3 fills this module with the two-phase
//! scheduler that BUILDS the map (the retired `expr_pass` machinery:
//! `collect_links`/`resolve_link`/`topo_order`), and a prop-link finally reads a value.
//!
//! ⚠️ **W3's map was a projection of the BINDING LIST, and that was the bug behind
//! *"Follow não segue o objeto referido"*.** Both halves — the name and the value — came
//! from `doc.bindings()`, so a link could only read a property the timeline already
//! animated; anything else took the evaluator's total contract and resolved to **0.0**,
//! which reads as *"my object jumped to the origin"*. Today [`build_names`] covers the
//! whole SCENE and [`seed_unbound_links`] reads the world for the sources it does not
//! animate. See those two for the measurement.
//!
//! An empty [`LinkFrame`] allocates nothing — an empty `BTreeMap` never touches the
//! heap until its first insert — so a formula-free apply carries one at zero cost
//! (HR-3; gate `no_expression_allocates_no_link_frame`).

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, Name, World, stable_name_id};
use ph2d_expr::{Bindings, Expr, eval};

use crate::TargetBinding;
use crate::apply_prop::read_prop;
use crate::doc::TimelineDoc;
use crate::prop::PropKind;

/// What the blend knows about other channels this frame, for prop-links (ADR-0152).
///
/// Empty on the formula-free path (the common case), where it is never read: an
/// empty `BTreeMap` never allocates, so carrying one costs nothing. The blend reads it
/// through [`ExprBindings`] when a per-clip expression names a `Name.prop` link.
#[derive(Default, Debug, Clone)]
pub(crate) struct LinkFrame {
    /// The composed value per `(entity, prop)` — what a `Name.prop` link reads. A
    /// source composes *before* its reader (W3's topological order) and writes its
    /// already-faded value here; the reader reads it, and its own strip fades the
    /// result again (the "double fade" of ADR-0152).
    pub links: BTreeMap<(u64, PropKind), f64>,
    /// `stable_name_id(Name)` -> entity bits, resolving the name half of a
    /// `Name.prop` link to the entity whose channel it names.
    pub names: BTreeMap<u64, u64>,
}

/// Spread between two bindings' wiggle seeds — large enough that the noise phases of
/// distinct properties never coincide (the hash decorrelates any distinct input, so this
/// only needs to keep the numbers apart). `LITERAL-PX-OK`: a seed spacing, not a UI value.
pub(crate) const SEED_SPACING: f32 = 100.0;

/// **The one door to `__seed`** — this channel's noise seed, for every consumer.
///
/// ⚠️ It exists because the SAME question had **three answers**, and the audit of
/// 2026-07-29 (§4 D-J) measured all three: the scene said `target * SEED_SPACING`; the
/// card's preview ribbon fell through to its `_ => 0.0` arm and said **0**, always; and
/// the coverage census — the instrument that decided which recipes were "alive" — let
/// `__seed` land in its *link* arm and said **0.96**. One formula, one knob, five
/// different numbers: a `Jitter` on the third object displaced **0.0089 u ≈ 0.9 px**
/// while the ribbon drew the wobble of object zero. *"Jitter não funciona"* was literal,
/// and literal for SOME objects and not others.
///
/// ⚠️ Três instrumentos independentes erraram a MESMA binding, e é por isso que isto é uma
/// função e não uma convenção: quem deriva o seed por conta própria desenha um wobble que o
/// objeto não roda.
#[must_use]
pub fn seed_of_target(target: u64) -> f32 {
    target as f32 * SEED_SPACING
}

/// Evaluate a parsed expression `ir` as a channel's value: `value` is this channel's
/// pre-expression value (the keyed sample or rest), `time` is the strip-LOCAL clip clock,
/// `seed` is the wiggle seed, and `links` carries the faded value of any source channel a
/// `Name.prop` link reads (ADR-0152).
///
/// The evaluator is f32 (`ph2d-expr`); the blend works in f64 and casts the result back.
/// A `Name.prop` whose source is not in `links` resolves to 0.0 (the evaluator's total
/// contract) — through W2 the map is empty, so every prop-link is 0 until W3 fills it.
pub(crate) fn eval_expr(ir: &Expr, value: f64, time: f64, seed: f64, links: &LinkFrame) -> f32 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the expression evaluator is f32; f64 is the blend's working precision"
    )]
    let bindings = ExprBindings {
        links,
        value: value as f32,
        time: time as f32,
        seed: seed as f32,
    };
    eval(ir, &bindings)
}

/// The [`Bindings`] the blend hands an expression: `time`, `value`, `__seed`, and
/// `Name.prop` prop-links resolved against the frame's [`LinkFrame`] (the faded value of
/// the source channel this sweep composed — ADR-0152). An unknown name is 0.0.
struct ExprBindings<'a> {
    links: &'a LinkFrame,
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
            #[expect(
                clippy::cast_possible_truncation,
                reason = "link values are stored f64 for the blend; the evaluator is f32"
            )]
            dotted => resolve_link(dotted, &self.links.names)
                .and_then(|k| self.links.links.get(&k).copied())
                .map(|v| v as f32)
                .unwrap_or(0.0),
        }
    }

    fn param(&self, _name: &str) -> f32 {
        0.0
    }
}

/// A `Name.prop` prop-link identifier -> the `(entity, prop)` it names, if the name
/// resolves to an animated object and the tail is a known property. `None` for a bare
/// identifier (no dot) or an unresolved name. Shared by the blend (W1) and the retired
/// post-pass's dependency collector (`expr_pass::collect_links`), so a link resolves the
/// same way wherever it is read.
pub(crate) fn resolve_link(name: &str, names: &BTreeMap<u64, u64>) -> Option<(u64, PropKind)> {
    let (nm, pr) = name.rsplit_once('.')?;
    Some((
        *names.get(&stable_name_id(nm))?,
        PropKind::from_expr_name(pr)?,
    ))
}

/// Every `(entity, prop)` an expression reads through a `Name.prop` prop-link — the
/// dependency edges for the topological order. Shared by the frame scheduler (W3) and the
/// global post-pass (`expr_pass`), so a link contributes the same edge wherever it is read.
pub(crate) fn collect_links(e: &Expr, names: &BTreeMap<u64, u64>, out: &mut Vec<(u64, PropKind)>) {
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

/// Build the `stable_name_id(Name) -> entity bits` map for every live binding — the name
/// half of a prop-link resolves through this. First name wins (deterministic under the
/// `BTreeMap` iteration a binding list already has).
/// **Does this frame have a formula to run?** — the predicate that decides whether the
/// apply builds `composed`, a names map and a topological order at all, or takes the
/// formula-free zero-alloc path (HR-3).
///
/// ⚠️ **One door, three callers.** It was written out identically in all three apply
/// views (Arrange, container, Keys solo), and the live-preview channel is exactly the
/// kind of fourth source that a copied predicate learns about in two places out of
/// three: the preview would then drive the object in Arrange and sit dead inside a
/// container, for no reason the artist could see.
///
#[must_use]
pub(crate) fn any_formula(doc: &TimelineDoc) -> bool {
    doc.bindings().iter().any(|b| b.expr.is_some())
        || doc.clips().iter().any(|c| !c.expr.is_empty())
        || crate::expr_owed::has_pending_restore()
}

/// **Every name a prop-link may resolve — the whole SCENE, not the binding list.**
///
/// ⚠️ Red-first against a report: *"Follow e outros da categoria ruins, não seguem o
/// objeto referido."* This map used to be built exclusively from `doc.bindings()`, so
/// a name only existed if the timeline already animated that object — and MEASURED,
/// `Ball.x*1 + 0` against a merely-placed sprite resolved to **0.0**, which does not
/// read as *"the link is dead"*, it reads as *"my object jumped to the origin"*. The
/// load-bearing measurement was the near-miss: an object **bound** on TranslationX
/// with **zero keys** already worked (7.0000), so what was missing was never the
/// track — it was the BINDING, a document fact the artist has no reason to connect to
/// *"be where that is"*.
///
/// Two passes, and the order carries the weight:
///
/// 1. **the bindings, exactly as before** — a bound object stays authoritative, so
///    every link that resolves today resolves to the same entity tomorrow;
/// 2. **the rest of the scene**, `or_insert`, so pass 1 always wins.
///
/// ⚠️ Duplicate `Name`s become user-visible the moment the map covers the whole scene,
/// and *"whichever came first"* is not an answer when the source is a query whose order
/// is an implementation detail. The rule is the **lowest entity bits**: arbitrary, but
/// STABLE across frames and across runs, which is the property that matters — an
/// ambiguous link that picks a different object every frame is worse than one that
/// picks the wrong object every frame.
pub(crate) fn build_names(world: &mut World, doc: &TimelineDoc) -> BTreeMap<u64, u64> {
    let mut names = build_names_bound(world, doc);
    let mut unbound: BTreeMap<u64, u64> = BTreeMap::new();
    let mut q = world.query::<(Entity, &Name)>();
    for (e, name) in q.iter(world) {
        let key = stable_name_id(name.0.as_str());
        let bits = e.to_bits();
        unbound
            .entry(key)
            .and_modify(|held| *held = (*held).min(bits))
            .or_insert(bits);
    }
    for (key, bits) in unbound {
        names.entry(key).or_insert(bits);
    }
    names
}

/// Pass 1 alone — the names of objects the timeline ANIMATES.
///
/// ⚠️ This is not a second answer to *"what does this name mean?"*: [`build_names`] calls
/// it and then widens. It exists as its own door for the ONE caller that cannot widen —
/// `pose_at`, the onion's ghost, which is handed a `&World` from inside the render
/// extract and whose prop-links are ALREADY declared approximate (it reads the source at
/// the live playhead, not at the ghost's time). Giving the ghost a mutable world to reach
/// unbound sources would push `&mut` through the render path to make an approximation
/// slightly less approximate.
pub(crate) fn build_names_bound(world: &World, doc: &TimelineDoc) -> BTreeMap<u64, u64> {
    let mut names = BTreeMap::new();
    for b in doc.bindings() {
        if b.missing {
            continue;
        }
        let Some(e) = Entity::try_from_bits(b.entity) else {
            continue;
        };
        if let Some(name) = world.get::<Name>(e) {
            names
                .entry(stable_name_id(name.0.as_str()))
                .or_insert(b.entity);
        }
    }
    names
}

/// **The value of a link whose source the timeline does not animate**, read straight
/// from the world.
///
/// The composition loop writes `links.links` for every BINDING it composes, and
/// [`seed_links`] pre-fills the same set from last frame — so a source that has a
/// binding is always covered. This fills the other case, and only the pairs some
/// formula actually NAMES: an unbound source contributes nothing to `topo_order`
/// (there is no channel to order), and it does not need to — its value is already
/// final for the frame, whoever wrote it (the gizmo, physics, a parent's transform).
/// Read-only, and never inserted over a value the loop will produce.
///
/// ⚠️ `Position` is honestly absent for an unbound source: it is a distance ALONG a
/// trajectory, and an object with no binding has no trajectory to measure against —
/// so [`read_prop_kind`] returns `None` and the link keeps the total contract's 0.0.
///
/// [`read_prop_kind`]: crate::apply_prop::read_prop_kind
pub(crate) fn seed_unbound_links(
    world: &World,
    doc: &TimelineDoc,
    names: &BTreeMap<u64, u64>,
    out: &mut BTreeMap<(u64, PropKind), f64>,
) {
    let bound: std::collections::BTreeSet<(u64, PropKind)> = doc
        .bindings()
        .iter()
        .filter(|b| !b.missing)
        .map(|b| (b.entity, b.prop))
        .collect();
    let mut wanted = Vec::new();
    for b in doc.bindings() {
        binding_links(doc, b, names, &mut wanted);
    }
    for c in doc.clips() {
        for expr in c.expr.values() {
            let Ok(ir) = ph2d_expr_parse::parse(expr) else {
                continue;
            };
            collect_links(&ir, names, &mut wanted);
        }
    }
    for key in wanted {
        if bound.contains(&key) || out.contains_key(&key) {
            continue;
        }
        let Some(e) = Entity::try_from_bits(key.0) else {
            continue;
        };
        if let Some(v) = crate::apply_prop::read_prop_kind(world, e, key.1) {
            out.insert(key, f64::from(v));
        }
    }
}

/// Seed the frame's `links` from the world — the value each channel held LAST frame — so a
/// genuine cycle's back edge (`A` reads `B` reads `A`) reads a previous-frame value instead
/// of 0 (ADR-0152 §2.1, the industry one-frame-delay). An ACYCLIC channel's seed is
/// immediately overwritten by its fresh composition in topo order, so seeding never changes
/// the acyclic result — it only gives a cycle something stable to read.
///
/// ⚠️ The C4 hole is CLOSED: `read_prop` takes the binding now, so Position (a distance
/// read through the trajectory) and Morph seed like every other kind, and a cycle back
/// edge naming them reads a real previous-frame value instead of 0.
pub(crate) fn seed_links(
    world: &World,
    doc: &TimelineDoc,
    out: &mut BTreeMap<(u64, PropKind), f64>,
) {
    for b in doc.bindings() {
        if b.missing {
            continue;
        }
        let Some(e) = Entity::try_from_bits(b.entity) else {
            continue;
        };
        if let Some(v) = read_prop(world, e, b, doc.path_for(b.target)) {
            out.insert((b.entity, b.prop), f64::from(v));
        }
    }
}

/// Every prop-link a binding's expressions read — its GLOBAL expr and EVERY clip's per-clip
/// expr for this target. Which clip actually plays is a runtime question the sample site
/// answers; the dependency order takes the UNION, which is safe because an extra edge only
/// orders more conservatively (it never drops a real dependency). Parses the exprs (335 ns
/// each; caching was measured and rejected, `expr_pass.rs`).
fn binding_links(
    doc: &TimelineDoc,
    b: &TargetBinding,
    names: &BTreeMap<u64, u64>,
    out: &mut Vec<(u64, PropKind)>,
) {
    if let Some(src) = &b.expr
        && let Ok(ir) = ph2d_expr_parse::parse(src)
    {
        collect_links(&ir, names, out);
    }
    for clip in doc.clips() {
        if let Some(src) = clip.expr.get(&b.target)
            && let Ok(ir) = ph2d_expr_parse::parse(src)
        {
            collect_links(&ir, names, out);
        }
    }
}

/// **The order the frame's channels compose in (ADR-0152 W3):** a prop-link SOURCE before
/// its reader, so `value + Sprite.x` reads Sprite's already-composed (faded) value in the
/// SAME frame — no one-frame lag. Kahn's algorithm over the binding graph; a genuine cycle
/// (A reads B reads A) has no valid order, so its members are appended in original order and
/// read the value STILL STANDING in the `LinkFrame` (the one-frame delay of the industry —
/// Houdini Feedback CHOP, ADR-0152 §3). Returns binding indices into `doc.bindings()`.
pub(crate) fn topo_order(doc: &TimelineDoc, names: &BTreeMap<u64, u64>) -> Vec<usize> {
    let bindings = doc.bindings();
    let n = bindings.len();
    let pos_of: BTreeMap<(u64, PropKind), usize> = bindings
        .iter()
        .enumerate()
        .map(|(i, b)| ((b.entity, b.prop), i))
        .collect();
    let mut indeg = vec![0usize; n];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut links = Vec::new();
    for (i, b) in bindings.iter().enumerate() {
        links.clear();
        binding_links(doc, b, names, &mut links);
        let mut seen: Vec<usize> = Vec::new();
        for k in &links {
            if let Some(&q) = pos_of.get(k)
                && q != i
                && !seen.contains(&q)
            {
                seen.push(q);
                indeg[i] += 1;
                dependents[q].push(i);
            }
        }
    }
    let mut queue: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    let mut qi = 0;
    while qi < queue.len() {
        let i = queue[qi];
        qi += 1;
        order.push(i);
        for &r in &dependents[i] {
            indeg[r] -= 1;
            if indeg[r] == 0 {
                queue.push(r);
            }
        }
    }
    for i in 0..n {
        if !order.contains(&i) {
            order.push(i);
        }
    }
    order
}
