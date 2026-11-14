//! ⭐ **A caixa de marcar de um cartão** (T64, `docs/CartoesUI/05` §4), a única: a linha do corpo, o
//! seletor de datas e a checklist do verso pintam ESTA.

use super::CheckboxValue;
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect};
use crate::widget::card_ink::CardInk;
use crate::widget::key_mark::Rotulo;
use crate::zones::Rect;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
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

/// ⭐ **As medidas da caixa em MOLDURA, à Godot** (dono 13/11,
/// `project-memory/project_checkbox_in_a_frame_like_godot_2026_11_13.md`): moram com a caixa, e não no
/// painel que a arruma, para que a linha de propriedade e quem pinta fora de um cartão perguntem a
/// mesma largura sem subir ao `panel`.
///
/// A palavra da moldura: nomeia o que a marca liga; quem diz o estado é o ✓.
#[must_use]
pub fn card_check_word() -> &'static str {
    ph2d_i18n::tr("chrome.checkbox.on")
}

/// A moldura com ✓ e «On» inteiros.
pub fn card_check_frame_w(text_system: &mut TextSystem) -> f32 {
    let on = text_system
        .prefix_width(card_check_word(), m::VALUE_PX)
        .ceil();
    m::FIELD_PAD_R + m::CHECK_BOX + m::CHECK_GAP + on + m::FIELD_PAD_R
}

/// A largura em que a linha de marcar com o nome EM CIMA mostra tudo: o maior entre o nome com o
/// losango e a moldura com ✓ e «On».
pub fn card_check_w(text_system: &mut TextSystem, label: Rotulo<'_>) -> f32 {
    let nome = text_system.prefix_width(label.texto, m::LABEL_PX).ceil() + m::KEY_MARK_BOX;
    nome.max(card_check_frame_w(text_system))
}

/// A largura da caixa EM FILA (o nome dentro da moldura, ao lado do ✓, e o losango ao lado).
pub fn card_check_in_row_w(text_system: &mut TextSystem, nome: &str) -> f32 {
    let tw = text_system.prefix_width(nome, m::VALUE_PX).ceil();
    m::FIELD_PAD_R + m::CHECK_BOX + m::CHECK_GAP + tw + m::FIELD_PAD_R + m::KEY_MARK_BOX
}
