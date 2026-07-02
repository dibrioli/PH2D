//! Per-kind render adapters that emit `ph2d_vector::VectorScene`
//! strokes consuming `ph2d_grid` math against the editor's
//! [`crate::grid::GridView`] (camera + canvas-clip params).
//!
//! Single entry point [`paint`] dispatches by [`super::state::GridKind`].
//! The Coordenador's planned refactor of `crates/ph2d-editor/src/grid.rs`
//! will call this from the host render loop in place of its current
//! hardcoded-square overlay.

pub mod chunks;
pub mod hex;
pub mod iso;
pub mod quadtree;
pub mod square;
pub mod staggered;
pub mod tri;
pub mod util;
pub mod voronoi;

use crate::grid::GridView;
use crate::grid_snap::state::{GridKind, GridSnapState};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color, VectorScene};

/// Compose the active-kind grid overlay into `scene`, clipped to
/// `view.canvas`. No-op when overlay display is disabled. The line
/// colour is derived from the canvas background (a same-hue lightness
/// shift — lighter on a dark backdrop, darker on a light one), so the
/// grid always reads as a subtle relative contrast (Enio 2026-07-02);
/// only the stored alpha × `opacity` is kept from the user's colour.
pub fn paint(scene: &mut VectorScene, view: &GridView, state: &GridSnapState, theme: Theme) {
    if !state.show_overlay {
        return;
    }
    let color = grid_line_color(theme, state.color_rgba[3], state.opacity);

    scene.push_clip(&crate::paint::rect_to_vello(view.canvas));
    match state.kind {
        GridKind::Square => square::paint(scene, view, color, &state.square_cfg),
        GridKind::Hex => hex::paint(scene, view, color, &state.hex_cfg),
        GridKind::Iso => iso::paint(scene, view, color, &state.iso_cfg),
        GridKind::StaggeredSquare => {
            staggered::paint_square(scene, view, color, &state.staggered_square_cfg)
        }
        GridKind::StaggeredHex => {
            staggered::paint_hex(scene, view, color, &state.staggered_hex_cfg)
        }
        GridKind::Tri => tri::paint(scene, view, color, &state.tri_cfg),
        GridKind::Quadtree => quadtree::paint(scene, view, color, &state.quadtree_cfg),
        GridKind::Voronoi => voronoi::paint(scene, view, color, &state.voronoi_cfg),
        GridKind::Chunks => chunks::paint(scene, view, color, &state.chunks_cfg),
    }
    scene.pop_layer();
}

/// The grid line colour, always RELATIVE to the canvas background ([`ColorToken::Bg0`]): a same-hue
/// lightness shift — toward white on a dark backdrop, toward black on a light one — so the grid stays a
/// subtle, always-legible contrast whatever the theme. `base_alpha` × `opacity` sets the transparency.
fn grid_line_color(theme: Theme, base_alpha: u8, opacity: f32) -> Color {
    let bg = ColorToken::Bg0.resolve(theme);
    let [r, g, b] = grid_rgb([bg.r, bg.g, bg.b]);
    let alpha = (f32::from(base_alpha) * opacity.clamp(0.0, 1.0)) as u8;
    Color::from_rgba8(r, g, b, alpha)
}

/// Shift a background RGB toward white (dark bg) or black (light bg) by [`GRID_SHIFT`], keeping the hue —
/// the "gray on gray, lighter/darker" rule. Pure so it's unit-testable without a theme.
fn grid_rgb(bg: [u8; 3]) -> [u8; 3] {
    let (r, g, b) = (f32::from(bg[0]), f32::from(bg[1]), f32::from(bg[2]));
    let luma = 0.299 * r + 0.587 * g + 0.114 * b;
    let target = if luma < 128.0 { 255.0 } else { 0.0 }; // dark bg → lighter grid; light bg → darker
    let mix = |c: f32| (c + (target - c) * GRID_SHIFT).round().clamp(0.0, 255.0) as u8;
    [mix(r), mix(g), mix(b)]
}

/// How far (0..1) the grid line shifts each channel toward white/black from the background lightness.
const GRID_SHIFT: f32 = 0.42;

#[cfg(test)]
mod color_tests {
    use super::grid_rgb;

    #[test]
    fn grid_rgb_lightens_dark_bg_darkens_light_bg_and_stays_neutral() {
        let dark = grid_rgb([20, 22, 26]);
        assert!(
            dark[0] > 20 && dark[1] > 22 && dark[2] > 26,
            "dark bg → lighter grid: {dark:?}"
        );
        let light = grid_rgb([230, 232, 235]);
        assert!(
            light[0] < 230 && light[1] < 232 && light[2] < 235,
            "light bg → darker grid: {light:?}"
        );
        // A neutral gray backdrop stays neutral gray (r == g == b) — "cinza com cinza".
        let neutral = grid_rgb([30, 30, 30]);
        assert!(
            neutral[0] == neutral[1] && neutral[1] == neutral[2],
            "stays neutral gray: {neutral:?}"
        );
    }
}
