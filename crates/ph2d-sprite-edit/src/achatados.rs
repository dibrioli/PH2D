//! ⭐ **Píxeis vindos de fora vencem as camadas do Painter** (decisão de 05/11, [ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)).
//! O objecto que recebe a imagem larga o documento, senão gravar e abrir re-assaria as camadas por cima; o Undo devolve-o.
//! O documento é `D`, e quem o tira e devolve é a composição (a folha não conhece o Painter).

use ph2d_asset::AssetId;
use ph2d_ecs::{Entity, PaintedDoc, SpritePixels, StableId, World};
use ph2d_render::{Sprite, texture_of};
use std::collections::BTreeMap;

/// O documento de um objecto: fora do dono enquanto o objecto mostra `depois`, de volta quando o Undo o devolve a `antes`.
struct Achatado<D> {
    id: StableId,
    antes: AssetId,
    depois: AssetId,
    guardado: Option<D>,
}

/// Os documentos à espera do Undo.
pub struct Achatados<D>(Vec<Achatado<D>>);

impl<D> Default for Achatados<D> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<D> Achatados<D> {
    /// Cada um de `quem` de que `tira` devolve documento passa de `antes` a `depois` sem ele. `true` = algum saiu: avisa-se.
    pub fn achata(
        &mut self,
        world: &mut World,
        quem: &[Entity],
        (antes, depois): (AssetId, AssetId),
        mut tira: impl FnMut(u64) -> Option<D>,
    ) -> bool {
        let mut algum = false;
        for e in quem {
            let Some(guardado) = tira(e.to_bits()) else {
                continue;
            };
            algum = true;
            let id = world.get::<StableId>(*e).copied();
            // O carimbo faria os píxeis próprios saltarem o ficheiro: «tem documento».
            world.entity_mut(*e).remove::<PaintedDoc>();
            if let Some(id) = id {
                self.0.push(Achatado {
                    id,
                    antes,
                    depois,
                    guardado: Some(guardado),
                });
            }
        }
        algum
    }

    /// Depois de um restauro: quem voltou a `antes` retoma o documento (`devolve`); quem voltou a `depois` larga-o (`tira`).
    pub fn reconcilia(
        &mut self,
        world: &mut World,
        atlas: &BTreeMap<u32, AssetId>,
        mut tira: impl FnMut(u64) -> Option<D>,
        mut devolve: impl FnMut(u64, D),
    ) {
        let quem: BTreeMap<StableId, Entity> = world
            .query::<(Entity, &StableId)>()
            .iter(world)
            .map(|(e, s)| (*s, e))
            .collect();
        for a in &mut self.0 {
            let Some(&e) = quem.get(&a.id) else {
                continue;
            };
            let mostra = texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas);
            if mostra == Some(a.antes) {
                if let Some(doc) = a.guardado.take() {
                    devolve(e.to_bits(), doc);
                }
            } else if mostra == Some(a.depois) && a.guardado.is_none() {
                a.guardado = tira(e.to_bits());
            }
        }
    }

    /// Um projecto novo: o que esperava era do que saiu.
    pub fn esquece(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
#[path = "achatados_tests.rs"]
mod tests;
