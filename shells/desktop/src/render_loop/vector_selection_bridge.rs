//! Vector selection overlay — per-frame highlight of the shared
//! [`VectorSelection`] over the committed vector scene + the live marquee.
//!
//! Free function called once per frame BEFORE `paint_hero_screen` (mirror of
//! the other vector bridges). Pure feedback — it never mutates the scene:
//! Direct-Select vertex/handle affordances + the in-progress Select-tool
//! marquee rectangle. The object-selection box itself is the transform gizmo
//! (ADR-0076); this overlay only adds the *vertex-level* editing affordances.
//!
//! **Visual language (Illustrator/Affinity/Figma convention).** Every glyph is
//! projected to SCREEN space and drawn at a constant pixel size (`Affine::
//! IDENTITY` + pre-multiplied point), so affordances never distort with a
//! gizmo-scaled/rotated shape or with zoom. Anchors are shaped by their
//! [`VertexKind`] — round = smooth ([`VertexKind::Mirror`]/[`Aligned`]),
//! square = corner ([`VertexKind::Free`]), diamond = renderer-chosen
//! ([`VertexKind::Auto`]). State is shown by colour + fill:
//!
//! | State        | Anchor                       | Tangent handle              |
//! |--------------|------------------------------|-----------------------------|
//! | idle ("rest")| hollow ring, `BorderStrong`  | thin `BorderStrong` line    |
//! | selected     | filled `Accent`              | (anchor of a grabbed handle)|
//! | **grabbed**  | filled `AccentHover` + ring  | thick `Accent` line + dot   |
//!
//! The grabbed handle/anchor is the saturated accent; everything else is the
//! neutral border colour — a real hue/chroma contrast that survives every
//! theme (the accent stack is monochromatic per theme). This satisfies the
//! "selected handle a different colour from the rest" request directly.
//!
//! All colours/strokes/sizes come from [`ph2d_tokens`] (HR-15: zero hex, zero
//! UI `f32` literal). Reads the shell-owned [`VectorSelection`] + committed
//! scene + the active tool (downcast to [`VectorDirectTool`]/[`VectorSelectTool`]
//! by the imported short name, off the central-dispatch downcast gate). Wired by
//! `render_loop/mod.rs` after the create-tool bridges.

use ph2d_core::Vec2;
use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_tool_vector_direct::{DirectGrab, GrabTarget, VectorDirectTool};
use ph2d_tool_vector_select::VectorSelectTool;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Scene, Stroke, VectorScene};
use ph2d_vector_doc::{Ph2dVectorAsset, TangentSide, VectorSelection, VertexKind};

/// Resolve a design-token colour for the active theme into a Vello [`Color`]
/// at full opacity (`from_oklch` tokens carry `a = 255`; we draw crisp
/// affordances, so we override any soft token alpha).
fn solid(tok: ColorToken, theme: Theme) -> Color {
    let c = tok.resolve(theme);
    Color::from_rgba8(c.r, c.g, c.b, 255)
}

/// Resolve a token to a Vello [`Color`] keeping an explicit alpha (marquee tint).
fn tint(tok: ColorToken, theme: Theme, alpha: u8) -> Color {
    let c = tok.resolve(theme);
    Color::from_rgba8(c.r, c.g, c.b, alpha)
}

