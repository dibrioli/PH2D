//! O roteamento dos controles do **FILTERS** (a pilha de FX raster, plano 24) — irmão do [`super`]
//! pelo teto de 600 LOC do painel.
//!
//! Os sliders passam pela MESMA fronteira `forward_track` dos outros: a conversão track→documento
//! mora aqui, com o MESMO mapa que o `populate` dá ao chip (senão slider e campo divergiriam). Os
//! BOTÕES da pilha (Add / ✕ / ↑ / ↓ / 👁 / a swatch) NÃO passam por aqui — são `Click` puros
//! encaminhados por `forwards_plain_click`, e o drain da shell os traduz em edições do `VecFilter`.
//!
//! ⚠️ A varredura é sobre o TETO de linhas (`MAX_FILTER_ROWS`): os ids são hashes de NOME, então
//! não há aritmética que os inverta. É barato, e é o mesmo padrão do bloco `vector_fx_*`.

use crate::ids;
use crate::state::filters::{FILTER_OFFSET_MAX, FILTER_RADIUS_MAX};
use ph2d_editor_core::panel::PanelHostInternal;

/// Este id é um BOTÃO da pilha de filtros (Add / Remove / Up / Down / Hide)?
///
/// ⚠️ **A swatch de cor NÃO é um deles** — ela é alvo de PICKER: o Down abre o OKLCH partilhado e
/// nenhum `Click` chega a sair (a shell lê a escolha pelo `picker_target`, como no Contour).
pub(super) fn is_filter_button(id: ph2d_a11y::NodeId) -> bool {
    (0..ids::MAX_FILTER_KINDS).any(|k| id == ids::filter_add_id(k))
        || (0..ids::MAX_FILTER_ROWS).any(|r| {
            id == ids::filter_remove_id(r)
                || id == ids::filter_up_id(r)
                || id == ids::filter_down_id(r)
                || id == ids::filter_hide_id(r)
                || (0..ids::MAX_FILTER_MODES).any(|m| id == ids::filter_mode_id(r, m))
        })
}

/// `Some(consumido)` se `id` é um slider do Filters; `None` deixa o irmão continuar a decidir.
pub(super) fn filters_slider_event(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<bool> {
    for row in 0..ids::MAX_FILTER_ROWS {
        if id == ids::filter_radius_id(row) {
            return Some(super::forward_track(host, id, 0.0, |t| {
                t * FILTER_RADIUS_MAX
            }));
        }
        if id == ids::filter_opacity_id(row) {
            // Track == valor (`0..1`); a fronteira não converte nada.
            return Some(super::forward_track(host, id, 1.0, |t| t));
        }
        if id == ids::filter_offx_id(row) || id == ids::filter_offy_id(row) {
            return Some(super::forward_track(host, id, 0.5, |t| {
                t.mul_add(2.0 * FILTER_OFFSET_MAX, -FILTER_OFFSET_MAX)
            }));
        }
    }
    None
}
