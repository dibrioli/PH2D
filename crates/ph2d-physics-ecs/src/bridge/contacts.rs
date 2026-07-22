//! **Contact state** (W-Contacts) — which entities are touching, where, and under
//! how much load.
//!
//! The solid sibling of [`super::triggers`], and deliberately the same shape: the
//! world reports body HANDLES ([`ph2d_physics::PhysicsWorld::contact_reports`]) and
//! this turns them into entities the overlay can draw. Split out of `bridge.rs` for
//! the same LOC reason, `impl PhysicsBridge` here like the other submodules.
//!
//! ## Why a flat list and not a map
//!
//! `triggers` is a `BTreeMap<sensor, Vec<inside>>` because a trigger is asked about
//! ONE entity ("is this sensor firing?"). A contact has no owner — it is a
//! *relationship*, symmetric by construction — so the honest shape is the list of
//! relationships, in the deterministic order the world already sorted them into. The
//! per-entity question is answered by scanning it, which is what
//! [`PhysicsBridge::contact_count`] does; at the counts this module sees that is
//! cheaper than maintaining a second index of the same fact.

use std::collections::BTreeMap;

use ph2d_ecs::Entity;

use super::PhysicsBridge;

/// One touching pair, in entity space — what the overlay draws.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyContact {
    /// The two entities, in the world's handle order (so a pair reads the same
    /// whichever way the narrow phase reported it).
    pub a: Entity,
    pub b: Entity,
    /// The deepest contact point, in world units.
    pub point: [f32; 2],
    /// The load this pair is carrying right now, in N·s — what the standing cross's
    /// size means. **Not the impact peak** — see [`Self::impact`].
    pub impulse: f32,
    /// The **peak** normal impulse this pair reached during the tick's sub-steps, in
    /// N·s — *how hard the hit was* (W-ImpactForce). `>= impulse` always. This is what
    /// the flash's size means, and it is a different channel from the load precisely
    /// so a hard hit on a lightly-loaded pair reads differently from a gentle touch on
    /// a heavy one. Straight through from [`ph2d_physics::ContactReport::impact`].
    pub impact: f32,
    /// Ticks since this pair BEGAN touching — the age the overlay fades its flash
    /// over.
    ///
    /// `None` means *"already touching when the timeline last jumped"*: a scrub or a
    /// disarm re-baselines the set without inventing a beginning for it (see
    /// [`PhysicsBridge::rebuild_contacts`]). Keeping that distinct from `Some(0)` is
    /// what stops a scrub from flashing every contact in the scene at once — the age
    /// is a fact about the SIMULATION, not about how long the readback has known.
    pub age_ticks: Option<u64>,
}

/// Which end of a contact's life an event describes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ContactPhase {
    /// The pair was not touching last tick and is touching now.
    Began,
    /// The pair was touching last tick and is not touching now — including
    /// because one of the two bodies was deleted, so an `Ended` event may name an
    /// entity that no longer exists. That is the honest report: the contact ended,
    /// and *why* is the caller's question.
    Ended,
}

