//! ⭐⭐ **A LISTA DE CAMADAS do painel do Painter** — cortada do `paint.rs` em 2026-09-09 pelo tecto
//! de LOC do FICHEIRO (601/600), logo a seguir ao corte que tirou a mesma lista da função `paint`.
//!
//! ⚠️ **Os dois cortes são o mesmo por responsabilidade, em duas escalas:** o `paint.rs` responde
//! *«que moldura é esta e onde começa o corpo?»*, e este ficheiro *«que linhas há, e como se
//! desenham»*. *Um tecto de função e um tecto de ficheiro medem coisas diferentes, e nesta crate
//! caiu-se nos dois no mesmo dia.*

use crate::paint_rows::{paint_drop_indicator, paint_layer_subtree};
use crate::state;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::PANEL_HEAD_PAD;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, TypeToken};
use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};

/// ⭐⭐ **A LISTA DE CAMADAS** — cortada do [`paint`] em 2026-09-09 pelo tecto de LOC (264/200), e o
/// corte é por RESPONSABILIDADE: aquele responde *«que moldura é esta e onde começa o corpo?»* e
/// esta responde *«que linhas há, e como se desenham»*.
///
/// ⚠️ **Os três valores que ela devolve são os que o resto do quadro precisa** — o `y` onde a lista
/// acabou (o rodapé desenha-se a partir dele), os ids das linhas ARRASTÁVEIS (que o despacho lê para
/// saber o que começa um reparent) e o **fantasma** do arrasto, que é pintado no fim, sem recorte.
/// ⛔ Devolvê-los por `&mut` em vez de os deixar aqui dentro seria dar dois donos ao mesmo cursor.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_layer_rows(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    // ⚠️ O rect do corpo entra para o indicador de largada ler a MESMA banda que o desenho usa.
    _body_rect: Rect,
    content_w: f32,
    y_in: f32,
) -> (
    f32,
    std::collections::BTreeSet<ph2d_a11y::NodeId>,
    Option<(String, f32, f32)>,
) {
    let mut y = y_in;
    let mut painter_row_ids: std::collections::BTreeSet<ph2d_a11y::NodeId> =
        std::collections::BTreeSet::new();
    let mut ghost: Option<(String, f32, f32)> = None;
    // ⚠️ **Filtra pela FAMÍLIA** (2026-09-14): o slot de arrasto passou a ser um só para todos os
    // painéis, e sem esta pergunta o fantasma das camadas desenhava-se enquanto alguém arrasta uma
    // TAG no painel ao lado.
    let dragging = ctx
        .host
        .store()
        .panel_row_drag()
        .filter(|&(f, d)| {
            f == ph2d_editor_core::interaction::PanelRowFamily::PainterLayer && d.active
        })
        .map(|(_, d)| d);
    match state::current_layers() {
        Some(stack) if !stack.is_empty() => {
            painter_row_ids = stack
                .all_ids()
                .filter(|&l| !stack.is_mask(l)) // masks are selectable but not draggable
                .map(|l| painter_layer_widget_id(l.0, PainterLayerWidget::Row))
                .collect();
            let active = stack.active();
            // W3 multi-select: the rows to highlight (active = strong outline,
            // the rest = soft wash). Published by the bridge each frame.
            let selected = state::current_selection();
            // Full-row drop bands collected during the walk (id, full-row rect,
            // is_group) so the indicator below mirrors `find_painter_layer_drop`
            // exactly (same rows, same 30/40/30) — WYSIWYG drop.
            let mut drag_rows: Vec<(ph2d_a11y::NodeId, Rect, bool)> = Vec::new();
            y = paint_layer_subtree(
                ctx,
                theme,
                &stack,
                stack.root(),
                active,
                &selected,
                0,
                rect.x + PANEL_HEAD_PAD,
                content_w,
                y,
                dragging,
                &mut drag_rows,
            );
            if let Some(d) = dragging {
                // Live drop indicator, on top of the rows (still inside the body
                // clip so it can't bleed over the header/footer).
                paint_drop_indicator(
                    ctx,
                    theme,
                    &drag_rows,
                    d.cursor_y,
                    d.dragged,
                    rect.x + PANEL_HEAD_PAD,
                    content_w,
                );
                // Decode the dragged NodeId → its layer name for the ghost pill.
                ghost = stack
                    .all_ids()
                    .find(|lid| {
                        painter_layer_widget_id(lid.0, PainterLayerWidget::Row) == d.dragged
                    })
                    .and_then(|lid| stack.get(lid))
                    .map(|l| (l.name.clone(), d.cursor_x, d.cursor_y));
            }
        }
        _ => {
            let font = TypeToken::Base.px();
            paint_text(
                ctx.text_system,
                ctx.scene,
                tr("panel.painter_layers.layers.no_layers"),
                rect.x + PANEL_HEAD_PAD,
                y,
                font,
                content_w,
                resolve(ColorToken::Text2, theme),
            );
            y += font + ph2d_tokens::control_gap_px();
        }
    }
    (y, painter_row_ids, ghost)
}
