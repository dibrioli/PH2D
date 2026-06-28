//! Shared **Sprite-gizmo-style** painting for the Painter's on-canvas transform gizmos (the Curve
//! whole-curve gizmo + the Stencil rect), so they read EXACTLY like the editor's Sprite transform gizmo
//! — a theme `Selection` box outline + `Accent` rounded-square handles with a `BorderEmph` outline, the
//! corners flipping to circles as the rotate cue — only a touch DARKER than the Sprite gizmo (Enio
//! 2026-06-28). Pure draw; resolves the active theme's tokens (like `vector_selection_bridge`).

use ph2d_tokens::{ColorToken, Theme};
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