/// A contact TRANSITION — the thing gameplay consumes (an impact sound, damage, a
/// trigger), as opposed to [`BodyContact`], which is a standing state.
///
/// ⚠️ **Sampled once per dispatch, not once per tick.** A frame that owes several
/// ticks (catching up, or a forward scrub) reports the difference between its two
/// ENDPOINTS, so a pair that both began and ended inside that span is never seen.
/// The alternative — reading the contact graph after every sub-tick — costs the read
/// in every scene to serve a case only a stalled frame produces, and it was not
/// measured to be worth that. Documented rather than hidden, because a consumer that
/// counts hits needs to know the sampling rate of its own input.
///
/// ⚠️ **And the sharper edge of the same fact: a FAST impact produces no event at
/// all.** The sample is taken after `step` returns, and the solver resolves a hard
/// landing and pushes the body back out WITHIN that step — measured on a ball of
/// radius 0.3 dropped 3.9 m, which reaches y = -0.478 against a contact depth of
/// -0.5 and is already climbing when the tick ends. So the pair is not touching at
/// either sampling instant, and the landing that a person plainly sees is invisible
/// here. The boundary was swept: dropped from 1.2 m (~5.8 m/s) every bounce is
/// reported; from 2.0 m (~7.0 m/s) the first one is not. Smoke scene 29 drops from
/// within the reported range on purpose, and says so.
///
/// **This is the same mechanism as the missing impulse peak** ([`Self::impulse`]),
/// and it has the same cure: sampling INSIDE the sub-step loop. Whoever builds "real
/// impact force" gets this for free and should take both — they are one wave, not
/// two, and pricing it is the open question (it is a cost paid by every scene for a
/// reading only some scenes want).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ContactEvent {
    /// The two entities, in the same fixed order [`BodyContact`] uses.
    pub a: Entity,
    pub b: Entity,
    pub phase: ContactPhase,
    /// Where the touch was. For `Began`, this tick's deepest point; for `Ended`, the
    /// LAST point the pair was known to touch at — the place they parted.
    pub point: [f32; 2],
    /// The load at that moment, in N·s. For `Ended`, the last load the pair carried.
    /// The load meter — for *how hard the hit was*, read [`Self::impact`].
    pub impulse: f32,
    /// The **impact peak** of this transition, in N·s — for a `Began`, how hard the
    /// pair hit; for an `Ended`, the last peak it was known at (W-ImpactForce). This
    /// is the number a hit sound sizes itself by, and it is the captured peak, not the
    /// settled load ([`ph2d_physics::ContactReport::impact`]).
    ///
    /// ⚠️ **Scope of this wave:** the peak is real for every transition that FIRES,
    /// but a *fast* impact — one that both begins and ends inside a single tick —
    /// still fires no event at all (the standing-set diff never sees it), so its
    /// peak, though captured in the world, is not delivered here. Making a fast impact
    /// eventful restructures the diff (standing set → per-tick touched set) and is the
    /// next wave; the capture this one adds is its prerequisite.
    pub impact: f32,
}

/// What the bridge remembers about a pair BETWEEN dispatches — the whole of the
/// state that turns a standing set into transitions.
#[derive(Copy, Clone, Debug)]
pub(super) struct ContactMemo {
    /// Tick the pair began touching, or `None` when it was already touching at the
    /// last re-baseline (see [`BodyContact::age_ticks`]).
    began: Option<u64>,
    /// Last known place, load, and impact peak, so an `Ended` event can say where
    /// they parted and how hard the pair last hit.
    point: [f32; 2],
    impulse: f32,
    impact: f32,
}

impl PhysicsBridge {
    /// Rebuild [`contacts`](Self::contacts) from the world's current touching pairs,
    /// and DIFF it against the previous dispatch to produce
    /// [`contact_events`](Self::contact_events).
    ///
    /// Returns early — before building the handle→entity map — when nothing is
    /// touching AND nothing was touching, which is every frame of a scene in free
    /// fall. The map is built here rather than maintained every frame for the same
    /// reason `triggers` does it: it is only needed when there is something to
    /// translate.
    ///
    /// ## The trap this function exists to avoid
    ///
    /// The standing set is recomputed from scratch every dispatch, so a naive diff
    /// turns every DISCONTINUOUS clock move into a storm of events: scrub backwards
    /// over a settled stack and each pair the replay lands on looks brand new. But
    /// nothing began — the artist moved the clock. An event describes a transition
    /// **the simulation actually stepped through**, so any discontinuity
    /// ([`rewind_to`](super::PhysicsBridge::rewind_to) and
    /// [`hold`](super::PhysicsBridge::hold)) drops `contacts_continuous`, and the next
    /// rebuild re-baselines in SILENCE.
    ///
    /// ⚠️ The baseline starts **empty and continuous**, so the first stepped frame
    /// emits `Began` for whatever it finds — including a stack authored already
    /// resting. That matches Unity (`OnCollisionEnter` fires on the first
    /// `FixedUpdate` for pre-touching bodies) and is the only defensible reading: the
    /// narrow phase had never run before, so there is no earlier truth to compare to.
    pub(super) fn rebuild_contacts(&mut self) {
        self.contacts.clear();
        self.contact_events.clear();
        let reports = self.world.contact_reports();
        if reports.is_empty() && self.contact_since.is_empty() {
            self.contacts_continuous = true;
            return;
        }
        let tick = self.last_stepped;
        let mut by_handle: std::collections::BTreeMap<(u32, u32), Entity> = Default::default();
        for (e, b) in &self.bodies {
            by_handle.insert(b.handle.into_raw_parts(), *e);
        }
        // Fresh memo per touching pair. `BTreeMap` and not a `Vec` because the
        // difference below is a SET operation, and because the order it iterates in
        // is the order `Ended` events come out in — deterministic, like everything
        // else the bridge publishes.
        let mut now: BTreeMap<(Entity, Entity), ContactMemo> = BTreeMap::new();
        for r in reports {
            let (Some(&a), Some(&b)) = (
                by_handle.get(&r.body1.into_raw_parts()),
                by_handle.get(&r.body2.into_raw_parts()),
            ) else {
                continue;
            };
            let began = if self.contacts_continuous {
                // Carry the beginning forward for a pair that was already touching;
                // a pair that was not is beginning NOW.
                self.contact_since
                    .get(&(a, b))
                    .map_or(Some(tick), |memo| memo.began)
            } else {
                // Re-baselining: the set is adopted without inventing a beginning
                // for any of it.
                None
            };
            now.insert(
                (a, b),
                ContactMemo {
                    began,
                    point: r.point,
                    impulse: r.impulse,
                    impact: r.impact,
                },
            );
            self.contacts.push(BodyContact {
                a,
                b,
                point: r.point,
                impulse: r.impulse,
                impact: r.impact,
                age_ticks: began.map(|t| tick.saturating_sub(t)),
            });
        }
        if self.contacts_continuous {
            // ENDED first, then BEGAN — a fixed order, so a consumer draining the
            // queue sees the same sequence on every machine. (The two halves cannot
            // name the same pair, so the order is presentation, not semantics.)
            for (&(a, b), memo) in &self.contact_since {
                if !now.contains_key(&(a, b)) {
                    self.contact_events.push(ContactEvent {
                        a,
                        b,
                        phase: ContactPhase::Ended,
                        point: memo.point,
                        impulse: memo.impulse,
                        impact: memo.impact,
                    });
                }
            }
            for (&(a, b), memo) in &now {
                if !self.contact_since.contains_key(&(a, b)) {
                    self.contact_events.push(ContactEvent {
                        a,
                        b,
                        phase: ContactPhase::Began,
                        point: memo.point,
                        impulse: memo.impulse,
                        impact: memo.impact,
                    });
                }
            }
        }
        self.contact_since = now;
        self.contacts_continuous = true;
    }

