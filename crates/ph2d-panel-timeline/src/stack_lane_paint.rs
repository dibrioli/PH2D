//! The clip-stack lanes: a row per lane, a rectangle per strip (ADR-0115 B1).
//!
//! **The crossfade is drawn from the same numbers the evaluator weights by.** The
//! snapshot's `blend_in`/`blend_out` come straight off `ClipLane::blend_in/out` —
//! the panel never re-derives them. So the wedge you see IS the blend you hear;
//! they cannot drift apart, which is the failure mode a "draw it approximately"
//! shortcut would have shipped.
//!
//! Nothing here knows what an overlap is either. Two strips whose rectangles touch
//! simply have a blend window at that end, because the lane says so.

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, Radius, Theme};

use crate::graph::TimeView;
use crate::stack_ease_grip::{EASE_IN, EASE_OUT, blend_px, overlaps, strip_grips};
use crate::state::TimelinePanelState;
use crate::strip_paint::{Fase, paint_strip};
use crate::{geom, ids};

/// Vertical inset of a strip inside its row, so lanes read as separate rows.
pub(crate) const STRIP_PAD_Y: f32 = 3.0; // LITERAL-PX-OK: a strip must not touch its row's edge
/// Grab width of a strip's trim edges.
pub(crate) const EDGE_W: f32 = 12.0; // LITERAL-PX-OK: a comfortable pointer target — 6 px was too thin to click (Enio, 2026-07-19)

/// Paint every lane of the clip stack, above the Summary channel and the tracks.
/// A no-op when the document has no stack — which is the norm.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    // The stack band holds LANES here and the container LIST at the Containers tab's root —
    // one band, two contents (`tab::rows`). Asking before the loop rather than filtering
    // inside it: the two painters answer the same question in the same words, so neither can
    // decide it is also the other's turn.
    if crate::tab::rows(state.tab, snap) != crate::tab::Rows::Lanes {
        return;
    }
    let region = g.rows;
    let bands: Vec<(usize, f32, f32)> =
        geom::stack_bands(snap, state.tab, region.y, state.scroll_y).collect();
    for (i, y, h) in bands {
        // Scrolled out of the band: neither paint nor leave hits behind.
        if y + h <= region.y || y >= region.y + region.h {
            continue;
        }
        paint_lane(
            ctx,
            theme,
            g,
            view,
            state,
            snap,
            i,
            Rect::new(region.x, y, region.w, h),
        );
    }
    crate::no_lugar_paint::paint(ctx, theme, g, view, state, snap);
}

