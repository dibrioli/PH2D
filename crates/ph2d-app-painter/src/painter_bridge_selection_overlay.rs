//! The **Selection** overlay — Procreate-style **marching ants** on the boundary + semi-transparent
//! diagonal **hatching** over the deselected area. The tool builds a canvas-sized straight-RGBA overlay
//! ([`PainterTool::selection_overlay_rgba`]); this blits it image→screen through the sprite affine, exactly
//! like `painter_bridge_overlays::draw_repeat_image`. Split from `painter_bridge_overlays` for the HR-18
//! file-LOC cap. Pure draw — mutates no tool/model state.
//!
//! B20.11: the ants step on the WALL CLOCK ([`passo_das_formigas`]) and the tool keeps the built image per
//! step, so an idle frame re-draws the same `StableImage` (no rebuild, no upload). The image is only the
//! hatch; the ants and the Offset lines are 1-screen-px strokes of the cached contour. View-only chrome.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;
use std::time::{Duration, Instant};

/// O passo das formigas em `agora` e o instante em que o seguinte começa.
pub(crate) fn passo_das_formigas(agora: Instant) -> (u32, Instant) {
    static INICIO: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let inicio = *INICIO.get_or_init(|| agora);
    let ms = ph2d_tool_painter::PASSO_DAS_FORMIGAS_MS;
    let passo = agora.saturating_duration_since(inicio).as_millis() as u64 / ms;
    (
        passo as u32,
        inicio + Duration::from_millis((passo + 1) * ms),
    )
}

/// Draw the active selection's overlay (marching ants + deselected-area hatching) into `vector_scene`.
/// No-op without a selected sprite or a live selection.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_selection_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    let passo = passo_das_formigas(Instant::now()).0;
    draw_selection_overlay_no_passo(painter, hero, sim, camera, window_size, vector_scene, passo);
}

/// [`draw_selection_overlay`] num passo dado das formigas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_selection_overlay_no_passo(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    passo: u32,
) {
    let Some(bits) = hero.gizmo.selection else {
        return;
    };
    if !painter.selection_active() {
        return;
    }
    let (iw, ih) = painter.canvas_size();
    if iw == 0 || ih == 0 {
        return;
    }
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let (Some(tr), Some(sprite)) = (
        ph2d_ecs::world_transform(sim.world(), entity),
        ph2d_sprite_screen::sprite_mostrada(sim.world(), entity),
    ) else {
        return;
    };
    // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula, e aí o quad do
    // afim é o de sempre, byte-idêntico.
    let sprite_grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    let Some(imagem) = painter.selection_overlay_imagem(passo) else {
        return;
    };
    // image-px → screen via the FULL sprite affine, so the overlay rides scale / aspect / rotation.
    let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        sprite_grid,
        camera,
        window_size,
    );
    vector_scene.draw_stable_image_transformed(&imagem, affine, ph2d_vector::ImageQuality::Low);
    if let Some(c) = painter.selection_contornos() {
        ph2d_tool_painter::desenha_formigas(vector_scene, &c, affine, passo);
    }
}

#[cfg(test)]
#[path = "painter_bridge_selection_overlay_tests.rs"]
mod tests;

/// The Selection tool's crosshair (over the canvas, not a panel), in place of the brush ring. Drawn with
/// the other cursor-following chrome, so it hides once the pointer leaves the window (AU-AU13-7).
pub(super) fn draw_selection_cursor(
    painter: &PainterTool,
    hero: &HeroScreen,
    vector_scene: &mut VectorScene,
    (cx, cy): (f32, f32),
) {
    if hero.gizmo.selection.is_some()
        && painter.is_selection_mode()
        && hero.chrome_panel_at(cx, cy).is_none()
    {
        draw_crosshair(vector_scene, cx, cy);
    }
}

/// A cor da mira da Seleção.
pub(crate) const COR_DA_MIRA: [f32; 4] = [0.12, 0.12, 0.12, 0.9]; // LITERAL-COLOR-OK: overlay cursor crosshair

/// A small screen-space crosshair (target `+` with a centre gap) at the cursor — the Selection tool's
/// cursor, in place of the brush ring.
fn draw_crosshair(vector_scene: &mut VectorScene, cx: f32, cy: f32) {
    use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke};
    const ARM: f64 = 9.0; // arm length (px)
    const GAP: f64 = 3.0; // centre gap (px)
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    let color = Color::new(COR_DA_MIRA);
    let mut path = BezPath::new();
    path.move_to(Point::new(cx - ARM, cy));
    path.line_to(Point::new(cx - GAP, cy));
    path.move_to(Point::new(cx + GAP, cy));
    path.line_to(Point::new(cx + ARM, cy));
    path.move_to(Point::new(cx, cy - ARM));
    path.line_to(Point::new(cx, cy - GAP));
    path.move_to(Point::new(cx, cy + GAP));
    path.line_to(Point::new(cx, cy + ARM));
    vector_scene.inner_mut().stroke(
        &Stroke::new(1.25),
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &path,
    );
}
