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

    /// **O collider que uma PEÇA tem no solver AGORA** — `None` se ela não está
    /// pendurada (W-PartFace).
    ///
    /// ⚠️ Existe para gatear **CHURN**, que nenhum outro oráculo enxerga: uma
    /// peça despendurada e re-pendurada a cada frame simula *quase* igual (ela
    /// assenta, o tremor medido foi 0,0), então nem pose nem aparência
    /// denunciam. O que muda é o HANDLE — e um handle que anda é a definição
    /// exata do defeito.
    ///
    /// Medido antes da correção, numa descida de rampa de 301 ticks: **133**
    /// re-pendurações pela pose de MUNDO dentro do `BodyDesc` (que `attach_part`
    /// nem lê) e **107** pelo `local` derivado por round-trip no solver. Com as
    /// duas curas: **1**, o spawn.
    #[doc(hidden)]
    #[must_use]
    pub fn part_handle(&self, e: ph2d_ecs::Entity) -> Option<ph2d_physics::ColliderHandle> {
        self.parts.get(&e).map(|p| p.handle)
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
        // `readback_order` entrou com a correção de ordem (ancestral antes de
        // descendente): ele cresce até o número de corpos uma vez e para.
        self.seen.capacity() + self.chain.capacity() + self.readback_order.capacity()
    }
}
