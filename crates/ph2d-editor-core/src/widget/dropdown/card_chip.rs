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
    chip_com_icone(scene, text_system, ink, r, (None, texto), aceso);
}

/// O espaço que um ícone à esquerda do valor ocupa no [`chip_com_icone`] (o glifo e o vão).
#[must_use]
pub fn largura_do_icone(icone: Option<IconId>) -> f32 {
    icone.map_or(0.0, |_| m::HEAD_ICON_GLYPH + m::SEG_ICON_GAP)
}

/// O [`chip`] com um ícone à esquerda do valor — o glifo do que está escolhido (o tipo de um clip na
/// Timeline: a lista fechada mostra o mesmo ícone que a aberta, dono 12/11).
pub fn chip_com_icone(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: CardInk,
    r: Rect,
    (icone, texto): (Option<IconId>, &str),
    aceso: bool,
) {
    let li = largura_do_icone(icone);
    if crate::widget::card_censo::ligado() {
        let tw = text_system.prefix_width(texto, m::VALUE_PX).ceil();
        let pede = li + tw + m::FIELD_PAD_R * 2.0 + m::ICON_GLYPH;
        crate::widget::card_censo::controlo_curto(r, pede, texto);
    }
    let fundo = if aceso {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    crate::paint::fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(fundo));
    let g = m::ICON_GLYPH;
    if let Some(icone) = icone {
        let s = m::HEAD_ICON_GLYPH;
        let caixa = Rect::new(r.x + m::FIELD_PAD_R, r.y + (r.h - s) * 0.5, s, s);
        let cor = ink.color(ColorToken::Text1);
        crate::paint::paint_icon(scene, icone, caixa, cor, StrokeToken::Default.px());
    }
    crate::paint::paint_text_elided(
        text_system,
        scene,
        texto,
        r.x + m::FIELD_PAD_R + li,
        r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        (r.w - m::FIELD_PAD_R * 2.0 - g - li).max(0.0),
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

/// O [`chip`], ou o que não abre quando `parado`.
#[allow(clippy::too_many_arguments)]
pub fn chip_ou_parado(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: CardInk,
    r: Rect,
    texto: &str,
    aceso: bool,
    parado: bool,
) {
    if parado {
        chip_parado(scene, text_system, ink, r, texto);
    } else {
        chip(scene, text_system, ink, r, texto, aceso);
    }
}

/// O [`chip`] que não abre: a caixa do cartão com o texto apagado e sem a seta.
fn chip_parado(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: CardInk,
    r: Rect,
    texto: &str,
) {
    crate::paint::fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(ColorToken::CardField));
    crate::paint::paint_text_elided(
        text_system,
        scene,
        texto,
        r.x + m::FIELD_PAD_R,
        r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        (r.w - m::FIELD_PAD_R * 2.0).max(0.0),
        ink.color(ColorToken::Text3),
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
