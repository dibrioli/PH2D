//! World-space grid overlay for the editor canvas (ADR-0025 M14.4b).
//!
//! Renders minor + major grid lines in world meters, projected into
//! canvas screen pixels. The same projection the sprite renderer uses
//! (camera center + `height_world` + window aspect), so the grid
//! visually aligns with sprite positions.
//!
//! # Why Vello, not a dedicated wgsl pipeline
//!
//! `editor` feature gates this module; release-game builds drop it
//! entirely (HR-7). A bespoke shader would cost a separate pipeline,
//! plus bind groups, plus a transient texture, all paid only by the
//! editor. Vello already runs as a post-sprite Vello pass — adding
//! a few-N stroked thin lines per frame is cheap (well below the
//! 3.5 ms render budget in §11.1).
//!
//! # LOD
//!
//! When the minor spacing maps to fewer than 6 screen pixels we
//! skip the minor lines and draw only the majors — keeps the
//! display from looking like a mesh blur when zoomed out. The
//! threshold matches Blender's grid LOD for the same reason.
//!
//! # Determinism
//!
//! Line counts are a pure function of `(GridView, GridConfig)`.
//! Same inputs → identical line endpoints. Useful for the `cargo
//! test`-side `grid_visible_lines` test (no GPU needed).

use crate::paint::{rect_to_vello, resolve};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Stroke, VectorScene};

/// Minor spacing in screen pixels below which minor lines are
/// suppressed. Same value Blender's 2D grid uses — empirically the
/// point at which two adjacent minor lines become indistinguishable.
pub const LOD_MIN_PX: f32 = 6.0;

/// Grid display configuration. Defaults to 1m minor + 5m major in
/// the editor's `Outline*` tokens. Game projects can swap these via
/// `HeroScreen::grid_config_mut()` to match their target world
/// scale (e.g. a pixel-art project with 16px = 1 unit might want
/// 16-unit minor / 256-unit major).
#[derive(Clone, Debug)]
pub struct GridConfig {
    /// Spacing between minor grid lines in world meters.
    pub spacing_minor: f32,
    /// Spacing between major grid lines in world meters. Should be
    /// an integer multiple of `spacing_minor` so major lines line
    /// up exactly with a subset of minors.
    pub spacing_major: f32,
    /// Stroke width of minor lines in screen pixels.
    pub stroke_minor: f32,
    /// Stroke width of major lines in screen pixels.
    pub stroke_major: f32,
    /// Color token for minor lines.
    pub color_minor: ColorToken,
    /// Color token for major lines.
    pub color_major: ColorToken,
    /// Color token for the world axes when no per-axis override is
    /// set. Kept for backward compat — `paint_grid` prefers the
    /// per-axis RGBA fields below when they're set.
    pub color_axis: ColorToken,
    /// Stroke width of the world axes (always painted, regardless
    /// of LOD).
    pub stroke_axis: f32,
    /// X-axis color (Blender / Maya convention: red). Bypasses the
    /// `color_axis` token so the axis cue stays consistent regardless
    /// of the active theme.
    pub color_axis_x_rgba: [u8; 4],
    /// Y-axis color (Blender / Maya convention: green).
    pub color_axis_y_rgba: [u8; 4],
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            spacing_minor: 1.0,
            spacing_major: 5.0,
            stroke_minor: 0.6,
            stroke_major: 1.0,
            color_minor: ColorToken::GridLine,
            color_major: ColorToken::GridAxis,
            color_axis: ColorToken::GridAxis,
            stroke_axis: 1.4,
            // Blender / Maya convention: X red, Y green. Tuned a
            // little darker than pure (255, 0, 0) / (0, 255, 0) so
            // the axis lines don't blow out HDR tonemap.
            color_axis_x_rgba: [0xD0, 0x40, 0x40, 0xFF],
            color_axis_y_rgba: [0x40, 0xC0, 0x60, 0xFF],
        }
    }
}

/// Per-frame view parameters for the grid painter. The host's
/// `render_frame` builds this from the demo `Camera2d` + window
/// size + computed canvas rect.
#[derive(Copy, Clone, Debug)]
pub struct GridView {
    /// Camera center in world meters. The grid's `(0,0)` axes pass
    /// through this point's screen projection.
    pub camera_center: [f32; 2],
    /// Vertical span of the visible region in world meters. Matches
    /// `ph2d_render::Camera2d::height_world`.
    pub camera_height_world: f32,
    /// Window width in screen pixels — used to derive the
    /// projection aspect ratio (same as the sprite renderer's).
    pub window_w: f32,
    /// Window height in screen pixels.
    pub window_h: f32,
    /// Editor canvas region (sub-rect of the window). Grid lines
    /// are clipped to this rect so they don't overlap the
    /// Inspector/Hierarchy panels.
    pub canvas: Rect,
}

