//! Vector selection overlay — per-frame highlight of the shared
//! [`VectorSelection`] over the committed vector scene + the live marquee.
//!
//! Free function called once per frame BEFORE `paint_hero_screen` (mirror
//! of the other vector bridges). Renders three layers of *feedback* (it
//! never mutates the scene):
//! 1. **Selected networks** — an accent outline around each selected
//!    network's bounding box.
//! 2. **Selected vertices** — accent dots (Direct-Select feedback).
//! 3. **Marquee rect** — the in-progress Select-tool drag rectangle.
//!
//! Reads the shell-owned [`VectorSelection`] + committed scene by-ref; the
//! marquee rect is read by downcasting the active tool to
//! [`VectorSelectTool`] (allowlisted bridge downcast, ADR-0040 §3).
//!
//! ## ⚠ Central wiring required (Coord) — see `docs/HANDOFF_vector_w2_t23_select_coord.md`
//!
//! `render_loop/mod.rs`: `mod vector_selection_bridge;` + call
//! `dispatch(tools, camera, window_size, &self.committed_vector_pen_paths,
//! &self.vector_selection, vector_scene)` after the tool bridges.

use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_vector_select::VectorSelectTool;
use ph2d_vector::{BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};
use ph2d_vector_doc::{Ph2dVectorAsset, VectorSelection};
use glam::Vec2;

/// Accent (R, G, B) for selection feedback — warm amber, distinct from the
/// pen/pencil blue overlay.
const ACCENT_RGB: (u8, u8, u8) = (255, 170, 60);
const OUTLINE_ALPHA: u8 = 210;
const VERTEX_ALPHA: u8 = 235;
const MARQUEE_FILL_ALPHA: u8 = 40;
const MARQUEE_LINE_ALPHA: u8 = 180;
/// Screen-px sizes (converted to world by dividing by the camera scale).
const OUTLINE_WIDTH_PX: f64 = 1.5;
const VERTEX_DOT_RADIUS_PX: f64 = 4.0;
const MARQUEE_WIDTH_PX: f64 = 1.0;

/// Per-frame selection overlay dispatch. Safe to call every frame.
pub(super) fn dispatch(
    tools: &mut ToolRegistry,
    camera: &Camera2d,
    window_size: WindowSize,
    committed: &[Ph2dVectorAsset],
    selection: &VectorSelection,
    vector_scene: &mut VectorScene,
) {
    let world_to_screen = camera.world_to_screen_affine(window_size);
    let k = (window_size.height as f64) / (camera.height_world as f64).max(1e-6);
    let scene = vector_scene.inner_mut();
    let accent = |a: u8| Color::from_rgba8(ACCENT_RGB.0, ACCENT_RGB.1, ACCENT_RGB.2, a);

    // Layer 1 — selected network bounding boxes (accent outline).
    let outline_w = OUTLINE_WIDTH_PX / k;
    for &idx in &selection.networks {
        let Some(asset) = committed.get(idx) else {
            continue;
        };
        let Some((min, max)) = asset.network.bounding_box() else {
            continue;
        };
        let mut path = BezPath::new();
        rect_path(&mut path, min, max);
        scene.stroke(
            &Stroke::new(outline_w),
            world_to_screen,
            &Brush::Solid(accent(OUTLINE_ALPHA)),
            None,
            &path,
        );
    }

    // Layer 2 — selected vertex dots.
    let dot_r = VERTEX_DOT_RADIUS_PX / k;
    for &(asset_idx, vid) in &selection.vertices {
        let Some(asset) = committed.get(asset_idx) else {
            continue;
        };
        let Some(v) = asset.network.vertices.iter().find(|v| v.id == vid) else {
            continue;
        };
        let c = Circle::new(Point::new(v.pos.x as f64, v.pos.y as f64), dot_r);
        scene.fill(
            Fill::NonZero,
            world_to_screen,
            &Brush::Solid(accent(VERTEX_ALPHA)),
            None,
            &c,
        );
    }

    // Layer 3 — live marquee rect (only while the Select tool is dragging).
    let marquee = tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<VectorSelectTool>())
        .and_then(VectorSelectTool::marquee_rect);
    if let Some((min, max)) = marquee {
        let mut rect = BezPath::new();
        rect_path(&mut rect, min, max);
        // Subtle fill + crisp outline.
        scene.fill(
            Fill::NonZero,
            world_to_screen,
            &Brush::Solid(accent(MARQUEE_FILL_ALPHA)),
            None,
            &rect,
        );
        scene.stroke(
            &Stroke::new(MARQUEE_WIDTH_PX / k),
            world_to_screen,
            &Brush::Solid(accent(MARQUEE_LINE_ALPHA)),
            None,
            &rect,
        );
    }
}

/// Build a closed axis-aligned rectangle path from `(min, max)`.
fn rect_path(path: &mut BezPath, min: Vec2, max: Vec2) {
    path.move_to(Point::new(min.x as f64, min.y as f64));
    path.line_to(Point::new(max.x as f64, min.y as f64));
    path.line_to(Point::new(max.x as f64, max.y as f64));
    path.line_to(Point::new(min.x as f64, max.y as f64));
    path.close_path();
}
