//! Hex `#RRGGBBAA` text field + eyedropper button painters.

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub fn paint_hex_field(
    hex: &str,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    let pad = Spacing::Md.px();
    let label_w = 36.0;
    let label_rect = Rect::new(rect.x + pad, rect.y, label_w, rect.h);
    paint_text(
        text_system,
        scene,
        "Hex",
        label_rect.x,
        label_rect.y + (label_rect.h - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        label_w,
        resolve(ColorToken::Text2, theme),
    );
    paint_text(
        text_system,
        scene,
        hex,
        rect.x + pad + label_w,
        rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        rect.w - pad * 2.0 - label_w,
        resolve(ColorToken::Text1, theme),
    );
}

pub fn paint_eyedropper(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    paint_icon(
        scene,
        IconId::EyePencil,
        rect,
        resolve(ColorToken::Text2, theme),
        1.5,
    );
}
