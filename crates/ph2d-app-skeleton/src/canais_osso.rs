//! ⭐⭐ **A família OSSO** (F40): [`publicar`]/[`aplicar`] o comprimento, a força e os segmentos do
//! `Bone` (pelas cercas do painel), e o losango do painel ([`CAMPOS`], [`marcas`], [`clique`]); as
//! alças são `PropKind::BoneBend*`. A curva é pré-visualização por campo (`FieldDrive`).

use std::collections::BTreeMap;

use ph2d_ecs::canal::familia::OSSO;
use ph2d_ecs::canal_constraint as c;
use ph2d_ecs::canal_osso as o;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld};
use ph2d_editor_core::ids as painel;
use ph2d_editor_core::{NodeId, widget::key_mark::KeyMark};
use ph2d_preview_drive::PreviewDrive;
use ph2d_skeleton_ecs::Bone;
use ph2d_timeline::{Forca, Instante, PropKind, TimelineDoc, TimelineIntent};

use crate::knobs::{BoneKnob, apply};

/// A chave do campo `campo` (o osso é a entidade: `i = j = 0`).
#[must_use]
pub const fn k(campo: u16) -> FieldKey {
    FieldKey::new(OSSO, campo, 0, 0)
}

/// ⭐ **A tabela canal → campo do painel** — lida pelo gate `architecture_every_editable_field_is_a_channel`.
pub const CAMPOS: &[(PropKind, NodeId)] = &[
    (
        PropKind::Field(k(o::COMPRIMENTO)),
        painel::VECTOR_BONE_LENGTH,
    ),
    (PropKind::Field(k(o::FORCA)), painel::VECTOR_BONE_STRENGTH),
    (
        PropKind::Field(k(o::SEGMENTOS)),
        painel::VECTOR_BONE_SEGMENTS,
    ),
    (PropKind::BoneBendInX, painel::VECTOR_BONE_CURVE_IN_X),
    (PropKind::BoneBendInY, painel::VECTOR_BONE_CURVE_IN_Y),
    (PropKind::BoneBendOutX, painel::VECTOR_BONE_CURVE_OUT_X),
    (PropKind::BoneBendOutY, painel::VECTOR_BONE_CURVE_OUT_Y),
];

/// Quanto vale `campo` no osso `b` — `None` fora do vocabulário.
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
pub fn ler(b: &Bone, campo: u16) -> Option<f32> {
    Some(match campo {
        o::COMPRIMENTO => b.length as f32,
        o::FORCA => b.strength as f32,
        o::SEGMENTOS => f32::from(b.segments),
        _ => return None,
    })
}

/// Escreve `v` em `campo` do osso pelas cercas do painel; os segmentos arredondam.
pub fn escrever(b: &mut Bone, campo: u16, v: f32) {
    let v = f64::from(v);
    match campo {
        o::COMPRIMENTO => apply(b, BoneKnob::Length, v),
        o::FORCA => apply(b, BoneKnob::Strength, v),
        o::SEGMENTOS => apply(b, BoneKnob::Segments, v.round()),
        _ => {}
    }
}

/// ⭐ **Publica o que cada osso vale AGORA**; esquece antes o que cada entidade publicou da família.
pub fn publicar(sim: &mut SimWorld, entidades: &[u64]) {
    let lotes: Vec<(u64, Bone)> = entidades
        .iter()
        .filter_map(|&b| Some((b, *sim.world().get::<Bone>(Entity::try_from_bits(b)?)?)))
        .collect();
    let w = sim.world_mut();
    let mut lidos = w.get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, OSSO);
    }
    for (b, os) in lotes {
        for c in o::CAMPOS {
            if let Some(v) = ler(&os, c) {
                lidos.publicar(b, k(c), v);
            }
        }
    }
}

/// A chave do registo (`slot_in`) de `campo`.
const fn chave(campo: u16) -> &'static str {
    match campo {
        o::FORCA => "strength",
        o::SEGMENTOS => "segments",
        _ => "length",
    }
}

