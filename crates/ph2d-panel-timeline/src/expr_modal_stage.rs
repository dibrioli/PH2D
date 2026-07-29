//! **The Expression STAGE** — the ghost, on a card of its own beside the modal.
//!
//! ```text
//!   ┌──────────────── the card ────────────────┐   ┌──── the stage ────┐
//!   │ Expression — Position X              [x] │   │ Stage · 4 m       │
//!   │ gallery │ sheet                          │   │   ┆   ┆   ┆   ┆   │
//!   │         │ Shake   Speed [2.0] Amt [0.3]  │   │   ┆  (ᴖ_ᴖ) ┆   ┆  │
//!   │ ─────── the wave, the whole card wide ── │   │   ┆   ┆   ┆   ┆   │
//!   │ fx  value + wiggle(2, 0.3)               │   │  0.30 m · 30 px   │
//!   └──────────────────────────────────────────┘   └───────────────────┘
//! ```
//!
//! ⚠️ **Outside the card, and that is the point** (Enio, smoke de 2026-07-29:
//! *"vamos nos desfazer do preview do objeto (sphere) no painel e vamos colocar o
//! preview fora do painel, ao lado direito do painel e vinculado a ele (pode ser
//! arrastado) usando a métrica real do canvas. Assim fica mais fácil visualizar o
//! efeito real no canvas"*). Inside the card it was a 190 px column and had to
//! invent its own scale; out here it can be a **metric window** — a fixed span of
//! world, drawn at the project's own `pixels_per_meter`, so *"is 0.3 a nudge or a
//! flight off the canvas?"* is answered by looking rather than by arithmetic.
//!
//! ⚠️ **LINKED, and by ONE number.** The stage's position is an OFFSET from the
//! card's top-left ([`ExprModal::stage_offset`]), so dragging the card carries the
//! stage with it for free and dragging the stage re-authors the offset. Two absolute
//! positions kept "in sync" would be two answers to where the pair is, and they
//! would drift the first time the viewport clamp moved one of them.
//!
//! ⚠️ **The ghost never evaluates anything.** It indexes the SAME sample vector the
//! wave strip plots, at the same phase (`expr_modal_preview::sample_at`). Two cards
//! showing the same instant of two different evaluations is precisely the kind of
//! disagreement nobody can see and everybody trusts.

use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, TimelineHitKind,
};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text, resolve, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::paint_shapes::fill_polygon;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::PropKind;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::expr_modal::ExprModal;
use crate::expr_modal_preview::sample_at;
use crate::ids;
use crate::state::TimelinePanelState;

/// Stage card width.
pub const STAGE_W: f32 = 260.0; // LITERAL-PX-OK: largura do card do palco
/// Rows the metric window is tall (the title and the readout take one each).
const WINDOW_ROWS: f32 = 7.0; // LITERAL-PX-OK: CONTAGEM de linhas da janela, nao medida

/// How much WORLD the window shows, across.
///
/// ⚠️ Fixed, not auto-fitted, and that is the whole value of it: a window that
/// resized itself to the motion would make every amount look the same size, which
/// is exactly the complaint this stage exists to answer. Four metres is character
/// scale — 400 px at the project default of 100 px/m, a tenth of a 4K canvas — so a
/// 0.3 m shake reads as a shake and a 12 m throw visibly leaves.
const SPAN_M: f32 = 4.0; // LITERAL-PX-OK: metros de mundo que a janela mostra

/// Grid spacing, in metres. One line per metre is a ruler you can count.
const GRID_M: f32 = 1.0; // LITERAL-PX-OK: metros por linha de grade

/// The ghost's height, in metres — about a small character.
const GHOST_M: f32 = 0.55; // LITERAL-PX-OK: altura do fantasma em metros de mundo

/// Half, for centring.
const HALF: f32 = 0.5; // LITERAL-PX-OK: aritmetica de centralizacao, nao medida

