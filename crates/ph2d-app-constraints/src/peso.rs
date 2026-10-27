//! **O peso por distância no cartão** (W5.4, `docs/BlenderConstraints/06_W5_resto.md` §3.2): o «Weight
//! By» de um *behaviour* arma o clique no palco que escolhe o objecto (o idioma do «Target»); o
//! «Clear» tira-o.

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use ph2d_constraints::Kind;
use ph2d_constraints::comportamento::Peso;
use ph2d_constraints_ecs::{Constraints, Item};
use ph2d_ecs::{Name, SimWorld, StableId, World, entity_with_stable_id, stable_id_of};
use ph2d_editor_core::constraint_edits::PesoDaLinha;

use crate::inspector::Escolha;

/// O item cujo objecto do peso o próximo clique no palco escolhe: `(dono, Item::id)`.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PesoArmado(pub Option<(u64, u32)>);

/// O peso de fábrica ao escolher o objecto: cai de 1 em cima dele a 0 a este raio, Smooth.
const PESO_NOVO: Peso = Peso {
    objecto: 0,
    raio: 150.0,
    curva: 2,
    inverte: false,
};

pub(crate) fn armado(w: &World) -> Option<(u64, u32)> {
    w.get_resource::<PesoArmado>().and_then(|a| a.0)
}

/// Arma (ou, se já armado para este item, desarma) o clique do peso; o do alvo e o da cola caem.
pub(crate) fn alternar(sim: &mut SimWorld, dono: u64, id: u32) {
    let novo = (armado(sim.world()) != Some((dono, id))).then_some((dono, id));
    crate::inspector::armar(sim, None);
    crate::cola::armar(sim, None);
    sim.world_mut().insert_resource(PesoArmado(novo));
}

/// Tira o peso do item. `true` se mudou.
pub(crate) fn largar(sim: &mut SimWorld, e: Entity, id: u32) -> bool {
    sim.world_mut()
        .get_mut::<Constraints>(e)
        .and_then(|mut c| {
            let it = c.item_mut(id)?;
            let Kind::Comportamento(b) = &mut it.kind else {
                return None;
            };
            Some(b.peso.take().is_some())
        })
        .unwrap_or(false)
}

/// O clique no palco com o peso armado (`None` = não está armado). O primeiro objecto debaixo do
/// cursor que não é o dono vira o objecto do peso, mantendo o raio e a curva que já havia.
pub(crate) fn escolher(sim: &mut SimWorld, debaixo: &[u64]) -> Option<Escolha> {
    let (dono, id) = armado(sim.world())?;
    let Some(outro) = debaixo
        .iter()
        .copied()
        .find(|&b| b != dono)
        .and_then(Entity::try_from_bits)
    else {
        if debaixo.contains(&dono) {
            return Some(Escolha::Recusado);
        }
        sim.world_mut().insert_resource(PesoArmado(None));
        return Some(Escolha::Desistiu);
    };
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let Some(StableId(sid)) = stable_id_of(sim.world(), outro) else {
        return Some(Escolha::Recusado);
    };
    sim.world_mut().insert_resource(PesoArmado(None));
    let gravou = Entity::try_from_bits(dono)
        .and_then(|e| sim.world_mut().get_mut::<Constraints>(e))
        .and_then(|mut c| {
            let Kind::Comportamento(b) = &mut c.item_mut(id)?.kind else {
                return None;
            };
            b.peso = Some(Peso {
                objecto: sid,
                ..b.peso.unwrap_or(PESO_NOVO)
            });
            Some(())
        });
    Some(if gravou.is_some() {
        Escolha::Gravado
    } else {
        Escolha::Desistiu
    })
}

/// A linha «Weight By» do item (`None` = não é um *behaviour*).
pub(crate) fn linha(w: &World, dono: u64, it: &Item) -> Option<PesoDaLinha> {
    let Kind::Comportamento(b) = &it.kind else {
        return None;
    };
    let nome = b
        .peso
        .and_then(|p| entity_with_stable_id(w, StableId(p.objecto)))
        .map(|o| w.get::<Name>(o).map_or_else(String::new, |n| n.0.clone()));
    Some(PesoDaLinha {
        nome,
        escolhendo: armado(w) == Some((dono, it.id)),
    })
}
