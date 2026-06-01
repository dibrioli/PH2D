//! Vector Pencil tool ⟷ shell bridge — per-frame committed strokes +
//! in-progress overlay.
//!
//! Free function called once per frame BEFORE `paint_hero_screen` (mirror
//! of `vector_pen_bridge.rs`; HR-1 + DIRETRIZ §3.A.4).
//!
//! ## Coordinate model
//!
//! Identical to the Pen bridge: the vector network IS the asset (ADR-0056
//! §1.1). Pointer samples are converted `screen px → camera.screen_to_world
//! → world` in `vector_pencil_input.rs` and stored verbatim; rendering
//! builds the world→screen `Affine` from the camera + window.
//!
//! ## Two render layers
//!
//! - **(a) Committed stroked paths** — scene state, rendered every frame
//!   regardless of the active tool. A Pencil path is an OPEN path; it has
//!   no region, so `ph2d_vector::draw_vector_network` (fill-only) never
//!   draws it. This bridge strokes every segment whose `style_ref`
//!   resolves to a [`StrokeStyle`] in the asset's table. Pen-tool segments
//!   carry no `style_ref` (their region carries the fill), so they are
//!   skipped here — the committed list can safely mix Pen + Pencil assets
//!   with no double-draw.
//! - **(b) In-progress overlay** — only while the Pencil is the active
//!   tool: the live (un-smoothed) sample polyline, so the user sees their
//!   raw gesture until pointer-up swaps in the Hobby-smoothed curve.
//!
//! ## INTERIM stroke rendering — Coord item
//!
//! Layer (a) lives in this bridge **pending a canonical stroke pass in
//! `ph2d_vector::draw_vector_network`** (which is fill-only today). When
//! that lands (Coord; touches the frozen `ph2d-vector` surface), this
//! shell-side stroke loop is deleted and committed Pencil paths render
//! through the canonical path like committed Pen regions do. See
//! `docs/HANDOFF_vector_w2_pencil_coord.md`.

use std::collections::BTreeMap;

use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_vector_pencil::VectorPencilTool;
use ph2d_vector::{
    BezPath, Brush, Color, Ph2dVectorAsset, Point, Stroke, VectorScene, oklch_to_color,
};

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
/// the Pen bridge). Safe to call every frame: the committed-stroke pass is
/// unconditional, the overlay early-returns when the Pencil isn't active.
pub(super) fn dispatch(
    tools: &mut ToolRegistry,
    camera: &Camera2d,
    window_size: WindowSize,
    committed_paths: &mut Vec<Ph2dVectorAsset>,
    vector_scene: &mut VectorScene,
) {
    let world_to_screen = camera.world_to_screen_affine(window_size);
    let scene = vector_scene.inner_mut();

    // Layer (a) — committed stroked paths (INTERIM; see module docs).
    for asset in committed_paths.iter() {
        stroke_styled_segments(scene, asset, world_to_screen);
    }

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
    // the shared scene (held in memory only — W1/W2 no disk persistence).
    if let Some(asset) = pencil.take_committed_asset() {
        committed_paths.push(asset);
    }

    // Layer (b) — in-progress overlay: the raw sample polyline. `k =
    // window.height / camera.height_world`; world width = px / k keeps the
    // overlay a constant screen thickness at any zoom.
    let samples = pencil.current_samples();
    if samples.len() < 2 {
        return;
    }
    let k = (window_size.height as f64) / (camera.height_world as f64).max(1e-6);
    let line_width_world = OVERLAY_LINE_WIDTH_PX / k;
    let color = Color::from_rgba8(OVERLAY_RGB.0, OVERLAY_RGB.1, OVERLAY_RGB.2, OVERLAY_ALPHA);

    let mut path = BezPath::new();
    path.move_to(Point::new(
        samples[0].pos.x as f64,
        samples[0].pos.y as f64,
    ));
    for s in &samples[1..] {
        path.line_to(Point::new(s.pos.x as f64, s.pos.y as f64));
    }
    scene.stroke(
        &Stroke::new(line_width_world),
        world_to_screen,
        &Brush::Solid(color),
        None,
        &path,
    );
}

/// Stroke every segment of `asset` whose `style_ref` resolves to a stroke
/// in the asset's table. Open Pencil paths render here; closed Pen regions
/// (no per-segment `style_ref`) are skipped.
///
/// The stroke width is the document-space [`StrokeStyle::width`], so it
/// scales with zoom under `world_to_screen` — correct vector behavior
/// (a 2 px stroke gets visually thicker as you zoom in, like Illustrator).
fn stroke_styled_segments(
    scene: &mut ph2d_vector::Scene,
    asset: &Ph2dVectorAsset,
    world_to_screen: ph2d_vector::Affine,
) {
    let network = &asset.network;
    if network.segments.is_empty() {
        return;
    }
    let vpos: BTreeMap<u32, Point> = network
        .vertices
        .iter()
        .map(|v| (v.id, Point::new(v.pos.x as f64, v.pos.y as f64)))
        .collect();

    let mut seg_path = BezPath::new();
    for seg in &network.segments {
        let Some(style_ref) = seg.style_ref else {
            continue; // unstyled (e.g. a Pen region edge) — not our draw.
        };
        let Some(stroke_style) = asset.styles.strokes.get(&style_ref) else {
            continue;
        };
        let (Some(&start), Some(&end)) = (vpos.get(&seg.start), vpos.get(&seg.end)) else {
            continue;
        };
        // Cubic control points from the tangent offset vectors (renderer
        // convention: c1 = start + out_at_start, c2 = end + in_at_end).
        let c1 = Point::new(
            start.x + seg.out_at_start.x as f64,
            start.y + seg.out_at_start.y as f64,
        );
        let c2 = Point::new(
            end.x + seg.in_at_end.x as f64,
            end.y + seg.in_at_end.y as f64,
        );
        seg_path.truncate(0);
        seg_path.move_to(start);
        seg_path.curve_to(c1, c2, end);
        scene.stroke(
            &Stroke::new(f64::from(stroke_style.width)),
            world_to_screen,
            &Brush::Solid(oklch_to_color(stroke_style.color)),
            None,
            &seg_path,
        );
    }
}
