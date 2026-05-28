//! Vector Pen tool ⟷ shell bridge — per-frame preview + commit save.
//!
//! Mirror of `bgremoval_preview.rs` (HR-1 + DIRETRIZ §3.A.4): a free
//! function called once per frame BEFORE `paint_hero_screen`, when
//! the Vector Pen tool is the active tool.
//!
//! ## What this does (W1.T1.7)
//!
//! 1. Drain `VectorPenTool::take_committed_asset()` from the previous
//!    tick's close-path → save to disk via `save_vector_asset`. Toast
//!    success/error so the smoke Day-7 user sees confirmation.
//! 2. Build the network-local → screen [`Affine`] for the active
//!    sprite footprint (reuses the same chain `bgremoval_preview` uses
//!    for its overlay — see [`image_to_screen_transform_affine`]).
//! 3. Call `ph2d_vector::draw_vector_network(scene, &network, &styles,
//!    affine)` so the live triangle (or in-progress path) paints over
//!    the canvas while the user clicks.
//!
//! Pointer dispatch (screen-px click → `pen.on_canvas_click(net-local)`)
//! lives in [`crate::input_dispatch::vector_pen_input`], mirroring the
//! split between `painter_bridge.rs` (render) + `painter_input.rs`
//! (input).
//!
//! ## Anti-padrões evitados
//!
//! - Sem touch em editor-core core. Pen tool surface é toda em
//!   `ph2d-tool-vector-pen` (ADR-0040 §3 documented exception via
//!   `as_any_mut` downcast).
//! - Sem variant novo em `EditorAction`. Click → tool inherent method.
//! - Sem helper duplicado da affine math além do mínimo necessário;
//!   `image_to_screen_transform_affine` é fork local de
//!   `bgremoval_preview::image_to_screen_transform_affine` (private
//!   there) com nota TODO para extrair shared helper W2+ quando o
//!   terceiro overlay consumidor aparecer.

use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_editor::toast::{Toast, ToastQueue};
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use ph2d_tool_vector_pen::VectorPenTool;
use ph2d_vector::{Affine, VectorScene};
use std::time::{SystemTime, UNIX_EPOCH};

/// Per-frame Vector Pen bridge dispatch.
///
/// Early-returns if the active tool isn't `vector_pen` OR if there's
/// no sprite selection. No mutation on early-return — safe to call
/// every frame unconditionally.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    hero: &HeroScreen,
    tools: &mut ToolRegistry,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    toasts: &mut ToastQueue,
) {
    let is_pen_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pen"))
        .unwrap_or(false);
    if !is_pen_active {
        return;
    }

    // Selection drives the canvas footprint. No selection = no canvas
    // = no overlay; pen tool still consumes clicks but the user sees
    // no preview until a sprite is picked.
    let Some(bits) = hero.gizmo.selection else {
        return;
    };
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let Some(tr) = sim.world().get::<Transform>(entity) else {
        return;
    };
    let Some(sprite) = sim.world().get::<Sprite>(entity) else {
        return;
    };

    // Downcast — ADR-0040 §3 documented exception (mirror of
    // `painter_bridge` + `bgremoval_preview`).
    let Some(active) = tools.active_mut() else {
        return;
    };
    let Some(pen) = active.as_any_mut().downcast_mut::<VectorPenTool>() else {
        return;
    };

    // Step 1 — drain pending commit + save to disk. We do this FIRST
    // (before the preview render) so the asset save happens on the
    // tick that the user closed the path; the preview then re-renders
    // the *next* in-progress path (empty post-reset) on the same
    // tick, so no stale triangle stays on screen.
    if let Some(asset) = pen.take_committed_asset() {
        match save_asset_to_disk(&asset) {
            Ok(path) => toasts.push(Toast::info(format!("Vector saved: {path}"))),
            Err(e) => toasts.push(Toast::error(format!("Vector save failed: {e}"))),
        }
    }

    // Step 2 — render in-progress preview. Skip when network is empty
    // (avoids issuing a zero-region Vello draw call).
    let network = pen.current_network();
    if network.vertices.is_empty() {
        return;
    }
    let styles = pen.current_styles();

    // Network-local coords are sprite-pixel space (the Pen tool calls
    // `on_canvas_click(net_local_pos)` with coordinates derived from
    // the same sprite source-pixel grid). Build the same affine that
    // `bgremoval_preview` uses for its image overlay so the triangle
    // appears anchored to the sprite footprint.
    let image_w = sprite.size[0].max(1.0) as u32;
    let image_h = sprite.size[1].max(1.0) as u32;
    let net_to_screen =
        image_to_screen_transform_affine(image_w, image_h, tr, sprite, camera, window_size);
    ph2d_vector::draw_vector_network(vector_scene.inner_mut(), network, styles, net_to_screen);
}

/// Save the committed Pen asset to disk under a timestamped filename.
///
/// **W1.T1.7 MVP convention**: dumps into
/// `vector_pen_<unix_secs>.ph2d-vector` in the process's current
/// directory. W2 will plumb the asset path through `AssetDb` + a real
/// "save as" dialog; for the smoke Day-7 the user verifies file
/// presence + can re-load via `load_and_validate_vector_asset`.
fn save_asset_to_disk(asset: &ph2d_vector_doc::Ph2dVectorAsset) -> Result<String, String> {
    let bytes = ph2d_vector_doc::save_vector_asset(asset).map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = format!("vector_pen_{ts}.ph2d-vector");
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Image-px → screen-px affine for the Pen overlay.
///
/// **Local duplicate** of `bgremoval_preview::image_to_screen_transform_affine`
/// (private there). When a third overlay consumer arrives (W2 Pencil
/// preview likely), extract to a shared `render_loop::overlay_affine`
/// helper. Until then duplicate-and-document avoids the cross-cutting
/// edit during the parallel Painter session.
///
/// Coordinate chain:
/// 1. Image-pixel `(px, py)` → sprite-local (centered, Y-flipped to
///    world-Y-up): `((px/W − 0.5) · sw, (0.5 − py/H) · sh)`.
/// 2. Sprite-local → world via Transform composite
///    (translate ⊕ rotate ⊕ anchor ⊕ scale).
/// 3. World → screen via camera (centered, Y-flipped) + uniform
///    `k = window.height / camera.height_world`.
fn image_to_screen_transform_affine(
    image_w: u32,
    image_h: u32,
    tr: &Transform,
    sprite: &Sprite,
    camera: &Camera2d,
    window_size: WindowSize,
) -> Affine {
    let image_w = image_w as f64;
    let image_h = image_h as f64;
    let size_w = sprite.size[0] as f64;
    let size_h = sprite.size[1] as f64;
    let img_to_local = Affine::scale_non_uniform(size_w / image_w, -size_h / image_h)
        * Affine::translate((-image_w * 0.5, -image_h * 0.5));
    let local_to_world = Affine::translate((tr.translation.x as f64, tr.translation.y as f64))
        * Affine::rotate(tr.rotation as f64)
        * Affine::translate((sprite.anchor[0] as f64, sprite.anchor[1] as f64))
        * Affine::scale_non_uniform(tr.scale.x as f64, tr.scale.y as f64);
    let k = (window_size.height as f64) / (camera.height_world as f64).max(1e-6);
    let world_to_screen = Affine::translate((
        window_size.width as f64 * 0.5,
        window_size.height as f64 * 0.5,
    )) * Affine::scale_non_uniform(k, -k)
        * Affine::translate((-camera.center[0] as f64, -camera.center[1] as f64));
    world_to_screen * local_to_world * img_to_local
}
