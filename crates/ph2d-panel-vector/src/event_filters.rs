//! O roteamento dos SLIDERS do **FILTERS** (FX raster, plano 24) — irmão do [`super`] pelo teto de
//! 600 LOC do painel.
//!
//! Os sliders passam pela MESMA fronteira `forward_track` dos outros: a conversão track→documento
//! mora aqui, com o MESMO mapa que o `populate` dá ao chip (senão slider e campo divergiriam). Os
//! chips de TIPO (None/Blur/Glow/Shadow) NÃO passam por aqui — são `Click` puros encaminhados por
//! `forwards_plain_click`, e o drain da shell os traduz em armar/remover o `VecFilter`.

use crate::ids;
use crate::state::filters::{FILTER_OFFSET_MAX, FILTER_RADIUS_MAX};
use ph2d_editor_core::panel::PanelHostInternal;

/// `Some(consumido)` se `id` é um slider do Filters; `None` deixa o irmão continuar a decidir.
pub(super) fn filters_slider_event(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<bool> {
    if id == ids::VECTOR_FILTER_RADIUS {
        return Some(super::forward_track(host, id, 0.0, |t| t * FILTER_RADIUS_MAX));
    }
    if id == ids::VECTOR_FILTER_OPACITY {
        // Track == valor (`0..1`); a fronteira não converte nada.
        return Some(super::forward_track(host, id, 1.0, |t| t));
    }
    if id == ids::VECTOR_FILTER_OFFX || id == ids::VECTOR_FILTER_OFFY {
        return Some(super::forward_track(host, id, 0.5, |t| {
            t.mul_add(2.0 * FILTER_OFFSET_MAX, -FILTER_OFFSET_MAX)
        }));
    }
    None
}
