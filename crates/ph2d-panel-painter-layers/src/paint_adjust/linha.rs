//! ⭐⭐⭐ **AS LINHAS DA PILHA DE AJUSTES — pela porta do manual, com a coluna da SECÇÃO.**
//!
//! Filho do [`super`] por responsabilidade: aquele diz *que editor cada ajuste tem*; este diz *como
//! uma linha de ajuste se reparte entre o nome e o controlo*.
//!
//! # ⛔⛔ Porque saiu
//!
//! Até 2026-09-16 a coluna do nome era o literal `ADJ_LABEL_W = 44,0` (com o comentário *«slider-
//! param label column ("Contrast")»*), pintada em `TypeToken::Base` por `paint_text`. Medido nesse
//! dia sobre os **44** nomes que as pilhas genéricas pintam:
//!
//! | | cortados |
//! |---|---|
//! | coluna de `44` px (antes) | **17 de 44** — em TODA largura de painel |
//! | coluna da secção (agora) | `0` a partir do mínimo do dock |
//!
//! ⛔ E o próprio exemplo do comentário estava cortado: `Contrast` mede `53,3 px` em `Base`. Os
//! nomes da tabela de ajustes já vêm ABREVIADOS (`Shad Amt`, `High Wid`, `Vib`) para caber ali — e
//! mesmo assim não cabiam.
//!
//! # ⚠️ A caixa de ligar é uma linha de propriedade como as outras
//!
//! A linha de interruptor era `nome à esquerda … interruptor encostado à direita`, e ao lado dela
//! as barras passam a ter o nome na coluna. *Duas famílias de linha com o nome em sítios diferentes
//! no mesmo cartão são duas colunas* (spec §6-quinquies) — logo o interruptor vai para o início da
//! coluna do controlo, onde a marca de uma caixa de marcar também vive.

use super::*;
use ph2d_editor_core::widget::{PropertyRow, Seccao, colunas_da_linha, paint_property_label};
use ph2d_text::TextSystem;

/// ⭐ **A secção de uma pilha, medida sobre os nomes que ELA pinta** — as barras e os
/// interruptores juntos, porque os dois vivem no mesmo cartão (spec §6-quinquies).
///
/// ⚠️ Uma pilha sem nomes (o *Invert*) não tem coluna a medir; cai na metade, que é o default.
pub(crate) fn seccao_de(
    text_system: &mut TextSystem,
    barras: &[(&str, f32)],
    interruptores: &[(&str, bool)],
) -> Seccao {
    let rotulos: Vec<&str> = barras
        .iter()
        .map(|(n, _)| *n)
        .chain(interruptores.iter().map(|(n, _)| *n))
        .collect();
    if rotulos.is_empty() {
        return Seccao::apenas_campos(1);
    }
    Seccao::medida(text_system, 1, &rotulos)
}

/// As colunas de uma linha da pilha.
fn colunas(row: Rect, sec: Seccao) -> PropertyRow {
    colunas_da_linha(row.x, row.w, row.y, ROW_H_PX, sec)
}

/// O nome da linha, na coluna — alinhado à direita, elidido, no peso em que foi medido.
fn nome(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, label: &str, linha: &PropertyRow) {
    let font = TypeToken::Sm.px();
    paint_property_label(
        ctx.text_system,
        ctx.scene,
        label,
        linha.label.x,
        linha.label.y + (linha.label.h - font) * 0.5,
        font,
        linha.label.w,
        resolve(ColorToken::Text2, theme),
    );
}

/// Render one labeled `0..1` slider row (name column + horizontal slider): the shared body of the
/// generic slider rack AND the bespoke editors' sliders. The slider STORES `0..1`; the value is
/// derived from the live params each frame, so the caller passes `val01`. Registers the hit rect;
/// the caller advances `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_labeled_slider(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    val01: f32,
    row: Rect,
    sec: Seccao,
) {
    register_slider(
        ctx.host.store_mut(),
        id,
        val01,
        SliderOrientation::Horizontal,
    );
    let linha = colunas(row, sec);
    nome(ctx, theme, label, &linha);
    let mut slider = Slider::new(id, "")
        .accent(true)
        .visual(ctx.host.store().slider_visual(id));
    slider.value = val01.clamp(0.0, 1.0);
    paint_slider(&slider, linha.control, ctx.scene, theme);
    ctx.host.hit_index_mut().register(id, linha.control);
}

/// Render one toggle row (name column + a switch at the START of the control column, where a
/// checkbox mark also lives) painted with the live param value: the shared body of the generic
/// toggle rack AND the Channel-Mixer / Black & White switches. Registers the switch as a button
/// (click → the tool flips the param); the caller advances `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_toggle_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
    row: Rect,
    sec: Seccao,
) {
    let linha = colunas(row, sec);
    nome(ctx, theme, label, &linha);
    let toggle_rect = Rect::new(
        linha.control.x,
        row.y + (ROW_H_PX - ADJ_TOGGLE_H) * 0.5,
        ADJ_TOGGLE_W.min(linha.control.w),
        ADJ_TOGGLE_H,
    );
    let toggle = Toggle::new(id, "")
        .visual(ctx.host.store().toggle_visual(id))
        .on(on);
    paint_toggle(&toggle, toggle_rect, ctx.scene, theme);
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, toggle_rect);
}