/// One lane: its label slice, then its strips.
#[allow(clippy::too_many_lines)] // the row IS the layout: splitting it hides the arithmetic
#[allow(clippy::too_many_arguments)] // one lane's worth of per-frame context; grouping hides it
fn paint_lane(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
    index: usize,
    row: Rect,
) {
    let lane = &snap.lanes[index];
    let dim = lane.muted || lane.weight <= 0.0;
    // The label column — name, weight, mute, "+ strip" and the menu surface (`faixas_coluna`).
    crate::faixas_coluna::paint(ctx, theme, g, lane, index, row);

    // ── the strips ──
    //
    // Two passes, and the order is load-bearing. The hit index resolves to the
    // LAST id registered over a point, and a lane's strips MAY OVERLAP — that is
    // the crossfade. Registering each strip whole (body, then its edges) before
    // the next one puts the right strip's BODY on top of the left strip's END
    // edge, which is precisely the edge you reach for to tune the crossfade you
    // just made. Bodies first for the whole lane, edges after, and every edge
    // outranks every body.
    // Clipped to the TIME area, never the label column: a strip that starts left of
    // the view has an x0 far off to the left. The HITS below intersect with this
    // band (`hit_plan`); the PAINT must too, or the body's fill blankets the lane's
    // name/weight/mute/"+ strip" while it scrolls left (Enio, 2026-07-16: *"a barra
    // da lane deslocada para esquerda aparece acima das opções do painel"*). One
    // band for the ink and the hit, so what is drawn and what is clickable end at
    // the same pixel.
    let time_band = Rect::new(
        view.time_x,
        g.rows.y,
        (view.right - view.time_x).max(0.0),
        g.rows.h,
    );
    let mut boxes: Vec<(&ph2d_timeline::StripView, Rect)> = Vec::new();
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    ctx.scene.push_clip(&rect_to_vello(time_band));
    for s in &lane.strips {
        let (x0, x1) = (view.x(s.t_start), view.x(s.t_end));
        // Off-screen, or collapsed to nothing: no rect, no hit.
        if x1 <= view.time_x || x0 >= view.right || x1 - x0 < 1.0 {
            continue;
        }
        let pad = geom::peca_margem();
        let body = Rect::new(
            x0,
            row.y + pad,
            (x1 - x0).max(0.0),
            (row.h - pad * 2.0).max(0.0),
        );
        let tinta = crate::pecas_cartoes::tinta_do_strip(s, snap);
        let fase = if cartoes { Fase::Base } else { Fase::Tudo };
        let marca = (dim, state.strip_escolhida == Some(s.id));
        paint_strip(ctx, theme, view, body, s, marca, index, tinta, fase);
        boxes.push((s, body));
    }
    if cartoes {
        for (s, body) in &boxes {
            let tinta = crate::pecas_cartoes::tinta_do_strip(s, snap);
            let marca = (dim, state.strip_escolhida == Some(s.id));
            paint_strip(ctx, theme, view, *body, s, marca, index, tinta, Fase::Topo);
        }
    }
    ctx.scene.pop_layer();
    paint_trim_preview(ctx, theme, state, index, &boxes, view, time_band);
    let spans: Vec<(u64, Rect)> = boxes.iter().map(|(s, b)| (s.id.0, *b)).collect();
    // The ease grips (B4) — only where the strip owns the edge. Where a neighbour
    // overlaps it, the OVERLAP is the fade (Unity's rule): the grip is painted greyed
    // and gets **no hit** at all, because a dimmed control that still dispatches is a
    // control that lies ([[feedback_disabled_button_still_dispatches]]).
    let eases: Vec<(u64, u8, Rect, bool)> = boxes
        .iter()
        .filter_map(|(s, body)| {
            let (a, b) = strip_grips(
                *body,
                blend_px(view, s.t_start, s.lead_in),
                blend_px(view, s.t_start, s.blend_in),
                blend_px(view, s.t_start, s.blend_out),
                blend_px(view, s.t_end, s.lead_out),
            )?;
            Some([
                (s.id.0, EASE_IN, a, s.ease_locked_in),
                (s.id.0, EASE_OUT, b, s.ease_locked_out),
            ])
        })
        .flatten()
        .collect();
    // **As FAIXAS de fade — a superfície que o menu de easing abre** (Enio, 2026-07-31).
    // Vêm da MESMA porta que o pintor listra (`fade_bands`), senão o menu abriria onde nada
    // foi desenhado. Registradas DEPOIS dos corpos e ANTES das quinas, então uma faixa
    // ganha do corpo (é ela que se quer) e perde para os grips de aparar/esticar, que são
    // alvos de precisão nas beiras.
    //
    // ⚠️ Numa SOBREPOSIÇÃO as faixas das duas strips se cruzam, e last-wins entrega a de
    // quem CHEGA — que é exatamente a strip cuja `curve_in` governa aquele crossfade
    // (`ClipLane::effective_curve`): a lane está ordenada por início, então a que chega é a
    // que registra por último, e o alinhamento sai de graça em vez de ser mantido à mão.
    let bands: Vec<(u64, u8, Rect)> = boxes
        .iter()
        .flat_map(|(s, body)| {
            crate::strip_paint::fade_bands(s, view, *body)
                .into_iter()
                .map(|(edge, rect, _)| (s.id.0, edge, rect))
        })
        .collect();
    for (strip, edge, rect) in hit_plan(&spans, &bands, &eases, time_band) {
        put_strip_hit(ctx, row, rect, index, strip, edge);
    }
}

