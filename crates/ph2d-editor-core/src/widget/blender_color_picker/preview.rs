//! **A amostra da cor resultante** — a faixa de largura inteira que mostra o `value.rgba` do picker
//! sobre um xadrez, para que uma alfa parcial se leia (as células claras e escuras aparecem
//! através de uma cor translúcida, como em todo app de pintura).
//!
//! ⚠️ O corte é por RESPONSABILIDADE, não por tamanho: o pai (`paint.rs`) orquestra o LAYOUT do
//! picker (roda · faixa · amostra · canais · hex · paletas) e este irmão sabe pintar UMA amostra.
//! Foi o teto de 500 LOC dos primitivos de widget que forçou a decisão na wave 3 do redesenho
//! (2026-09-05), e a linha de corte já estava lá — a função não lia nada do layout.

use super::state::BlenderColorPicker;
use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Theme};
use ph2d_vector::VectorScene;

/// Paint the resulting-color preview swatch — full-width strip
/// showing the current `value.rgba` over a checkerboard so partial
/// alpha is legible (light/dark squares show through translucent
/// colors, matching what every other paint app does).
pub fn paint_color_preview(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    // Light backdrop covering the whole rect (the "white" cells of
    // the checker).
    fill_rounded_rect(
        scene,
        rect,
        radius,
        ph2d_vector::Color::from_rgba8(220, 220, 220, 255),
    );
    // Darker squares — every other cell. Inset by `radius` on all
    // sides so the checker stays inside the rounded outline; with
    // a translucent overlay the corners would otherwise show tiny
    // squares poking past the curve.
    let cell = 6.0_f32;
    let chk_x = rect.x + radius;
    let chk_y = rect.y + radius;
    let chk_w = (rect.w - radius * 2.0).max(0.0);
    let chk_h = (rect.h - radius * 2.0).max(0.0);
    let cols = (chk_w / cell).ceil() as i32;
    let rows = (chk_h / cell).ceil() as i32;
    let dark = ph2d_vector::Color::from_rgba8(170, 170, 170, 255);
    for j in 0..rows {
        for i in 0..cols {
            if (i + j) % 2 == 0 {
                continue;
            }
            let cx = chk_x + (i as f32) * cell;
            let cy = chk_y + (j as f32) * cell;
            let w = cell.min(chk_x + chk_w - cx);
            let h = cell.min(chk_y + chk_h - cy);
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            let kr = ph2d_vector::Rect::new(cx as f64, cy as f64, (cx + w) as f64, (cy + h) as f64);
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(dark),
                None,
                &kr,
            );
        }
    }
    // Color overlay (with whatever alpha the picker carries).
    let [r, g, b, a] = cp.value.rgba;
    fill_rounded_rect(
        scene,
        rect,
        radius,
        ph2d_vector::Color::from_rgba8(r, g, b, a),
    );
    // ⭐ A amostra é plana num tema moderno (os presets do `ColorPicker` do Godot não têm borda).
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
}
