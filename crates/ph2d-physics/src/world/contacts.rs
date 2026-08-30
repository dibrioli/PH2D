//! **Contact readback — who hit whom, where, and how hard** (ADR-0131 W-Contacts).
//!
//! The solid counterpart of [`super::sensors`]. A sensor answers *"who is inside
//! me"* for a collider that passes through; this answers *"who am I touching"* for
//! one that does not — and, unlike an overlap, a contact has a **place** and a
//! **load**.
//!
//! ## Read-only, and that is the whole contract
//!
//! Nothing in [`super::PhysicsWorld::step`] calls this. It reads the narrow phase
//! the solver just filled and allocates a `Vec` for the caller; the world is not
//! touched, so installing it cannot move a single body — the C9 hash is unchanged
//! by construction, and there is a gate that asks the world for its hash before and
//! after a full read to keep it that way. That is why this wave adds **no** bodies
//! to the C9 harness: there is nothing new on the deterministic path to prove.
//!
//! ## Why one report per PAIR and not per contact point
//!
//! A box resting flat on the floor has **two** contact points (the two corners),
//! and a polygon can have more. Reporting each would answer *"how many corners are
//! touching"*, which is a fact about tessellation, not about the scene — two
//! objects touching is ONE event, and that is the question an artist (or, later, a
//! gameplay consumer) asks. The report carries the **deepest** point, which is where
//! the collision most is, and the **summed** normal impulse, which is how hard the
//! whole pair pushed.
//!
//! ⚠️ **That law held for contact POINTS and broke for SHAPES** (W-CompoundContact).
//! `contact_pairs()` iterates COLLIDER pairs, and until the W-Compound a body had
//! exactly one collider, so *"one entry per collider pair"* and *"one entry per body
//! pair"* were the same sentence. With a compound body they stop being: two rafts of
//! identical silhouette and identical mass resting on the same floor were measured at
//! **1 report / impulse 0.061313** (one-piece) against **2 reports / 0.030677 +
//! 0.030636** (two planks) — the overlay drawing two half-size crosses for one touch,
//! and `contact_count` answering *how many planks*, which is a fact about how the
//! artist decomposed the body. The merge is now this module's job, and the law above
//! is enforced at the level it was always about. Same aged premise as W-PartSensor and
//! W-CompoundZone, in the third channel.

use std::collections::BTreeMap;

use crate::rmath::{Rotation, Vector};
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::{ColliderSet, NarrowPhase};

use super::PhysicsWorld;

/// A body pair keyed for the peak map — the two handles' raw parts, lower first,
/// so the key matches the order [`ContactReport`] reports the pair in. `BTreeMap`
/// wants `Ord`, and this derives it (unlike the handles themselves).
///
/// `pub` because the bridge diffs the per-tick union ([`PhysicsWorld::tick_contacts`])
/// against its standing set, and it already speaks handle raw parts (its own
/// handle→entity map is keyed the same way).
pub type PeakKey = ((u32, u32), (u32, u32));

/// What the world remembers about one pair over the sub-steps of a single tick — the
/// data an EVENT needs (W-TickContacts), which the settled end-of-tick state cannot
/// carry for a pair that lifted off before the last sub-step.
///
/// `impact` is the hardest the pair pushed at any sub-step (the peak, W-ImpactForce);
/// `point`/`impulse` are the LAST sub-step it was active in. For a pair still touching
/// at the tick's end, "last active" IS the end, so these agree with
/// [`PhysicsWorld::contact_reports`]; for a FAST touch — one caught in a mid-tick
/// sub-step and gone by the last — they are the closest instant the world saw it, which
/// is where its event's place and load come from.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PeakSample {
    /// The peak normal impulse over the tick's sub-steps, N·s.
    pub impact: f32,
    /// The deepest contact point at the last sub-step the pair was active in, world units.
    pub point: [f32; 2],
    /// The surface normal at that point, world units, **pointing from `body1` to
    /// `body2`** — see [`ContactReport::normal`], same law, same door.
    pub normal: [f32; 2],
    /// The summed normal impulse at that same last-active sub-step, N·s.
    pub impulse: f32,
}