/// Total stage height.
#[must_use]
pub fn stage_h() -> f32 {
    Spacing::Sm.px() * 2.0 + ROW_H_PX * (WINDOW_ROWS + 2.0)
}

/// **Where the stage sits, given where the card sits.** The single door: the paint
/// places it, the hit test finds it and the gates read it from here.
#[must_use]
pub fn stage_rect(card: Rect, offset: (f32, f32)) -> Rect {
    Rect::new(card.x + offset.0, card.y + offset.1, STAGE_W, stage_h())
}

/// The offset a freshly opened card gives its stage: immediately to the right,
/// tops aligned.
#[must_use]
pub fn default_offset(card_w: f32) -> (f32, f32) {
    (card_w + Spacing::Sm.px(), 0.0)
}

/// What the number DRIVES about the ghost — chosen by the property.
///
/// ⚠️ One function, not a table. A second mapping would let the stage move a ghost
/// sideways for a property the wave strip is plotting as a turn, and the two cards
/// would disagree about what the artist is authoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drive {
    /// Slides along X (metres).
    SlideX,
    /// Slides along Y (metres).
    SlideY,
    /// Turns (radians).
    Turn,
    /// Stretches on X (a factor).
    WideNarrow,
    /// Stretches on Y (a factor).
    TallShort,
    /// Fades (`0..1`).
    Fade,
    /// Nothing about a pose — a normalised number with a bar under the ghost.
    Meter,
}

#[must_use]
pub fn drive_for(prop: PropKind) -> Drive {
    match prop {
        PropKind::TranslationX | PropKind::Position => Drive::SlideX,
        PropKind::TranslationY => Drive::SlideY,
        PropKind::Rotation => Drive::Turn,
        PropKind::ScaleX => Drive::WideNarrow,
        PropKind::ScaleY => Drive::TallShort,
        PropKind::Opacity => Drive::Fade,
        // Morph is a normalised `0..1`; TimeRemap is a clock, and a clock has no
        // pose — a bar under a still ghost is the honest figure for both.
        _ => Drive::Meter,
    }
}

/// Paint the stage. Returns nothing — like the card, it places itself.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    m: &ExprModal,
    ctx: &mut PaintCtx,
    theme: Theme,
    card: Rect,
    samples: &[f32],
    base: f32,
    px_per_m: f32,
) {
    let rect = stage_rect(card, m.stage_offset);
    let radius = Radius::Md.px();
    fill_rounded_rect(ctx.scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(
        ctx.scene,
        rect,
        radius,
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );

    let font = TypeToken::Sm.px();
    let pad = Spacing::Sm.px();
    let mut cy = rect.y + pad;

    // ── The title band doubles as the drag handle (the card's own pattern). ──
    ctx.host.store_mut().register(
        ids::EXPR_STAGE_HANDLE,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::ExprStageHandle,
            canvas: rect,
        },
    );
    let band = Rect::new(rect.x, rect.y, rect.w, ROW_H_PX + pad);
    ctx.host
        .hit_index_mut()
        .register(ids::EXPR_STAGE_HANDLE, band);
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!(
            "{}  ·  {SPAN_M} m",
            ph2d_i18n::tr("panel.timeline.expr_stage")
        ),
        rect.x + pad,
        cy + (ROW_H_PX - font) * HALF,
        font,
        rect.w - pad * 2.0,
        resolve(ColorToken::Text1, theme),
    );
    cy += ROW_H_PX;

    // ── The metric window. ──
    let win = Rect::new(rect.x + pad, cy, rect.w - pad * 2.0, ROW_H_PX * WINDOW_ROWS);
    // Pixels of SCREEN per metre of world, so the window shows exactly `SPAN_M`.
    let scale = win.w / SPAN_M;
    stroke_rounded_rect(
        ctx.scene,
        win,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_grid(ctx.scene, theme, win, scale);

    let value = sample_at(samples, m.preview_frame, base);
    let drive = drive_for(m.prop);
    paint_ghost(ctx, theme, win, scale, drive, value, base);

    cy += win.h + Spacing::Xs.px();

    // ── The readout: the number, in BOTH units. ──
    //
    // ⚠️ The pixels are the half the artist actually asked for. `0.3` means nothing
    // against a canvas until it says `30 px`, and that conversion is the project's,
    // not ours.
    let readout = match drive {
        Drive::SlideX | Drive::SlideY => {
            format!("{value:+.2} m  ·  {:+.0} px", value * px_per_m)
        }
        Drive::Turn => format!("{:+.1}°", value.to_degrees()),
        Drive::WideNarrow | Drive::TallShort => format!("x{value:.2}"),
        Drive::Fade | Drive::Meter => format!("{value:.2}"),
    };
    paint_text(
        ctx.text_system,
        ctx.scene,
        &readout,
        rect.x + pad,
        cy + (ROW_H_PX - font) * HALF,
        font,
        rect.w - pad * 2.0,
        resolve(ColorToken::Text2, theme),
    );
}

