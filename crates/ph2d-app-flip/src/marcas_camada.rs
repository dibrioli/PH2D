//! ⭐⭐ **O losango de cada campo da CAMADA do Flip** (plano 35 §A.8) — o irmão do
//! `ph2d_app_components::marcas_inspector`: os campos da família FLIP (`ph2d_ecs::canal_flip`) por
//! linha do painel. O id de cada linha é derivado da camada (`flip_layer_widget_id`), então a
//! tabela é por TIPO de controlo e o mapa enumera as camadas do objecto em edição. O estado e o
//! clique passam pela porta única [`ph2d_timeline::estado_do_grupo`] /
//! [`ph2d_timeline::clique_no_grupo`].

use std::collections::BTreeMap;

use ph2d_ecs::FieldKey;
use ph2d_ecs::canal_flip as f;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::key_mark::KeyMark;
use ph2d_flip::FlipObject;
use ph2d_panel_flip::ids::{FlipLayerWidget, flip_layer_widget_id};
use ph2d_timeline::{Forca, Instante, PropKind, TimelineDoc, TimelineIntent};

/// ⭐ **A tabela campo → controlos da linha da camada.** O gate
/// `architecture_every_editable_field_is_a_channel` lê-a: o slider e o seu chip partilham o
/// losango; o cadeado, o nome e as setas de ordem não estão aqui (plano 35 §8).
pub const CAMPOS: &[(u16, &[FlipLayerWidget])] = &[
    (f::VISIVEL, &[FlipLayerWidget::Visibility]),
    (f::MISTURA, &[FlipLayerWidget::Blend]),
    (
        f::OPACIDADE,
        &[FlipLayerWidget::Opacity, FlipLayerWidget::OpacityNum],
    ),
    (
        f::PROFUNDIDADE,
        &[FlipLayerWidget::Depth, FlipLayerWidget::DepthNum],
    ),
];

fn marca(m: Forca) -> KeyMark {
    match m {
        Forca::Livre => KeyMark::Livre,
        Forca::Animado => KeyMark::Animado,
        Forca::Chave => KeyMark::Chave,
        Forca::Mudado => KeyMark::Mudado,
    }
}

/// O objecto cujas camadas o painel mostra (o desenho em edição) e a entidade dele.
#[must_use]
pub fn alvo<'a>(
    state: &crate::state::FlipState,
    doc: &'a ph2d_flip::FlipDoc,
) -> Option<(u64, &'a FlipObject)> {
    let oid = state.target.object?;
    Some((*state.entities.get(&oid)?, doc.object(oid)?))
}

/// O canal que o controlo `id` de uma camada de `obj` mostra.
#[must_use]
pub fn canal_do_campo(obj: &FlipObject, id: NodeId) -> Option<FieldKey> {
    obj.layers().iter().find_map(|l| {
        CAMPOS.iter().find_map(|&(campo, kinds)| {
            kinds
                .iter()
                .any(|&k| flip_layer_widget_id(u64::from(l.id.0), k) == id)
                .then(|| crate::canais_camada::chave(campo, l.id))
                .flatten()
        })
    })
}

/// ⭐ **O mapa do quadro** para as camadas de `obj` (o objecto Flip em edição, a entidade
/// `entidade`).
#[must_use]
pub fn marcas(
    doc: &TimelineDoc,
    entidade: u64,
    obj: &FlipObject,
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
) -> BTreeMap<NodeId, KeyMark> {
    let mut out = BTreeMap::new();
    for l in obj.layers() {
        for &(campo, kinds) in CAMPOS {
            let Some(kk) = crate::canais_camada::chave(campo, l.id) else {
                continue;
            };
            let reps = [PropKind::Field(kk)];
            let m = marca(ph2d_timeline::estado_do_grupo(doc, &[entidade], &reps, inst, &valor).0);
            for &k in kinds {
                out.insert(flip_layer_widget_id(u64::from(l.id.0), k), m);
            }
        }
    }
    out
}

/// ⭐ **O que o clique num losango de uma camada escreve** — a porta única da timeline sobre o
/// grupo do campo. Fora das camadas de `obj`, nada.
#[must_use]
#[allow(clippy::too_many_arguments)] // o irmão do `marcas_inspector::clique`, mais o objecto
pub fn clique(
    doc: &TimelineDoc,
    campos: &[NodeId],
    entidade: u64,
    obj: &FlipObject,
    inst: Instante,
    lidos: impl Fn(u64) -> Vec<FieldKey>,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
    autorar: impl Fn(u64, PropKind) -> Option<TimelineIntent>,
) -> Vec<TimelineIntent> {
    let Some(kk) = campos.iter().find_map(|&c| canal_do_campo(obj, c)) else {
        return Vec::new();
    };
    ph2d_timeline::clique_no_grupo(
        doc,
        &[PropKind::Field(kk)],
        &[entidade],
        inst,
        lidos,
        valor,
        autorar,
    )
}

#[cfg(test)]
#[path = "marcas_camada_tests.rs"]
mod tests;
