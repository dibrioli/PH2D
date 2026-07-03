//! Shared **Sprite-gizmo-style** painting for the Painter's on-canvas transform gizmos (the Curve
//! whole-curve gizmo + the Stencil rect), so they read EXACTLY like the editor's Sprite transform gizmo
//! — a theme `Selection` box outline + `Accent` rounded-square handles with a `BorderEmph` outline, the
//! corners flipping to circles as the rotate cue — only a touch DARKER than the Sprite gizmo (Enio
//! 2026-06-28). Pure draw; resolves the active theme's tokens (like `vector_selection_bridge`).

use ph2d_tokens::{ColorToken, Theme};
use ph2d_tool_painter::TransformGizmo;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, RoundedRect, Scene, Stroke};

/// How much darker than the Sprite gizmo the painter gizmos read (per-channel RGB scale; `1.0` = same).
const DARKEN: f32 = 0.82;
/// Sprite-gizmo handle half-size in screen px (a 12 px square) + the rounded-corner radius.
const HANDLE_HALF: f64 = 6.0;
const HANDLE_RADIUS: f64 = 2.0;
/// Box-outline stroke width (the Sprite gizmo's bbox) + the handle-outline width.
const BOX_STROKE: f64 = 1.5;
const HANDLE_STROKE: f64 = 1.0;

/// The darkened Sprite-gizmo palette for the active theme: the `Selection` box + `Accent` handle fill +
/// `BorderEmph` handle outline, each scaled toward black by [`DARKEN`].
pub(super) struct GizmoPalette {
    frame: Color,
    fill: Color,
    stroke: Color,
}

/// Resolve a token for `theme` and darken it a touch (the only difference from the Sprite gizmo).
fn darkened(tok: ColorToken, theme: Theme) -> Color {
    let c = tok.resolve(theme);
    Color::from_rgba8(
        (f32::from(c.r) * DARKEN) as u8,
        (f32::from(c.g) * DARKEN) as u8,
        (f32::from(c.b) * DARKEN) as u8,
        255,
    )
}

/// Build the painter-gizmo palette for the active theme.
pub(super) fn palette(theme: Theme) -> GizmoPalette {
    GizmoPalette {
        frame: darkened(ColorToken::Selection, theme),
        fill: darkened(ColorToken::Accent, theme),
        stroke: darkened(ColorToken::BorderEmph, theme),
    }
}

/// Distinct **fluorescent** gizmo accent colours (mirrors the Mask overlay palette) — so multiple
/// simultaneous gizmos each read a unique colour. Cycle by index.
pub(super) const GIZMO_ACCENTS: [[u8; 3]; 4] = [
    [220, 255, 0],  // fluorescent yellow
    [255, 42, 160], // fluorescent pink
    [80, 255, 60],  // fluorescent green
    [255, 120, 0],  // fluorescent orange
];

/// A gizmo palette whose box outline + handle fill are a specific fluorescent `accent` (the handle outline
/// stays the themed `BorderEmph` for contrast) — for distinctly-coloured simultaneous gizmos.
pub(super) fn palette_accent(theme: Theme, accent: [u8; 3]) -> GizmoPalette {
    let c = Color::from_rgba8(accent[0], accent[1], accent[2], 255);
    GizmoPalette {
        frame: c,
        fill: c,
        stroke: darkened(ColorToken::BorderEmph, theme),
    }
}

/// Stroke the gizmo box outline through `pts` as a closed polygon — the Sprite gizmo's bbox style.
pub(super) fn stroke_box(scene: &mut Scene, pts: &[Point], pal: &GizmoPalette) {
    let Some((&first, rest)) = pts.split_first() else {
        return;
    };
    let mut path = BezPath::new();
    path.move_to(first);
    for &p in rest {
        path.line_to(p);
    }
    path.close_path();
    scene.stroke(
        &Stroke::new(BOX_STROKE),
        Affine::IDENTITY,
        &Brush::Solid(pal.frame),
        None,
        &path,
    );
}

/// Stroke an OPEN polyline (a shape spine / segment guide) in the gizmo frame colour — the themed twin of a
/// literal "guide" stroke, so the shape overlays' guide lines match the Sprite-gizmo box.
pub(super) fn stroke_open(scene: &mut Scene, pts: &[Point], pal: &GizmoPalette) {
    let Some((&first, rest)) = pts.split_first() else {
        return;
    };
    let mut path = BezPath::new();
    path.move_to(first);
    for &p in rest {
        path.line_to(p);
    }
    scene.stroke(
        &Stroke::new(BOX_STROKE),
        Affine::IDENTITY,
        &Brush::Solid(pal.frame),
        None,
        &path,
    );
}

