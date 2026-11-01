//! ⭐⭐ **Os painéis à ESQUERDA de Fill e Stroke** (F44, dono 01/11) — o mecanismo da F43
//! ([`crate::fx_side::side_panel`]) com três corpos: o preenchimento de BASE (tipo, degradê, token),
//! uma camada EXTRA da pilha (largura, deslocamento, offset, mistura) e o «Stroke settings».
//!
//! Cada um é mantido aberto por um `Dropdown` (a caixa da linha, ou o ⚙ da linha fixa): o
//! light-dismiss, a roda e o polegar vêm de graça. Uma linha que o corpo não pintou neste quadro
//! (secção dobrada, seleção vazia) FECHA — senão o painel voltava sozinho ao reaparecer a linha.
//!
//! ⚠️ **A camada aberta da pilha** (`state::open_layer`, o índice que a shell lê para gravar a
//! largura, o deslocamento e a mistura) passa a ser DERIVADA do `Dropdown` aberto, aqui, uma vez
//! por quadro — duas verdades para «que camada está aberta?» divergiriam no primeiro clique fora.

use crate::fx_side::{set_open, side_head, side_panel};
use crate::paint_sections::fill_stroke::take_abertos;
use crate::paint_sections::fx_list::is_open;
use crate::{ids, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Theme;
use ph2d_vec_scene::MAX_PAINT_LAYERS;

/// O passe diferido: o painel aberto (há no máximo um — abrir outro fecha este pelo clique fora).
pub(crate) fn paint(ctx: &mut PaintCtx, panel: Rect, theme: Theme) {
    let abertos = take_abertos();
    sync_open_layer(ctx.host.store());
    match abertos.fill {
        Some(line) => side_panel(
            ctx,
            ids::VECTOR_FILL_ROW,
            line,
            panel,
            theme,
            &mut |b, y| {
                let y = side_head(
                    b,
                    None,
                    tr("panel.vector.section.fill"),
                    ids::VECTOR_FILL_PROPS_CLOSE,
                    None,
                    None,
                    y,
                );
                b.fill_props_body(y)
            },
        ),
        None => set_open(ctx.host.store_mut(), ids::VECTOR_FILL_ROW, false),
    }
    match abertos.stroke {
        Some(line) => {
            let snap = state::current_snapshot();
            side_panel(
                ctx,
                ids::VECTOR_STROKE_SETTINGS,
                line,
                panel,
                theme,
                &mut |b, y| {
                    let y = side_head(
                        b,
                        None,
                        tr("panel.vector.stroke.settings"),
                        ids::VECTOR_STROKE_PROPS_CLOSE,
                        None,
                        None,
                        y,
                    );
                    b.stroke_settings_body(&snap, y)
                },
            );
        }
        None => set_open(ctx.host.store_mut(), ids::VECTOR_STROKE_SETTINGS, false),
    }
    let camada = abertos.layer.and_then(|(i, line)| {
        let a = state::current_appearance()?;
        Some((i, line, *a.layers.get(i)?, a.layers.len()))
    });
    match camada {
        Some((i, line, row, n)) => {
            let name = tr(if row.is_fill {
                "panel.vector.paint.fill"
            } else {
                "panel.vector.paint.stroke"
            });
            // ↑ sobe a camada no DOCUMENTO (para o topo, `i + 1`); ↓ desce.
            let up = (i + 1 < n).then(|| ids::vector_paint_up_id(i));
            let down = (i > 0).then(|| ids::vector_paint_down_id(i));
            side_panel(
                ctx,
                ids::vector_paint_row_id(i),
                line,
                panel,
                theme,
                &mut |b, y| {
                    let y = side_head(b, None, name, ids::VECTOR_PAINT_PROPS_CLOSE, down, up, y);
                    b.layer_props(&row, y)
                },
            );
        }
        None => close_layers(ctx.host.store_mut()),
    }
}

/// ⭐ **Abre o painel da camada `i`** — o gesto do clique na caixa da linha, para quem monta uma cena
/// (smoke, teste) sem rato.
pub fn open_paint_layer(store: &mut WidgetStore, i: usize) {
    close_layers(store);
    set_open(store, ids::vector_paint_row_id(i), true);
    state::set_open_layer(Some(i));
}

/// A camada aberta da pilha = a linha cujo `Dropdown` está aberto.
fn sync_open_layer(store: &WidgetStore) {
    state::set_open_layer(open_layer_row(store));
}

fn close_layers(store: &mut WidgetStore) {
    for i in 0..MAX_PAINT_LAYERS {
        set_open(store, ids::vector_paint_row_id(i), false);
    }
}

/// O `Dropdown` da camada aberta, se há.
fn open_layer_row(store: &WidgetStore) -> Option<usize> {
    (0..MAX_PAINT_LAYERS).find(|&i| is_open(store, ids::vector_paint_row_id(i)))
}

/// ⭐ **Os cliques que mexem no que está aberto**, antes do verbo do painel (que corre a seguir,
/// salvo no X). `true` = consumido.
///
/// - os três X → fecham o painel deles;
/// - «−» de uma camada, «+ Fill», «+ Stroke» → fecham a camada aberta (a shell fecha a dela: um
///   índice guardado não sobrevive à lista que o explicava);
/// - ↑ / ↓ da camada aberta → o painel ACOMPANHA a camada para o índice novo.
pub(crate) fn on_click(store: &mut WidgetStore, id: NodeId) -> bool {
    let fechar: [(NodeId, NodeId); 2] = [
        (ids::VECTOR_FILL_PROPS_CLOSE, ids::VECTOR_FILL_ROW),
        (ids::VECTOR_STROKE_PROPS_CLOSE, ids::VECTOR_STROKE_SETTINGS),
    ];
    if let Some(&(_, alvo)) = fechar.iter().find(|(x, _)| *x == id) {
        set_open(store, alvo, false);
        return true;
    }
    if id == ids::VECTOR_PAINT_PROPS_CLOSE {
        close_layers(store);
        return true;
    }
    if id == ids::VECTOR_PAINT_ADD_FILL
        || id == ids::VECTOR_PAINT_ADD_STROKE
        || (0..MAX_PAINT_LAYERS).any(|i| id == ids::vector_paint_del_id(i))
    {
        close_layers(store);
        return false;
    }
    if let Some(i) = open_layer_row(store) {
        let to = if id == ids::vector_paint_up_id(i) {
            Some(i + 1)
        } else if id == ids::vector_paint_down_id(i) && i > 0 {
            Some(i - 1)
        } else {
            None
        };
        if let Some(to) = to.filter(|&t| t < MAX_PAINT_LAYERS) {
            set_open(store, ids::vector_paint_row_id(i), false);
            set_open(store, ids::vector_paint_row_id(to), true);
        }
    }
    false
}
