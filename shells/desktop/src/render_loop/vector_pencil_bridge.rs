//! Vector Pencil tool ⟷ shell bridge — commit drain + in-progress overlay.
//!
//! Free function called once per frame BEFORE `paint_hero_screen` (mirror
//! of `vector_pen_bridge.rs`; HR-1 + DIRETRIZ §3.A.4).
//!
//! ## Coordinate model
//!
//! Identical to the Pen bridge: the vector network IS the asset (ADR-0056
//! §1.1). Pointer samples are converted `screen px → camera.screen_to_world
//! → world` in `vector_pencil_input.rs` and stored verbatim; the overlay
//! builds the world→screen `Affine` from the camera + window.
//!
//! ## What this bridge does
//!
//! - **Drains** the committed asset emitted by the previous tick's
//!   pointer-up into the shared committed-vector-scene list
//!   (`App::committed_vector_pen_paths`, also fed by the Pen).
//! - **Renders the in-progress overlay** — the live (un-smoothed) sample
//!   polyline, so the user sees their raw gesture until pointer-up swaps in
//!   the Hobby-smoothed curve.
//!
//! ## What this bridge does NOT do
//!
//! Committed Pencil paths are **open stroked** paths; they render through
//! the canonical `ph2d_vector::draw_vector_network` stroke-pass (every
//! segment whose `style_ref` resolves to a `StrokeStyle`), which the Pen
//! bridge already invokes on the shared committed list every frame. So
//! this bridge only handles the volatile overlay + the commit drain — no
//! committed-geometry rendering here (avoids a double-stroke).

use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_vector_pencil::VectorPencilTool;
use ph2d_vector::{Ph2dVectorAsset, VectorScene, draw_variable_width_stroke};
use ph2d_vector_doc::srgb8_to_oklch;

/// Pencil-blue overlay tint for the in-progress (un-smoothed) polyline.
const OVERLAY_RGB: (u8, u8, u8) = (90, 140, 255);
const OVERLAY_ALPHA: u8 = 170;
/// In-progress overlay line width, in screen px (kept constant on screen
/// by dividing by the camera scale `k`).
const OVERLAY_LINE_WIDTH_PX: f64 = 1.5;

/// Returns `true` iff the Vector Pencil is the active tool AND has an
/// in-progress (un-committed) stroke. The shell calls this before a
/// Pencil-pill toggle-off / tool switch runs `Tool::on_deactivate →
/// reset`, which silently discards the recorded samples (mirror of
/// `super::vector_pen_bridge::pen_has_in_progress_path` /
/// `painter_bridge::painter_has_unflushed_strokes`). The tool-concrete
/// downcast lives in this allowlisted bridge so central dispatch stays
/// downcast-free (`architecture_no_downcast_to_concrete_tool_in_shell`).
#[must_use]
pub(super) fn pencil_has_in_progress_stroke(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<VectorPencilTool>()
                .map(|p| p.has_in_progress_stroke())
        })
        .unwrap_or(false)
}

/// Per-frame Vector Pencil bridge dispatch.
///
/// `committed_paths` is the shared committed-vector-scene list (also fed by
/// the Pen bridge, which renders it via `draw_vector_network`). Safe to
/// call every frame: early-returns when the Pencil isn't active.
pub(super) fn dispatch(
    tools: &mut ToolRegistry,
    camera: &Camera2d,
    window_size: WindowSize,
    committed_paths: &mut Vec<Ph2dVectorAsset>,
    vector_scene: &mut VectorScene,
) {
    let is_pencil_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pencil"))
        .unwrap_or(false);
    if !is_pencil_active {
        return;
    }

    // Downcast — ADR-0040 §3 documented exception (mirror of the Pen bridge).
    let Some(active) = tools.active_mut() else {
        return;
    };
    let Some(pencil) = active.as_any_mut().downcast_mut::<VectorPencilTool>() else {
        return;
    };

    // Drain the asset emitted by the previous tick's pointer-up commit into
    // the shared scene (rendered canonically by `draw_vector_network`;
    // held in memory only — W1/W2 no disk persistence).
    if let Some(asset) = pencil.take_committed_asset() {
        committed_paths.push(asset);
    }

    // In-progress overlay: the raw sample polyline. `k = window.height /
    // camera.height_world`; world width = px / k keeps the overlay a
    // constant screen thickness at any zoom.
    let samples = pencil.current_samples();
    if samples.len() < 2 {
        return;
    }
    let world_to_screen = camera.world_to_screen_affine(window_size);
    let scene = vector_scene.inner_mut();
    let k = (window_size.height as f64) / (camera.height_world as f64).max(1e-6);
    let line_width_world = OVERLAY_LINE_WIDTH_PX / k;

    // W5 live preview: expand the in-progress polyline into a variable-width band
    // driven by per-sample pen pressure (`StrokeSample.pressure`, 0..=1; 1.0 on
    // pressure-less devices → constant width = the W2 look). The base width is the
    // constant-screen-thickness overlay width, so the taper scales with zoom.
    let overlay = srgb8_to_oklch([OVERLAY_RGB.0, OVERLAY_RGB.1, OVERLAY_RGB.2, OVERLAY_ALPHA]);
    let centerline: Vec<_> = samples.iter().map(|s| s.pos).collect();
    let widths: Vec<f32> = samples
        .iter()
        .map(|s| line_width_world as f32 * s.pressure)
        .collect();
    draw_variable_width_stroke(scene, &centerline, &widths, overlay, world_to_screen);
}
