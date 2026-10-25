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
    let card_h = 80.0_f32; // LITERAL-PX-OK: demo Card height (showcase-specific)
    let r = Rect::new(x, y, w, card_h);
    let card = Card::new(NodeId(0)).title("Quick actions");
    paint_card(&card, r, scene, text_system, theme);
    // Body caption — Card itself doesn't paint into body_rect, so we
    // write a single label line inside it as a static demo.
    let body = card.body_rect(r);
    paint_text(
        text_system,
        scene,
        "Card body slot — host content here.",
        body.x,
        body.y,
        TypeToken::Xs.px(),
        body.w,
        resolve(ColorToken::Text2, theme),
    );
    // Illustration — o desenho em DADOS de uma dica (o menu Add): um corpo, o chão, a seta do salto e
    // o contorno tracejado da colisão.
    let pic_h = 64.0_f32; // LITERAL-PX-OK: demo illustration height (showcase-specific)
    let pic = Rect::new(x, y + card_h, w, pic_h);
    crate::widget::illustration::paint_illustration(scene, pic, DEMO_ILLUSTRATION, theme);
    y + card_h + pic_h
}

/// O desenho de exemplo da galeria (coordenadas da moldura, `0..1`).
const DEMO_ILLUSTRATION: &[crate::widget::illustration::Mark] = {
    use crate::widget::illustration::Mark;
    &[
        Mark::Rect {
            at: [0.1, 0.8],
            size: [0.8, 0.1],
            fill: ColorToken::Text3,
            round: 0.02,
        },
        Mark::Capsule {
            c: [0.35, 0.55],
            half_h: 0.1,
            r: 0.12,
            fill: ColorToken::Accent,
        },
        Mark::Dashed {
            at: [0.27, 0.3],
            size: [0.16, 0.5],
            stroke: ColorToken::Text2,
        },
        Mark::Line {
            a: [0.5, 0.6],
            b: [0.7, 0.25],
            stroke: ColorToken::Text2,
            arrow: true,
        },
        Mark::Disc {
            c: [0.78, 0.2],
            r: 0.06,
            fill: ColorToken::Warn,
        },
    ]
};

// ── Store accessor helpers ─────────────────────────────────────────────────
