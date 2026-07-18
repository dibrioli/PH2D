//! **A pintura de UM strip** — a caixa, as áreas de fade, os gizmos de canto e o nome.
//!
//! Mora fora do `stack_lane_paint` pelo cap de 600 LOC dos painéis, e porque é uma unidade: tudo
//! aqui responde "como este retângulo se explica sozinho ao artista".
//!
//! **Cada operação da strip é um gizmo próprio** (Enio, 2026-07-16): os cantos SUPERIORES são
//! VERDES (esticar/retime), os INFERIORES são VERMELHOS (cortar/trim), e a alça de fade é um
//! traço com duas setinhas. E cada uma se ANUNCIA: a área de fade é listrada, a área cortada é
//! vermelha cruzada (durante o arraste), e o fator de esticamento aparece na label.

use std::borrow::Cow;

use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::text_elide::paint_text_elided;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, BezPath, Color, Fill, Stroke};

use crate::graph::TimeView;
use crate::stack_ease_grip::{EASE_BAR_W, blend_px, strip_grips};
use crate::stack_lane_paint::EDGE_W;

/// Below this, a rate is real time and the strip says nothing about it.
const SPEED_EPS: f64 = 1e-6; // LITERAL-PX-OK: not a length — a "is it exactly 1" epsilon
/// How far each arm of a corner bracket runs along its edge — the grab region's own width
/// (`hit_plan` registers `EDGE_W`), so the quina traces what is actually clickable.
const CORNER_ARM: f32 = EDGE_W;
/// Thickness of a corner bracket's arms.
const CORNER_W: f32 = 2.0; // LITERAL-PX-OK: a bracket stroke, like the fade bar's
/// Spacing between hatch lines (the fade stripes / the cut cross-hatch).
pub(crate) const HATCH_STEP: f32 = 5.0; // LITERAL-PX-OK: hatch line pitch
/// Half-size of a fade-handle arrowhead.
const ARROW_SZ: f32 = 3.0; // LITERAL-PX-OK: fade arrowhead half-size

/// The four corners as `(stretch, edge)` — GREEN top pair retimes, RED bottom pair
/// trims; `edge` is the document's `0` = start, `1` = end.
///
/// ONE list, walked by both the bracket and its change bar, so a corner cannot be
/// drawn in one place and forgotten in the other.
const CORNERS: [(bool, u8); 4] = [(true, 0), (true, 1), (false, 0), (false, 1)];

/// A corner's colour. The operation IS the colour, at the gizmo and at its bar.
const fn corner_token(stretch: bool) -> ColorToken {
    if stretch {
        ColorToken::Success
    } else {
        ColorToken::Danger
    }
}

/// The bar one corner's **change mark** occupies — `None` where that corner has not
/// been edited, or where its edit rounds away at this zoom.
///
/// The span runs from the edge to `edge + mark`, and **which side of the edge it
/// lands on is the sign's doing, not a second rule to keep in step**: pull a start
/// edge outward and the time it gained is now INSIDE the strip; push it inward and
/// the time it lost is OUTSIDE. One expression, both edges, both operations.
///
/// Pure, and separate from the painting, so the geometry can be stated in a test
/// without a scene — the `hit_plan` pattern.
pub(crate) fn mark_bar(
    s: &ph2d_timeline::StripView,
    view: TimeView,
    body: Rect,
    stretch: bool,
    edge: u8,
) -> Option<Rect> {
    let t_edge = if edge == 0 { s.t_start } else { s.t_end };
    let d = s.marks[ph2d_timeline::mark_index(stretch, edge)];
    let (a, b) = (view.x(t_edge), view.x(t_edge + d));
    let (x0, x1) = if a <= b { (a, b) } else { (b, a) };
    // The bar rides the edge its corner hugs: the top for a stretch, the bottom for
    // a trim — the same band the bracket's arm runs along.
    let y = if stretch {
        body.y
    } else {
        body.y + body.h - CORNER_W
    };
    (x1 > x0).then(|| Rect::new(x0, y, x1 - x0, CORNER_W))
}

