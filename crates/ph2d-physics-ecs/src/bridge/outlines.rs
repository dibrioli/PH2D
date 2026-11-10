//! A poda das peças do «Outline»: o mundo e o cache da ponte guardam só o que um corpo ou peça
//! viva usa (o ABERTO 2 do 10/11 — arrastar Detail ou escala registava um conjunto por valor).

use std::collections::BTreeSet;

use ph2d_physics::{OutlineKey, ShapeDesc};

use super::PhysicsBridge;

fn key_of(shape: ShapeDesc) -> Option<OutlineKey> {
    match shape {
        ShapeDesc::Outline { key, .. } => Some(key),
        _ => None,
    }
}

impl PhysicsBridge {
    /// Depois das peças (`prepare`): os `rest` dos corpos e das peças são as únicas chaves que um
    /// re-spawn ou um rewind ainda pedem.
    pub(super) fn sweep_outlines(&mut self) {
        let vivas = || {
            (self.bodies.values().map(|b| b.rest.shape))
                .chain(self.parts.values().map(|p| p.rest.shape))
                .filter_map(key_of)
        };
        if !self.outlines.needs_sweep(&self.world, vivas().count()) {
            return;
        }
        let live: BTreeSet<OutlineKey> = vivas().collect();
        self.outlines.sweep(&mut self.world, &live);
    }

    /// Os tamanhos das tabelas do Outline: peças Q4, conversões, peças no mundo (gates).
    #[doc(hidden)]
    #[must_use]
    pub fn outline_tables(&self) -> [usize; 3] {
        self.outlines.tables(&self.world)
    }
}