/// **The cut area, made evident** (Enio, 2026-07-16): while a TRIM drag shortens a strip,
/// the region it removes shows red cross-hatched — the frames about to be dropped. A live
/// preview: when the drag ends the strip is simply shorter and the red is gone. A trim that
/// REVEALS more (grows the strip) removes nothing, so it paints nothing. Clipped to the
/// lane's time-band so it never spills over the label column.
fn paint_trim_preview(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    lane: usize,
    boxes: &[(&ph2d_timeline::StripView, Rect)],
    view: TimeView,
    time_band: Rect,
) {
    let Some(cut) = trim_cut_region(state, lane, boxes, view) else {
        return;
    };
    let x0 = cut.x.max(time_band.x);
    let clipped = Rect::new(
        x0,
        cut.y,
        (cut.x + cut.w).min(time_band.x + time_band.w) - x0,
        cut.h,
    );
    if clipped.w <= 0.0 {
        return;
    }
    ctx.scene.push_clip(&rect_to_vello(time_band));
    fill_rounded_rect(
        ctx.scene,
        clipped,
        Radius::Xs.px(),
        resolve(ColorToken::DangerSoft, theme),
    );
    crate::strip_paint::paint_hatch(ctx, clipped, resolve(ColorToken::Danger, theme), true);
    ctx.scene.pop_layer();
}

/// The red cut region for a live TRIM drag on lane `lane`, or `None` when no trim is in
/// progress there or the drag REVEALS clip (grows the strip) rather than removing it.
///
/// Pure, so the "only when shrinking, only the removed span" rule has an oracle a paint
/// test cannot give it. The removed span runs from the trimmed edge's position at
/// drag-start (`start_span`) to where the snapshot now shows it; its `y`/`h` come from the
/// strip's live box.
fn trim_cut_region(
    state: &TimelinePanelState,
    lane: usize,
    boxes: &[(&ph2d_timeline::StripView, Rect)],
    view: TimeView,
) -> Option<Rect> {
    let d = state
        .strip_drag
        .filter(|d| d.lane == lane && (d.edge == 0 || d.edge == 1))?;
    let (s, body) = boxes.iter().find(|(s, _)| s.id == d.id)?;
    let (a0, b0) = d.start_span;
    // The removed span in TIME — its width is negative (`x1 <= x0` below, since `view.x`
    // is monotonic) when the drag REVEALS more clip rather than cutting, and then no hatch
    // is drawn: revealing removes nothing.
    let (t0, t1) = if d.edge == 0 {
        (a0, s.t_start) // start edge dragged inward hides [original_start, now]
    } else {
        (s.t_end, b0) // end edge dragged inward hides [now, original_end]
    };
    let (x0, x1) = (view.x(t0), view.x(t1));
    (x1 > x0).then(|| Rect::new(x0, body.y, x1 - x0, body.h))
}