/// Number of grid lines a [`paint_grid`] call would emit, for tests
/// + telemetry. Doesn't paint anything.
pub fn count_visible_lines(view: &GridView, config: &GridConfig) -> GridLineCounts {
    let (world_bounds, ppm_y) = world_bounds(view);
    let minor_px_y = config.spacing_minor * ppm_y;
    let draw_minor = minor_px_y >= LOD_MIN_PX;

    let count_in_range = |start: f32, end: f32, spacing: f32| -> u32 {
        let first = (start / spacing).ceil() as i32;
        let last = (end / spacing).floor() as i32;
        (last - first + 1).max(0) as u32
    };

    let major_v = count_in_range(world_bounds.left, world_bounds.right, config.spacing_major);
    let major_h = count_in_range(world_bounds.bottom, world_bounds.top, config.spacing_major);
    let (minor_v, minor_h) = if draw_minor {
        let all_v = count_in_range(world_bounds.left, world_bounds.right, config.spacing_minor);
        let all_h = count_in_range(world_bounds.bottom, world_bounds.top, config.spacing_minor);
        // Minor pass skips lines that also coincide with majors to
        // avoid double-painting. Subtract the major counts.
        (all_v.saturating_sub(major_v), all_h.saturating_sub(major_h))
    } else {
        (0, 0)
    };
    GridLineCounts {
        minor_vertical: minor_v,
        minor_horizontal: minor_h,
        major_vertical: major_v,
        major_horizontal: major_h,
        minor_visible: draw_minor,
    }
}

/// Aggregate counts returned by [`count_visible_lines`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GridLineCounts {
    pub minor_vertical: u32,
    pub minor_horizontal: u32,
    pub major_vertical: u32,
    pub major_horizontal: u32,
    /// `true` when the LOD threshold lets minor lines render.
    /// `false` means only majors + axes will be drawn.
    pub minor_visible: bool,
}

impl GridLineCounts {
    pub fn total(&self) -> u32 {
        self.minor_vertical + self.minor_horizontal + self.major_vertical + self.major_horizontal
    }
}

/// Paint the grid overlay onto `scene`, clipped to `view.canvas`.
///
/// Visit order (back-to-front):
/// 1. Minor vertical + horizontal lines (skipped at LOD threshold).
/// 2. Major vertical + horizontal lines (always).
/// 3. The X=0 and Y=0 axes (always; emphasized stroke).
///
/// Each pass walks the visible world range at the relevant spacing
/// and emits one `BezPath` per pass — fewer scene-graph nodes than
/// one path per line.
pub fn paint_grid(scene: &mut VectorScene, theme: Theme, view: &GridView, config: &GridConfig) {
    if view.canvas.w <= 0.0 || view.canvas.h <= 0.0 {
        return;
    }
    if view.window_w <= 0.0 || view.window_h <= 0.0 {
        return;
    }
    if config.spacing_minor <= 0.0 || config.spacing_major <= 0.0 {
        return;
    }

    let (bounds, ppm_y) = world_bounds(view);
    let minor_px_y = config.spacing_minor * ppm_y;
    let draw_minor = minor_px_y >= LOD_MIN_PX;

    // Clip every emitted line to the canvas region so panels are
    // not painted over.
    scene.push_clip(&rect_to_vello(view.canvas));

    // Minor pass (skip lines that overlap a major to avoid
    // double-painting). Draw before majors so majors paint on top.
    if draw_minor {
        let path = build_grid_path(
            &bounds,
            view,
            config.spacing_minor,
            Some(config.spacing_major),
        );
        stroke_path(
            scene,
            &path,
            config.stroke_minor,
            resolve(config.color_minor, theme),
        );
    }

    // Major pass.
    let major_path = build_grid_path(&bounds, view, config.spacing_major, None);
    stroke_path(
        scene,
        &major_path,
        config.stroke_major,
        resolve(config.color_major, theme),
    );

    // Axis pass: paint X and Y separately so each gets its canonical
    // color (Blender/Maya convention: X red, Y green). Only emits a
    // path when the camera actually sees that axis.
    let _ = theme; // axis colors come from the RGBA literals, not the theme
    if bounds.left <= 0.0 && bounds.right >= 0.0 {
        // The vertical line at world x=0 IS the Y axis.
        let sx = world_to_screen_x(0.0, &bounds, view);
        let mut path = BezPath::new();
        path.move_to((sx as f64, view.canvas.y as f64));
        path.line_to((sx as f64, (view.canvas.y + view.canvas.h) as f64));
        let [r, g, b, a] = config.color_axis_y_rgba;
        stroke_path(
            scene,
            &path,
            config.stroke_axis,
            ph2d_vector::Color::from_rgba8(r, g, b, a),
        );
    }
    if bounds.bottom <= 0.0 && bounds.top >= 0.0 {
        // The horizontal line at world y=0 IS the X axis.
        let sy = world_to_screen_y(0.0, &bounds, view);
        let mut path = BezPath::new();
        path.move_to((view.canvas.x as f64, sy as f64));
        path.line_to(((view.canvas.x + view.canvas.w) as f64, sy as f64));
        let [r, g, b, a] = config.color_axis_x_rgba;
        stroke_path(
            scene,
            &path,
            config.stroke_axis,
            ph2d_vector::Color::from_rgba8(r, g, b, a),
        );
    }

    scene.pop_layer();
}

