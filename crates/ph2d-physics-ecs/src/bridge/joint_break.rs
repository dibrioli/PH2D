//! **The joint that gave way, on the ECS side** (ADR-0131 W-J7).
//!
//! The wrapper decides the break — inside the sub-step loop, because a break
//! noticed a frame late is a different simulation ([`ph2d_physics::JointBreak`]).
//! This half does the two things only the bridge can: turn a joint handle back
//! into the **entity** the artist selects, and keep the report alive for a frame
//! so the overlay and the toast can both see it.
//!
//! ## Why a channel and not a flag on the view
//!
//! A break is a **transition**, and the state that follows it cannot carry it:
//! a moment after parting, the joint reads a load of zero — it is not holding
//! anything. `JointView::broken` (the state) says *this joint is not holding*,
//! which is what the overlay draws every frame; `JointBreakEvent` (the
//! transition) says *it gave way right here, at this load*, which is what a
//! toast and a flash want. Same split W-TickContacts made for contacts, for the
//! same reason.
//!
//! ## A rewind un-breaks, and it has to
//!
//! Nothing here is authored state. `rebuild_from_rest` re-spawns every joint
//! from its descriptor, so a scrub back before the break brings the joint back
//! and re-breaks it at the same tick — the world stays a function of
//! `(tick, authored rest)`. That is also why the break never writes into
//! [`crate::PhysicsJoint`]: the solver writing authored state is what poisons
//! undo (a diff every frame) and what makes a replay disagree with the live run.

use ph2d_ecs::Entity;
use ph2d_physics::JointLoad;

use super::PhysicsBridge;

/// One joint that gave way, in the vocabulary the editor speaks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JointBreakEvent {
    /// The joint ENTITY — what the Hierarchy lists and the toast names.
    pub joint: Entity,
    /// Where it parted, world metres (the midpoint of the two anchors).
    pub point: [f32; 2],
    /// The reaction force at that instant, newtons.
    pub force: f32,
    /// The reaction torque at that instant, newton-metres.
    pub torque: f32,
}

impl PhysicsBridge {
    /// Fold the wrapper's breaks of the tick just stepped into this dispatch's
    /// list, mapping each handle back to its entity.
    ///
    /// Called from inside the stepping loop, next to `accumulate_contact_events`
    /// and for the same reason: a dispatch can owe several ticks, and a break in
    /// the third of them is not visible from the world's state at the end of the
    /// fifth (the wrapper clears its own list at the start of every `step`).
    pub(super) fn accumulate_joint_breaks(&mut self) {
        if self.world.joint_breaks().is_empty() {
            return;
        }
        // Handle → entity, from the bridge's own joint table. Walked rather than
        // indexed because a scene has a handful of joints where it has hundreds
        // of bodies, and the alternative is a second map to keep in step with
        // `self.joints` — which is exactly the kind that drifts.
        for b in self.world.joint_breaks() {
            if let Some((&entity, _)) = self.joints.iter().find(|(_, j)| j.handle == b.joint) {
                self.joint_breaks.push(JointBreakEvent {
                    joint: entity,
                    point: b.point,
                    force: b.force,
                    torque: b.torque,
                });
            }
        }
    }

    /// **Fold this frame's joint loads into the high-water mark of the RUN.**
    ///
    /// The per-tick peak the wrapper keeps ([`ph2d_physics::PhysicsWorld::joint_load`])
    /// is exact but *transient* — it describes one tick, and a yank is over
    /// before the artist can read it. What tuning a threshold needs is the
    /// hardest this joint has been pulled since the clock last restarted: play,
    /// do the thing, read the number, type it. That is a search with a feedback
    /// signal instead of a binary one.
    ///
    /// ⚠️ **Cleared only by a REWIND, not by a hold.** A rebuild is where a new
    /// run begins; pausing (or disarming the transport's Physics toggle) does not
    /// make the last run's number untrue — and the artist pauses *precisely to
    /// read it*, so clearing there would erase the answer at the moment it is
    /// wanted.
    ///
    /// ⚠️ A BROKEN joint stops contributing, and that is the whole readout of a
    /// break: the wrapper skips a disabled joint, so its live load reads zero
    /// while the high-water keeps the load that crossed. No special case — the
    /// number the artist wants to see frozen freezes itself.
    pub(super) fn accumulate_joint_peaks(&mut self) {
        for (&entity, j) in &self.joints {
            let Some(load) = self.world.joint_load(j.handle) else {
                continue;
            };
            let slot = self.joint_peaks.entry(entity).or_insert(JointLoad::ZERO);
            slot.force = slot.force.max(load.force);
            slot.torque = slot.torque.max(load.torque);
        }
    }

    /// The high-water load of this run, per joint. `ZERO` for one never stepped.
    #[must_use]
    pub(super) fn joint_peak(&self, entity: Entity) -> JointLoad {
        self.joint_peaks
            .get(&entity)
            .copied()
            .unwrap_or(JointLoad::ZERO)
    }

    /// Forget every high-water mark — a new run starts here.
    pub(super) fn discard_joint_peaks(&mut self) {
        self.joint_peaks.clear();
    }

    /// The joints that gave way during this dispatch. Empty in almost every
    /// frame — this is a transition channel.
    ///
    /// Drained by the shell (a toast) and read by the overlay (the burst). Not
    /// cleared by a hold or a rewind the way the contact baseline is: a break is
    /// *reported*, not *remembered*, so there is no stale baseline to re-adopt —
    /// the list is emptied at the top of every dispatch regardless.
    #[must_use]
    pub fn joint_breaks(&self) -> &[JointBreakEvent] {
        &self.joint_breaks
    }
}
