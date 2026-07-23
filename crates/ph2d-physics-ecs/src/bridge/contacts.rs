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

/// One pair TOUCHING right now, in entity space — what the overlay draws a `+` on.
///
/// The standing set: rebuilt every dispatch from the world's end-of-tick state
/// ([`ph2d_physics::PhysicsWorld::contact_reports`]). *Who began touching* is a
/// separate channel ([`ContactEvent`]), and *the flash that marks a beginning* is a
/// third ([`ContactFlash`]) — a beginning is an event, not a property of the standing
/// pair, and (W-TickContacts) a beginning can even belong to a pair that is no longer
/// in this list.
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
    /// N·s — *how hard the hit was* (W-ImpactForce). `>= impulse` always. A different
    /// channel from the load, so a hard hit on a lightly-loaded pair reads differently
    /// from a gentle touch on a heavy one. Straight through from
    /// [`ph2d_physics::ContactReport::impact`].
    pub impact: f32,
}

/// One begin-flash — a `×` the overlay draws for a few ticks after a pair began
/// touching, sized by the impact of the hit (W-ContactEvents' visible half,
/// W-ImpactForce's size).
///
/// A SEPARATE channel from [`BodyContact`], and that is the whole of W-TickContacts on
/// the visible side: a flash marks a BEGINNING, which is an event with a fixed lifetime,
/// not a property of a pair that happens to still be touching. Sourced from `Began`
/// transitions and decayed in SIM ticks by the bridge, it therefore flashes a pair for
/// its full life *whether or not it is still touching* — including a FAST touch that
/// began and ended inside one tick and never enters `contacts` at all. (The old flash
/// rode `BodyContact`, so a short bounce flashed for only the ticks it touched, and a
/// fast touch never flashed; both are fixed by moving here.)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ContactFlash {
    /// The pair that began — same fixed order as everywhere else, so a re-light of the
    /// same pair finds its own flash.
    pub a: Entity,
    pub b: Entity,
    /// Where the touch began, world units.
    pub point: [f32; 2],
    /// The impact peak of the beginning, N·s — the flash's size.
    pub impact: f32,
    /// Ticks since the beginning. The overlay sizes and expands the flash from this;
    /// the bridge drops it past [`CONTACT_FLASH_TICKS`].
    pub age_ticks: u64,
}

/// How many sim ticks a begin-flash lives.
///
/// It lives here, not in the overlay, because the bridge ages the flashes in SIM ticks
/// (in the stepping loop) and drops them past this — so the overlay never has to. The
/// overlay reads it to size the age expansion; a single source keeps the drop and the
/// draw agreeing. At the 60 Hz tick this is ~100 ms — long enough to catch, short enough
/// that a busy scene does not stay lit (the reason the old overlay chose it).
pub const CONTACT_FLASH_TICKS: u64 = 6;

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
/// **Diffed once per TICK, not once per dispatch** (W-TickContacts). A frame that owes
/// several ticks — catching up, or a forward scrub — steps the world tick by tick, and
/// the diff runs after EACH, so a pair that both began and ended inside that span is
/// reported, not lost between the endpoints. And the diff is against the world's
/// per-tick UNION ([`ph2d_physics::PhysicsWorld::tick_contacts`], the same sub-step
/// ledger that captures the impact peak), so a FAST touch — one the solver resolves and
/// pushes back out within a single tick, never touching at the tick's end — is caught in
/// the sub-step it was active and fires a `Began`. (The old per-dispatch diff, sampling
/// only `contact_reports`, missed both: measured, a 3 m drop's first landing was
/// invisible and an 8 m drop fired nothing at all.)
///
/// The only touch this cannot report is one that begins and ends within a SINGLE
/// sub-step — which the discrete solver cannot produce (a contact it never resolved
/// across a sub-step boundary is a tunnel, and preventing that is CCD's job).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ContactEvent {
    /// The two entities, in the same fixed order [`BodyContact`] uses.
    pub a: Entity,
    pub b: Entity,
    pub phase: ContactPhase,
    /// Where the touch was. For `Began`, the last-active sub-step's deepest point; for
    /// `Ended`, the LAST point the pair was known to touch at — the place they parted.
    pub point: [f32; 2],
    /// The load at that moment, in N·s. For `Ended`, the last load the pair carried.
    /// The load meter — for *how hard the hit was*, read [`Self::impact`].
    pub impulse: f32,
    /// The **impact peak** of this transition, in N·s — for a `Began`, how hard the
    /// pair hit; for an `Ended`, the last peak it was known at (W-ImpactForce). This
    /// is the number a hit sound sizes itself by, and it is the captured peak, not the
    /// settled load ([`ph2d_physics::ContactReport::impact`]). Real for every transition
    /// that fires, including the fast touches this wave now delivers.
    pub impact: f32,
}