/// One touching pair, in plain data — no rapier type escapes except the body
/// handles the caller already speaks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ContactReport {
    /// The two bodies, in a fixed order (lower handle first) so a pair reads the
    /// same whichever way the narrow phase happened to report it.
    pub body1: RigidBodyHandle,
    pub body2: RigidBodyHandle,
    /// The **deepest** contact point, in world units. Where the touch most is.
    pub point: [f32; 2],
    /// The surface normal at [`Self::point`], world units, unit length —
    /// ***pointing from [`Self::body1`] toward [`Self::body2`]***.
    ///
    /// This is the *"against what did I hit"* half that a place and a load cannot
    /// answer: it is what orients a spark, decides *wall or floor*, and lets a
    /// caller slide along what it struck (the `normal` of Godot's
    /// `get_last_slide_collision` and Unity's `OnControllerColliderHit`).
    ///
    /// ⚠️ **The direction is stated here because it CANNOT be inferred.** The pair
    /// is published in handle order, which has nothing to do with which of the two
    /// the narrow phase happened to call *collider1*; a reader who is `body2`
    /// negates. Publishing rapier's raw `local_n1` would make the sign a coin flip
    /// from the caller's side — the same trap [`crate::CharacterHit::normal`] names
    /// and the one-way hook already pays for with its `s = -1`.
    ///
    /// ⚠️ **It is the normal of the DEEPEST contact, not an average.** A compound
    /// body touching a corner has two manifolds with perpendicular normals, and
    /// their mean points somewhere no surface faces; the deepest one is the same
    /// choice [`Self::point`] already makes, and the two must agree because they
    /// describe one contact.
    pub normal: [f32; 2],
    /// The summed normal impulse over the pair's manifolds, in N·s — **the load this
    /// pair is carrying right now**.
    ///
    /// ⚠️ **Not the impact peak** — see [`Self::impact`], which is. A ball landing from
    /// 6 m reports the same *impulse* as the same ball sitting still (0.010032237 vs
    /// 0.010032236): `step` returns after the solver has already stopped the body, so
    /// what survives out here is the settled load, not the peak.
    ///
    /// What it IS is exactly and usefully physical: in a stack of four identical
    /// boxes the impulses come out **4 : 3 : 2 : 1** from the floor up, because the
    /// bottom contact holds four boxes and the top one holds one. It is a load meter,
    /// and that is what the overlay's standing cross size means.
    pub impulse: f32,
    /// The **peak** normal impulse this pair reached during the tick's sub-steps, in
    /// N·s — *how hard the hit was*, as opposed to [`Self::impulse`], which is the load
    /// it settled to (W-ImpactForce).
    ///
    /// This is the number a hit sound wants. It exists because [`Self::impulse`]
    /// **cannot** carry it: the impact happens *between* the sub-steps and is gone by
    /// the time `step` returns. Captured by a `max` over the sub-steps inside the step
    /// loop (measured: ≤ 1.93% of the HR-4 budget at 500 contact pairs, so always-on);
    /// see [`PhysicsWorld::step`].
    ///
    /// **`impact >= impulse` always** for a live pair: the tick's peak is at least its
    /// endpoint. For a pair that is touching now but whose peak was not captured (it
    /// began touching only on the readback, never during a stepped sub-step — which the
    /// current step loop cannot produce), this falls back to `impulse`.
    pub impact: f32,
}

