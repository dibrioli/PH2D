//! As ALÇAS das strips (cantos e alça de fade) e a sombra delas nos cartões — split do
//! `strip_paint.rs` pelo tecto de 600 LOC dos painéis; filho dele, lê as consts por `super::*`.

use super::*;

/// A corner gizmo: an **L-shaped bracket** hugging the two edges that meet at this corner —
/// a QUINA, not a dot (Enio, 2026-07-16). One arm runs along the top/bottom edge, the other
/// down/up the left/right edge, both from the corner itself, so the mark reads as the corner
/// you grab rather than a bead sitting near it.
///
/// The horizontal arm is the grab region's own width (`EDGE_W`, what `hit_plan` registers),
/// so what is drawn and what is clickable start at the same pixel.
pub(super) fn paint_corner(
    ctx: &mut PaintCtx,
    (theme, estado): (Theme, EstadoAlca),
    body: Rect,
    left: bool,
    top: bool,
    color: Color,
) {
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
    let bracos = [
        Rect::new(x_out, y_edge, arm, w),
        Rect::new(x_edge, y_out, w, arm),
    ];
    crate::alca_estado::pinta(ctx, theme, estado, color, &bracos, &[]);
}

/// A guia (a altura da strip, `Xs` além de cada lado) e a leitura de `segundos` junto de `x`,
/// do lado de dentro quando `a_esquerda` — só no arrasto (`alca_estado`).
pub(super) fn guia_e_leitura(
    ctx: &mut PaintCtx,
    (theme, estado): (Theme, EstadoAlca),
    body: Rect,
    x: f32,
    segundos: f64,
    a_esquerda: bool,
) {
    let m = Spacing::Xs.px();
    crate::alca_estado::pinta_guia(ctx, theme, estado, x, body.y - m, body.y + body.h + m);
    let cy = body.y - m - TypeToken::Xs.px();
    crate::alca_estado::pinta_leitura(ctx, theme, estado, segundos, x, cy, a_esquerda);
}

/// A sombra de uma alça feita de `rects` e `tris`, pintada ANTES dela, só nos cartões (a lei da
/// câmera, `paint_flight::sombra_de_alca`: a mesma forma, `Thin` mais larga de cada lado). As
/// alças das strips e as da régua (loop, duração) passam todas por aqui.
pub(crate) fn pinta_sombra_de_alca(ctx: &mut PaintCtx, rects: &[Rect], tris: &[[(f32, f32); 3]]) {
    if !ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        return;
    }
    let (sombra, g) = ph2d_editor_core::paint_flight::sombra_de_alca(ctx.host.theme());
    for r in rects {
        let cresce = Rect::new(r.x - g, r.y - g, r.w + g * 2.0, r.h + g * 2.0);
        fill_rounded_rect(ctx.scene, cresce, Radius::Xs.px() + g, sombra);
    }
    for pts in tris {
        let contorno = Stroke::new(f64::from(g * 2.0));
        let p = tri(*pts);
        ctx.scene
            .inner_mut()
            .stroke(&contorno, Affine::IDENTITY, sombra, None, &p);
    }
}

/// The fade handle: a vertical line centred at `xc` plus a ◀ ▶ arrow pair at mid-height.
pub(super) fn paint_fade_handle(
    ctx: &mut PaintCtx,
    (theme, estado): (Theme, EstadoAlca),
    xc: f32,
    y: f32,
    h: f32,
    color: Color,
) {
    let barra = Rect::new(xc - EASE_BAR_W * 0.5, y, EASE_BAR_W, h);
    let my = y + h * 0.5;
    let gap = EASE_BAR_W * 0.5 + 1.0; // LITERAL-PX-OK: clear of the line
    // Left arrow (points left) and right arrow (points right).
    let setas = [
        [
            (xc - gap - ARROW_SZ, my),
            (xc - gap, my - ARROW_SZ),
            (xc - gap, my + ARROW_SZ),
        ],
        [
            (xc + gap + ARROW_SZ, my),
            (xc + gap, my - ARROW_SZ),
            (xc + gap, my + ARROW_SZ),
        ],
    ];
    crate::alca_estado::pinta(ctx, theme, estado, color, &[barra], &setas);
}

fn tri(pts: [(f32, f32); 3]) -> BezPath {
    let mut p = BezPath::new();
    p.move_to((f64::from(pts[0].0), f64::from(pts[0].1)));
    p.line_to((f64::from(pts[1].0), f64::from(pts[1].1)));
    p.line_to((f64::from(pts[2].0), f64::from(pts[2].1)));
    p.close_path();
    p
}