/// What the bridge remembers about a pair between TICKS — the last place, load, and
/// peak it was seen at, so an `Ended` event can say where the pair parted and how hard
/// it last hit. The whole of the state that turns a standing set into transitions.
#[derive(Copy, Clone, Debug)]
pub(super) struct ContactMemo {
    point: [f32; 2],
    impulse: f32,
    impact: f32,
}

impl PhysicsBridge {
    /// The handle→entity map, rebuilt from `bodies`. Built on demand rather than
    /// maintained, the same call `triggers` makes — it is only needed when there is
    /// something to translate.
    pub(super) fn handle_map(&self) -> BTreeMap<(u32, u32), Entity> {
        let mut by_handle: BTreeMap<(u32, u32), Entity> = BTreeMap::new();
        for (e, b) in &self.bodies {
            by_handle.insert(b.handle.into_raw_parts(), *e);
        }
        by_handle
    }

    /// Diff the world's PER-TICK union of touching pairs against the standing set to
    /// emit [`contact_events`](Self::contact_events), and light a flash for each
    /// beginning. Called after EVERY `step` in the forward loop (W-TickContacts), so a
    /// touch that lives less than a whole tick is still reported — the source is
    /// [`tick_contacts`](ph2d_physics::PhysicsWorld::tick_contacts), the sub-step ledger,
    /// not the settled end-of-tick state a fast touch has already left.
    ///
    /// ## The trap this exists to avoid
    ///
    /// The standing set is recomputed from scratch, so a naive diff turns every
    /// DISCONTINUOUS clock move into a storm: scrub back over a settled stack and every
    /// pair looks brand new. But nothing began — the artist moved the clock. An event
    /// describes a transition **the simulation actually stepped through**, so any
    /// discontinuity ([`rewind_to`](super::PhysicsBridge::rewind_to),
    /// [`hold`](super::PhysicsBridge::hold)) drops `contacts_continuous`, and the next
    /// forward tick RE-BASELINES in silence.
    ///
    /// ⚠️ The baseline starts **empty and continuous**, so the first stepped tick emits
    /// `Began` for whatever it finds — a stack authored already resting reports its
    /// contact (Unity's reading: the narrow phase had never run, so there is no earlier
    /// truth to compare against). The re-baseline path (`!continuous`) fires only after a
    /// discontinuity has explicitly dropped the flag.
    pub(super) fn accumulate_contact_events(&mut self, by_handle: &BTreeMap<(u32, u32), Entity>) {
        // Age the flashes one tick; drop those the overlay would no longer draw so the
        // list stays bounded. A pair that begins again re-lights its own flash to age 0.
        self.flashes.retain_mut(|f| {
            f.age_ticks += 1;
            f.age_ticks < CONTACT_FLASH_TICKS
        });

        // `now` = the per-tick union, mapped to entities. `BTreeMap`, not a `Vec`,
        // because the diff below is a SET operation and its iteration order is the order
        // events come out in — deterministic, like everything the bridge publishes.
        let mut now: BTreeMap<(Entity, Entity), ContactMemo> = BTreeMap::new();
        for (key, sample) in self.world.tick_contacts() {
            let (Some(&a), Some(&b)) = (by_handle.get(&key.0), by_handle.get(&key.1)) else {
                continue;
            };
            now.insert(
                (a, b),
                ContactMemo {
                    point: sample.point,
                    impulse: sample.impulse,
                    impact: sample.impact,
                },
            );
        }

        if !self.contacts_continuous {
            // Re-baseline after a discontinuity: adopt the set without reporting any of
            // it as having begun, and light nothing.
            self.contact_since = now;
            self.contacts_continuous = true;
            return;
        }

        // ENDED first, then BEGAN — a fixed order, so a consumer draining the queue sees
        // the same sequence on every machine. (The two halves cannot name the same pair,
        // so the order is presentation, not semantics.)
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
                // The visible half: a beginning flashes, whether or not it survives to
                // the tick's end.
                self.light_flash(a, b, memo.point, memo.impact);
            }
        }
        self.contact_since = now;
    }

    /// Re-light this pair's begin-flash to age 0, or start one — so a pair that bounces
    /// rapidly shows ONE flash, re-lit, rather than a growing pile of them.
    fn light_flash(&mut self, a: Entity, b: Entity, point: [f32; 2], impact: f32) {
        if let Some(f) = self.flashes.iter_mut().find(|f| f.a == a && f.b == b) {
            f.point = point;
            f.impact = impact;
            f.age_ticks = 0;
        } else {
            self.flashes.push(ContactFlash {
                a,
                b,
                point,
                impact,
                age_ticks: 0,
            });
        }
    }

    /// Rebuild [`contacts`](Self::contacts) (the standing `+` crosses) from the world's
    /// end-of-tick touching pairs. Runs once at the END of a dispatch, whichever branch
    /// produced the state — a scrub still publishes what touches AT the tick the artist
    /// is looking at. Empty (and no map built) when nothing touches.
    ///
    /// Purely the standing set: the transitions and flashes are the forward loop's job
    /// (`accumulate_contact_events`), which is the only place the clock moved through
    /// them.
    pub(super) fn rebuild_standing_contacts(&mut self) {
        self.contacts.clear();
        let reports = self.world.contact_reports();
        if reports.is_empty() {
            return;
        }
        let by_handle = self.handle_map();
        for r in reports {
            let (Some(&a), Some(&b)) = (
                by_handle.get(&r.body1.into_raw_parts()),
                by_handle.get(&r.body2.into_raw_parts()),
            ) else {
                continue;
            };
            self.contacts.push(BodyContact {
                a,
                b,
                point: r.point,
                impulse: r.impulse,
                impact: r.impact,
            });
        }
    }

    /// Forget what was touching, WITHOUT reporting any of it as having ended — and put
    /// out any live flashes, since a scrub or a disarm is not a moment to be lighting up.
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
        self.flashes.clear();
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
    /// Empty on any frame where nothing began or ended, and empty on the frame after a
    /// discontinuity (see [`Self::accumulate_contact_events`]). Ordered `Ended` then
    /// `Began` within each tick, each half sorted by entity pair; a multi-tick dispatch
    /// concatenates its ticks in order.
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

    /// The live begin-flashes — the `×` marks the overlay draws for a few ticks after a
    /// pair began touching, sized by the impact of the hit ([`ContactFlash`]).
    ///
    /// A SEPARATE channel from [`Self::contacts`] because a flash marks a BEGINNING and
    /// lives a fixed span whether or not the pair still touches — so a FAST touch, one
    /// that never enters `contacts`, still flashes (W-TickContacts). Aged in sim ticks
    /// and dropped past [`CONTACT_FLASH_TICKS`]; cleared by a discontinuity.
    #[must_use]
    pub fn contact_flashes(&self) -> &[ContactFlash] {
        &self.flashes
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