/// The ordered body pair, deepest world point, and summed normal impulse of one
/// **actively touching** contact pair — or `None` for a near-miss (bounding
/// volumes overlap, no active manifold point).
///
/// The single door both readers go through, so `contact_reports` (the load meter)
/// and `accumulate_peaks` (the impact capture) cannot disagree about *which*
/// pairs are touching, in *what* order, at *what* load — a second copy of this
/// collider→body→order logic is exactly the kind that drifts.
fn active_pair(
    pair: &rapier2d::geometry::ContactPair,
    colliders: &ColliderSet,
) -> Option<ActivePair> {
    // ⚠️ rapier 0.35: `has_any_active_contact` deixou de ser um CAMPO e passou a ser um método.
    if !pair.has_any_active_contact() {
        return None;
    }
    let c1 = colliders.get(pair.collider1)?;
    let c2 = colliders.get(pair.collider2)?;
    let (b1, b2) = (c1.parent()?, c2.parent()?);
    // ⚠️ `local_p1` is in **collider1's** frame, so it has to go through
    // collider1's world position — the same whose-frame-is-this care the one-way
    // hook pays, and for the same reason: the pair is not ordered for us.
    let (manifold, deepest) = pair.find_deepest_contact()?;
    let world = c1.position() * deepest.local_p1;
    let impulse = pair.total_impulse_magnitude();
    let swapped = b1.into_raw_parts() > b2.into_raw_parts();
    let (body1, body2) = if swapped { (b2, b1) } else { (b1, b2) };
    // ⚠️ **A normal do manifold é `local_n1`: no frame de COLLIDER1 e apontando
    // de 1 para 2** — a mesma convenção que o hook one-way lê ao comparar contra
    // o *up* da plataforma (`oneway.rs`). Duas conversões, e nenhuma é opcional:
    //
    //  1. **para o MUNDO**, pela rotação de collider1 — a mesma
    //     de-que-frame-é-isto que o `local_p1` acima já paga;
    //  2. **para a ORDEM PUBLICADA**, negando quando o par de corpos foi trocado
    //     para caber em (handle menor primeiro).
    //
    // Sem (2) o SINAL vira cara-ou-coroa contra o `a`/`b` que o leitor recebe —
    // exactamente o perigo que o [`crate::CharacterHit::normal`] nomeia (*qual
    // das duas testemunhas produz a normal é convenção da biblioteca*) e que o
    // one-way já pagou com o `s = -1`.
    let normal = published_normal(manifold.local_n1, c1.position().rotation, swapped);
    let (s1, s2) = (
        pair.collider1.into_raw_parts(),
        pair.collider2.into_raw_parts(),
    );
    Some(ActivePair {
        key: (body1.into_raw_parts(), body2.into_raw_parts()),
        seq: if s1 <= s2 { (s1, s2) } else { (s2, s1) },
        point: [world.x, world.y],
        normal,
        dist: deepest.dist,
        impulse,
    })
}

/// A normal de um manifold, levada ao MUNDO e à ORDEM PUBLICADA — a porta única
/// das duas conversões que separam `local_n1` do que um leitor recebe.
///
/// ⚠️ **Existe como função porque nenhuma CENA a distingue.** A troca de ordem
/// dispara de facto (medido: **976 vezes** numa cena com corpo composto), mas o
/// vencedor do teste de profundidade nunca foi um par trocado em nenhum fixture
/// deste repo — apagar o `swapped` deixa a suíte INTEIRA de `ph2d-physics-ecs`
/// verde. Uma defesa que nenhum gate pode ver não é uma defesa: extraída, ela
/// vira uma lei que se afirma directamente.
///
/// * `local_n1` está no frame de **collider1** e aponta de **1 para 2** — a mesma
///   convenção que o hook one-way lê ao compará-la com o *up* da plataforma.
/// * `rot` é a rotação de collider1: leva ao mundo.
/// * `swapped` diz que o par de CORPOS foi trocado para caber em *handle menor
///   primeiro*; então o que aponta de 1 para 2 aponta de `body2` para `body1`, e
///   a publicação (que promete `body1 → body2`) tem de negar.
fn published_normal(local_n1: Vector, rot: Rotation, swapped: bool) -> [f32; 2] {
    let n = rot * local_n1;
    let s = if swapped { -1.0 } else { 1.0 };
    [s * n.x, s * n.y]
}