/// One strip: its box, its name, the fade areas, and the four corner gizmos.
pub(crate) fn paint_strip(
    ctx: &mut PaintCtx,
    theme: Theme,
    view: TimeView,
    body: Rect,
    s: &ph2d_timeline::StripView,
    dim: bool,
) {
    fill_rounded_rect(
        ctx.scene,
        body,
        Radius::Sm.px(),
        resolve(
            if dim {
                ColorToken::Bg3
            } else {
                ColorToken::TimelineKey
            },
            theme,
        ),
    );
    stroke_rounded_rect(
        ctx.scene,
        body,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::BorderStrong, theme),
    );

    // **The fade areas are LISTRADAS (gray-striped)** so the crossfade reads at a glance
    // (Enio, 2026-07-16). A window at an end means the strip is fading there — into its
    // neighbour if it has one (the overlap), out of nothing if it does not.
    for (t_from, secs) in [
        (s.t_start, s.blend_in),
        (s.t_end - s.blend_out, s.blend_out),
    ] {
        if secs <= 0.0 {
            continue;
        }
        let a = view.x(t_from).max(body.x);
        let b = view.x(t_from + secs).min(body.x + body.w);
        if b > a {
            paint_fade_region(ctx, theme, Rect::new(a, body.y, b - a, body.h));
        }
    }

    // **The OUTWARD fade-in area — the travel band in the gap BEFORE the strip.** It sits
    // to the LEFT of the box (over empty lane time), striped like any fade. It reads as
    // "this clip's entry reaches back here": the object crosses from the previous strip's
    // pose to this clip's first frame over this band, then the clip plays clean.
    if s.lead_in > 0.0 {
        let a = view.x(s.t_start - s.lead_in);
        let b = view.x(s.t_start);
        if b > a {
            paint_fade_region(ctx, theme, Rect::new(a, body.y, b - a, body.h));
        }
    }

    // **The change bars** — what each corner's last edit DID, still on screen long
    // after the pointer let go (Enio, 2026-07-16). Drawn under the brackets, in the
    // corner's own colour and on the corner's own edge, so a bar reads as that
    // gizmo's arm grown to the length of the change it made.
    for (stretch, edge) in CORNERS {
        if let Some(bar) = mark_bar(s, view, body, stretch, edge) {
            fill_rounded_rect(
                ctx.scene,
                bar,
                Radius::Xs.px(),
                resolve(corner_token(stretch), theme),
            );
        }
    }

    // **The four corner gizmos** — GREEN top = stretch (retime), RED bottom = trim (cut).
    // The colour IS the operation, so the artist never wonders which corner does what.
    for (stretch, edge) in CORNERS {
        paint_corner(
            ctx,
            body,
            edge == 0,
            stretch,
            resolve(corner_token(stretch), theme),
        );
    }

    // **The fade handle: a traço with two little arrows** (Enio, 2026-07-16) — a vertical
    // line at the wedge tip plus a ◀ ▶ pair, so it reads as "drag me either way to fade".
    // GRAY where a neighbour defines the window (the overlap IS the fade; the way to change
    // it is to move the strips) and NOT registered — a dimmed control that dispatches lies.
    if let Some((a, b)) = strip_grips(
        body,
        blend_px(view, s.t_start, s.lead_in),
        blend_px(view, s.t_start, s.blend_in),
        blend_px(view, s.t_start, s.blend_out),
    ) {
        for (g, locked, at_left) in [(a, s.ease_locked_in, true), (b, s.ease_locked_out, false)] {
            let color = resolve(
                if locked || dim {
                    ColorToken::Border
                } else {
                    ColorToken::TimelinePlayhead
                },
                theme,
            );
            let x = if at_left { g.x } else { g.x + g.w - EASE_BAR_W };
            paint_fade_handle(ctx, x + EASE_BAR_W * 0.5, g.y, g.h, color);
        }
    }

    // The clip name, centred. A STRETCHED strip does not change how it looks, so the only
    // sign it was retimed is its factor in parentheses — `Main (2x)` / `Main (0.5x)` (Enio,
    // 2026-07-16). The middle is the only band the corner gizmos and fade handles do not
    // contest, which is why the label lives there.
    let label: Cow<'_, str> = if (s.speed - 1.0).abs() > SPEED_EPS {
        Cow::Owned(format!("{} ({})", s.clip_name, factor_str(s.speed)))
    } else {
        Cow::Borrowed(s.clip_name.as_str())
    };
    let font = TypeToken::Sm.px();
    let budget = (body.w - Spacing::Xs.px() * 2.0).max(0.0);
    let text_w = ctx.text_system.prefix_width(&label, font).min(budget);
    // **ACCENT, e NÃO `Text1`.** O corpo do strip é pintado com `TimelineKey`, que é o MESMO valor
    // de `Text1` em todos os temas (o token da chave é de PRIMEIRO PLANO). Escrever `Text1` ali é
    // escrever branco no branco: o nome só aparecia onde a cunha do fade escurecia o fundo (Enio,
    // smoke). O accent é o único token de meia-luminosidade E saturado: destaca-se dos dois
    // extremos, nos dois temas.
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &label,
        body.x + (body.w - text_w) * 0.5,
        body.y + (body.h - font) * 0.5,
        font,
        budget,
        resolve(ColorToken::Accent, theme),
    );
}

