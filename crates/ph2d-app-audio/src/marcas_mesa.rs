//! ⭐⭐ **O losango de cada linha da mesa de som** (F30 §B2) — o irmão do
//! `ph2d_app_physics::marcas_mundo`: os campos da família MESA (`ph2d_ecs::canal_mesa`) sobre a
//! entidade da Cena. O estado e o clique passam pela porta única [`ph2d_timeline::Ligacoes`] /
//! [`ph2d_timeline::clique_no_grupo`]; a chave mora num clip da Cena (a mesma guarda).

use std::collections::BTreeMap;

use ph2d_ecs::canal::familia::MESA;
use ph2d_ecs::canal_mesa::{self as m, bus};
use ph2d_ecs::{FieldKey, World};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::key_mark::KeyMark;
use ph2d_panel_audio_mixer as painel;
use ph2d_timeline::{Forca, Instante, PropKind, SCENE, TimelineDoc, TimelineIntent};

/// ⭐ **A tabela campo → controlos da mesa.** O gate `architecture_every_editable_field_is_a_channel`
/// lê-a. A escolha da chave do ducking são quatro segmentos do mesmo campo.
pub const CAMPOS: &[(u16, &[NodeId])] = &[
    (m::GANHO, &[painel::AMIX_FADER]),
    (m::CORTE_ALTO, &[painel::AMIX_CUTOFF]),
    (m::CORTE_BAIXO, &[painel::AMIX_LOWCUT]),
    (m::MUDO, &[painel::AMIX_MASTER_MUTE]),
    (m::PAN, &[painel::AMIX_PAN]),
    (m::LIMITADOR, &[painel::AMIX_LIMITER]),
    (m::REVERB, &[painel::AMIX_REVERB]),
    (m::REVERB_TAMANHO, &[painel::AMIX_REVERB_SIZE]),
    (m::REVERB_RETORNO, &[painel::AMIX_REVERB_MIX]),
    (m::EQ_GRAVE, &[painel::AMIX_EQ_LOW]),
    (m::EQ_MEDIO, &[painel::AMIX_EQ_MID]),
    (m::EQ_AGUDO, &[painel::AMIX_EQ_HIGH]),
    (m::DELAY, &[painel::AMIX_DELAY]),
    (m::DELAY_TEMPO, &[painel::AMIX_DELAY_TIME]),
    (m::DELAY_FEEDBACK, &[painel::AMIX_DELAY_FEEDBACK]),
    (m::DELAY_RETORNO, &[painel::AMIX_DELAY_MIX]),
    (m::DUCKING, &[painel::AMIX_DUCK]),
    (m::DUCKING_PROFUNDIDADE, &[painel::AMIX_DUCK_DEPTH]),
    (m::DUCKING_CHAVE, &painel::AMIX_DUCK_KEY_BUS),
    (bus(0, m::B_GANHO), &[painel::SUB_FADER[0]]),
    (bus(0, m::B_MUDO), &[painel::SUB_MUTE[0]]),
    (bus(0, m::B_SOLO), &[painel::SUB_SOLO[0]]),
    (bus(0, m::B_PAN), &[painel::SUB_PAN[0]]),
    (bus(0, m::B_TOM), &[painel::SUB_TONE[0]]),
    (bus(0, m::B_CORTE_BAIXO), &[painel::SUB_LOWCUT[0]]),
    (bus(0, m::B_ENVIO_REVERB), &[painel::SUB_SEND[0]]),
    (bus(0, m::B_ENVIO_DELAY), &[painel::SUB_DELAY_SEND[0]]),
    (bus(0, m::B_COMP), &[painel::SUB_COMP[0]]),
    (bus(1, m::B_GANHO), &[painel::SUB_FADER[1]]),
    (bus(1, m::B_MUDO), &[painel::SUB_MUTE[1]]),
    (bus(1, m::B_SOLO), &[painel::SUB_SOLO[1]]),
    (bus(1, m::B_PAN), &[painel::SUB_PAN[1]]),
    (bus(1, m::B_TOM), &[painel::SUB_TONE[1]]),
    (bus(1, m::B_CORTE_BAIXO), &[painel::SUB_LOWCUT[1]]),
    (bus(1, m::B_ENVIO_REVERB), &[painel::SUB_SEND[1]]),
    (bus(1, m::B_ENVIO_DELAY), &[painel::SUB_DELAY_SEND[1]]),
    (bus(1, m::B_COMP), &[painel::SUB_COMP[1]]),
    (bus(2, m::B_GANHO), &[painel::SUB_FADER[2]]),
    (bus(2, m::B_MUDO), &[painel::SUB_MUTE[2]]),
    (bus(2, m::B_SOLO), &[painel::SUB_SOLO[2]]),
    (bus(2, m::B_PAN), &[painel::SUB_PAN[2]]),
    (bus(2, m::B_TOM), &[painel::SUB_TONE[2]]),
    (bus(2, m::B_CORTE_BAIXO), &[painel::SUB_LOWCUT[2]]),
    (bus(2, m::B_ENVIO_REVERB), &[painel::SUB_SEND[2]]),
    (bus(2, m::B_ENVIO_DELAY), &[painel::SUB_DELAY_SEND[2]]),
    (bus(2, m::B_COMP), &[painel::SUB_COMP[2]]),
    (bus(3, m::B_GANHO), &[painel::SUB_FADER[3]]),
    (bus(3, m::B_MUDO), &[painel::SUB_MUTE[3]]),
    (bus(3, m::B_SOLO), &[painel::SUB_SOLO[3]]),
    (bus(3, m::B_PAN), &[painel::SUB_PAN[3]]),
    (bus(3, m::B_TOM), &[painel::SUB_TONE[3]]),
    (bus(3, m::B_CORTE_BAIXO), &[painel::SUB_LOWCUT[3]]),
    (bus(3, m::B_ENVIO_REVERB), &[painel::SUB_SEND[3]]),
    (bus(3, m::B_ENVIO_DELAY), &[painel::SUB_DELAY_SEND[3]]),
    (bus(3, m::B_COMP), &[painel::SUB_COMP[3]]),
];

fn marca(f: Forca) -> KeyMark {
    match f {
        Forca::Livre => KeyMark::Livre,
        Forca::Animado => KeyMark::Animado,
        Forca::Chave => KeyMark::Chave,
        Forca::Mudado => KeyMark::Mudado,
    }
}

/// O canal que o controlo `id` mostra (cada campo da mesa é o seu grupo).
#[must_use]
pub fn canal_do_campo(id: NodeId) -> Vec<PropKind> {
    CAMPOS
        .iter()
        .find(|(_, ids)| ids.contains(&id))
        .map(|&(campo, _)| vec![PropKind::Field(FieldKey::new(MESA, campo, 0, 0))])
        .unwrap_or_default()
}

/// ⭐ **O mapa do quadro** para as linhas da mesa (sobre a entidade da Cena de `world`).
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
        let mk = marca(lig.estado(&canal_do_campo(ids[0]), &valor).0);
        for &id in ids {
            out.insert(id, mk);
        }
    }
    out
}

/// ⭐ **O que o clique num losango da mesa escreve** — a porta única da timeline sobre o campo.
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
#[path = "marcas_mesa_tests.rs"]
mod tests;