/// One actively-touching **collider** pair, already resolved to bodies — the raw
/// material both readers fold into per-BODY answers (W-CompoundContact).
///
/// # Why this type exists at all
///
/// A compound body (W-Compound) touches through as many collider pairs as it has
/// shapes in reach, and every one of them resolves to the SAME body pair. The
/// narrow phase hands them over one at a time; turning that into *"these two objects
/// are touching, here, this hard"* is a **fold**, and the fold has to be written once
/// — the collider→body resolution already was ([`active_pair`]), and splitting the
/// merge out of it would put the drift one level up instead of removing it.
///
/// # `seq` fixes the SUM's order — and is NOT observable today
///
/// Merging means adding `f32` impulses, and float addition is **not associative**,
/// while `NarrowPhase::contact_pairs()` iterates rapier's internal graph — an order
/// nobody promised us. Sorting by (body pair, collider pair) makes the sum
/// **specified**, the same HR-5 care [`super::shapes::sorted_shapes`] takes one
/// module over.
///
/// ⚠️ **Measured: removing `seq` from the key leaves every gate green**, and that is
/// honest rather than a hole. A two-shape body has two addends and IEEE-754 addition
/// is **commutative** (only *associativity* fails), and on a slice this small
/// `sort_unstable` happens to keep the input order anyway. What `seq` buys is that
/// the order is *stated* instead of *observed*: it becomes load-bearing at three
/// shapes on the same pair, and it is exactly the kind of unspecified behaviour that
/// changes under a dependency update without anything failing. Documented, not gated
/// — the precedent of the CAS in ADR-0145.
#[derive(Copy, Clone, Debug)]
pub(super) struct ActivePair {
    /// The ordered body pair (lower handle first) — the key both readers group on.
    key: PeakKey,
    /// The ordered collider pair — the tiebreak that fixes the summation order.
    seq: ((u32, u32), (u32, u32)),
    point: [f32; 2],
    /// A normal em MUNDO, orientada **de `key.0` para `key.1`** — ver
    /// [`active_pair`], que é onde as duas conversões acontecem.
    normal: [f32; 2],
    /// Depth of `point` (negative = penetrating), so the DEEPEST contact across a
    /// compound body's shapes wins the merge — the literal extension of what one
    /// collider pair already answers with `find_deepest_contact`.
    dist: f32,
    impulse: f32,
}

/// Every actively-touching collider pair right now, resolved to bodies and sorted
/// into the fixed order [`ActivePair::seq`] documents. Written into `out`, which the
/// caller owns and reuses (the step loop's scratch keeps its capacity — the
/// hot-path zero-alloc gate).
fn collect_active(narrow_phase: &NarrowPhase, colliders: &ColliderSet, out: &mut Vec<ActivePair>) {
    out.clear();
    for pair in narrow_phase.contact_pairs() {
        if let Some(p) = active_pair(pair, colliders) {
            out.push(p);
        }
    }
    out.sort_unstable_by_key(|p| (p.key, p.seq));
}

/// Fold a sorted [`collect_active`] list into **one answer per BODY pair** — summed
/// impulse, deepest point — and hand each to `emit` in key order.
///
/// This is the module's own law, applied one level up from where it was written:
/// *two objects touching is ONE event*. A box resting flat reports one pair, not two
/// corners; a compound raft resting flat reports one pair, not one per plank. Both
/// are facts about how the thing was built, not about the scene.
fn for_each_body_pair(
    sorted: &[ActivePair],
    mut emit: impl FnMut(PeakKey, [f32; 2], [f32; 2], f32),
) {
    let mut i = 0;
    while i < sorted.len() {
        let key = sorted[i].key;
        let (mut point, mut normal, mut dist, mut impulse) =
            (sorted[i].point, sorted[i].normal, sorted[i].dist, 0.0);
        while i < sorted.len() && sorted[i].key == key {
            impulse += sorted[i].impulse;
            if sorted[i].dist < dist {
                dist = sorted[i].dist;
                point = sorted[i].point;
                // ⚠️ **A normal viaja COM o ponto, sob o mesmo teste de
                // profundidade** — as duas descrevem o MESMO contato, e escolhê-las
                // por critérios diferentes daria a orientação de um plano da jangada
                // no lugar onde o outro toca. Somá-las (uma média) seria pior: num
                // corpo composto encostado numa quina, a média de duas normais
                // perpendiculares aponta para um lado que nenhuma superfície tem.
                normal = sorted[i].normal;
            }
            i += 1;
        }
        emit(key, point, normal, impulse);
    }
}

