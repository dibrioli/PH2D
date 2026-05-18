//! Selection overlay painter — dashed marquee + 4 corner handles +
//! floating tag above the marquee.

use super::HeroLayout;
use super::HeroSelection;
use crate::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, rect_to_vello, resolve, stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Stroke, VectorScene};

pub fn paint_selection_overlay(
    layout: &HeroLayout,
    selection: &HeroSelection,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let marquee_w = (layout.canvas.w * 0.55).clamp(280.0, 520.0); // LITERAL-PX-OK: mockup marquee geometry (canvas-ratio + min/max chrome)
    let marquee_h = (layout.canvas.h * 0.5).clamp(220.0, 440.0); // LITERAL-PX-OK: mockup marquee geometry
    let cx = layout.canvas.x + layout.canvas.w * 0.5;
    let cy = layout.canvas.y + layout.canvas.h * 0.5;
    let marquee = Rect::new(
        cx - marquee_w * 0.5,
        cy - marquee_h * 0.5,
        marquee_w,
        marquee_h,
    );
    let stroke = Stroke::new(1.0).with_dashes(0.0, [6.0, 4.0]); // LITERAL-PX-OK: marching-ants dash pattern (gap+dash design-specific)
    let r = rect_to_vello(marquee);
    scene.inner_mut().stroke(
        &stroke,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &r,
    );
    let handle = Spacing::Md.px();
    for (hx, hy) in [
        (marquee.x - handle * 0.5, marquee.y - handle * 0.5),
        (
            marquee.x + marquee.w - handle * 0.5,
            marquee.y - handle * 0.5,
        ),
        (
            marquee.x - handle * 0.5,
            marquee.y + marquee.h - handle * 0.5,
        ),
        (
            marquee.x + marquee.w - handle * 0.5,
            marquee.y + marquee.h - handle * 0.5,
        ),
    ] {
        let h_rect = Rect::new(hx, hy, handle, handle);
        fill_rounded_rect(scene, h_rect, 1.0, resolve(ColorToken::Bg0, theme));
        stroke_rounded_rect(scene, h_rect, 1.0, 2.0, resolve(ColorToken::Accent, theme));
    }
    let tag_w = 220.0_f32; // LITERAL-PX-OK: selection tag width (chrome-specific)
    let tag_h = 22.0_f32; // LITERAL-PX-OK: selection tag height (chrome-specific compact)
    let tag_rect = Rect::new(
        marquee.x,
        marquee.y - tag_h - Spacing::Sm.px(),
        tag_w,
        tag_h,
    );
    fill_rounded_rect(
        scene,
        tag_rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        scene,
        tag_rect,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let pad = Spacing::Md.px();
    let label_y = tag_rect.y + (tag_rect.h - TypeToken::Xs.px()) * 0.5;
    paint_text(
        text_system,
        scene,
        &selection.label,
        tag_rect.x + pad,
        label_y,
        TypeToken::Xs.px(),
        80.0, // LITERAL-PX-OK: selection label width budget (chrome-specific)
        resolve(ColorToken::Text1, theme),
    );
    let badge_x = tag_rect.x + pad + 60.0; // LITERAL-PX-OK: badge offset within selection tag (chrome-specific)
    let badge_w = Spacing::Xl3.px();
    let badge_rect = Rect::new(
        badge_x,
        tag_rect.y + Spacing::Xs.px(),
        badge_w,
        tag_rect.h - Spacing::Md.px(),
    );
    fill_rounded_rect(
        scene,
        badge_rect,
        Radius::Xs.px(),
        resolve(ColorToken::AccentSoft, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        &selection.kind,
        badge_rect,
        TypeToken::Xs.px() - 2.0,
        resolve(ColorToken::Accent, theme),
    );
    let pos_text = format!(
        "\u{00b7} {:.0}, {:.0}",
        selection.world_pos.0, selection.world_pos.1
    );
    paint_text(
        text_system,
        scene,
        &pos_text,
        badge_x + badge_w + Spacing::Md.px(),
        label_y,
        TypeToken::Xs.px(),
        100.0, // LITERAL-PX-OK: pos-text width budget (chrome-specific)
        resolve(ColorToken::Text3, theme),
    );
}