#[derive(Copy, Clone, Debug)]
struct WorldBounds {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

fn world_bounds(view: &GridView) -> (WorldBounds, f32) {
    let aspect = view.window_w / view.window_h;
    let half_h = view.camera_height_world * 0.5;
    let half_w = half_h * aspect;
    let cx = view.camera_center[0];
    let cy = view.camera_center[1];
    let bounds = WorldBounds {
        left: cx - half_w,
        right: cx + half_w,
        bottom: cy - half_h,
        top: cy + half_h,
    };
    let ppm_y = view.window_h / view.camera_height_world;
    (bounds, ppm_y)
}

fn world_to_screen_x(wx: f32, bounds: &WorldBounds, view: &GridView) -> f32 {
    let t = (wx - bounds.left) / (bounds.right - bounds.left);
    t * view.window_w
}

fn world_to_screen_y(wy: f32, bounds: &WorldBounds, view: &GridView) -> f32 {
    // Y-flip: world Y-up → screen Y-down (SKILL §11.1).
    let t = (wy - bounds.bottom) / (bounds.top - bounds.bottom);
    view.window_h - t * view.window_h
}

fn build_grid_path(
    bounds: &WorldBounds,
    view: &GridView,
    spacing: f32,
    skip_multiples_of: Option<f32>,
) -> BezPath {
    let mut path = BezPath::new();
    // Vertical lines (constant world X, varying Y).
    let first_v = (bounds.left / spacing).ceil() as i32;
    let last_v = (bounds.right / spacing).floor() as i32;
    for i in first_v..=last_v {
        let wx = i as f32 * spacing;
        if should_skip(wx, skip_multiples_of) {
            continue;
        }
        let sx = world_to_screen_x(wx, bounds, view);
        path.move_to((sx as f64, view.canvas.y as f64));
        path.line_to((sx as f64, (view.canvas.y + view.canvas.h) as f64));
    }
    // Horizontal lines (constant world Y, varying X).
    let first_h = (bounds.bottom / spacing).ceil() as i32;
    let last_h = (bounds.top / spacing).floor() as i32;
    for j in first_h..=last_h {
        let wy = j as f32 * spacing;
        if should_skip(wy, skip_multiples_of) {
            continue;
        }
        let sy = world_to_screen_y(wy, bounds, view);
        path.move_to((view.canvas.x as f64, sy as f64));
        path.line_to(((view.canvas.x + view.canvas.w) as f64, sy as f64));
    }
    path
}

fn should_skip(world_coord: f32, skip_multiples_of: Option<f32>) -> bool {
    let Some(major) = skip_multiples_of else {
        return false;
    };
    if major <= 0.0 {
        return false;
    }
    let q = world_coord / major;
    // Tolerance: f32 round-trip jitter on coords that *are* a major
    // multiple can land ~1e-5 off. Treat anything within 1e-3 of an
    // integer as a major coincidence.
    (q.round() - q).abs() < 1e-3
}

fn stroke_path(scene: &mut VectorScene, path: &BezPath, width: f32, color: ph2d_vector::Color) {
    if path.is_empty() {
        return;
    }
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, path);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view_unit() -> GridView {
        // 800×600 window with a centered 400×300 canvas, camera at
        // origin with 10m height → 5px = 1m (vertical).
        GridView {
            camera_center: [0.0, 0.0],
            camera_height_world: 10.0,
            window_w: 800.0,
            window_h: 600.0,
            canvas: Rect::new(200.0, 150.0, 400.0, 300.0),
        }
    }

