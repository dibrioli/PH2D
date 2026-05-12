//! Canvas background painter — full viewport `Bg0` fill + canvas
//! region `Bg1` tint. Mockup HTML uses a radial gradient + perspective
//! grid; we deliberately ship a solid fill until the screenshot
//! harness lands (M14+) to keep theme-correctness obvious.

use super::HeroLayout;
use crate::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub fn paint_canvas_bg(layout: &HeroLayout, scene: &mut VectorScene, theme: Theme) {
    scene.fill_rect(
        rect_to_vello(layout.viewport),
        resolve(ColorToken::Bg0, theme),
    );
    scene.fill_rect(
        rect_to_vello(layout.canvas),
        resolve(ColorToken::Bg1, theme),
    );
}

/// M14.4e drag-and-drop overlay. Painted on top of the canvas when
/// the host reports files being hovered over the window. Visually:
/// translucent accent-tinted scrim covering the canvas region + a
/// centered floating card showing the file count and the first
/// filename (so the user knows what they're about to import).
pub fn paint_drop_overlay(
    layout: &HeroLayout,
    paths: &[std::path::PathBuf],
    _cursor_px: (f32, f32),
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if paths.is_empty() {
        return;
    }
    // Scrim: accent-tinted soft fill covering the canvas. Uses
    // `AccentSoft` so the user gets a brand-coherent hint rather than
    // a generic gray. Lower α than the design tokens supply so the
    // canvas content stays partially visible underneath.
    let scrim = layout.canvas;
    scene.fill_rect(rect_to_vello(scrim), resolve(ColorToken::AccentSoft, theme));

    // Centered card with file count + first filename. The card sits
    // in the canvas center; size auto-grows with the filename but is
    // capped so the layout stays readable on small windows.
    let count = paths.len();
    let first_name = paths
        .first()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("?");
    let caption = if count == 1 {
        format!("Drop to import: {first_name}")
    } else {
        format!("Drop to import {count} files (first: {first_name})")
    };

    let card_w = 420.0_f32.min(scrim.w - 32.0).max(220.0);
    let card_h = 64.0_f32;
    let card = Rect::new(
        scrim.x + (scrim.w - card_w) * 0.5,
        scrim.y + (scrim.h - card_h) * 0.5,
        card_w,
        card_h,
    );
    fill_rounded_rect(
        scene,
        card,
        Radius::Md.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        scene,
        card,
        Radius::Md.px(),
        1.0,
        resolve(ColorToken::Accent, theme),
    );

    let label_y = card.y + (card.h - TypeToken::Base.px()) * 0.5;
    paint_text(
        text_system,
        scene,
        &caption,
        card.x + Spacing::Lg.px(),
        label_y,
        TypeToken::Base.px(),
        card.w - Spacing::Lg.px() * 2.0,
        resolve(ColorToken::Text1, theme),
    );
}