/// Fold this instant's contact loads into the peak map by `max` — the impact
/// capture, called once per sub-step from [`PhysicsWorld::step`]. Keyed by the
/// **body** pair (lower first), so it lines up with [`ContactReport`] when the
/// readback reads it back out.
///
/// The `impact` is a `max` and not a sum: it is the hardest the pair pushed at any
/// single instant of the tick, not the total over the tick (which would grow with the
/// sub-step count and mean nothing physical). The `point`/`impulse`, by contrast, are
/// **overwritten** each sub-step, so at the tick's end they hold the LAST sub-step the
/// pair was active in — which is what a fast touch's event needs (W-TickContacts): a
/// place and a load for a pair that is no longer touching when `step` returns.
///
/// ⚠️ **The `max` is over SUB-STEPS, of the pair's SUMMED load** — never over the
/// pair's individual colliders (W-CompoundContact). Taking the max across shapes
/// would report a compound body's hit at the strength of its single hardest-hit
/// plank: measured, a two-plank raft landing read **0.0307 N·s** where the identical
/// one-piece raft read **0.0613**. Half, in the number a hit sound sizes itself by.
pub(super) fn accumulate_peaks(
    narrow_phase: &NarrowPhase,
    colliders: &ColliderSet,
    scratch: &mut Vec<ActivePair>,
    out: &mut BTreeMap<PeakKey, PeakSample>,
) {
    collect_active(narrow_phase, colliders, scratch);
    for_each_body_pair(scratch, |key, point, normal, impulse| {
        let s = out.entry(key).or_insert(PeakSample {
            impact: 0.0,
            point,
            normal,
            impulse,
        });
        s.impact = s.impact.max(impulse);
        s.point = point;
        s.normal = normal;
        s.impulse = impulse;
    });
}

impl PhysicsWorld {
    /// Every pair of bodies **actually touching** right now, deepest point and
    /// summed impulse each.
    ///
    /// **The near-miss band is real, and it was measured.** The contact graph keeps a
    /// pair alive while the *bounding volumes* overlap: two circles 0.566 apart (radii
    /// 0.25) are 1 pair in the graph with 0 active contacts, while the same circles
    /// 0.003 apart on an axis are not in the graph at all. Reporting the graph as-is
    /// would call the first pair a collision — the same distinction
    /// `intersecting_body_pairs` draws with its `intersecting` flag, and the one a
    /// box-shaped fixture cannot see (a box's shape IS its AABB).
    ///
    /// ⚠️ **Two layers honour that, and each alone is enough today** — the flag, and
    /// the `?` on `find_deepest_contact` (no manifold point, no report). Mutating
    /// EITHER leaves all six gates green; mutating BOTH turns the near-miss gate red,
    /// which is how the gate was proven to be about something
    /// ([[feedback_layered_defenses_need_per_layer_gates]]). The flag stays as the
    /// primary predicate because it is rapier's own statement of the fact — the
    /// lookup merely happens to imply it, and would stop implying it the day
    /// speculative manifold points are kept.
    ///
    /// **Sorted by handle**, because the contact graph's own order is an internal
    /// detail and a readout built on it would flicker frame to frame. Empty for a
    /// world where nothing touches, so a scene in free fall pays one iteration of
    /// an empty graph.
    #[must_use]
    pub fn contact_reports(&self) -> Vec<ContactReport> {
        // A `&self` readout that already allocates its answer, so it brings its own
        // scratch rather than borrowing the step loop's — the caller is not on the
        // hot path and the alternative is a `&mut self` on a pure query.
        let mut active = Vec::new();
        collect_active(&self.narrow_phase, &self.colliders, &mut active);
        let mut out = Vec::new();
        for_each_body_pair(&active, |key, point, normal, impulse| {
            // The impact peak the step loop captured for this pair. `impulse`
            // (the settled load) is the floor: the peak is at least the
            // endpoint, and a pair with no captured peak (impossible from the
            // current loop — a live pair touched the last sub-step) reads its
            // load.
            let impact = self
                .contact_peaks
                .get(&key)
                .map_or(impulse, |s| s.impact)
                .max(impulse);
            out.push(ContactReport {
                body1: RigidBodyHandle::from_raw_parts(key.0.0, key.0.1),
                body2: RigidBodyHandle::from_raw_parts(key.1.0, key.1.1),
                point,
                normal,
                impulse,
                impact,
            });
        });
        // Already in key order — `for_each_body_pair` walks a list sorted by
        // (body pair, collider pair), so the reports come out sorted by body pair
        // without a second pass.
        out
    }

