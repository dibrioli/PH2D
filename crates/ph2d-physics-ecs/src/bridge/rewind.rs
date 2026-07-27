//! **Putting the clock BACK** — the rewind half of the dispatch (W1 + W1.5).
//!
//! Split out of `bridge.rs` for the LOC cap, along the seam the module already uses:
//! [`super::hold`] is what the bridge does when the clock STANDS STILL, and this is
//! what it does when the clock goes BACKWARDS. Both are behaviours of a
//! non-advancing timeline, and neither belongs in the middle of the stepping path.

use ph2d_ecs::SimWorld;
use ph2d_physics::PhysicsWorld;

use super::{PhysicsBridge, SceneAtTick};

impl PhysicsBridge {
    /// Put the world back at tick 0 and replay forward to `target`.
    ///
    /// rapier cannot step backwards, and the live `Transform` is no help —
    /// the readback has already overwritten it with the simulated pose. So
    /// each body carries the description it was SPAWNED with
    /// ([`BodyRef::rest`]); a fresh world built from those, replayed
    /// `target` steps, reproduces the state exactly (the sim is
    /// deterministic). `target == 0` is the common case — Reset — and costs
    /// no steps at all.
    ///
    /// **W1.5:** the checkpoint ring makes this `O(STRIDE)` instead of
    /// `O(target)`. The ring seeds the world from the newest cached state at
    /// or before `target` and only the remainder is replayed; on a miss (a
    /// target older than the cached window, and Reset, which is `target = 0`)
    /// the world is rebuilt from the bodies' rest descriptions — the path
    /// that shipped in W1, still the only correctness-critical one.
    pub(super) fn rewind_to(
        &mut self,
        sim: &mut SimWorld,
        target: u64,
        scene: &mut dyn SceneAtTick,
    ) {
        // ⚠️ **Um salto de relógio SOLTA a mão** (W-Grab, regra 2 de
        // `bridge::grab`): o cutucão é um gesto sobre a corrida que acabou de
        // terminar, e o replay abaixo é uma corrida NOVA a partir do estado
        // AUTORADO — arrastar a tralha da mão para dentro dela puxaria cada tick
        // replayado na direção de onde o cursor está AGORA. É também o que Reset
        // promete: voltar à cena que o artista autorou.
        self.release_grab();
        // The clock jumped, so whatever was touching describes a run that is over.
        // Reporting the difference across the jump would call a scrub a hundred
        // collisions; the rebuild at the end of the dispatch re-baselines in silence.
        self.discard_contact_history();
        // And the tuning high-water marks: a rebuild is where a new RUN begins,
        // so the hardest each joint was pulled describes a run that is over.
        // ⚠️ Deliberately NOT cleared by `hold` — pausing is exactly when the
        // artist stops to read the number.
        self.discard_joint_peaks();
        let anchor = self.ring.seed(&mut self.world, target);
        let (from, replayed) = match anchor {
            Some(tick) => (tick, target - tick),
            None => {
                self.rebuild_from_rest();
                (0, target)
            }
        };
        for i in 0..replayed {
            // ⚠️ The replay drives scene-owned bodies too, and skipping this
            // was the defect that made a scrub disagree with a play. A
            // kinematic body frozen at its rest pose for the whole replay is a
            // platform that is not where it was, so every dynamic body that
            // touched it lands somewhere else — measured at 3.4 cm on a partial
            // replay, and at "the box never travelled at all" on a ring miss,
            // which meant the answer also depended on whether the cache
            // happened to hold the anchor.
            //
            // With a scene that answers, this restores the invariant the whole
            // bridge rests on: the world is a function of the tick, given the
            // authored rest state AND the authored curves — both reproducible.
            scene.put(sim, from + i + 1);
            self.drive_kinematic(sim, 1.0);
            self.world.step();
            self.steps_taken += 1;
        }
        self.readback(sim);
        self.last_stepped = target;
    }

    /// Put the world back at tick 0 from the descriptions the bodies were
    /// spawned with.
    ///
    /// The live `Transform` is no help here — the readback has already
    /// overwritten it with the simulated pose — which is why every body
    /// carries its [`BodyRef::rest`].
    ///
    /// **Clears the ring**, because this hands out fresh rapier handles: a
    /// checkpoint captured before the rebuild indexes bodies through the old
    /// arena, and restoring it would leave the bridge's handles addressing
    /// nothing. The pose published would then be stale, in silence — the
    /// worst kind of wrong. (The handles very likely come back identical,
    /// insertion order being the same; "very likely" is not a thing to build
    /// a cache on, and clearing here costs nothing since the ring already
    /// missed.)
    pub(super) fn rebuild_from_rest(&mut self) {
        self.ring.clear();
        self.world = PhysicsWorld::new();
        self.settings.apply_to(&mut self.world);
        // BTreeMap → entity order, so the fresh handles are assigned in the
        // same deterministic order as the original spawn (HR-5).
        for b in self.bodies.values_mut() {
            b.handle = self.world.spawn_body(b.rest);
        }
        // Joints come back in the SAME call: the rewind replays its owed steps
        // immediately, and a replay missing the joints is a different
        // simulation — the chain would fall apart and re-assemble a frame later.
        self.respawn_joints_from_rest();
    }
}
