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
use crate::state::filters::{
    FILTER_DETAIL_MAX, FILTER_OFFSET_MAX, FILTER_RADIUS_MAX, FILTER_SCALE_MAX, FILTER_SEED_MAX,
};
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
                // As opções do popover de MISTURA — `Click` puros, como as pontas do traço. O
                // CHIP não entra: ele é um `Dropdown`, e abrir/fechar é do dispatch genérico.
                || (0..ids::MAX_FILTER_BLENDS).any(|m| id == ids::filter_blend_option_id(r, m))
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
        if id == ids::filter_scale_id(row) {
            return Some(super::forward_track(host, id, 0.0, |t| {
                t * FILTER_SCALE_MAX
            }));
        }
        // ⚠️ Detail e Seed atravessam a fronteira como CONTAGEM — o arredondamento mora aqui, no
        // MESMO mapa que o `populate` deu ao chip, senão o slider e o campo diriam números
        // diferentes sobre a mesma oitava.
        if id == ids::filter_detail_id(row) {
            return Some(super::forward_track(host, id, 0.0, |t| {
                t.mul_add(FILTER_DETAIL_MAX - 1.0, 1.0).round()
            }));
        }
        if id == ids::filter_seed_id(row) {
            return Some(super::forward_track(host, id, 0.0, |t| {
                (t * FILTER_SEED_MAX).round()
            }));
        }
        if id == ids::filter_offx_id(row) || id == ids::filter_offy_id(row) {
            return Some(super::forward_track(host, id, 0.5, |t| {
                t.mul_add(2.0 * FILTER_OFFSET_MAX, -FILTER_OFFSET_MAX)
            }));
        }
    }
    None
}