/// ⭐ **Aplica ao osso o que a curva mandou neste quadro**, regista o eco e declara a escrita à
/// pré-visualização POR CAMPO (`campo_do_osso`): o undo e o save fotografam o autorado.
pub fn aplicar(sim: &mut SimWorld, drive: &mut PreviewDrive) -> bool {
    let Some(escritas) =
        (sim.world_mut().get_resource_mut::<CanaisEscritos>()).map(|mut w| w.drenar(OSSO))
    else {
        return false;
    };
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for (b, kk, v) in escritas {
        let Some(e) = Entity::try_from_bits(b) else {
            continue;
        };
        let Some(mut os) = sim.world_mut().get_mut::<Bone>(e) else {
            continue;
        };
        let was = *os;
        escrever(&mut os, kk.campo, v);
        let now = *os;
        if let Some(ficou) = ler(&now, kk.campo) {
            ecos.push((b, kk, v, ficou));
        }
        mudou |= was != now;
        let ch = [chave(kk.campo)];
        ph2d_timeline_preview::declara_osso(sim.world(), e, &was, &now, &ch, drive);
    }
    let w = sim.world_mut();
    let mut lidos = w.get_resource_or_insert_with(CanaisLidos::default);
    for (b, kk, v, f) in ecos {
        lidos.eco(b, kk, v, f);
    }
    mudou
}

/// ⭐ **Os números da ÂNCORA** → o campo do item dela na família CONSTRAINT (pelo `Item::id`).
pub const ANCORA: &[(u16, NodeId)] = &[
    (c::FORCA, painel::VECTOR_BONE_IK_MIX),
    (c::SUAVIDADE, painel::VECTOR_BONE_IK_SOFTNESS),
    (c::CORRENTE, painel::VECTOR_BONE_IK_CHAIN),
    (c::DESVIO, painel::VECTOR_BONE_IK_OFFSET),
];

/// O canal que o campo `id` do painel mostra.
#[must_use]
pub fn canal_do_campo(id: NodeId) -> Option<PropKind> {
    CAMPOS.iter().find(|&&(_, n)| n == id).map(|&(p, _)| p)
}

/// As linhas do osso `e`: [`CAMPOS`] e, com âncora, as [`ANCORA`] sobre o item dela.
fn linhas(sim: &SimWorld, e: u64) -> Vec<(PropKind, NodeId)> {
    let item = ph2d_constraints_ecs::ancora_de(sim.world(), Entity::from_bits(e)).map(|a| a.0);
    let ancora = ANCORA
        .iter()
        .filter_map(|&(f, n)| Some((PropKind::Field(c::chave(f, item?)), n)));
    CAMPOS.iter().copied().chain(ancora).collect()
}

/// ⭐ **O mapa do quadro** para os números do osso escolhido em `sel`.
#[must_use]
pub fn marcas(
    doc: &TimelineDoc,
    sim: &SimWorld,
    sel: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
) -> BTreeMap<NodeId, KeyMark> {
    let Some(e) = crate::bone_gesture::selected_bone(sim, sel.iter().copied()) else {
        return BTreeMap::new();
    };
    let lig = ph2d_timeline::Ligacoes::de(doc, &[e], inst);
    let marca = |f| match f {
        Forca::Livre => KeyMark::Livre,
        Forca::Animado => KeyMark::Animado,
        Forca::Chave => KeyMark::Chave,
        Forca::Mudado => KeyMark::Mudado,
    };
    (linhas(sim, e).into_iter())
        .map(|(p, n)| (n, marca(lig.estado(&[p], &valor).0)))
        .collect()
}

/// ⭐ **O clique num losango do painel** sobre o osso escolhido; fora da tabela ou sem osso, nada.
#[must_use]
#[allow(clippy::too_many_arguments)] // o irmão do `ph2d_app_constraints::marcas::clique`
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
    let Some(e) = crate::bone_gesture::selected_bone(sim, sel.iter().copied()) else {
        return Vec::new();
    };
    let ls = linhas(sim, e);
    let Some(&(p, _)) = ls.iter().find(|(_, n)| campos.contains(n)) else {
        return Vec::new();
    };
    ph2d_timeline::clique_no_grupo(doc, &[p], &[e], inst, lidos, valor, autorar)
}

#[cfg(test)]
#[path = "canais_osso_tests.rs"]
mod tests;