/// One line per metre, so the window is a ruler rather than a box.
fn paint_grid(scene: &mut VectorScene, theme: Theme, win: Rect, scale: f32) {
    let ink = resolve(ColorToken::Border, theme);
    let (cx, cy) = (win.x + win.w * HALF, win.y + win.h * HALF);
    let step = GRID_M * scale;
    let mut k = 1.0;
    while k * step < win.w * HALF {
        for x in [cx - k * step, cx + k * step] {
            stroke_polyline(
                scene,
                &[(x, win.y), (x, win.y + win.h)],
                StrokeToken::Thin.px(),
                ink,
            );
        }
        k += 1.0;
    }
    let mut k = 1.0;
    while k * step < win.h * HALF {
        for y in [cy - k * step, cy + k * step] {
            stroke_polyline(
                scene,
                &[(win.x, y), (win.x + win.w, y)],
                StrokeToken::Thin.px(),
                ink,
            );
        }
        k += 1.0;
    }
    // The centre cross — where the property rests.
    const CROSS_FRAC: f32 = 0.2; // LITERAL-PX-OK: fracao do passo da grade, nao medida
    let arm = step * CROSS_FRAC;
    stroke_polyline(
        scene,
        &[(cx - arm, cy), (cx + arm, cy)],
        StrokeToken::Thin.px(),
        resolve(ColorToken::Text2, theme),
    );
    stroke_polyline(
        scene,
        &[(cx, cy - arm), (cx, cy + arm)],
        StrokeToken::Thin.px(),
        resolve(ColorToken::Text2, theme),
    );
}

/// The ghost's outline, in a unit box: `x` and `y` in `[-0.5, 0.5]`, `y` DOWN.
///
/// A dome over a body with a scalloped hem — the figure everyone recognises, built
/// as ONE polygon so it can be turned and stretched by the same transform.
fn ghost_outline() -> Vec<(f32, f32)> {
    const DOME_STEPS: usize = 12; // LITERAL-PX-OK: CONTAGEM de amostras do domo
    const SCALLOPS: usize = 3; // LITERAL-PX-OK: CONTAGEM de bicos da barra
    const SHOULDER_Y: f32 = -0.1; // LITERAL-PX-OK: fracao do desenho, nao medida
    /// ⚠️ The dome is an ELLIPSE, not a half circle: at radius `HALF` its crown
    /// would reach `-0.6` and the figure would no longer fit the unit box it
    /// claims — which is exactly what `the_ghost_outline_is_a_fillable_figure`
    /// caught. `HALF + SHOULDER_Y` is the tallest crown that still fits.
    const DOME_RY: f32 = HALF + SHOULDER_Y; // LITERAL-PX-OK: fracao do desenho, nao medida
    const HEM_Y: f32 = 0.42; // LITERAL-PX-OK: fracao do desenho, nao medida
    const HEM_RISE: f32 = 0.1; // LITERAL-PX-OK: fracao do desenho, nao medida
    let mut p = Vec::with_capacity(DOME_STEPS + SCALLOPS * 2 + 4);
    // The dome: a half turn from the left shoulder over the top to the right.
    for i in 0..=DOME_STEPS {
        let a = core::f32::consts::PI * (1.0 + i as f32 / DOME_STEPS as f32);
        p.push((HALF * a.cos(), SHOULDER_Y + DOME_RY * a.sin()));
    }
    p.push((HALF, HEM_Y));
    // The hem, right to left, dipping between each scallop.
    for i in 0..SCALLOPS {
        let t0 = (i as f32 + HALF) / SCALLOPS as f32;
        let t1 = (i as f32 + 1.0) / SCALLOPS as f32;
        p.push((HALF - t0 * 1.0, HEM_Y - HEM_RISE));
        p.push((HALF - t1 * 1.0, HEM_Y));
    }
    p.push((-HALF, SHOULDER_Y));
    p
}

