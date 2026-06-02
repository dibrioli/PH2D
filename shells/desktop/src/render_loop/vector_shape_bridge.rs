//! Vector Shape tool ⟷ shell bridge — commit drain + live drag preview.
//!
//! Free function called once per frame BEFORE `paint_hero_screen` (mirror
//! of `vector_pen_bridge.rs` / `vector_pencil_bridge.rs`).
//!
//! ## What this bridge does
//!
//! - **Drains** the asset committed on the previous tick's pointer-up into
//!   the shared committed-vector-scene list (`App::committed_vector_pen_paths`,
//!   also fed by the Pen + Pencil) — committed shapes render canonically
//!   via the Pen bridge's per-frame `draw_vector_network`.
//! - **Renders the live preview** of the in-progress drag through the SAME
//!   canonical `draw_vector_network` (fill-pass for closed shapes,
//!   stroke-pass for the spiral) — so the preview is pixel-identical to the
//!   committed result.
//!
//! ## ⚠ Central wiring required (Coord) — see `docs/HANDOFF_vector_w2_shape_coord.md`
//!
//! Not yet declared/called centrally. To activate, mirror the Pencil
//! wiring (`69b3788`): `shells/desktop/Cargo.toml` dep,
//! `render_loop/mod.rs` `mod` + dispatch call + destructive-deactivate warn,
//! `input_dispatch.rs` drag routing, `keyboard.rs` Esc.

use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_vector_shape::VectorShapeTool;
use ph2d_vector::{Ph2dVectorAsset, VectorScene, draw_vector_network};

/// `true` iff the Shape tool is active AND mid-drag (used before a
/// destructive tool switch — mirror of
/// `vector_pencil_bridge::pencil_has_in_progress_stroke`). The downcast
/// lives in this allowlisted bridge so central dispatch stays
/// downcast-free.
#[must_use]
pub(super) fn shape_has_in_progress_shape(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<VectorShapeTool>()
                .map(|s| s.has_in_progress_shape())
        })
        .unwrap_or(false)
}

/// Per-frame Vector Shape bridge dispatch. `committed_paths` is the shared
/// committed-vector-scene list. Early-returns when the Shape tool isn't
/// active.
pub(super) fn dispatch(
    tools: &mut ToolRegistry,
    camera: &Camera2d,
    window_size: WindowSize,
    committed_paths: &mut Vec<Ph2dVectorAsset>,
    vector_scene: &mut VectorScene,
) {
    let is_shape_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("vector_shape"))
        .unwrap_or(false);
    if !is_shape_active {
        return;
    }

    // Downcast — ADR-0040 §3 documented exception.
    let Some(active) = tools.active_mut() else {
        return;
    };
    let Some(shape) = active.as_any_mut().downcast_mut::<VectorShapeTool>() else {
        return;
    };

    // Drain the previous tick's commit into the shared scene.
    if let Some(asset) = shape.take_committed_asset() {
        committed_paths.push(asset);
    }

    // Live preview of the current drag, via the canonical renderer.
    if let Some(preview) = shape.preview_network() {
        let world_to_screen = camera.world_to_screen_affine(window_size);
        let styles = shape.current_styles();
        let scene = vector_scene.inner_mut();
        draw_vector_network(scene, &preview, styles, world_to_screen);
    }
}
