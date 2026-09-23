//! **A semente da secção PARALLAX** (plano 24, W7).
//!
//! ⚠️ **Ela não corre por quadro** — a aresta é a ASSINATURA do instantâneo, e o foco e o arrasto
//! ganham-lhe: *a mão do artista ganha ao instantâneo*. É a lei que as seis irmãs já escrevem.
//!
//! ⚠️⚠️ **As leituras do MUNDO (`camera`, `atravessa`, `outro_motor`…) ficam FORA da assinatura, e
//! é deliberado:** elas não semeiam widget nenhum — só decidem uma frase — e pô-las ali faria a
//! secção re-semear-se no quadro em que alguém apaga uma câmera, com o valor que o artista estava a
//! arrastar.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::parallax_edits::InspectorParallaxInfo;

use crate::state::InspectorState;

/// ⚠️ **Só o que SEMEIA um widget entra** — ver o cabeçalho.
fn assinatura(info: &InspectorParallaxInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    let (lmin, lmax) = info.limits.unwrap_or(([0.0, 0.0], [0.0, 0.0]));
    // ⚠️ **A PRESENÇA de cada bloco entra na assinatura**, e não só o valor: anexar um
    // `ScrollRepeat` de fábrica escreve `[0, 0]`, que é o que a ausência também semeia — sem estes
    // três bits o painel não re-semearia o quadro em que a fileira nasce.
    for b in [
        info.repeat.is_some(),
        info.motion.is_some(),
        info.limits.is_some(),
    ] {
        b.hash(&mut h);
    }
    for v in info
        .factor
        .iter()
        .chain(info.repeat.unwrap_or([0.0, 0.0]).iter())
        .chain(info.motion.unwrap_or([0.0, 0.0]).iter())
        .chain(lmin.iter())
        .chain(lmax.iter())
    {
        v.to_bits().hash(&mut h);
    }
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_parallax() else {
        inspector_state.last_parallax_sig = None;
        return;
    };
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_parallax_sig == Some(sig) {
        return;
    }
    inspector_state.last_parallax_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    let rep = info.repeat.unwrap_or([0.0, 0.0]);
    let mov = info.motion.unwrap_or([0.0, 0.0]);
    let (lmin, lmax) = info.limits.unwrap_or(([0.0, 0.0], [0.0, 0.0]));
    // ⚠️ **A ORDEM é a do [`super::populate_parallax::NUMEROS`]**, e é assim que ela não pode
    // divergir: os dez ids são os mesmos dez, na mesma ordem, e um campo novo tem de entrar nos
    // dois.
    for (id, v) in [
        (crate::ids::INSP_PARALLAX_K_X, info.factor[0]),
        (crate::ids::INSP_PARALLAX_K_Y, info.factor[1]),
        (crate::ids::INSP_PARALLAX_TILE_X, rep[0]),
        (crate::ids::INSP_PARALLAX_TILE_Y, rep[1]),
        (crate::ids::INSP_PARALLAX_VEL_X, mov[0]),
        (crate::ids::INSP_PARALLAX_VEL_Y, mov[1]),
        (crate::ids::INSP_PARALLAX_MIN_X, lmin[0]),
        (crate::ids::INSP_PARALLAX_MIN_Y, lmin[1]),
        (crate::ids::INSP_PARALLAX_MAX_X, lmax[0]),
        (crate::ids::INSP_PARALLAX_MAX_Y, lmax[1]),
    ] {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, f64::from(v));
    }
}