    /// Forget what was touching, WITHOUT reporting any of it as having ended.
    ///
    /// The two callers are the two discontinuities: a scrub/Reset
    /// ([`rewind_to`](super::PhysicsBridge::rewind_to)) and disarming the transport's
    /// Physics toggle ([`hold`](super::PhysicsBridge::hold)). Both move the clock in a
    /// way the simulation did not step through, and the events channel only reports
    /// what it stepped through.
    pub(super) fn discard_contact_history(&mut self) {
        self.contacts.clear();
        self.contact_events.clear();
        self.contact_since.clear();
        self.contacts_continuous = false;
    }

    /// Every pair of entities touching right now — what the overlay draws a spark on.
    /// Sorted (the world sorted the handles, and the map preserves that order). Empty
    /// in a scene where nothing touches.
    #[must_use]
    pub fn contacts(&self) -> &[BodyContact] {
        &self.contacts
    }

    /// The contact TRANSITIONS of this dispatch — who started touching, who stopped.
    ///
    /// Empty on any frame where the standing set did not change, and empty on the
    /// frame after a discontinuity (see [`Self::rebuild_contacts`]). Ordered
    /// `Ended` then `Began`, each half sorted by entity pair.
    ///
    /// ⚠️ **This is the channel a gameplay consumer would drain** (an impact sound, a
    /// timeline marker, a script callback) — and that consumer is deliberately NOT
    /// built here: it is cross-line and its design is the Enio's call. What this wave
    /// owes is the primitive plus a VISIBLE reading of it (the overlay's flash), so
    /// the channel is not a dead flag — the precedence W7 set.
    #[must_use]
    pub fn contact_events(&self) -> &[ContactEvent] {
        &self.contact_events
    }

    /// How many bodies `entity` is touching right now. A scan, not an index — see the
    /// module header for why a contact has no owner to key a map on.
    #[must_use]
    pub fn contact_count(&self, entity: Entity) -> usize {
        self.contacts
            .iter()
            .filter(|c| c.a == entity || c.b == entity)
            .count()
    }
}

impl PhysicsBridge {
    /// A linha d'água de cada zona com empuxo — o que o overlay desenha (W-Buoyancy).
    ///
    /// Passagem direta para [`ph2d_physics::PhysicsWorld::waterlines`]: a shell não
    /// alcança o `PhysicsWorld`, e a ponte não tem opinião nenhuma sobre a superfície —
    /// ela é da física, calculada pela mesma função que o empuxo usa.
    #[must_use]
    pub fn waterlines(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.world.waterlines()
    }
}