    #[test]
    fn world_origin_maps_to_window_center() {
        let view = view_unit();
        let (bounds, _) = world_bounds(&view);
        let sx = world_to_screen_x(0.0, &bounds, &view);
        let sy = world_to_screen_y(0.0, &bounds, &view);
        assert!((sx - 400.0).abs() < 1e-3, "screen x for world 0: {sx}");
        assert!((sy - 300.0).abs() < 1e-3, "screen y for world 0: {sy}");
    }

    #[test]
    fn positive_world_y_maps_to_upper_screen() {
        // Per SKILL §11.1: world Y-up → screen Y-down. World +5
        // (top edge of camera view) should map to screen y ≈ 0
        // (top of window).
        let view = view_unit();
        let (bounds, _) = world_bounds(&view);
        let sy = world_to_screen_y(5.0, &bounds, &view);
        assert!(sy < 1.0, "world +5 should map to top of screen: got {sy}");
    }

    #[test]
    fn counts_match_default_camera() {
        // Default camera: height_world 10, aspect 800/600 ≈ 1.333.
        // Visible world rect: x in [-6.66, 6.66], y in [-5, 5].
        // Major spacing 5 → 3 vertical (-5, 0, 5) + 3 horizontal.
        // Minor spacing 1 → 14 vertical (range ceil(-6.66)..floor(6.66)
        // = -6..6 = 13 lines), minus 3 majors (-5, 0, 5) → 10 minor v.
        // Horizontal minor: range -5..5 = 11 lines minus 3 majors = 8.
        let view = view_unit();
        let config = GridConfig::default();
        let counts = count_visible_lines(&view, &config);
        assert!(counts.minor_visible);
        assert_eq!(counts.major_vertical, 3, "majors_v: {counts:?}");
        assert_eq!(counts.major_horizontal, 3, "majors_h: {counts:?}");
        assert_eq!(counts.minor_vertical, 10, "minors_v: {counts:?}");
        assert_eq!(counts.minor_horizontal, 8, "minors_h: {counts:?}");
    }

    #[test]
    fn lod_suppresses_minor_below_threshold() {
        // Zoom far out so 1m maps to < 6px vertically.
        // window_h=600, height_world=600 → 1px/m.
        let view = GridView {
            camera_center: [0.0, 0.0],
            camera_height_world: 600.0,
            window_w: 800.0,
            window_h: 600.0,
            canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
        };
        let counts = count_visible_lines(&view, &GridConfig::default());
        assert!(!counts.minor_visible, "minor should be suppressed");
        assert_eq!(counts.minor_vertical, 0);
        assert_eq!(counts.minor_horizontal, 0);
        // Majors still drawn.
        assert!(counts.major_vertical > 0);
        assert!(counts.major_horizontal > 0);
    }

    #[test]
    fn lod_keeps_minor_above_threshold() {
        // Zoom in: height_world = 5, window_h = 600 → 120 px/m.
        // Minor spacing 1m × 120 = 120 px, well above the 6px LOD.
        let view = GridView {
            camera_height_world: 5.0,
            ..view_unit()
        };
        let counts = count_visible_lines(&view, &GridConfig::default());
        assert!(counts.minor_visible);
    }

    #[test]
    fn empty_canvas_paint_is_safe_noop() {
        // Pathological dims shouldn't panic; paint_grid early-outs.
        // This test only ensures the function doesn't crash; we
        // can't easily verify "scene unchanged" without a real
        // VectorScene inspector.
        let view = GridView {
            canvas: Rect::new(0.0, 0.0, 0.0, 0.0),
            ..view_unit()
        };
        let mut scene = VectorScene::new();
        paint_grid(&mut scene, Theme::Forge, &view, &GridConfig::default());
    }

    #[test]
    fn skip_multiples_of_filters_majors() {
        // World coord 5.0 is a major (multiple of 5); should_skip
        // with major=5 returns true. World coord 1.0 (not multiple
        // of 5) returns false.
        assert!(should_skip(5.0, Some(5.0)));
        assert!(should_skip(-5.0, Some(5.0)));
        assert!(should_skip(0.0, Some(5.0))); // origin is a major
        assert!(!should_skip(1.0, Some(5.0)));
        assert!(!should_skip(2.5, Some(5.0)));
        // No filter → no skip.
        assert!(!should_skip(5.0, None));
    }

    #[test]
    fn paint_axis_only_when_origin_visible() {
        // Move camera so x=0 is outside the visible range. The
        // paint_grid call should still succeed (no panic) and the
        // counts function ignores axes anyway. This is a smoke test.
        let view = GridView {
            camera_center: [100.0, 100.0],
            ..view_unit()
        };
        let mut scene = VectorScene::new();
        paint_grid(&mut scene, Theme::Forge, &view, &GridConfig::default());
    }
}
