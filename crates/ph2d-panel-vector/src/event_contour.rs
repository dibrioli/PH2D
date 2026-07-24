//! O roteamento dos controles do **CONTOUR** — irmão do [`super`] pelo teto de 600 LOC do painel.
//!
//! Os três sliders passam pelas portas ÚNICAS do [`crate::contour_params`] — as mesmas de que o
//! `populate` deriva o `scale`/`offset` do chip e o `paint` tira a posição do trilho.

use crate::contour_params::{
    CONTOUR_ACCEL_MAX, CONTOUR_STEPS_DEFAULT, accel_from_track, accel_to_track, d_from_track,
    steps_from_track, steps_to_track,
};
use crate::ids;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;

/// `Some(consumido)` se `id` é um controle do Contour; `None` deixa o irmão continuar a decidir.
pub(super) fn contour_slider_event(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<bool> {
    if id == ids::VECTOR_CONTOUR_STEPS {
        return Some(super::forward_track(
            host,
            id,
            steps_to_track(CONTOUR_STEPS_DEFAULT),
            steps_from_track,
        ));
    }
    if id == ids::VECTOR_CONTOUR_OFFSET {
        return Some(super::forward_track(host, id, 0.5, d_from_track));
    }
    if id == ids::VECTOR_CONTOUR_ACCEL {
        return Some(super::forward_track(host, id, 0.5, accel_from_track));
    }
    // ⚠️ O campo numérico da Accel é o ÚNICO chip do painel que fala com o bus direto, e é
    // deliberado: a faixa dela é GEOMÉTRICA, e o `link_slider_number_mapped` do store só sabe
    // mapas afins (ver `populate_contour`). Sem link, o chip tem de fazer as duas coisas que o
    // link faria — empurrar o valor para o bus **e** pôr o slider onde o valor cai —, e as duas
    // acontecem aqui, num sítio só. Ele emite sob o id do SLIDER para a shell ter UMA porta.
    if id == ids::VECTOR_CONTOUR_ACCEL_NUM {
        // ⚠️ O número vem de um campo que o artista DIGITA, então não-finito é entrada
        // possível — e um `NaN` **passa pelo clamp intacto** (a comparação é falsa nos dois
        // lados) e envenenaria o `accel` do componente, que a `ring_distance` eleva a expoente.
        // A finitude é decidida ANTES; o clamp só cuida da faixa.
        let raw = host.store().number_value(id).unwrap_or(1.0);
        let value = if raw.is_finite() {
            raw.clamp(1.0 / CONTOUR_ACCEL_MAX, CONTOUR_ACCEL_MAX) // CLAMP-OK: bounds constantes finitas, valor provado finito acima
        } else {
            1.0
        };
        host.store_mut()
            .set_slider_value(ids::VECTOR_CONTOUR_ACCEL, accel_to_track(value));
        host.bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                ids::VECTOR_CONTOUR_ACCEL,
                value,
            )));
        return Some(true);
    }
    None
}
