//! ⭐ **A caixa de marcar de um cartão** (T64, `docs/CartoesUI/05` §4), a única: a linha do corpo, o
//! seletor de datas e a checklist do verso pintam ESTA.

use super::CheckboxValue;
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect};
use crate::widget::card_ink::CardInk;
use crate::zones::Rect;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

/// A caixa `CHECK_BOX` centrada na altura em `x`, numa fila que começa em `y` e mede `h`.
#[must_use]
pub fn card_check_rect(x: f32, y: f32, h: f32) -> Rect {
    Rect::new(x, y + (h - m::CHECK_BOX) * 0.5, m::CHECK_BOX, m::CHECK_BOX)
}

/// [`paint_card_check`] com o valor inteiro: «misto» é o acento com um traço, como o `Checkbox`.
pub fn paint_card_check_value(
    scene: &mut VectorScene,
    ink: CardInk,
    caixa: Rect,
    valor: CheckboxValue,
    hover: bool,
) {
    if valor != CheckboxValue::Indeterminate {
        return paint_card_check(scene, ink, caixa, valor == CheckboxValue::Checked, hover);
    }
    let fundo = ink.color(ColorToken::KanbanBlueBold);
    fill_rounded_rect(scene, caixa, m::SEG_RADIUS, fundo);
    let traco = StrokeToken::Default.px();
    let lado = caixa.w * 0.5;
    fill_rounded_rect(
        scene,
        Rect::new(
            caixa.x + (caixa.w - lado) * 0.5,
            caixa.y + (caixa.h - traco) * 0.5,
            lado,
            traco,
        ),
        traco * 0.5,
        ink.color(ColorToken::KanbanBtnOnFg),
    );
}

/// Cheia de acento com o ✓, ou vazia com o contorno do rótulo (acesa no hover).
pub fn paint_card_check(scene: &mut VectorScene, ink: CardInk, caixa: Rect, on: bool, hover: bool) {
    let borda = ink.color(ColorToken::CardLabel);
    paint_card_check_com_borda(scene, ink, caixa, on, hover, borda);
}

/// [`paint_card_check`] com o contorno da caixa VAZIA em `borda`: uma barra mais escura que o cartão
/// (a da Timeline) pede um contorno mais claro para a caixa não sumir no fundo.
pub fn paint_card_check_com_borda(
    scene: &mut VectorScene,
    ink: CardInk,
    caixa: Rect,
    on: bool,
    hover: bool,
    borda: ph2d_vector::Color,
) {
    if on {
        let fundo = ink.color(ColorToken::KanbanBlueBold);
        fill_rounded_rect(scene, caixa, m::SEG_RADIUS, fundo);
        let marca = ink.color(ColorToken::KanbanBtnOnFg);
        paint_icon(
            scene,
            IconId::Check,
            caixa,
            marca,
            StrokeToken::Default.px(),
        );
    } else {
        let fundo = if hover {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(scene, caixa, m::SEG_RADIUS, ink.color(fundo));
        // FRAME-RAW-OK: o quadradinho da caixa de marcar É o contorno (T64): sem ele a caixa desapareceria
        stroke_rounded_rect(scene, caixa, m::SEG_RADIUS, m::FIELD_BORDER, borda);
    }
}