/// **The order the strip hits are registered in — and it is load-bearing.**
///
/// The hit index resolves a point to the LAST id registered over it, and a lane's
/// strips MAY OVERLAP: that is the crossfade. Registering each strip whole (body,
/// then its own edges) put the right strip's BODY on top of the left strip's END
/// edge — precisely the grip you reach for to tune the crossfade you just made.
/// So: every body first, every edge after. An edge always outranks a box.
///
/// Pure, and tested, because no headless paint can reach it: the defect lived in
/// the order of two loops and no `apply_event` test could ever have seen it.
///
/// The ease grips (B4) come LAST, and only over their own strip's body: a fade handle
/// sits ON the body it belongs to, so it has to outrank it. It never lands on a trim
/// grip — [`ease_grip`] declines the handle entirely on a strip too narrow to hold
/// both — which is why "last" is safe here and would not be otherwise.
///
/// **A LOCKED ease grip is painted and not registered** (`locked` = a neighbour's overlap
/// defines that fade, so the number is not the strip's to author). The refusal lives HERE,
/// in the pure function, and not in the paint loop that calls it — a rule that only exists
/// where no test can reach it is a rule that will be deleted by someone tidying up, and the
/// artist would find out by dragging a handle that silently writes a number nobody reads
/// ([[feedback_disabled_button_still_dispatches]]).
pub(crate) fn hit_plan(
    spans: &[(u64, Rect)],
    bands: &[(u64, u8, Rect)],
    eases: &[(u64, u8, Rect, bool)],
    band: Rect,
) -> Vec<(u64, u8, Rect)> {
    let mut out = Vec::with_capacity(spans.len() * 3 + bands.len() + eases.len());
    for &(id, body) in spans {
        if let Some(r) = clipped(body, band) {
            out.push((id, 2, r));
        }
    }
    // The fade BANDS sit between the bodies and the corner grips: a band outranks the body
    // it is drawn on (that is the point — the menu is the band's whole reason to be hittable)
    // and yields to the trim/stretch corners, which are the precision targets at the edges.
    for &(id, edge, rect) in bands {
        if let Some(r) = clipped(rect, band) {
            out.push((id, edge, r));
        }
    }
    for &(id, body) in spans {
        let w = EDGE_W.min(body.w * 0.5);
        let top = body.h * 0.5;
        // Each strip operation is its own CORNER (Enio, 2026-07-16): the GREEN top
        // corners STRETCH (retime), the RED bottom corners TRIM (cut). Top-half vs
        // bottom-half of each edge — disjoint, so a pointer picks one unambiguously.
        for (edge, rect) in [
            (5, Rect::new(body.x, body.y, w, top)), // stretch-start (top-left)
            (6, Rect::new(body.x + body.w - w, body.y, w, top)), // stretch-end (top-right)
            (0, Rect::new(body.x, body.y + top, w, body.h - top)), // trim-start (bottom-left)
            (
                1,
                Rect::new(body.x + body.w - w, body.y + top, w, body.h - top),
            ), // trim-end (bottom-right)
        ] {
            // The WHOLE grip must be inside the band, not merely overlap it: a
            // sliver of an edge poking into view is not something a pointer can aim
            // at, and the start edge of a strip that begins off-screen left would
            // otherwise be trimmable blind.
            if rect.x >= band.x
                && rect.x + rect.w <= band.x + band.w
                && let Some(r) = clipped(rect, band)
            {
                out.push((id, edge, r));
            }
        }
    }
    // Every CORNER grip registered above — a fade handle may not land on ANY of them
    // (its own strip's, or a neighbour's it overlaps in a crossfade).
    let trims: Vec<Rect> = out
        .iter()
        .filter(|(_, e, _)| matches!(*e, 0 | 1 | 5 | 6))
        .map(|(_, _, r)| *r)
        .collect();
    for &(id, edge, rect, locked) in eases {
        // Same rule as the trim grips: a handle half-visible at the edge of the view is
        // not one a pointer can aim at. And a locked one is not grabbable at all.
        //
        // **And it must not cover a trim grip — not even a NEIGHBOUR's.** `ease_grips` keeps a
        // handle clear of its own strip's edges, but strips OVERLAP (that is the crossfade), so
        // one strip's fade handle can land squarely on the strip next door's trim grip — and,
        // registered after every trim, it would win the click. The artist would reach for the
        // edge of B and author a fade on A. Dropping the handle is the right refusal: the trim
        // is the gesture that cannot be replaced, and the fade is one scroll away.
        if !locked
            && rect.x >= band.x
            && rect.x + rect.w <= band.x + band.w
            && let Some(r) = clipped(rect, band)
            && !trims.iter().any(|t| overlaps(*t, r))
        {
            out.push((id, edge, r));
        }
    }
    out
}

/// The part of `r` the panel can actually show — its intersection with the band.
///
/// A hit rect that reaches outside the band is a control you cannot see and can
/// still click. A lane scrolled half under the ruler would otherwise register its
/// mute where "+ Lane" is painted, and a strip that starts left of the view would
/// register its body across the whole label column, on top of every control there.
/// Both are the same bug, and this is the one place it dies.
pub(crate) fn clipped(r: Rect, band: Rect) -> Option<Rect> {
    let (x0, y0) = (r.x.max(band.x), r.y.max(band.y));
    let (x1, y1) = (
        (r.x + r.w).min(band.x + band.w),
        (r.y + r.h).min(band.y + band.h),
    );
    (x1 > x0 && y1 > y0).then(|| Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// One strip hit target. `edge`: `0` = start, `1` = end, `2` = body.
pub(crate) fn put_strip_hit(
    ctx: &mut PaintCtx,
    row: Rect,
    rect: Rect,
    lane: usize,
    strip: u64,
    edge: u8,
) {
    let id = ids::timeline_strip_hit_id(lane as u64, strip, edge);
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ph2d_editor_core::ids::TIMELINE_PANEL,
            kind: TimelineHitKind::Strip { lane, strip, edge },
            canvas: row,
        },
    );
    ctx.host.hit_index_mut().register(id, rect);
}

#[cfg(test)]
#[path = "stack_lane_paint_tests.rs"]
mod tests;