    /// The **per-tick union** of touching pairs — every pair that had an active contact
    /// in *any* sub-step of the last [`step`](PhysicsWorld::step), with its peak, its
    /// last-active place, and its last-active load ([`PeakSample`]).
    ///
    /// This is the SUPERSET of [`contact_reports`](Self::contact_reports) (the end-of-tick
    /// live state): a pair that lands and rebounds within one tick is here, and is not
    /// there. The bridge diffs this against its standing set to report a fast touch that
    /// the end-of-tick state can never show (W-TickContacts). It is the same ledger the
    /// impact peak already accumulates — no extra work on the stepping path, and cleared
    /// (capacity kept) at the start of every `step`.
    #[must_use]
    pub fn tick_contacts(&self) -> &BTreeMap<PeakKey, PeakSample> {
        &self.contact_peaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    fn close(a: [f32; 2], b: [f32; 2]) -> bool {
        (a[0] - b[0]).abs() < EPS && (a[1] - b[1]).abs() < EPS
    }

    /// **As duas conversões são independentes, e cada uma é necessária.**
    ///
    /// ⚠️ Este gate existe porque **nenhuma cena as distingue** (ver o doc da
    /// [`published_normal`]): a troca de ordem dispara no mundo real mas o
    /// vencedor do teste de profundidade nunca é um par trocado em fixture
    /// nenhum deste repo. Aqui a lei é afirmada onde ela mora.
    #[test]
    fn the_published_normal_goes_to_the_world_and_to_the_published_order() {
        let x = Vector::new(1.0, 0.0);
        let id = Rotation::IDENTITY;
        let quarter = Rotation::new(std::f32::consts::FRAC_PI_2);

        // Sem rotação e sem troca: passa verbatim.
        assert!(close(published_normal(x, id, false), [1.0, 0.0]));
        // Só a rotação: o frame de collider1 girou um quarto de volta.
        assert!(close(published_normal(x, quarter, false), [0.0, 1.0]));
        // Só a troca: `1 -> 2` vira `body2 -> body1`, logo nega.
        assert!(close(published_normal(x, id, true), [-1.0, 0.0]));
        // As duas COMPÕEM — e é este caso que uma implementação que esqueça
        // uma delas acerta por acaso nos três de cima.
        assert!(close(published_normal(x, quarter, true), [0.0, -1.0]));
    }

    /// **A normal viaja com o PONTO, sob o mesmo teste de profundidade.**
    ///
    /// ⚠️ O fixture põe a entrada mais funda em **segundo** lugar e com uma
    /// normal PERPENDICULAR à da primeira — é o corpo composto encostado numa
    /// quina, o caso que o doc do [`for_each_body_pair`] descreve. Sem ele o
    /// vencedor seria sempre o primeiro da lista e a lei ficaria por afirmar.
    #[test]
    fn the_deepest_contact_brings_its_own_normal() {
        let key = ((1, 0), (2, 0));
        let pairs = [
            ActivePair {
                key,
                seq: ((1, 0), (2, 0)),
                point: [0.0, 0.0],
                normal: [0.0, 1.0],
                dist: -0.01,
                impulse: 1.0,
            },
            ActivePair {
                key,
                seq: ((1, 0), (3, 0)),
                point: [5.0, 5.0],
                normal: [1.0, 0.0],
                dist: -0.50,
                impulse: 2.0,
            },
        ];
        let mut got = None;
        for_each_body_pair(&pairs, |_, point, normal, impulse| {
            got = Some((point, normal, impulse));
        });
        let (point, normal, impulse) = got.expect("um par de corpos");
        assert!(close(point, [5.0, 5.0]), "o ponto mais fundo: {point:?}");
        assert!(
            close(normal, [1.0, 0.0]),
            "a normal tem de ser a DO ponto mais fundo, nao a da primeira \
             entrada: {normal:?}"
        );
        assert!((impulse - 3.0).abs() < EPS, "a carga SOMA: {impulse}");
    }
}
