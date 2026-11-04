//! As peças pintadas da lista de estilos: fundo, título, amostra e ícone.

use super::*;

pub(super) fn fundo(scene: &mut VectorScene, r: Rect, ink: &CardInk) {
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanListShadow),
        m::LIST_SHADOW_FAR,
    );
    fill_rounded_rect(
        scene,
        r,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanCardRaised),
    );
}

pub(super) fn titulo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: &CardInk,
    r: Rect,
    t: &str,
) {
    crate::paint::paint_text_title_elided(
        text_system,
        scene,
        t,
        r.x + m::CARD_PAD_X,
        r.y + crate::paint::folga_do_texto(m::CARD_HEAD_H, m::CARD_TITLE_PX),
        m::CARD_TITLE_PX,
        r.w - m::CARD_PAD_X * 2.0 - m::HEAD_ICON_BOX,
        ink.color(ColorToken::Text1),
    );
}

pub(super) fn amostra(scene: &mut VectorScene, r: Rect, c: [u8; 4], ink: &CardInk) {
    fill_rounded_rect(
        scene,
        r,
        m::SEG_RADIUS,
        ph2d_vector::Color::from_rgba8(c[0], c[1], c[2], c[3]),
    );
    // FRAME-RAW-OK: o contorno da AMOSTRA de cor é a borda que a separa do cartão — a cor dela é a mensagem
    crate::paint::stroke_rounded_rect(
        scene,
        r,
        m::SEG_RADIUS,
        m::FIELD_BORDER,
        ink.color(ColorToken::Border),
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn icone(
    scene: &mut VectorScene,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    ink: &CardInk,
    b: Rect,
    id: NodeId,
    icon: IconId,
) {
    hit_index.register(id, b);
    if store.hover_live(id) > 0.0 {
        fill_rounded_rect(
            scene,
            b,
            m::HEAD_ICON_RADIUS,
            ink.color(ColorToken::CardFieldHover),
        );
    }
    let g = m::HEAD_ICON_GLYPH;
    paint_icon(
        scene,
        icon,
        Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
        ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
}
