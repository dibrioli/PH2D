//! Vector Pen tool ⟷ shell bridge — per-frame scene + in-progress overlay.
//!
//! Free function called once per frame BEFORE `paint_hero_screen`
//! (mirror of `bgremoval_preview.rs` shape; HR-1 + DIRETRIZ §3.A.4).
//!
//! ## Coordinate model
//!
//! The vector network IS the asset — there is no parent sprite (ADR-0056
//! §1.1, unlike the RasterEditTool family). A canvas click is converted
//! `screen px → camera.screen_to_world → world coords` and stored
//! directly in `VectorNetwork.vertices[i].pos`. Rendering builds a
//! world→screen `Affine` from the camera + window only.
//!
//! ## Three render layers
//!
//! - **(a) Committed paths** — scene state, rendered every frame
//!   regardless of the active tool. Closed triangles persist across tool
//!   switches; they are NOT Pen-tool state. Cleared via Esc when no path
//!   is in progress (`vector_pen_input::try_vector_pen_escape`).
//! - **(b) In-progress overlay** — only while Pen is the active tool:
//!   vertex dots + segment guidelines + rubber-band to the cursor.
//!
//! ## Persistence
//!
//! W1 holds committed paths in memory only — no disk write. Real
//! persistence (AssetDb + save-as) lands in W2; see
//! `docs/AUDIT_vector_module_W1_results.md` §6.

use std::collections::BTreeMap;

use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_vector_pen::VectorPenTool;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

/// Pen-blue overlay tint (R, G, B). Distinct per-layer alpha below.
const PEN_RGB: (u8, u8, u8) = (80, 130, 255);
const VERTEX_ALPHA: u8 = 230;
const SEGMENT_ALPHA: u8 = 200;
const RUBBER_ALPHA: u8 = 120;
/// Screen-space sizes (world units derived by dividing by the camera
/// scale `k`, so they stay constant on screen at any zoom).
const DOT_RADIUS_PX: f64 = 5.0;
const LINE_WIDTH_PX: f64 = 2.0;
/// Vertex 0 is drawn larger as the visible "close-path target".
const FIRST_VERTEX_SCALE: f64 = 1.6;
/// Below this world distance the rubber-band is a degenerate zero-length
/// line (emits nothing meaningful, skipped).
const RUBBER_BAND_MIN_WORLD: f32 = 0.001;

/// Returns `true` iff the Vector Pen is the active tool AND has an
/// in-progress (un-closed) path.
///
/// Audit H7 (destructive-deactivate warn): the shell calls this BEFORE a
/// Pen-pill toggle-off / tool switch runs `Tool::on_deactivate →
/// reset_path`, which silently discards the in-progress vertices. Mirror
/// of [`super::painter_bridge::painter_has_unflushed_strokes`]. The
/// tool-concrete downcast lives here in the allowlisted bridge so the
/// central dispatch stays downcast-free per the
/// `architecture_no_downcast_to_concrete_tool_in_shell` gate.
#[must_use]
pub(super) fn pen_has_in_progress_path(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<VectorPenTool>()
                .map(|p| p.has_in_progress_path())
        })
        .unwrap_or(false)
}

