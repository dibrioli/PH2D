//! **A semente dos campos das secções NAV REGION e NAV AGENT** (plano 30, W4).
//!
//! ⛔⛔ **Nasce no MESMO commit que as secções** — a lição do RAIO (19/09): sem ela o painel mostra
//! os números de FÁBRICA do [`super::populate_nav`] sobre um objecto autorado, e quatro números
//! plausíveis são a pior forma deste defeito (eles não parecem partidos).
//!
//! ⚠️ **De ARESTA, nunca por quadro** (a assinatura do instantâneo), e o foco e o arrasto ganham-lhe:
//! *a mão do artista ganha ao instantâneo*. ⚠️ **A leitura VIVA fica de fora da assinatura** — ela
//! muda a cada tique e não semeia widget nenhum.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::nav_edits::InspectorNavInfo;
use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

/// Os números a semear, na ordem do [`super::populate_nav::NUMEROS`] — ⚠️ só os da secção que o
/// objecto TEM (os da outra ficam com o que estiverem; não são pintados).
fn numeros(info: &InspectorNavInfo) -> Vec<(ph2d_a11y::NodeId, f32)> {
    let mut v = Vec::new();
    if let Some(r) = info.region {
        v.push((crate::ids::INSP_NAV_HALF_W, r.half_w));
        v.push((crate::ids::INSP_NAV_HALF_H, r.half_h));
    }
    if let Some(a) = info.agent.as_ref() {
        v.push((crate::ids::INSP_NAV_TARGET_X, a.alvo_ponto[0]));
        v.push((crate::ids::INSP_NAV_TARGET_Y, a.alvo_ponto[1]));
        v.push((crate::ids::INSP_NAV_RADIUS, a.radius));
        v.push((crate::ids::INSP_NAV_ARRIVE, a.arrive));
        v.push((crate::ids::INSP_NAV_REPATH, a.repath));
        v.push((crate::ids::INSP_NAV_STUCK, a.stuck_after));
    }
    v
}

/// ⚠️ **Só o que SEMEIA um widget entra** — ver o cabeçalho sobre a leitura viva.
fn assinatura(info: &InspectorNavInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    for (id, v) in numeros(info) {
        id.hash(&mut h);
        v.to_bits().hash(&mut h);
    }
    if let Some(a) = info.agent.as_ref() {
        a.alvo_nome.hash(&mut h);
        a.on_arrived.hash(&mut h);
        a.on_no_path.hash(&mut h);
        a.on_stuck.hash(&mut h);
    }
    h.finish()
}

/// **A semente das secções** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_nav() else {
        inspector_state.last_nav_sig = None;
        return;
    };
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_nav_sig == Some(sig) {
        return;
    }
    inspector_state.last_nav_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, v) in numeros(&info) {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, f64::from(v));
    }
    if let Some(a) = info.agent.as_ref() {
        for (id, t) in [
            (crate::ids::INSP_NAV_TARGET_NAME, a.alvo_nome.as_str()),
            (crate::ids::INSP_NAV_ON_ARRIVED, a.on_arrived.as_str()),
            (crate::ids::INSP_NAV_ON_NO_PATH, a.on_no_path.as_str()),
            (crate::ids::INSP_NAV_ON_STUCK, a.on_stuck.as_str()),
        ] {
            crate::sync_text_field::escreve_texto(host, focus, id, t);
        }
    }
}
