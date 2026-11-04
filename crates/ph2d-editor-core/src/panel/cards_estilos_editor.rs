//! O «Edit style» da lista de estilos de cor (maquete G): nome, cor e quantas camadas mudam juntas.

use super::*;

/// A altura do «Edit style».
pub(super) fn altura() -> f32 {
    let rotulo = m::LABEL_TOP + m::LABEL_PX + m::LABEL_BOTTOM;
    m::CARD_HEAD_H + (rotulo + m::FIELD_H) * 2.0 + m::LABEL_TOP + m::LABEL_PX + m::CARD_PAD_BOTTOM
}

/// O «Edit style» (G), logo abaixo da lista: nome, cor e quantas camadas mudam juntas.
pub(super) fn editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    ink: &CardInk,
    lista: Rect,
    e: &Estilo,
) {
    let rotulo = m::LABEL_TOP + m::LABEL_PX + m::LABEL_BOTTOM;
    let caixa = Rect::new(lista.x, lista.y + lista.h + m::CARD_GAP, lista.w, altura());
    fundo(scene, caixa, ink);
    hit_index.register(FUNDO, caixa);
    titulo(
        scene,
        text_system,
        ink,
        caixa,
        ph2d_i18n::tr("card.styles.edit_title"),
    );
    let x = caixa.x + m::CARD_PAD_X;
    let w = caixa.w - m::CARD_PAD_X * 2.0;
    let mut y = caixa.y + m::CARD_HEAD_H;
    for (chave, id) in [("card.styles.name", NOME), ("card.styles.color", HEX)] {
        crate::paint::paint_text(
            text_system,
            scene,
            ph2d_i18n::tr(chave),
            x,
            y + m::LABEL_TOP,
            m::LABEL_PX,
            w,
            ink.color(ColorToken::CardLabel),
        );
        y += rotulo;
        let r = Rect::new(x, y, w, m::FIELD_H);
        let caixa_txt = if id == HEX {
            let g = m::ICON_GLYPH;
            fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(ColorToken::CardField));
            amostra(
                scene,
                Rect::new(r.x + (m::ICON_CELL - g) * 0.5, r.y + (r.h - g) * 0.5, g, g),
                e.rgba,
                ink,
            );
            Rect::new(r.x + m::ICON_CELL, r.y, r.w - m::ICON_CELL, r.h)
        } else {
            r
        };
        campo(scene, text_system, hit_index, store, ink, caixa_txt, id);
        y += m::FIELD_H;
    }
    let usos = ph2d_i18n::tr("card.styles.used_by").replace("{n}", &e.usos.to_string());
    crate::paint::paint_text_elided(
        text_system,
        scene,
        &usos,
        x,
        y + m::LABEL_TOP,
        m::LABEL_PX,
        w,
        ink.color(ColorToken::Text3),
    );
}

pub(super) fn campo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    ink: &CardInk,
    r: Rect,
    id: NodeId,
) {
    use crate::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
    hit_index.register(id, r);
    let Some(InteractiveState::TextInput {
        state,
        text,
        caret,
        selection_anchor,
    }) = store.get(id)
    else {
        return;
    };
    if matches!(state, TextInputState::Normal | TextInputState::Hovered) {
        if id != HEX {
            let fundo = if store.hover_live(id) > 0.0 {
                ColorToken::CardFieldHover
            } else {
                ColorToken::CardField
            };
            fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(fundo));
        }
        let (texto, cor) = if id == BUSCA && text.is_empty() {
            (ph2d_i18n::tr("card.styles.search"), ColorToken::Text3)
        } else {
            (text.as_str(), ColorToken::Text1)
        };
        crate::paint::paint_text_elided(
            text_system,
            scene,
            texto,
            r.x + m::FIELD_PAD_R,
            r.y + (r.h - m::VALUE_PX) * 0.5,
            m::VALUE_PX,
            (r.w - m::FIELD_PAD_R * 2.0).max(0.0),
            ink.color(cor),
        );
        return;
    }
    let input = TextInput::new(id, "").visual((*state, store.hover_live(id)));
    paint_text_input_with_buffer(
        &input,
        Some(text),
        Some(*caret),
        *selection_anchor,
        r,
        scene,
        text_system,
        ink.theme,
    );
}
