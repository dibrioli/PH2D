//! Vector selection overlay — per-frame highlight of the shared
//! [`VectorSelection`] over the committed vector scene + the live marquee.
//!
//! Free function called once per frame BEFORE `paint_hero_screen` (mirror of
//! the other vector bridges). Pure feedback — it never mutates the scene:
//! selected-network bounding-box outlines, selected-vertex dots (Direct-Select),
//! and the in-progress Select-tool marquee rectangle.
//!
//! Reads the shell-owned [`VectorSelection`] + committed scene by-ref; the
//! marquee rect is read by downcasting the active tool to [`VectorSelectTool`]
//! (the imported short name keeps this off the central-dispatch downcast gate,
//! same as the other vector input/bridge files). Wired by the W2.T2.3 shell
//! pass: `render_loop/mod.rs` declares the module + calls `dispatch(...)` after
//! the create-tool bridges, passing `&App::committed_vector_pen_paths` +
//! `&App::vector_selection`.

use ph2d_core::Vec2;
use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_vector_select::VectorSelectTool;
use ph2d_vector::{BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};
use ph2d_vector_doc::{Ph2dVectorAsset, VectorSelection};

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

    // Layer 1.5 — Direct-Select editable affordances. When Direct is the
    // active tool, surface EVERY vertex (a grab target) + every non-zero
    // tangent handle across the committed scene, so the user has something
    // to aim at — `selection.vertices` is empty until a grab succeeds, a
    // chicken-and-egg that made Direct feel dead. Drawn before Layer 2 so a
    // grabbed/selected vertex renders brighter on top. Pure feedback.
    let direct_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("vector_direct"))
        .unwrap_or(false);
    if direct_active {
        let handle_w = MARQUEE_WIDTH_PX / k;
        let aff_dot_r = VERTEX_DOT_RADIUS_PX / k;
        let ctrl_r = (VERTEX_DOT_RADIUS_PX * 0.7) / k;
        for asset in committed {
            // Tangent handles: a thin line vertex→control-point + a control
            // dot, per non-zero tangent (straight segments have zero tangents
            // and draw nothing — mirror of `nearest_tangent_handle`'s skip).
            for seg in &asset.network.segments {
                let (Some(start), Some(end)) = (
                    asset.network.vertices.iter().find(|v| v.id == seg.start),
                    asset.network.vertices.iter().find(|v| v.id == seg.end),
                ) else {
                    continue;
                };
                for (base, tan) in [(start.pos, seg.out_at_start), (end.pos, seg.in_at_end)] {
                    if tan.length_squared() < 1e-6 {
                        continue;
                    }
                    let ctrl = base + tan;
                    let mut line = BezPath::new();
                    line.move_to(Point::new(base.x as f64, base.y as f64));
                    line.line_to(Point::new(ctrl.x as f64, ctrl.y as f64));
                    scene.stroke(
                        &Stroke::new(handle_w),
                        world_to_screen,
                        &Brush::Solid(accent(MARQUEE_LINE_ALPHA)),
                        None,
                        &line,
                    );
                    let dot = Circle::new(Point::new(ctrl.x as f64, ctrl.y as f64), ctrl_r);
                    scene.fill(
                        Fill::NonZero,
                        world_to_screen,
                        &Brush::Solid(accent(VERTEX_ALPHA)),
                        None,
                        &dot,
                    );
                }
            }
            // Vertex dots (grab targets) — dimmer than the selected-vertex
            // highlight that Layer 2 paints on top.
            for v in &asset.network.vertices {
                let c = Circle::new(Point::new(v.pos.x as f64, v.pos.y as f64), aff_dot_r);
                scene.fill(
                    Fill::NonZero,
                    world_to_screen,
                    &Brush::Solid(accent(OUTLINE_ALPHA)),
                    None,
                    &c,
                );
            }
        }
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
        .and_then(|t| t.marquee_rect());
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
