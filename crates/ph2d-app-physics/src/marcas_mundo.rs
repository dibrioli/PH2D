//! ⭐⭐ **O losango de cada linha do painel Physics** (F30 §B) — o irmão do
//! `ph2d_app_flip::marcas_camada`: os campos da família MUNDO (`ph2d_ecs::canal_mundo`) sobre a
//! entidade da Cena. O estado e o clique passam pela porta única [`ph2d_timeline::Ligacoes`] /
//! [`ph2d_timeline::clique_no_grupo`].
//!
//! ⚠️ A chave da Cena mora num clip da Cena: com a animação de um objecto em vista o clique não
//! grava (o Auto Key di-lo num toast, `KeyRefusal::SceneNotInView`).

use std::collections::BTreeMap;

use ph2d_ecs::canal::familia::MUNDO;
use ph2d_ecs::canal_mundo as m;
use ph2d_ecs::{FieldKey, World};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::key_mark::KeyMark;
use ph2d_panel_physics::ids as painel;
use ph2d_timeline::{Forca, Instante, PropKind, SCENE, TimelineDoc, TimelineIntent};

/// ⭐ **A tabela campo → controlos da linha do painel.** O gate
/// `architecture_every_editable_field_is_a_channel` lê-a: o slider e o seu chip partilham o
/// losango.
pub const CAMPOS: &[(u16, &[NodeId])] = &[
    (
        m::GRAVIDADE_X,
        &[painel::PHYSICS_GRAVITY_X, painel::PHYSICS_GRAVITY_X_NUM],
    ),
    (
        m::GRAVIDADE_Y,
        &[painel::PHYSICS_GRAVITY_Y, painel::PHYSICS_GRAVITY_Y_NUM],
    ),
    (
        m::SUBPASSOS,
        &[painel::PHYSICS_SUBSTEPS, painel::PHYSICS_SUBSTEPS_NUM],
    ),
    (
        m::ITERACOES,
        &[painel::PHYSICS_ITERATIONS, painel::PHYSICS_ITERATIONS_NUM],
    ),
    (
        m::CONTACTO_HZ,
        &[painel::PHYSICS_CONTACT_HZ, painel::PHYSICS_CONTACT_HZ_NUM],
    ),
    (
        m::AMORT_LINEAR,
        &[
            painel::PHYSICS_LINEAR_DAMPING,
            painel::PHYSICS_LINEAR_DAMPING_NUM,
        ],
    ),
    (
        m::AMORT_ANGULAR,
        &[
            painel::PHYSICS_ANGULAR_DAMPING,
            painel::PHYSICS_ANGULAR_DAMPING_NUM,
        ],
    ),
    (
        m::AR,
        &[painel::PHYSICS_AIR_DRAG, painel::PHYSICS_AIR_DRAG_NUM],
    ),
    (
        m::SONO_VELOCIDADE,
        &[painel::PHYSICS_SLEEP_SPEED, painel::PHYSICS_SLEEP_SPEED_NUM],
    ),
    (
        m::SONO_DEMORA,
        &[painel::PHYSICS_SLEEP_DELAY, painel::PHYSICS_SLEEP_DELAY_NUM],
    ),
    (m::ADORMECE, &[painel::PHYSICS_SLEEP_SPIN]),
];

fn marca(f: Forca) -> KeyMark {
    match f {
        Forca::Livre => KeyMark::Livre,
        Forca::Animado => KeyMark::Animado,
        Forca::Chave => KeyMark::Chave,
        Forca::Mudado => KeyMark::Mudado,
    }
}

/// O GRUPO que o controlo `id` mostra (a gravidade são os dois eixos).
#[must_use]
pub fn canal_do_campo(id: NodeId) -> Vec<PropKind> {
    let Some(&(campo, _)) = CAMPOS.iter().find(|(_, ids)| ids.contains(&id)) else {
        return Vec::new();
    };
    let kk = FieldKey::new(MUNDO, campo, 0, 0);
    CAMPOS
        .iter()
        .map(|&(c, _)| FieldKey::new(MUNDO, c, 0, 0))
        .filter(|&o| o.mesmo_grupo(kk))
        .map(PropKind::Field)
        .collect()
}

/// ⭐ **O mapa do quadro** para as linhas do painel (sobre a entidade da Cena de `world`).
#[must_use]
pub fn marcas(
    doc: &TimelineDoc,
    world: &World,
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
) -> BTreeMap<NodeId, KeyMark> {
    let Some(cena) = ph2d_ecs::bits_da_cena(world) else {
        return BTreeMap::new();
    };
    let lig = ph2d_timeline::Ligacoes::de(doc, &[cena], inst);
    let mut out = BTreeMap::new();
    for &(_, ids) in CAMPOS {
        let reps = canal_do_campo(ids[0]);
        let mk = marca(lig.estado(&reps, &valor).0);
        for &id in ids {
            out.insert(id, mk);
        }
    }
    out
}

/// ⭐ **O que o clique num losango do painel escreve** — a porta única da timeline sobre o grupo.
/// Fora da tabela, sem Cena, ou com a animação de um objecto em vista, nada.
#[must_use]
pub fn clique(
    doc: &TimelineDoc,
    campos: &[NodeId],
    world: &World,
    inst: Instante,
    lidos: impl Fn(u64) -> Vec<FieldKey>,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
    autorar: impl Fn(u64, PropKind) -> Option<TimelineIntent>,
) -> Vec<TimelineIntent> {
    let Some(reps) = campos
        .iter()
        .map(|&c| canal_do_campo(c))
        .find(|r| !r.is_empty())
    else {
        return Vec::new();
    };
    let Some(cena) = ph2d_ecs::bits_da_cena(world) else {
        return Vec::new();
    };
    if !doc.clip_belongs_to(doc.active_index(), SCENE) {
        return Vec::new();
    }
    ph2d_timeline::clique_no_grupo(doc, &reps, &[cena], inst, lidos, valor, autorar)
}

#[cfg(test)]
#[path = "marcas_mundo_tests.rs"]
mod tests;
