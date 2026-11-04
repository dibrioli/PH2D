//! Showcase `paint_card_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_card_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_CARD,
        "Card",
        1,
    );
    if !open {
        return y;
    }
    paint_card_body(scene, text_system, theme, hit_index, store, x, w, y)
}

/// O CORPO da secção «Card», sem o cabeçalho, a partir de `y`; devolve o fundo. O cartão
/// da galeria chama-o (`SHOWCASE_CARDS`, doc 08 §1.2); a janela flutuante, pelo `*_section`.
#[allow(clippy::too_many_arguments)]
pub fn paint_card_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let card_h = super::encaixe::no_cartao(
        m::CARD_HEAD_H + m::FIELD_H + m::CARD_PAD_BOTTOM,
        80.0_f32, // LITERAL-PX-OK: demo Card height (showcase-specific)
    );
    let r = Rect::new(x, y, w, card_h);
    let card = Card::new(NodeId(0)).title("Quick actions");
    paint_card(&card, r, scene, text_system, theme);
    // Body caption — Card itself doesn't paint into body_rect, so we
    // write a single label line inside it as a static demo.
    let body = card.body_rect(r);
    super::encaixe::frase(
        scene,
        text_system,
        "Card body slot — host content here.",
        body.x,
        body.y,
        super::encaixe::no_cartao(m::LABEL_PX, TypeToken::Xs.px()),
        body.w,
        crate::widget::card_ink::tinta(ColorToken::Text2, theme),
    );
    // Illustration — o desenho em DADOS de uma dica (o menu Add): um corpo, o chão, a seta do salto e
    // o contorno tracejado da colisão.
    let pic_h = 64.0_f32; // LITERAL-PX-OK: demo illustration height (showcase-specific)
    let pic = Rect::new(x, y + card_h, w, pic_h);
    crate::widget::illustration::paint_illustration(scene, pic, DEMO_ILLUSTRATION, theme);
    super::card_back::paint_card_back_widgets(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y + card_h + pic_h,
    )
}

/// O desenho de exemplo da galeria (coordenadas da moldura, `0..1`).
const DEMO_ILLUSTRATION: &[crate::widget::illustration::Mark] = {
    use crate::widget::illustration::Mark;
    &[
        Mark::Rect {
            at: [0.1, 0.8],   // LITERAL-PX-OK: coordenada da moldura 0..1
            size: [0.8, 0.1], // LITERAL-PX-OK: coordenada da moldura 0..1
            fill: ColorToken::Text3,
            round: 0.02, // LITERAL-PX-OK: coordenada da moldura 0..1
        },
        Mark::Capsule {
            c: [0.35, 0.55], // LITERAL-PX-OK: coordenada da moldura 0..1
            half_h: 0.1,     // LITERAL-PX-OK: coordenada da moldura 0..1
            r: 0.12,         // LITERAL-PX-OK: coordenada da moldura 0..1
            fill: ColorToken::Accent,
        },
        Mark::Dashed {
            at: [0.27, 0.3],   // LITERAL-PX-OK: coordenada da moldura 0..1
            size: [0.16, 0.5], // LITERAL-PX-OK: coordenada da moldura 0..1
            stroke: ColorToken::Text2,
        },
        Mark::Line {
            a: [0.5, 0.6],  // LITERAL-PX-OK: coordenada da moldura 0..1
            b: [0.7, 0.25], // LITERAL-PX-OK: coordenada da moldura 0..1
            stroke: ColorToken::Text2,
            arrow: true,
        },
        Mark::Disc {
            c: [0.78, 0.2], // LITERAL-PX-OK: coordenada da moldura 0..1
            r: 0.06,        // LITERAL-PX-OK: coordenada da moldura 0..1
            fill: ColorToken::Warn,
        },
    ]
};

// ── Store accessor helpers ─────────────────────────────────────────────────
