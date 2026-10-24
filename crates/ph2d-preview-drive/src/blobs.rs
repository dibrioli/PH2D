//! ⭐ **Os COMPONENTES INTEIROS que um motor põe e tira** (macros W19 §2.4, `docs/Macros/17_W19_plano.md`) —
//! o irmão de [`crate::fields`], com a chave `(entidade, tipo)` e o valor como o blob postcard do registo
//! (`None` = ausente). A reprodução de uma macro do mundo põe o `Collider` que ainda não existia, tira o
//! que existe, esconde o objecto que ainda não nasceu — de qualquer componente registado.
//!
//! A lei é a do memo: o autorado é o 1.º `antes`; outra mão a meio passa a ser o autorado; a
//! [`BlobDrive::settle`] esquece quem não foi declarado neste quadro. As portas da vtable vêm com a
//! declaração, por isso a captura não precisa do registo.

use std::collections::BTreeMap;

use ph2d_ecs::scene::registry::{
    ComponentTypeEntry, ComponentTypeId, InsertFromBytesFn, RemoveFn, SerializeFn,
};
use ph2d_ecs::{Entity, SimWorld};

/// Um tipo registado, com as portas que o leem, põem e tiram.
#[derive(Clone, Copy, Debug)]
pub struct BlobRef {
    pub type_id: ComponentTypeId,
    pub insert_from_bytes: InsertFromBytesFn,
    pub serialize: SerializeFn,
    pub remove: RemoveFn,
}

impl BlobRef {
    #[must_use]
    pub fn of(entry: &ComponentTypeEntry) -> Self {
        Self {
            type_id: entry.type_id,
            insert_from_bytes: entry.insert_from_bytes,
            serialize: entry.serialize,
            remove: entry.remove,
        }
    }

    /// O blob na entidade; `None` de fora = a entidade não existe.
    #[must_use]
    pub fn le(&self, sim: &SimWorld, entity: Entity) -> Option<Option<Vec<u8>>> {
        (self.serialize)(sim.world(), entity).ok()
    }

    /// Põe `v` (ou tira, se `None`).
    pub fn poe(&self, sim: &mut SimWorld, entity: Entity, v: Option<&[u8]>) {
        match v {
            Some(b) => {
                let _ = (self.insert_from_bytes)(sim.world_mut(), entity, b);
            }
            None => (self.remove)(sim.world_mut(), entity),
        }
    }
}

#[derive(Clone)]
struct BlobEntry {
    authored: Option<Vec<u8>>,
    last_written: Option<Vec<u8>>,
    porta: BlobRef,
    /// Declarado desde a última [`BlobDrive::settle`].
    seen: bool,
    /// A ronda do motor em que foi declarado ([`BlobDrive::ronda`]).
    ronda: u64,
}

type Key = (u64, ComponentTypeId);

/// O ledger dos componentes conduzidos inteiros. ⚠️ `BTreeMap`: a ordem atravessa a captura.
#[derive(Default)]
pub struct BlobDrive {
    memo: BTreeMap<Key, BlobEntry>,
    ronda: u64,
}

/// O que a [`BlobDrive::substitute`] deslocou — o vivo, para o [`BlobDrive::restore`].
pub struct DisplacedBlobs(Vec<(u64, BlobRef, Option<Vec<u8>>)>);

impl BlobDrive {
    /// **O motor conduz o tipo `porta` de `entity`**: `antes` é o que lá estava, `depois` o que ele
    /// deixou. Sem entrada e sem mudança não nasce nada (não há o que devolver); com entrada, mantém-na.
    pub fn driven(
        &mut self,
        entity: Entity,
        porta: BlobRef,
        antes: Option<Vec<u8>>,
        depois: Option<Vec<u8>>,
    ) {
        let k = (entity.to_bits(), porta.type_id);
        let ronda = self.ronda;
        match self.memo.get_mut(&k) {
            None if antes == depois => {}
            None => {
                self.memo.insert(
                    k,
                    BlobEntry {
                        authored: antes,
                        last_written: depois,
                        porta,
                        seen: true,
                        ronda,
                    },
                );
            }
            Some(e) => {
                if e.last_written != antes {
                    e.authored = antes;
                }
                e.last_written = depois;
                e.seen = true;
                e.ronda = ronda;
            }
        }
    }

    /// **Lê, escreve se difere e declara** — o caminho de um motor por quadro. `false` = a entidade
    /// não existe.
    pub fn conduz(
        &mut self,
        sim: &mut SimWorld,
        entity: Entity,
        porta: BlobRef,
        depois: Option<&[u8]>,
    ) -> bool {
        let Some(antes) = porta.le(sim, entity) else {
            return false;
        };
        if antes.as_deref() != depois {
            porta.poe(sim, entity, depois);
        }
        self.driven(entity, porta, antes, depois.map(<[u8]>::to_vec));
        true
    }

    /// Abre uma ronda do motor: o que ele não declarar nela, a [`Self::devolve_os_largados`] devolve.
    pub fn ronda(&mut self) {
        self.ronda += 1;
    }

    /// **O motor largou-os** (a macro parou de tocar): os não declarados nesta ronda voltam ao
    /// autorado e saem. ⚠️ Não é a `settle`: o que a reprodução mostrou nunca vira documento.
    pub fn devolve_os_largados(&mut self, sim: &mut SimWorld) -> usize {
        let ronda = self.ronda;
        let largados: Vec<Key> = self
            .memo
            .iter()
            .filter(|(_, e)| e.ronda != ronda)
            .map(|(k, _)| *k)
            .collect();
        for k in &largados {
            if let Some(e) = self.memo.remove(k) {
                let ent = Entity::from_bits(k.0);
                if e.porta.le(sim, ent).is_some() {
                    e.porta.poe(sim, ent, e.authored.as_deref());
                }
            }
        }
        largados.len()
    }

    /// Uma vez por quadro: esquece quem não foi declarado.
    pub fn settle(&mut self) {
        self.memo.retain(|_, e| e.seen);
        for e in self.memo.values_mut() {
            e.seen = false;
        }
    }

    /// O mundo passa ao autorado para a captura; devolve o vivo.
    #[must_use]
    pub fn substitute(&self, sim: &mut SimWorld) -> DisplacedBlobs {
        let mut live = Vec::with_capacity(self.memo.len());
        for (&(bits, _), e) in &self.memo {
            let ent = Entity::from_bits(bits);
            let Some(now) = e.porta.le(sim, ent) else {
                continue;
            };
            if now != e.authored {
                e.porta.poe(sim, ent, e.authored.as_deref());
                live.push((bits, e.porta, now));
            }
        }
        DisplacedBlobs(live)
    }

    /// Devolve ao mundo o que a [`Self::substitute`] deslocou.
    pub fn restore(sim: &mut SimWorld, live: &DisplacedBlobs) {
        for (bits, porta, v) in &live.0 {
            porta.poe(sim, Entity::from_bits(*bits), v.as_deref());
        }
    }

    /// O autorado de um tipo conduzido (`None` de fora = não conduzido).
    #[must_use]
    pub fn autorado(&self, entity: Entity, type_id: ComponentTypeId) -> Option<Option<&[u8]>> {
        self.memo
            .get(&(entity.to_bits(), type_id))
            .map(|e| e.authored.as_deref())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.memo.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.memo.is_empty()
    }
}

#[cfg(test)]
#[path = "blobs_tests.rs"]
mod tests;