/// Draw the ghost with `value` driving whatever [`Drive`] says it drives.
fn paint_ghost(
    ctx: &mut PaintCtx,
    theme: Theme,
    win: Rect,
    scale: f32,
    drive: Drive,
    value: f32,
    base: f32,
) {
    /// A ghost never vanishes completely — an empty window reads as broken, not as
    /// transparent.
    const ALPHA_FLOOR: f32 = 0.08; // LITERAL-PX-OK: piso de alfa, nao medida
    /// Eye offsets and size, as fractions of the figure.
    const EYE_X: f32 = 0.17; // LITERAL-PX-OK: fracao do desenho, nao medida
    const EYE_Y: f32 = -0.14; // LITERAL-PX-OK: fracao do desenho, nao medida
    const EYE_R: f32 = 0.08; // LITERAL-PX-OK: fracao do desenho, nao medida
    /// The bar under a `Meter` ghost.
    const BAR_W_FRAC: f32 = 0.6; // LITERAL-PX-OK: fracao do desenho, nao medida
    const BAR_H_FRAC: f32 = 0.12; // LITERAL-PX-OK: fracao do desenho, nao medida

    let (cx, cy) = (win.x + win.w * HALF, win.y + win.h * HALF);
    let size = GHOST_M * scale;

    // What the value does to the figure. Everything not named here stays neutral,
    // which is what keeps a Rotation ghost from also sliding.
    let (mut dx, mut dy, mut turn, mut sx, mut sy, mut alpha) = (0.0, 0.0, 0.0, 1.0, 1.0, 1.0_f32);
    match drive {
        // ⚠️ Metres × the PROJECT's px/m — the whole reason the stage exists.
        Drive::SlideX => dx = value * scale,
        Drive::SlideY => dy = -value * scale,
        Drive::Turn => turn = value,
        Drive::WideNarrow => sx = value,
        Drive::TallShort => sy = value,
        Drive::Fade => alpha = value,
        Drive::Meter => {}
    }

    // ⚠️ CLAMPED to the window, with the ghost pinned to the wall it left. A figure
    // drawn outside its box is a figure the artist cannot see leaving — and "the
    // object disappears off the canvas" is the report this stage answers.
    let reach_x = win.w * HALF - size * HALF;
    let reach_y = win.h * HALF - size * HALF;
    let out = dx.abs() > reach_x || dy.abs() > reach_y;
    dx = dx.clamp(-reach_x, reach_x); // CLAMP-OK: bounds simetricos nao-NaN e ordenados
    dy = dy.clamp(-reach_y, reach_y); // CLAMP-OK: bounds simetricos nao-NaN e ordenados

    let ink = resolve(
        if out {
            ColorToken::Warn
        } else {
            ColorToken::TimelinePlayhead
        },
        theme,
    );
    let (s, c) = (turn.sin(), turn.cos());
    // ⚠️ Scale FIRST, then rotate: the other order shears a stretched figure, the
    // same law the Painter's dab frame pays.
    let place = |(ux, uy): (f32, f32)| {
        let (px, py) = (ux * size * sx, uy * size * sy);
        (cx + dx + px * c - py * s, cy + dy + px * s + py * c)
    };

    let body: Vec<(f32, f32)> = ghost_outline().into_iter().map(place).collect();
    let a = if matches!(drive, Drive::Fade) {
        alpha.clamp(ALPHA_FLOOR, 1.0) // CLAMP-OK: bounds literais nao-NaN e ordenados
    } else {
        1.0
    };
    fill_polygon(ctx.scene, &body, ink.multiply_alpha(a));
    let eye = resolve(ColorToken::BgElev, theme);
    for ex in [-EYE_X, EYE_X] {
        let (px, py) = place((ex, EYE_Y));
        fill_circle(ctx.scene, px, py, EYE_R * size, eye);
    }

    if matches!(drive, Drive::Meter) {
        // A clock and a morph have no pose, so the number gets a bar of its own
        // rather than a pretend one.
        let bw = win.w * BAR_W_FRAC;
        let bh = ROW_H_PX * BAR_H_FRAC;
        const BAR_DROP: f32 = 0.8; // LITERAL-PX-OK: fracao da figura, nao medida
        let by = cy + size * BAR_DROP;
        let track = Rect::new(cx - bw * HALF, by, bw, bh);
        stroke_rounded_rect(
            ctx.scene,
            track,
            Radius::Xs.px(),
            StrokeToken::Thin.px(),
            resolve(ColorToken::Border, theme),
        );
        let t = (value - base).abs().clamp(0.0, 1.0); // CLAMP-OK: bounds literais nao-NaN e ordenados
        fill_rounded_rect(
            ctx.scene,
            Rect::new(track.x, track.y, bw * t, bh),
            Radius::Xs.px(),
            ink,
        );
    }
}

