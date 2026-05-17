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
        ids::INSP_SECTION_CARD_COLOR,
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
    y + card_h
}

// ── Store accessor helpers ─────────────────────────────────────────────────
