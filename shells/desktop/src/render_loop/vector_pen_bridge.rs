//! Vector Pen tool ⟷ shell bridge — per-frame preview + commit save.
//!
//! Mirror of `bgremoval_preview.rs` shape (HR-1 + DIRETRIZ §3.A.4):
//! a free function called once per frame BEFORE `paint_hero_screen`
//! when the Vector Pen tool is the active tool.
//!
//! ## Why no sprite selection requirement (R4 redesign)
//!
//! Initial T1.7 R1-R3 implementation copied the bgremoval pattern of
//! "convert screen → sprite-local pixel coords + render overlay over
//! sprite footprint". Enio caught the conceptual error: bgremoval EDITS
//! an existing raster sprite, but the Pen tool **creates new vector
//! content** — the vector network IS the asset, no parent sprite is
//! involved. Per ADR-0056 §1.1: Vector Module is "single in two
//! dimensions fundamentally" vs RasterEditTool family.
//!
//! Post-R4 contract:
//!
//! - Click screen px → `camera.screen_to_world(...)` → **world-space
//!   coords** stored directly in `VectorNetwork.vertices[i].pos`.
//! - Render: build world-→-screen `Affine` from camera + window only
//!   (no per-sprite transform chain).
//! - Save: `.ph2d-vector` carries world-coordinate paths; reload draws
//!   identically. The vector IS the spatial asset.
//!
//! ## What this does
//!
//! 1. Drain `VectorPenTool::take_committed_asset()` from the previous
//!    tick's close-path → save to disk via `save_vector_asset`. Toast
//!    success/error feedback.
//! 2. Build the world→screen `Affine` directly from the camera (no
//!    sprite footprint involved).
//! 3. Call `ph2d_vector::draw_vector_network(scene, &network, &styles,
//!    affine)` so the live triangle (or in-progress path) paints over
//!    the canvas wherever the user clicked.

use ph2d_editor::ToolRegistry;
use ph2d_editor::toast::{Toast, ToastQueue};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_vector_pen::VectorPenTool;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};
use std::time::{SystemTime, UNIX_EPOCH};

