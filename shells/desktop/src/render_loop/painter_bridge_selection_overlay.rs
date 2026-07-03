//! The **Selection** overlay — Procreate-style **marching ants** on the boundary + semi-transparent
//! diagonal **hatching** over the deselected area. The tool builds a canvas-sized straight-RGBA overlay
//! ([`PainterTool::selection_overlay_rgba`]); this blits it image→screen through the sprite affine, exactly
//! like `painter_bridge_overlays::draw_repeat_image`. Split from `painter_bridge_overlays` for the HR-18
//! file-LOC cap. Pure draw — mutates no tool/model state.
//!
//! The ants/hatch animate off a shell-local frame counter. This is view-only chrome (never simulation
//! state), so it is exempt from HR-5 cross-platform determinism — a plain monotonic counter is correct.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;
use std::sync::atomic::{AtomicU32, Ordering};

/// Monotonic per-frame animation phase (advanced once per drawn frame while a selection is visible).
static SELECTION_ANIM_PHASE: AtomicU32 = AtomicU32::new(0);

/// Draw the active selection's overlay (marching ants + deselected-area hatching) into `vector_scene`.
/// No-op without a selected sprite or a live selection.
pub(super) fn draw_selection_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
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
        sim.world().get::<crate::Transform>(entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    let phase = SELECTION_ANIM_PHASE.fetch_add(1, Ordering::Relaxed);
    let Some((rgba, w, h)) = painter.selection_overlay_rgba(phase) else {
        return;
    };
    let rgba = std::sync::Arc::new(rgba);
    // image-px → screen via the FULL sprite affine, so the overlay rides scale / aspect / rotation.
    let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        camera,
        window_size,
    );
    vector_scene.draw_image_rgba_transformed(&rgba, w, h, affine, ph2d_vector::ImageQuality::Low);
}
