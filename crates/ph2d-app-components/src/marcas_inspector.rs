//! ⭐⭐ **O losango de cada campo do Inspector** (plano 35 §5; plano 34 §9 metade B) — o irmão do
//! `ph2d_app_vec::marcas` para os campos cujo canal é um `PropKind` de sempre: o Transform, a
//! opacidade do sprite e o motor das juntas. O estado e o clique passam pela porta única
//! [`ph2d_timeline::estado_do_grupo`] / [`ph2d_timeline::clique_no_grupo`].
//!
//! ⚠️ A POSIÇÃO de um objecto numa trajectória é o canal `Position` (a K grava um `AddPathKey`) e a
//! de um objecto livre são `TranslationX/Y`: o clique grava a do modo em que o objecto está, nunca
//! as duas.

use std::collections::BTreeMap;

use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::key_mark::KeyMark;
use ph2d_panel_inspector::ids as insp;
use ph2d_timeline::{Forca, Instante, PropKind, TimelineDoc, TimelineIntent};

const POSICAO: &[PropKind] = &[
    PropKind::TranslationX,
    PropKind::TranslationY,
    PropKind::Position,
];
const ESCALA: &[PropKind] = &[PropKind::ScaleX, PropKind::ScaleY];

/// ⭐ **A tabela campo → grupo do Inspector.** O gate `architecture_every_editable_field_is_a_channel`
/// lê-a: um campo do Inspector pintado com coluna de losango tem de estar aqui ou na tabela
/// `⛔ não-animável` / de dívida do gate.
const CAMPOS: &[(NodeId, &[PropKind])] = &[
    (insp::INSP_TRANSFORM_POS_X, POSICAO),
    (insp::INSP_TRANSFORM_POS_Y, POSICAO),
    (insp::INSP_TRANSFORM_ROT, &[PropKind::Rotation]),
    (insp::INSP_TRANSFORM_SCALE_X, ESCALA),
    (insp::INSP_TRANSFORM_SCALE_Y, ESCALA),
    (insp::INSP_SPRITE_OPACITY, &[PropKind::Opacity]),
    (insp::INSP_SPRITE_OPACITY_CHIP, &[PropKind::Opacity]),
    (insp::INSP_JOINT_MOTOR_TARGET, &[PropKind::JointMotorTarget]),
    (insp::INSP_JOINT_MOTOR_SPEED, &[PropKind::JointMotorSpeed]),
    (insp::INSP_JOINT_REST_LENGTH, &[PropKind::JointRestLength]),
    (insp::INSP_JOINT_MAX_LENGTH, &[PropKind::JointMaxLength]),
];

/// O grupo que o campo `id` do Inspector mostra (vazio = sem canal).
#[must_use]
pub fn canal_do_campo(id: NodeId) -> &'static [PropKind] {
    CAMPOS
        .iter()
        .find(|(c, _)| *c == id)
        .map_or(&[], |&(_, g)| g)
}

fn marca(f: Forca) -> KeyMark {
    match f {
        Forca::Livre => KeyMark::Livre,
        Forca::Animado => KeyMark::Animado,
        Forca::Chave => KeyMark::Chave,
        Forca::Mudado => KeyMark::Mudado,
    }
}

/// ⭐ **O mapa do quadro** para os campos do Inspector (os objectos `entidades`, o mais forte).
#[must_use]
pub fn marcas(
    doc: &TimelineDoc,
    entidades: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
) -> BTreeMap<NodeId, KeyMark> {
    let mut out = BTreeMap::new();
    if entidades.is_empty() {
        return out;
    }
    for &(id, reps) in CAMPOS {
        let f = ph2d_timeline::estado_do_grupo(doc, entidades, reps, inst, &valor).0;
        out.insert(id, marca(f));
    }
    out
}

/// ⭐ **O que o clique num losango do Inspector escreve.** Numa chave, apaga o grupo neste quadro;
/// senão grava-o pela porta da K, cada objecto no modo que a K escolheria
/// (`TimelineDoc::position_key_mode`: `Position` na trajectória, `TranslationX/Y` nos eixos
/// separados). Fora da tabela, nada.
#[must_use]
pub fn clique(
    doc: &TimelineDoc,
    campos: &[NodeId],
    entidades: &[u64],
    inst: Instante,
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
    let apagar = ph2d_timeline::clique_no_grupo(
        doc,
        reps,
        entidades,
        inst,
        |_| Vec::new(),
        &valor,
        |_, _| None,
    );
    if apagar.contains(&TimelineIntent::DeleteSelection) {
        return apagar;
    }
    let mut out = vec![TimelineIntent::BeginEdit];
    for &e in entidades {
        // A porta da K: a animação que existe decide; um objecto novo vai para a trajectória.
        let na_trajetoria = doc.position_key_mode(e, true) == ph2d_timeline::PositionKeyMode::Path;
        out.extend(
            reps.iter()
                .filter(|&&p| (p == PropKind::Position) == na_trajetoria || !POSICAO.contains(&p))
                .filter_map(|&p| autorar(e, p)),
        );
    }
    out.push(TimelineIntent::EndEdit);
    out
}

#[cfg(test)]
#[path = "marcas_inspector_tests.rs"]
mod tests;