/// Per-frame Vector Pen bridge dispatch.
///
/// Early-returns if the active tool isn't `vector_pen`. No mutation
/// on early-return; safe to call every frame unconditionally.
pub(super) fn dispatch(
    tools: &mut ToolRegistry,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
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
        let _ = match save_asset_to_disk(&asset) {
            Ok(path) => toasts.push(Toast::info(format!("Vector saved: {path}"))),
            Err(e) => toasts.push(Toast::error(format!("Vector save failed: {e}"))),
        };
    }

    // Step 2 — render. Two layers:
    //
    // (a) Closed regions via `draw_vector_network` — covers the
    //     post-close-path filled triangle (rare during build; common
    //     after first commit when an unclosed second path overlays a
    //     prior closed region if that ever happens).
    //
    // (b) **In-progress preview overlay** — vertex DOTS + segment
    //     LINES drawn directly. R5 fix: `draw_vector_network` only
    //     renders REGIONS with fill; vertices+segments alone (the
    //     standard state during clicks 1..3 of a triangle) produce
    //     nothing visible. Standard Pen-tool UX (Illustrator/Figma)
    //     shows vertex dots + segment guidelines DURING the build so
    //     the user gets feedback per-click.
    let network = pen.current_network();
    if network.vertices.is_empty() {
        return;
    }
    let styles = pen.current_styles();
    let world_to_screen = world_to_screen_affine(camera, window_size);
    let scene = vector_scene.inner_mut();

    // Layer (a): fills.
    ph2d_vector::draw_vector_network(scene, network, styles, world_to_screen);

    // Layer (b): in-progress vertex dots + segment guidelines.
    // The affine scales world→screen by `k = window.height /
    // camera.height_world`. To get a ~5 px dot regardless of zoom,
    // size in world units = `5 / k` (the affine then scales back to
    // 5 px on screen).
    let k = (window_size.height as f64) / (camera.height_world as f64).max(1e-6);
    let dot_radius_world = 5.0 / k;
    let line_width_world = 2.0 / k;
    let vertex_color = Color::from_rgba8(80, 130, 255, 230);
    let segment_color = Color::from_rgba8(80, 130, 255, 200);

    // Segments first (so dots paint on top of segment endpoints).
    for seg in &network.segments {
        let Some(start) = network.vertices.iter().find(|v| v.id == seg.start) else {
            continue;
        };
        let Some(end) = network.vertices.iter().find(|v| v.id == seg.end) else {
            continue;
        };
        let mut path = BezPath::new();
        path.move_to(Point::new(start.pos.x as f64, start.pos.y as f64));
        path.line_to(Point::new(end.pos.x as f64, end.pos.y as f64));
        scene.stroke(
            &Stroke::new(line_width_world),
            world_to_screen,
            &Brush::Solid(segment_color),
            None,
            &path,
        );
    }

    // Vertex dots — vertex 0 painted LAST + larger so it stands out
    // as the "close-path target" the user can click to finish the
    // path. (Mirror Illustrator/Figma convention: first vertex
    // visually distinct.)
    for v in &network.vertices {
        let radius = if Some(v.id) == network.vertices.first().map(|f| f.id) {
            dot_radius_world * 1.6
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

    // Rubber-band preview line from last vertex → current cursor in
    // world coords. Standard Pen-tool UX (Illustrator / Figma /
    // Inkscape): user sees the candidate next segment before
    // committing the click. Without this the user has no spatial
    // feedback for "where would my next click land".
    if let Some(last_v) = network.vertices.last() {
        let cursor_world = camera.screen_to_world(last_pointer, window_size);
        // Skip drawing the rubber-band if the cursor coincides with
        // the last vertex (zero-length line emits nothing meaningful
        // and could surface as a degenerate stroke in some Vello
        // versions).
        if (cursor_world[0] - last_v.pos.x).abs() > 0.001
            || (cursor_world[1] - last_v.pos.y).abs() > 0.001
        {
            let rubber_color = Color::from_rgba8(80, 130, 255, 120);
            let mut rubber = BezPath::new();
            rubber.move_to(Point::new(last_v.pos.x as f64, last_v.pos.y as f64));
            rubber.line_to(Point::new(cursor_world[0] as f64, cursor_world[1] as f64));
            scene.stroke(
                &Stroke::new(line_width_world),
                world_to_screen,
                &Brush::Solid(rubber_color),
                None,
                &rubber,
            );
        }
    }
}

/// World-meters → screen-pixel affine derived from the camera.
///
/// Matches the same projection as
/// `Camera2d::world_to_screen([x, y], window)` — uniform scale
/// `k = window.height / camera.height_world` (square pixels), Y
/// inverted (world Y-up → screen Y-down), translated by window center
/// and camera center.
fn world_to_screen_affine(camera: &Camera2d, window: WindowSize) -> Affine {
    let k = (window.height as f64) / (camera.height_world as f64).max(1e-6);
    Affine::translate((
        window.width as f64 * 0.5,
        window.height as f64 * 0.5,
    )) * Affine::scale_non_uniform(k, -k)
        * Affine::translate((-camera.center[0] as f64, -camera.center[1] as f64))
}

/// Save the committed Pen asset to disk under a timestamped filename.
///
/// **W1.T1.7 MVP convention**: dumps into
/// `vector_pen_<unix_secs>.ph2d-vector` in the process's current
/// directory. W2 will plumb the asset path through `AssetDb` + a real
/// "save as" dialog; for the smoke Day-7 the user verifies file
/// presence + can re-load via `load_and_validate_vector_asset`.
fn save_asset_to_disk(asset: &ph2d_vector::Ph2dVectorAsset) -> Result<String, String> {
    let bytes = ph2d_vector::save_vector_asset(asset).map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = format!("vector_pen_{ts}.ph2d-vector");
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}