/// A corner gizmo: an **L-shaped bracket** hugging the two edges that meet at this corner —
/// a QUINA, not a dot (Enio, 2026-07-16). One arm runs along the top/bottom edge, the other
/// down/up the left/right edge, both from the corner itself, so the mark reads as the corner
/// you grab rather than a bead sitting near it.
///
/// The horizontal arm is the grab region's own width (`EDGE_W`, what `hit_plan` registers),
/// so what is drawn and what is clickable start at the same pixel.
fn paint_corner(ctx: &mut PaintCtx, body: Rect, left: bool, top: bool, color: Color) {
    let arm = CORNER_ARM.min(body.w * 0.5).min(body.h * 0.5);
    let w = CORNER_W.min(arm);
    let (x_out, x_edge) = if left {
        (body.x, body.x)
    } else {
        (body.x + body.w - arm, body.x + body.w - w)
    };
    let (y_edge, y_out) = if top {
        (body.y, body.y)
    } else {
        (body.y + body.h - w, body.y + body.h - arm)
    };
    // The arm along the top/bottom edge, then the one down/up the side edge.
    fill_rounded_rect(
        ctx.scene,
        Rect::new(x_out, y_edge, arm, w),
        Radius::Xs.px(),
        color,
    );
    fill_rounded_rect(
        ctx.scene,
        Rect::new(x_edge, y_out, w, arm),
        Radius::Xs.px(),
        color,
    );
}

/// Paint a fade area: a faint darkening + gray diagonal STRIPES, so it reads as "a fade
/// lives here" at a glance.
fn paint_fade_region(ctx: &mut PaintCtx, theme: Theme, rect: Rect) {
    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Xs.px(),
        resolve(ColorToken::Bg3, theme),
    );
    paint_hatch(ctx, rect, resolve(ColorToken::Border, theme), false);
}

/// Fill `rect` with diagonal hatch lines. `crossed` draws BOTH diagonals (an X, for the
/// cut region); otherwise a single `/` set (the fade stripes). Clipped to `rect`, one
/// stroke call. Public so the trim preview (`stack_lane_paint`) draws its cut-hatch the
/// same way.
pub(crate) fn paint_hatch(ctx: &mut PaintCtx, rect: Rect, color: Color, crossed: bool) {
    if rect.w <= 0.5 || rect.h <= 0.5 {
        return;
    }
    ctx.scene.push_clip(&rect_to_vello(rect));
    let mut p = BezPath::new();
    let h = f64::from(rect.h);
    let step = f64::from(HATCH_STEP);
    let (x0, x1, y0, y1) = (
        f64::from(rect.x),
        f64::from(rect.x + rect.w),
        f64::from(rect.y),
        f64::from(rect.y + rect.h),
    );
    // `/` lines: bottom-left to top-right.
    let mut x = x0 - h;
    while x < x1 {
        p.move_to((x, y1));
        p.line_to((x + h, y0));
        x += step;
    }
    if crossed {
        // `\` lines: top-left to bottom-right.
        let mut x = x0 - h;
        while x < x1 {
            p.move_to((x, y0));
            p.line_to((x + h, y1));
            x += step;
        }
    }
    ctx.scene.inner_mut().stroke(
        &Stroke::new(f64::from(StrokeToken::Thin.px())),
        Affine::IDENTITY,
        color,
        None,
        &p,
    );
    ctx.scene.pop_layer();
}

/// The fade handle: a vertical line centred at `xc` plus a ◀ ▶ arrow pair at mid-height.
fn paint_fade_handle(ctx: &mut PaintCtx, xc: f32, y: f32, h: f32, color: Color) {
    fill_rounded_rect(
        ctx.scene,
        Rect::new(xc - EASE_BAR_W * 0.5, y, EASE_BAR_W, h),
        Radius::Xs.px(),
        color,
    );
    let my = y + h * 0.5;
    let gap = EASE_BAR_W * 0.5 + 1.0; // LITERAL-PX-OK: clear of the line
    // Left arrow (points left) and right arrow (points right).
    fill_tri(
        ctx,
        [
            (xc - gap - ARROW_SZ, my),
            (xc - gap, my - ARROW_SZ),
            (xc - gap, my + ARROW_SZ),
        ],
        color,
    );
    fill_tri(
        ctx,
        [
            (xc + gap + ARROW_SZ, my),
            (xc + gap, my - ARROW_SZ),
            (xc + gap, my + ARROW_SZ),
        ],
        color,
    );
}

/// Fill a triangle through three points.
fn fill_tri(ctx: &mut PaintCtx, pts: [(f32, f32); 3], color: Color) {
    let mut p = BezPath::new();
    p.move_to((f64::from(pts[0].0), f64::from(pts[0].1)));
    p.line_to((f64::from(pts[1].0), f64::from(pts[1].1)));
    p.line_to((f64::from(pts[2].0), f64::from(pts[2].1)));
    p.close_path();
    ctx.scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &p);
}

/// `2x`, `0.5x`, `1.5x` — the stretch factor, trailing zeros trimmed.
fn factor_str(speed: f64) -> String {
    let s = format!("{speed:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    format!("{s}x")
}

#[cfg(test)]
#[path = "strip_paint_tests.rs"]
mod tests;
