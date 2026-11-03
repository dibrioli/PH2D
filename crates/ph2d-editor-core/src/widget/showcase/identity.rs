//! Showcase `paint_identity_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_identity_section(
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
        ids::INSP_SECTION_IDENTITY,
        "Identity",
        2,
    );
    if !open {
        return y;
    }
    paint_identity_body(scene, text_system, theme, hit_index, store, x, w, y)
}

/// O CORPO da secção «Identity», sem o cabeçalho, a partir de `y`; devolve o fundo. O cartão
/// da galeria chama-o (`SHOWCASE_CARDS`, doc 08 §1.2); a janela flutuante, pelo `*_section`.
#[allow(clippy::too_many_arguments)]
pub fn paint_identity_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    _hit_index: &mut HitIndex,
    _store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let size = ICON_BTN_SIZE_PX;
    let gap = Spacing::Md.px();
    let circle = Rect::new(x, y, size, size);
    paint_avatar(
        &Avatar::new(NodeId(0), "Enio", 'E'),
        circle,
        scene,
        text_system,
        theme,
    );
    let square = Rect::new(x + size + gap, y, size, size);
    paint_avatar(
        &Avatar::new(NodeId(0), "Player", 'P').shape(AvatarShape::Square),
        square,
        scene,
        text_system,
        theme,
    );
    // A legenda à direita; na coluna estreita ela desce para debaixo dos avatares e quebra
    // (`super::encaixe`, D13 «rótulo em cima» lido ao contrário: a legenda segue o que nomeia).
    let caption = "Avatar · circle + square";
    let font = TypeToken::Base.px();
    let cx = x + size * 2.0 + gap * 2.0;
    let side_w = (w - size * 2.0 - gap * 2.0).max(0.0);
    if super::encaixe::cabe(text_system, caption, font, side_w) {
        paint_left_label(scene, text_system, theme, cx, caption, side_w, y, size);
        return y + size;
    }
    let below = y + size + row_gap();
    let h = super::encaixe::frase(
        scene,
        text_system,
        caption,
        x,
        below,
        font,
        w,
        resolve(ColorToken::Text1, theme),
    );
    below + h
}
