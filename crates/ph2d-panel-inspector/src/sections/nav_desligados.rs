//! ⭐ (dono 09/11) **O aviso dos corpos que não barram o caminho** — a frase curta da NAV REGION e,
//! por baixo, um botão por corpo de formas todas desligadas, que o seleciona (o idioma do «Objects on
//! this layer» da aba Collision Layers). Os botões fluem e quebram na largura do cartão; passados os
//! [`crate::ids::MAX_NAV_OFF_BODIES`], um «+N more» sem clique.
//!
//! ⚠️ O clique só chega com o registo no `populate_nav` e o braço no `event_nav`.

use super::*;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::nav_edits::CorpoDesligado;
use ph2d_editor_core::widget::button_label_font;
use ph2d_editor_core::widget::card_ink::card_body;
use ph2d_i18n::{tr, tr_with};

/// Pinta o aviso e os botões; sem corpos, nada. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    corpos: &[CorpoDesligado],
) -> f32 {
    if corpos.is_empty() {
        return y;
    }
    let y = super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        tr("panel.inspector.nav.bodies_with_every_shape_off"),
        ColorToken::Warn,
    );
    let em_cartao = card_body().is_some();
    let (h, vao, topo) = if em_cartao {
        (m::FIELD_H, m::FIELD_GAP, y + m::FIELD_ROW_GAP)
    } else {
        (ALTURA_DE_CAMPO, Spacing::Xs.px(), y)
    };
    let ids = &crate::ids::INSP_NAV_OFF_BODY;
    let resto = corpos.len().saturating_sub(ids.len());
    let mais =
        (resto > 0).then(|| tr_with("panel.inspector.nav.more_off_bodies", &[("n", &resto)]));
    let font = button_label_font();
    let pecas = corpos
        .iter()
        .zip(ids)
        .map(|(c, &id)| (Some(id), c.name.as_str()))
        .chain(mais.as_deref().map(|t| (None, t)));
    let (mut cx, mut cy) = (x, topo);
    for (id, rotulo) in pecas {
        let bw = (text_system.prefix_width(rotulo, font) + m::BUTTON_TEXT_PAD * 2.0).min(w);
        if cx > x && cx + bw > x + w {
            cx = x;
            cy += h + vao;
        }
        let rect = Rect::new(cx, cy, bw, h);
        match id {
            Some(id) => {
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
            }
            None => paint_text(
                text_system,
                scene,
                rotulo,
                rect.x,
                rect.y + super::rows::folga_da_linha(h, font),
                font,
                rect.w,
                resolve(ColorToken::Text3, theme),
            ),
        }
        cx += bw + vao;
    }
    if em_cartao {
        cy + h
    } else {
        cy + h + ph2d_tokens::control_gap_px()
    }
}