/// Per-frame Vector Pen bridge dispatch.
///
/// Early-returns if the active tool isn't `vector_pen`. No mutation on
/// early-return; safe to call every frame unconditionally.
pub(super) fn dispatch(
    tools: &mut ToolRegistry,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    committed_paths: &mut Vec<ph2d_vector::Ph2dVectorAsset>,
    placements: &[[f32; 6]],
    vector_scene: &mut VectorScene,
) {
    let world_to_screen = camera.world_to_screen_affine(window_size);
    let scene = vector_scene.inner_mut();

    // Layer (a) — committed scene paths: always rendered, each composed with its
    // ADR-0076 placement affine (`world_to_screen ∘ placement`). An asset committed
    // THIS frame isn't reconciled into `placements` yet → identity (its rest pose,
    // which is exactly where a brand-new vector belongs).
    for (i, asset) in committed_paths.iter().enumerate() {
        let placement = placements.get(i).map_or(Affine::IDENTITY, |m| {
            Affine::new([
                m[0] as f64,
                m[1] as f64,
                m[2] as f64,
                m[3] as f64,
                m[4] as f64,
                m[5] as f64,
            ])
        });
        ph2d_vector::draw_vector_network(
            scene,
            &asset.network,
            &asset.styles,
            world_to_screen * placement,
        );
    }

    let is_pen_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pen"))
        .unwrap_or(false);
    if !is_pen_active {
        return;
    }

    // Downcast — ADR-0040 §3 documented exception (mirror of
    // `painter_bridge` + `bgremoval_preview`).
    let Some(active) = tools.active_mut() else {
        return;
    };
    let Some(pen) = active.as_any_mut().downcast_mut::<VectorPenTool>() else {
        return;
    };

    // Drain the asset emitted by the previous tick's close-path into the
    // scene. Held in memory only (W1 — no disk persistence).
    if let Some(asset) = pen.take_committed_asset() {
        committed_paths.push(asset);
    }

    // No in-progress path → no overlay to draw.
    let network = pen.current_network();
    if network.vertices.is_empty() {
        return;
    }
    let styles = pen.current_styles();

    // Render the in-progress network's regions/fills (defensive — the W1
    // Pen never leaves a region open mid-build, but future tools sharing
    // this bridge might).
    ph2d_vector::draw_vector_network(scene, network, styles, world_to_screen);

    // Layer (b): in-progress overlay. `k = window.height /
    // camera.height_world`; sizing in world units = `px / k` yields a
    // constant screen size at any zoom.
    let k = (window_size.height as f64) / (camera.height_world as f64).max(1e-6);
    let dot_radius_world = DOT_RADIUS_PX / k;
    let line_width_world = LINE_WIDTH_PX / k;
    let vertex_color = Color::from_rgba8(PEN_RGB.0, PEN_RGB.1, PEN_RGB.2, VERTEX_ALPHA);
    let segment_color = Color::from_rgba8(PEN_RGB.0, PEN_RGB.1, PEN_RGB.2, SEGMENT_ALPHA);

    // One vertex-position lookup for the whole overlay (avoids the O(N²)
    // linear scan per segment). One reusable BezPath, truncated + refilled
    // per segment (one buffer instead of one alloc per segment).
    let vpos: BTreeMap<u32, Point> = network
        .vertices
        .iter()
        .map(|v| (v.id, Point::new(v.pos.x as f64, v.pos.y as f64)))
        .collect();
    let mut seg_path = BezPath::new();

    // Segments first (so dots paint on top of segment endpoints).
    for seg in &network.segments {
        let (Some(&start), Some(&end)) = (vpos.get(&seg.start), vpos.get(&seg.end)) else {
            continue;
        };
        seg_path.truncate(0);
        seg_path.move_to(start);
        seg_path.line_to(end);
        scene.stroke(
            &Stroke::new(line_width_world),
            world_to_screen,
            &Brush::Solid(segment_color),
            None,
            &seg_path,
        );
    }

    // Vertex dots — vertex 0 painted larger so it stands out as the
    // close-path target (Illustrator/Figma convention: first vertex
    // visually distinct).
    let first_id = network.vertices.first().map(|f| f.id);
    for v in &network.vertices {
        let radius = if Some(v.id) == first_id {
            dot_radius_world * FIRST_VERTEX_SCALE
        } else {
            dot_radius_world
        };
        let c = Circle::new(Point::new(v.pos.x as f64, v.pos.y as f64), radius);
        scene.fill(
            Fill::NonZero,
            world_to_screen,
            &Brush::Solid(vertex_color),
            None,
            &c,
        );
    }

    // Rubber-band preview from the last vertex → current cursor, only
    // while the path is still OPEN (no region yet). Gives spatial
    // feedback for where the next click would land.
    if network.regions.is_empty()
        && let Some(last_v) = network.vertices.last()
    {
        let cursor_world = camera.screen_to_world(last_pointer, window_size);
        if (cursor_world[0] - last_v.pos.x).abs() > RUBBER_BAND_MIN_WORLD
            || (cursor_world[1] - last_v.pos.y).abs() > RUBBER_BAND_MIN_WORLD
        {
            let rubber_color = Color::from_rgba8(PEN_RGB.0, PEN_RGB.1, PEN_RGB.2, RUBBER_ALPHA);
            seg_path.truncate(0);
            seg_path.move_to(Point::new(last_v.pos.x as f64, last_v.pos.y as f64));
            seg_path.line_to(Point::new(cursor_world[0] as f64, cursor_world[1] as f64));
            scene.stroke(
                &Stroke::new(line_width_world),
                world_to_screen,
                &Brush::Solid(rubber_color),
                None,
                &seg_path,
            );
        }
    }
}