/// Per-frame selection overlay dispatch. Safe to call every frame.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    tools: &mut ToolRegistry,
    theme: Theme,
    camera: &Camera2d,
    window_size: WindowSize,
    committed: &[Ph2dVectorAsset],
    placements: &[[f32; 6]],
    selection: &VectorSelection,
    vector_scene: &mut VectorScene,
) {
    let world_to_screen = camera.world_to_screen_affine(window_size);
    let scene = vector_scene.inner_mut();

    // Token palette (resolved once per frame).
    let rest = solid(ColorToken::BorderStrong, theme); // idle affordances
    let sel = solid(ColorToken::Accent, theme); // selected / grabbed handle
    let grab_fill = solid(ColorToken::AccentHover, theme); // grabbed anchor body
    let line_thin = StrokeToken::Thin.px() as f64;
    let line_thick = StrokeToken::Thick.px() as f64;
    let r_rest = Spacing::Xs.px() as f64; // 4 px idle anchor
    let r_active = Spacing::Sm.px() as f64; // 6 px selected/grabbed anchor
    let r_ctrl = Spacing::Xxs.px() as f64; // 2 px tangent control dot

    // ADR-0076: compose each asset's placement so the overlay tracks a gizmo-
    // moved shape instead of ghosting at its rest pose. Identity for un-moved
    // shapes. The returned affine maps a network-local point → SCREEN pixels.
    let placement_xf = |idx: usize| -> Affine {
        let local = placements.get(idx).map_or(Affine::IDENTITY, |m| {
            Affine::new([
                m[0] as f64,
                m[1] as f64,
                m[2] as f64,
                m[3] as f64,
                m[4] as f64,
                m[5] as f64,
            ])
        });
        world_to_screen * local
    };

    // Direct-Select active? + the live grab (which exact vertex/handle is being
    // dragged) — one borrow of the tool registry. Affordances are Direct-only:
    // when Direct isn't active the object gizmo owns selection feedback.
    let (direct_active, grab): (bool, Option<DirectGrab>) = match tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<VectorDirectTool>())
    {
        Some(t) => (true, t.grab()),
        None => (false, None),
    };

    if direct_active {
        for (idx, asset) in committed.iter().enumerate() {
            let xf = placement_xf(idx);
            let to_screen = |p: Vec2| xf * Point::new(p.x as f64, p.y as f64);

            // ── Tangent handles: line vertex→control + a control dot, per
            // non-zero tangent (straight segments have zero tangents → nothing,
            // mirror of `nearest_tangent_handle`'s skip). The grabbed handle is
            // the saturated accent (thick); the rest are the neutral border.
            for seg in &asset.network.segments {
                let (Some(start), Some(end)) = (
                    asset.network.vertices.iter().find(|v| v.id == seg.start),
                    asset.network.vertices.iter().find(|v| v.id == seg.end),
                ) else {
                    continue;
                };
                for (base, tan, side) in [
                    (start.pos, seg.out_at_start, TangentSide::OutAtStart),
                    (end.pos, seg.in_at_end, TangentSide::InAtEnd),
                ] {
                    if tan.length_squared() < 1e-6 {
                        continue;
                    }
                    let grabbed = grab
                        == Some(DirectGrab {
                            asset: idx,
                            target: GrabTarget::Tangent(seg.id, side),
                        });
                    let (col, w, dot_r) = if grabbed {
                        (sel, line_thick, r_rest)
                    } else {
                        (rest, line_thin, r_ctrl)
                    };
                    let p0 = to_screen(base);
                    let p1 = to_screen(base + tan);
                    let mut line = BezPath::new();
                    line.move_to(p0);
                    line.line_to(p1);
                    scene.stroke(
                        &Stroke::new(w),
                        Affine::IDENTITY,
                        &Brush::Solid(col),
                        None,
                        &line,
                    );
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(col),
                        None,
                        &Circle::new(p1, dot_r),
                    );
                }
            }

            // ── Anchors (grab targets). Shape = VertexKind; state = colour/fill.
            for v in &asset.network.vertices {
                let center = to_screen(v.pos);
                let grabbed = grab
                    == Some(DirectGrab {
                        asset: idx,
                        target: GrabTarget::Vertex(v.id),
                    });
                let selected = selection.vertices.contains(&(idx, v.id));
                if grabbed {
                    // Brightest body + an accent ring so the active point pops.
                    paint_anchor(
                        scene,
                        center,
                        v.kind,
                        r_active,
                        Some(&Brush::Solid(grab_fill)),
                        Some((&Brush::Solid(sel), line_thin)),
                    );
                } else if selected {
                    paint_anchor(
                        scene,
                        center,
                        v.kind,
                        r_active,
                        Some(&Brush::Solid(sel)),
                        None,
                    );
                } else {
                    // Idle: hollow neutral ring (low clutter across many points).
                    paint_anchor(
                        scene,
                        center,
                        v.kind,
                        r_rest,
                        None,
                        Some((&Brush::Solid(rest), line_thin)),
                    );
                }
            }
        }
    }

    // ── Live marquee rect (only while the Select tool is dragging). Projected
    // to screen + drawn at a constant 1 px border, accent-tinted fill.
    let marquee = tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<VectorSelectTool>())
        .and_then(|t| t.marquee_rect());
    if let Some((min, max)) = marquee {
        let a = world_to_screen * Point::new(min.x as f64, min.y as f64);
        let b = world_to_screen * Point::new(max.x as f64, max.y as f64);
        let mut rect = BezPath::new();
        rect_path(&mut rect, a, b);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(tint(ColorToken::Accent, theme, 40)),
            None,
            &rect,
        );
        scene.stroke(
            &Stroke::new(line_thin),
            Affine::IDENTITY,
            &Brush::Solid(tint(ColorToken::Accent, theme, 180)),
            None,
            &rect,
        );
    }
}

