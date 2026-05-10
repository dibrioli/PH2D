//! Vertical value (lightness) slider painter.

use super::state::BlenderColorPicker;
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Theme};
use ph2d_vector::VectorScene;

pub fn paint_value_slider(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    let l = cp.value.oklch.0 as f32;
    let fill_h = rect.h * l;
    let fill_rect = Rect::new(rect.x, rect.y + rect.h - fill_h, rect.w, fill_h);
    fill_rounded_rect(scene, fill_rect, radius, resolve(ColorToken::Text1, theme));
}
