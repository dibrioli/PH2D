//! **A semente dos números da secção TOP-DOWN PLAYER** (plano 28, W5).
//!
//! ⛔⛔ **Ela NÃO existia desde o TOP-20 #13, e foi achada ao acrescentar uma linha à secção:** os
//! sete números do mover eram registados com os valores de PARTIDA do `populate` e **nunca** eram
//! reescritos com os do objecto — um mover autorado a `9 m/s` mostrava `4`, e escrever por cima de
//! outro campo gravava os `4` no documento. É a MESMA forma que o raio, a vigia do contador e as duas
//! de 10/09 pagaram, e a mesma frase: *eles não parecem partidos*.
//!
//! # ⚠️ De ARESTA, nunca por quadro
//!
//! A assinatura do instantâneo é a aresta; o foco e o arrasto ganham-lhe (a mão do artista vence).

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::topdown_edits::InspectorTopDownInfo;

use crate::ids;
use crate::state::InspectorState;

/// Os números a semear, com o valor do instantâneo — um por linha do [`crate::populate_topdown`].
pub(crate) fn numeros(i: &InspectorTopDownInfo) -> [(ph2d_a11y::NodeId, f64); 8] {
    [
        (ids::INSP_TD_SPEED, f64::from(i.speed)),
        (ids::INSP_TD_ACCEL, f64::from(i.acceleration)),
        (ids::INSP_TD_DECEL, f64::from(i.deceleration)),
        (
            ids::INSP_TD_KNOCKBACK_RECOVERY,
            f64::from(i.knockback_recovery),
        ),
        (ids::INSP_TD_VIEW_ANGLE, f64::from(i.viewpoint_angle_deg)),
        (ids::INSP_TD_TURN_SPEED, f64::from(i.turn_speed_deg)),
        (ids::INSP_TD_MIN_SLIDE, f64::from(i.min_slide_angle_deg)),
        (ids::INSP_TD_MAX_SLIDES, f64::from(i.max_slides)),
    ]
}

fn assinatura(i: &InspectorTopDownInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    i.entity_bits.hash(&mut h);
    for (id, v) in numeros(i) {
        id.0.hash(&mut h);
        v.to_bits().hash(&mut h);
    }
    h.finish()
}

/// **A semente** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_topdown() else {
        inspector_state.last_topdown_sig = None;
        return;
    };
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_topdown_sig == Some(sig) {
        return;
    }
    inspector_state.last_topdown_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, v) in numeros(&info) {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, v);
    }
}