/// Paint one anchor glyph at screen `center`: round (smooth), square (corner)
/// or diamond (auto) per [`VertexKind`], with an optional filled `body` and an
/// optional `ring` stroke `(brush, width)`. Constant-px (caller pre-projects).
fn paint_anchor(
    scene: &mut Scene,
    center: Point,
    kind: VertexKind,
    r: f64,
    body: Option<&Brush>,
    ring: Option<(&Brush, f64)>,
) {
    match kind {
        VertexKind::Mirror | VertexKind::Aligned => {
            let c = Circle::new(center, r);
            if let Some(b) = body {
                scene.fill(Fill::NonZero, Affine::IDENTITY, b, None, &c);
            }
            if let Some((b, w)) = ring {
                scene.stroke(&Stroke::new(w), Affine::IDENTITY, b, None, &c);
            }
        }
        VertexKind::Free => paint_path(scene, &square_path(center, r), body, ring),
        VertexKind::Auto => paint_path(scene, &diamond_path(center, r), body, ring),
    }
}

/// Fill + optionally stroke a closed [`BezPath`] glyph (square / diamond).
fn paint_path(
    scene: &mut Scene,
    path: &BezPath,
    body: Option<&Brush>,
    ring: Option<(&Brush, f64)>,
) {
    if let Some(b) = body {
        scene.fill(Fill::NonZero, Affine::IDENTITY, b, None, path);
    }
    if let Some((b, w)) = ring {
        scene.stroke(&Stroke::new(w), Affine::IDENTITY, b, None, path);
    }
}

/// Axis-aligned square of half-extent `h`, centred at `c` (corner anchor).
fn square_path(c: Point, h: f64) -> BezPath {
    let mut p = BezPath::new();
    p.move_to(Point::new(c.x - h, c.y - h));
    p.line_to(Point::new(c.x + h, c.y - h));
    p.line_to(Point::new(c.x + h, c.y + h));
    p.line_to(Point::new(c.x - h, c.y + h));
    p.close_path();
    p
}

/// 45°-rotated square (diamond) of half-extent `h`, centred at `c` (auto anchor).
fn diamond_path(c: Point, h: f64) -> BezPath {
    let mut p = BezPath::new();
    p.move_to(Point::new(c.x, c.y - h));
    p.line_to(Point::new(c.x + h, c.y));
    p.line_to(Point::new(c.x, c.y + h));
    p.line_to(Point::new(c.x - h, c.y));
    p.close_path();
    p
}

/// Build a closed axis-aligned rectangle path between screen corners `a`/`b`.
fn rect_path(path: &mut BezPath, a: Point, b: Point) {
    let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
    let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
    path.move_to(Point::new(x0, y0));
    path.line_to(Point::new(x1, y0));
    path.line_to(Point::new(x1, y1));
    path.line_to(Point::new(x0, y1));
    path.close_path();
}
