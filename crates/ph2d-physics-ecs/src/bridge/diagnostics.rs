//! Read-only diagnostics on [`super::PhysicsBridge`] — the cross-OS hash and
//! the scratch-capacity probe. Split out of `bridge.rs` to keep it under the
//! workspace LOC cap; a child module reaches the bridge's private fields exactly
//! as an inline method did.

use ph2d_ecs::{SimWorld, Transform};

use super::PhysicsBridge;

impl PhysicsBridge {
    /// A blake3 digest over the readback poses (the ECS-visible result of
    /// the whole bridge: sync + step + our conversion + readback). This is
    /// what the `physics_ecs_c9` harness prints and CI compares cross-OS
    /// (ADR-0131 D7) — it proves OUR code on the deterministic path, not
    /// just the wrapper's internal `deterministic_hash`.
    ///
    /// The `bodies` `BTreeMap` iterates in `Entity` order, which is
    /// deterministic per run and identical cross-OS (sequential entity
    /// allocation), so no sort is needed to pin the order.
    pub fn deterministic_hash(&self, sim: &SimWorld) -> [u8; 32] {
        let world = sim.world();
        let mut hasher = blake3::Hasher::new();
        for &e in self.bodies.keys() {
            if let Some(t) = world.get::<Transform>(e) {
                hasher.update(&t.translation.x.to_bits().to_le_bytes());
                hasher.update(&t.translation.y.to_bits().to_le_bytes());
                hasher.update(&t.rotation.to_bits().to_le_bytes());
            }
        }
        *hasher.finalize().as_bytes()
    }

    /// Capacity of the main per-frame scratch buffer — the zero-alloc gate
    /// asserts this is stable across steady-state frames (HR-3, capacity
    /// stability rather than a flaky global allocation counter).
    #[doc(hidden)]
    pub fn scratch_capacity(&self) -> usize {
        // Every per-frame buffer, summed — a gate that watches one of them
        // reports "no growth" while another doubles beside it. `chain` is the
        // ancestor walk W5 added; it grows to the deepest hierarchy once and
        // must never grow again.
        self.seen.capacity() + self.chain.capacity()
    }
}
