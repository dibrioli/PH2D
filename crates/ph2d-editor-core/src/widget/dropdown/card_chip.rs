//! A cara de uma lista suspensa DENTRO de um cartão (T29, `docs/CartoesUI`): a caixa do cartão, o
//! valor e a seta, pela tinta dele ([`CardInk`]).

use crate::icons::IconId;
use crate::widget::card_ink::CardInk;
use crate::zones::Rect;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

/// T29: a cara de uma lista suspensa do Inspector dentro de um cartão — a caixa do cartão (um degrau
/// abaixo, acesa no hover), o valor e a seta, como o `Widget::Dropdown`. O id, o estado e as opções
/// continuam os do Inspector.
pub fn chip(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: CardInk,
    r: Rect,
    texto: &str,
    aceso: bool,
) {
    let fundo = if aceso {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    crate::paint::fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(fundo));
    let g = m::ICON_GLYPH;
    crate::paint::paint_text_elided(
        text_system,
        scene,
        texto,
        r.x + m::FIELD_PAD_R,
        r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        (r.w - m::FIELD_PAD_R * 2.0 - g).max(0.0),
        ink.color(ColorToken::Text1),
    );
    let seta = Rect::new(r.x + r.w - m::FIELD_PAD_R - g, r.y + (r.h - g) * 0.5, g, g);
    let cor = ink.color(ColorToken::Text2);
    crate::paint::paint_icon(
        scene,
        IconId::ChevronDown,
        seta,
        cor,
        StrokeToken::Default.px(),
    );
}

/// O raio do realce de uma linha da lista aberta de um cartão.
pub(super) const RAIO_DA_LINHA: f32 = m::SEG_RADIUS;

/// O nome de uma linha dessa lista: à esquerda, como a T16.
pub(super) fn texto_da_linha(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    r: Rect,
    texto: &str,
    cor: ph2d_vector::Color,
) {
    crate::paint::paint_text(
        text_system,
        scene,
        texto,
        r.x + m::FIELD_PAD_R,
        r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        (r.w - m::FIELD_PAD_R * 2.0).max(0.0),
        cor,
    );
}