/// A scale / move handle — a rounded square (Accent fill + BorderEmph 1 px outline).
pub(super) fn square_handle(scene: &mut Scene, p: Point, pal: &GizmoPalette) {
    let r = RoundedRect::new(
        p.x - HANDLE_HALF,
        p.y - HANDLE_HALF,
        p.x + HANDLE_HALF,
        p.y + HANDLE_HALF,
        HANDLE_RADIUS,
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(pal.fill),
        None,
        &r,
    );
    scene.stroke(
        &Stroke::new(HANDLE_STROKE),
        Affine::IDENTITY,
        &Brush::Solid(pal.stroke),
        None,
        &r,
    );
}

/// Draw a whole-shape **transform gizmo** (the Sprite-gizmo box + 9 handles) for a shape editor — shared
/// by the Curve and Line overlays so they read IDENTICALLY. `affine` maps image px → screen; `cursor` is
/// the screen cursor, used for the rotate-ring hover cue (corners flip to circles when hovering the band
/// just outside them, matching the tool's hit-test). Drawn UNDER the shape's spine + dots by the caller.
pub(super) fn draw_transform_gizmo(
    scene: &mut Scene,
    gz: &TransformGizmo,
    affine: Affine,
    pal: &GizmoPalette,
    cursor: (f32, f32),
) {
    let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
    let [mn, mx] = gz.bbox;
    let box_pts = [map(mn), map([mx[0], mn[1]]), map(mx), map([mn[0], mx[1]])];
    stroke_box(scene, &box_pts, pal);
    // Corners flip to circles on mouse-OVER the rotate ring (not only mid-drag) — the cue must match the
    // tool's hit-test: the band just OUTSIDE a corner (farther from the centre than it). Image-px tol →
    // screen via the affine's per-pixel scale.
    let scale = {
        let c = affine.as_coeffs();
        (c[0] * c[0] + c[1] * c[1]).sqrt()
    };
    let cur = Point::new(f64::from(cursor.0), f64::from(cursor.1));
    let center_sp = map(gz.handles[8]);
    let inner = f64::from(gz.scale_tol_px) * scale;
    let outer = f64::from(gz.rotate_tol_px) * scale;
    let over_rotate = gz.handles[..4].iter().any(|&h| {
        let sp = map(h);
        let d = sp.distance(cur);
        d > inner && d <= outer && cur.distance(center_sp) > sp.distance(center_sp)
    });
    let circle_corners = gz.rotating || over_rotate;
    for (i, &h) in gz.handles.iter().enumerate() {
        let p = map(h);
        if circle_corners && i < 4 {
            circle_handle(scene, p, pal);
        } else {
            square_handle(scene, p, pal);
        }
    }
}

/// A **sides** handle — a diamond (rotated square: Accent fill + BorderEmph 1 px outline), so the polygon's
/// side-count handle reads distinctly from the round rotate handle.
pub(super) fn diamond_handle(scene: &mut Scene, p: Point, pal: &GizmoPalette) {
    let mut path = BezPath::new();
    path.move_to(Point::new(p.x, p.y - HANDLE_HALF)); // top
    path.line_to(Point::new(p.x + HANDLE_HALF, p.y)); // right
    path.line_to(Point::new(p.x, p.y + HANDLE_HALF)); // bottom
    path.line_to(Point::new(p.x - HANDLE_HALF, p.y)); // left
    path.close_path();
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(pal.fill),
        None,
        &path,
    );
    scene.stroke(
        &Stroke::new(HANDLE_STROKE),
        Affine::IDENTITY,
        &Brush::Solid(pal.stroke),
        None,
        &path,
    );
}

/// A rotate-cue handle — a circle (Accent fill + BorderEmph 1 px outline), same radius as the square.
pub(super) fn circle_handle(scene: &mut Scene, p: Point, pal: &GizmoPalette) {
    let c = Circle::new(p, HANDLE_HALF);
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(pal.fill),
        None,
        &c,
    );
    scene.stroke(
        &Stroke::new(HANDLE_STROKE),
        Affine::IDENTITY,
        &Brush::Solid(pal.stroke),
        None,
        &c,
    );
}
