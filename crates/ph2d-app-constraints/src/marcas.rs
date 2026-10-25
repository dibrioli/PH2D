//! **O losango do Auto Key no cartão** (plano 35 §5): que canal cada controlo do cartão mostra, o
//! estado dele no instante do autor e o clique no losango. A linha `k` do cartão é o item `k` da pilha
//! da entidade escolhida; o canal é endereçado pelo `Item::id`.

use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::canal_constraint::{FORCA, LIGADA, chave};
use ph2d_ecs::{FieldKey, SimWorld};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::key_mark::KeyMark;
use ph2d_panel_inspector::ids as insp;
use ph2d_timeline::{Forca, Instante, PropKind, TimelineDoc, TimelineIntent};

/// Os controlos de uma linha e o campo que cada um mostra.
fn controlos(k: usize) -> [(u16, Option<&'static NodeId>); 2] {
    [
        (FORCA, insp::INSP_CONSTRAINT_FORCA.get(k)),
        (LIGADA, insp::INSP_CONSTRAINT_LIGADA.get(k)),
    ]
}

fn marca(m: Forca) -> KeyMark {
    match m {
        Forca::Livre => KeyMark::Livre,
        Forca::Animado => KeyMark::Animado,
        Forca::Chave => KeyMark::Chave,
        Forca::Mudado => KeyMark::Mudado,
    }
}

/// A entidade cujo cartão está à vista: a escolha, se for UMA e tiver pilha.
fn dono<'a>(sim: &'a SimWorld, sel: &[u64]) -> Option<(u64, &'a Constraints)> {
    let [b] = sel else {
        return None;
    };
    Some((*b, sim.world().get::<Constraints>(Entity::try_from_bits(*b)?)?))
}

/// O canal que o controlo `id` mostra.
#[must_use]
pub fn canal_do_campo(c: &Constraints, id: NodeId) -> Option<FieldKey> {
    c.items.iter().enumerate().find_map(|(k, it)| {
        controlos(k)
            .into_iter()
            .find(|(_, n)| n.is_some_and(|&n| n == id))
            .map(|(campo, _)| chave(campo, it.id))
    })
}

/// O estado de cada losango do cartão no instante do autor.
#[must_use]
pub fn marcas(
    doc: &TimelineDoc,
    sim: &SimWorld,
    sel: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
) -> BTreeMap<NodeId, KeyMark> {
    let mut out = BTreeMap::new();
    let Some((e, c)) = dono(sim, sel) else {
        return out;
    };
    let lig = ph2d_timeline::Ligacoes::de(doc, &[e], inst);
    for (k, it) in c.items.iter().enumerate() {
        for (campo, n) in controlos(k) {
            if let Some(&n) = n {
                let reps = [PropKind::Field(chave(campo, it.id))];
                out.insert(n, marca(lig.estado(&reps, &valor).0));
            }
        }
    }
    out
}

/// O clique num losango do cartão: grava (ou tira) a chave do campo no instante do autor.
#[must_use]
#[allow(clippy::too_many_arguments)] // o irmão do `marcas_camada::clique`
pub fn clique(
    doc: &TimelineDoc,
    campos: &[NodeId],
    sim: &SimWorld,
    sel: &[u64],
    inst: Instante,
    lidos: impl Fn(u64) -> Vec<FieldKey>,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
    autorar: impl Fn(u64, PropKind) -> Option<TimelineIntent>,
) -> Vec<TimelineIntent> {
    let Some((e, c)) = dono(sim, sel) else {
        return Vec::new();
    };
    let Some(kk) = campos.iter().find_map(|&n| canal_do_campo(c, n)) else {
        return Vec::new();
    };
    ph2d_timeline::clique_no_grupo(doc, &[PropKind::Field(kk)], &[e], inst, lidos, valor, autorar)
}
