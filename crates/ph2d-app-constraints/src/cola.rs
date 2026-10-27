//! **A cola no cartão e no palco** (`docs/BlenderConstraints/05_W5_diferenciais.md` §2.5): «Glue · Rigid» e
//! «Glue · Pin» no menu do cartão ARMAM o clique no outro objecto ([`ColaArmada`]) — não são itens da pilha; o
//! clique grava a [`Cola`] no `Colas` do dono (o `a`); o cartão mostra-a nos DOIS objectos; «Unglue» desfaz; o
//! gizmo arrasta o ponto do pino ([`mover_pino`]).

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_constraints::Pose2;
use ph2d_constraints_cola::{Cola, Estado, Tipo, local};
use ph2d_constraints_ecs::{Colas, EstadosDaCola};
use ph2d_ecs::{Name, SimWorld, StableId, entity_with_stable_id, stable_id_of, world_transform};
use ph2d_editor_core::constraint_edits::{EstadoDaCola, InspectorColaRow};

use crate::inspector::Escolha;

/// A cola cujo outro objecto o próximo clique no palco escolhe: `(dono, pino?)`. Estado de sessão.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColaArmada(pub Option<(u64, bool)>);

/// Toda cola do mundo, com a entidade que a guarda.
#[must_use]
pub fn todas(w: &World) -> Vec<(Entity, Cola)> {
    w.iter_entities()
        .filter_map(|r| Some((r.id(), r.get::<Colas>()?)))
        .flat_map(|(e, c)| c.items.iter().map(move |&cola| (e, cola)))
        .collect()
}

/// A pose MUNDIAL do objecto `stable` (o que a cola compara).
#[must_use]
pub fn pose_mundial(w: &World, stable: u64) -> Option<Pose2> {
    let t = world_transform(w, entity_with_stable_id(w, StableId(stable))?)?;
    Some(Pose2 {
        x: f64::from(t.translation.x),
        y: f64::from(t.translation.y),
        rot: f64::from(t.rotation),
        sx: f64::from(t.scale.x),
        sy: f64::from(t.scale.y),
    })
}

/// Leva o pino `id` do `dono` para o ponto `mundo`, nos DOIS objectos — eles não se mexem. `true` = mudou.
pub fn mover_pino(sim: &mut SimWorld, dono: Entity, id: u32, mundo: [f64; 2]) -> bool {
    let w = sim.world();
    let Some(cola) = w
        .get::<Colas>(dono)
        .and_then(|c| c.items.iter().find(|c| c.id == id).copied())
    else {
        return false;
    };
    let Tipo::Pino { .. } = cola.tipo else {
        return false;
    };
    let (Some(pa), Some(pb)) = (pose_mundial(w, cola.a), pose_mundial(w, cola.b)) else {
        return false;
    };
    let (Some(em_a), Some(em_b)) = (local(pa, mundo), local(pb, mundo)) else {
        return false;
    };
    let novo = Tipo::Pino { em_a, em_b };
    if novo == cola.tipo {
        return false;
    }
    let Some(mut c) = sim.world_mut().get_mut::<Colas>(dono) else {
        return false;
    };
    c.items
        .iter_mut()
        .filter(|c| c.id == id)
        .for_each(|c| c.tipo = novo);
    true
}

/// Arma (ou, com `None`, desarma) o clique da cola.
pub(crate) fn armar(sim: &mut SimWorld, armada: Option<(u64, bool)>) {
    sim.world_mut().insert_resource(ColaArmada(armada));
}

pub(crate) fn armada(w: &World) -> Option<(u64, bool)> {
    w.get_resource::<ColaArmada>().and_then(|a| a.0)
}

/// Desfaz a cola `id` do `dono`. `true` = o documento mudou.
pub(crate) fn descolar(sim: &mut SimWorld, dono: u64, id: u32) -> bool {
    Entity::try_from_bits(dono)
        .and_then(|e| sim.world_mut().get_mut::<Colas>(e))
        .is_some_and(|mut c| c.remove(id))
}

/// As linhas do cartão de `e`: toda cola que o toca, guardada por ele ou pelo parceiro.
#[must_use]
pub(crate) fn linhas(w: &World, e: Entity) -> Vec<InspectorColaRow> {
    let Some(s) = stable_id_of(w, e) else {
        return Vec::new();
    };
    let estados = w.get_resource::<EstadosDaCola>();
    todas(w)
        .into_iter()
        .filter_map(|(dono, cola)| {
            let outro = entity_with_stable_id(w, StableId(cola.outro(s.0)?));
            Some(InspectorColaRow {
                dono_bits: dono.to_bits(),
                id: cola.id,
                pino: matches!(cola.tipo, Tipo::Pino { .. }),
                parceiro: outro
                    .and_then(|o| w.get::<Name>(o))
                    .map_or_else(String::new, |n| n.0.clone()),
                estado: match estados.and_then(|x| x.0.get(&(cola.a, cola.id))) {
                    Some(Estado::Definido) => EstadoDaCola::Definido,
                    Some(Estado::Conflito) => EstadoDaCola::Conflito,
                    Some(Estado::Parado) | None => EstadoDaCola::Parado,
                },
            })
        })
        .collect()
}

/// O clique no palco com a cola armada: o alvo é o de cima sob o cursor que não é o dono; a Rígida guarda as
/// poses MUNDIAIS de agora, o Pino prende no ponto `mundo`. `None` = nada armado.
pub(crate) fn escolher(sim: &mut SimWorld, debaixo: &[u64], mundo: [f64; 2]) -> Option<Escolha> {
    let (dono_bits, pino) = armada(sim.world())?;
    let candidato = debaixo.iter().copied().find(|&b| b != dono_bits);
    let (Some(dono), Some(alvo)) = (
        Entity::try_from_bits(dono_bits),
        candidato.and_then(Entity::try_from_bits),
    ) else {
        if debaixo.contains(&dono_bits) {
            return Some(Escolha::Recusado);
        }
        armar(sim, None);
        return Some(Escolha::Desistiu);
    };
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let w = sim.world();
    let (Some(sa), Some(sb)) = (stable_id_of(w, dono), stable_id_of(w, alvo)) else {
        return Some(Escolha::Recusado);
    };
    let (Some(pa), Some(pb)) = (pose_mundial(w, sa.0), pose_mundial(w, sb.0)) else {
        armar(sim, None);
        return Some(Escolha::Desistiu);
    };
    armar(sim, None);
    let tinha = sim.world().get::<Colas>(dono).is_some();
    let mut colas = sim
        .world_mut()
        .entity_mut(dono)
        .take::<Colas>()
        .unwrap_or_default();
    let feita = colas.push(|id| {
        if pino {
            Cola::pino(id, sa.0, sb.0, pa, pb, mundo)
        } else {
            Cola::rigida(id, sa.0, sb.0, pa, pb)
        }
    });
    if tinha || feita.is_some() {
        sim.world_mut().entity_mut(dono).insert(colas);
    }
    // Um espaço colapsado (escala 0) não tem pose relativa: nada se grava.
    Some(if feita.is_some() {
        Escolha::Gravado
    } else {
        Escolha::Desistiu
    })
}
