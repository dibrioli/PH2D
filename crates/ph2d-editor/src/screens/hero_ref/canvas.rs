//! Canvas background painter — full viewport `Bg0` fill + canvas
//! region `Bg1` tint. Mockup HTML uses a radial gradient + perspective
//! grid; we deliberately ship a solid fill until the screenshot
//! harness lands (M14+) to keep theme-correctness obvious.

use super::HeroLayout;
use crate::paint::{rect_to_vello, resolve};
use ph2d_tokens::{ColorToken, Theme};
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
