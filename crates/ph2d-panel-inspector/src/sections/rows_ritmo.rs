//! **As portas do ritmo vertical** ([05 §4](../../../../../docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md)):
//! rótulo solto, título de grupo, lista e fila de botões. Num cartão, os vãos são os de `metrics`; fora
//! dele, os de sempre. O portão é `lei_do_ritmo`.

use super::*;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::cards::censo;
use ph2d_editor_core::widget::card_ink::card_body;

/// O nome solto sobre uma lista ou grade: o rótulo de um campo, com os vãos dele. Devolve onde o controlo
/// começa. Fora de um cartão, o texto de sempre seguido do vão de sempre.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rotulo_do_bloco(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
) -> f32 {
    let Some(ink) = card_body() else {
        let font = TypeToken::Sm.px();
        paint_text(
            text_system,
            scene,
            texto,
            x,
            y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        return y + font + ph2d_tokens::control_gap_px();
    };
    let topo = y + m::LABEL_TOP;
    paint_text(
        text_system,
        scene,
        texto,
        x,
        topo,
        m::LABEL_PX,
        w,
        ink.color(ColorToken::CardLabel),
    );
    topo + m::LABEL_PX + m::LABEL_BOTTOM
}

/// O título de um grupo de linhas dentro do cartão: `GROUP_TOP` acima. Quem vem a seguir traz o vão dele.
#[allow(clippy::too_many_arguments)]
pub(crate) fn titulo_de_grupo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
    token: ColorToken,
) -> f32 {
    let font = TypeToken::Sm.px();
    let em_cartao = card_body().is_some();
    let topo = if em_cartao { y + m::GROUP_TOP } else { y };
    paint_text(
        text_system,
        scene,
        texto,
        x,
        topo,
        font,
        w,
        resolve(token, theme),
    );
    if em_cartao {
        topo + font
    } else {
        topo + font + ph2d_tokens::control_gap_px()
    }
}

/// O `y` onde uma lista começa: num cartão ela é uma linha, com o vão da grelha em cima.
pub(crate) fn antes_da_lista(y: f32) -> f32 {
    if card_body().is_some() {
        y + m::FIELD_ROW_GAP
    } else {
        y
    }
}

/// O `y` depois da última linha de uma lista: num cartão, quem vem a seguir traz o vão dele.
pub(crate) fn depois_da_lista(y: f32) -> f32 {
    if card_body().is_some() {
        y
    } else {
        y + ph2d_tokens::control_gap_px()
    }
}

/// A caixa de uma lista suspensa sem rótulo em cima, e a coluna da marca: num cartão, da altura de todo
/// controlo, com o vão da grelha. Devolve também o `y` seguinte.
pub(crate) fn caixa_do_chip(x: f32, w: f32, y: f32) -> (Rect, Rect, f32) {
    if card_body().is_some() {
        let topo = y + m::FIELD_ROW_GAP;
        let cw = w - m::KEY_MARK_BOX - m::FIELD_GAP;
        let marca = Rect::new(x + w - m::KEY_MARK_BOX, topo, m::KEY_MARK_BOX, m::FIELD_H);
        return (Rect::new(x, topo, cw, m::FIELD_H), marca, topo + m::FIELD_H);
    }
    let (cw, marca) = ph2d_editor_core::widget::form_row_columns(x, w, y, ph2d_tokens::ROW_H_PX);
    (
        Rect::new(x, y, cw, ph2d_tokens::ROW_H_PX),
        marca,
        y + ph2d_tokens::row_pitch_px(),
    )
}

/// As larguras de uma fila de botões num cartão: colunas iguais se todos os textos cabem (lei 4 de
/// 05 §4); se não, pelo que cada um pede. Nunca um texto cortado.
fn colunas_da_fila(text_system: &mut TextSystem, rotulos: &[&str], w: f32) -> Vec<f32> {
    let font = ph2d_editor_core::widget::button_label_font();
    let n = rotulos.len();
    let util = w - m::FIELD_GAP * n.saturating_sub(1) as f32;
    let pede: Vec<(f32, bool)> = rotulos
        .iter()
        .map(|l| {
            let t = text_system.prefix_width(l, font);
            (t + 2.0 * m::BUTTON_TEXT_PAD, false)
        })
        .collect();
    let igual = util / n.max(1) as f32;
    if pede.iter().all(|p| p.0 <= igual) {
        return vec![igual; n];
    }
    m::seg_widths(&pede, util)
}

/// ⭐ **A fila de botões de uma lista** (`+ Add` · `Remove`, `Browse` · `Preview` · `Stop`): só os ativos.
/// Num cartão, botões soltos da altura de todo controlo; fora dele, a porta do grupo. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fila_de_botoes(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    pecas: &[(bool, NodeId, &str)],
) -> f32 {
    let ativas: Vec<(NodeId, &str)> = pecas
        .iter()
        .filter(|p| p.0)
        .map(|p| (p.1, p.2))
        .collect();
    if ativas.is_empty() {
        return y;
    }
    let rotulos: Vec<&str> = ativas.iter().map(|p| p.1).collect();
    if card_body().is_none() {
        let seg = ph2d_editor_core::widget::segment_rects_for(
            Rect::new(x, y, w, ALTURA_DE_CAMPO),
            &rotulos,
            ph2d_editor_core::widget::button_label_font(),
            text_system,
        );
        for (&(id, rotulo), &(rect, group)) in ativas.iter().zip(&seg) {
            hit_index.register(id, rect);
            paint_button(
                &Button::new(id, rotulo)
                    .kind(ButtonKind::Default)
                    .visual(store.button_visual(id))
                    .in_group(group),
                rect,
                scene,
                text_system,
                theme,
            );
        }
        return y + ALTURA_DE_CAMPO + ph2d_tokens::control_gap_px();
    }
    let topo = y + m::FIELD_ROW_GAP;
    let larguras = colunas_da_fila(text_system, &rotulos, w);
    let mut cx = x;
    for (&(id, rotulo), &cw) in ativas.iter().zip(&larguras) {
        let rect = Rect::new(cx, topo, cw, m::FIELD_H);
        hit_index.register(id, rect);
        paint_button(
            &Button::new(id, rotulo)
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
        let pede = text_system.prefix_width(rotulo, ph2d_editor_core::widget::button_label_font())
            + 2.0 * m::BUTTON_TEXT_PAD;
        censo::fora_da_linha(rotulo, pede, rect);
        cx += cw + m::FIELD_GAP;
    }
    topo + m::FIELD_H
}