/// Move the stage by its title band — and, unlike the card, what the drag writes is
/// an OFFSET, so the pair stays together afterwards.
pub(crate) fn apply_drag(state: &mut TimelinePanelState, g: TimelineGesture) {
    let Some(m) = state.expr_modal.as_mut() else {
        return;
    };
    match g.phase {
        GesturePhase::Begin => {
            let (x, y) = m.stage_offset;
            m.stage_drag = Some((x, y, g.x, g.y));
        }
        GesturePhase::Update => {
            if let Some((x0, y0, px, py)) = m.stage_drag {
                m.stage_offset = (x0 + (g.x - px), y0 + (g.y - py));
            }
        }
        _ => m.stage_drag = None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The ghost is a closed figure with an inside**, whatever the drawing does to
    /// it. A polygon of two points fills nothing, and a stage that silently drew
    /// nothing would look exactly like a stage whose expression is neutral.
    #[test]
    fn the_ghost_outline_is_a_fillable_figure() {
        let p = ghost_outline();
        assert!(p.len() >= 3, "a polygon needs an inside, got {}", p.len());
        for (x, y) in &p {
            assert!(
                x.abs() <= 0.51 && y.abs() <= 0.51,
                "the outline lives in the unit box: ({x}, {y})"
            );
        }
    }

    /// **The stage follows the card by construction.** Moving the card moves the
    /// stage by exactly the same delta, because the stage has no position of its
    /// own — that is the one number the link is made of.
    #[test]
    fn the_stage_travels_with_the_card() {
        let off = default_offset(600.0);
        let a = stage_rect(Rect::new(100.0, 50.0, 600.0, 400.0), off);
        let b = stage_rect(Rect::new(140.0, 90.0, 600.0, 400.0), off);
        assert!(
            (b.x - a.x - 40.0).abs() < f32::EPSILON && (b.y - a.y - 40.0).abs() < f32::EPSILON,
            "the stage moves with the card: {a:?} -> {b:?}"
        );
        assert!(
            a.x >= 100.0 + 600.0,
            "and it opens to the RIGHT of the card, not on top of it"
        );
    }
}
