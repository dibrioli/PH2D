//! **The Containers tab's root: the LIST of containers** (ADR-0133, amended 2026-07-21).
//!
//! # A list, not a stack
//!
//! *"A aba conteiner só serve como uma lista de containers criados"* (Enio, 2026-07-21). The
//! first cut made the tab a second Arrange over a container's interior, reached through a
//! dropdown chip that navigated — so making a container and going into one were the same
//! press, the tab never showed what the document HELD, and the only way to see your
//! containers was to open a menu. This is the Library of Animate / the Project panel of After
//! Effects: the assets, listed, with the two verbs they have.
//!
//! # Exactly two verbs, and the second one is why the first is a button
//!
//! A container's row does **rename** and **enter**, and nothing else — *"ela não pode ser
//! redimensionada e nem pode sofrer nenhuma outra operação"*. Enter is the double-click, so
//! rename cannot be; it is the pencil, the same affordance the clip selector uses one bar up.
//!
//! The bar spans the **whole time area** rather than the container's duration, and that is
//! the load-bearing part of the drawing: a container is an ASSET, with no position in time
//! and no span to trim. Drawn as its own length it would read as a strip you may drag — and
//! a brand-new container is EMPTY, so its bar would be zero pixels wide and the double-click
//! that is the only way in would have nothing to land on.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, TimelineHitKind,
};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::text_elide::paint_text_elided;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::graph::TimeView;
use crate::stack_lane_paint::STRIP_PAD_Y;
use crate::state::TimelinePanelState;
use crate::{geom, ids};

/// Paint one bar per container. A no-op wherever the rows are not the container list.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    if crate::tab::rows(state.tab, snap) != crate::tab::Rows::Containers {
        return;
    }
    let region = g.rows;
    let bands: Vec<(usize, f32, f32)> =
        geom::stack_bands(snap, state.tab, region.y, state.scroll_y).collect();
    for (i, y, h) in bands {
        if y + h <= region.y || y >= region.y + region.h {
            continue;
        }
        paint_row(
            ctx,
            theme,
            g,
            view,
            snap,
            i,
            Rect::new(region.x, y, region.w, h),
        );
    }
}

/// **Where a container's rename field floats** — over the label column of its own row.
///
/// Pure, and the one door: the paint lays the name out inside this rect and the deferred
/// overlay puts the field on it, so a field can never hover over a row it is not renaming.
/// `None` when no container rename is open, or its row is scrolled out of the band.
pub(crate) fn rename_anchor(
    g: &geom::Geom,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) -> Option<Rect> {
    let cr = state
        .clip_rename
        .filter(|c| c.kind == crate::state::RenameKind::Container)?;
    if crate::tab::rows(state.tab, snap) != crate::tab::Rows::Containers {
        return None;
    }
    let region = g.rows;
    let (_, y, h) = geom::stack_bands(snap, state.tab, region.y, state.scroll_y)
        .find(|(i, _, _)| *i == cr.index)?;
    (y >= region.y && y + h <= region.y + region.h).then(|| Rect::new(region.x, y, g.label_w, h))
}

/// One container: `name … ✎ | ▬▬▬▬▬▬▬▬`.
fn paint_row(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    snap: &TimelineViewSnapshot,
    index: usize,
    row: Rect,
) {
    let Some(c) = snap.containers.get(index) else {
        return;
    };
    fill_rounded_rect(
        ctx.scene,
        row,
        Radius::Xs.px(),
        resolve(ColorToken::BgElev, theme),
    );

    // ── the label column: the name, then the pencil ──
    let btn_w = ROW_H_PX - STRIP_PAD_Y * 2.0;
    let pencil = Rect::new(
        row.x + g.label_w - btn_w - Spacing::Xs.px(),
        row.y + STRIP_PAD_Y,
        btn_w,
        btn_w,
    );
    let font = TypeToken::Sm.px();
    // The name's budget stops at the pencil: text that elides UNDER a button reads as a
    // button with a word behind it (the lane row learned this first).
    // Sm on the left, Xs before the pencil, and one more Sm of air between the two — three
    // gaps, so the number is a COUNT of them, not a scale factor.
    let gaps = Spacing::Sm.px() + Spacing::Sm.px() + Spacing::Xs.px();
    let name_w = (g.label_w - btn_w - gaps).max(0.0);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &c.name,
        row.x + Spacing::Sm.px(),
        row.y + (row.h - font) * 0.5,
        font,
        name_w,
        resolve(ColorToken::Text1, theme),
    );
    // The band every hit is clipped to: a row scrolled half under the ruler must not register
    // its pencil where "+ Container" is painted.
    let band = Rect::new(g.rows.x, g.rows.y, g.label_w, g.rows.h);
    paint_pencil(ctx, theme, ids::TIMELINE_CONT_RENAME[index], pencil, band);

    // ── the bar: the whole time area, blank ──
    //
    // Blank because there is nothing true to write in it. A strip carries a name because it
    // says WHICH clip plays there; here the name is already in the label column, and a second
    // copy inside the bar would be the only thing making it look like a span.
    let bar = Rect::new(
        view.time_x,
        row.y + STRIP_PAD_Y,
        (g.rows.x + g.rows.w - view.time_x).max(0.0),
        (row.h - STRIP_PAD_Y * 2.0).max(0.0),
    );
    if bar.w <= 0.0 {
        return;
    }
    fill_rounded_rect(
        ctx.scene,
        bar,
        Radius::Xs.px(),
        resolve(ColorToken::Bg3, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        bar,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    // ONE rect, no edges, no grips: the bar has no operation but the double-click that enters
    // it. What is not registered cannot be dragged.
    ctx.host.store_mut().register(
        ids::TIMELINE_CONT_ROW[index],
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::ContainerRow { index },
            canvas: row,
        },
    );
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_CONT_ROW[index], bar);
}

/// The rename pencil on a container's row — the transport chip's, one bar down.
fn paint_pencil(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    r: Rect,
    band: Rect,
) {
    use ph2d_editor_core::paint::paint_icon;
    fill_rounded_rect(
        ctx.scene,
        r,
        Radius::Xs.px(),
        resolve(ColorToken::Bg3, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        r,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    let pad = Spacing::Xs.px();
    paint_icon(
        ctx.scene,
        IconId::Text,
        Rect::new(
            r.x + pad,
            r.y + pad,
            (r.w - pad * 2.0).max(0.0),
            (r.h - pad * 2.0).max(0.0),
        ),
        resolve(ColorToken::Text3, theme),
        StrokeToken::Default.px(),
    );
    ctx.host.store_mut().register(id, InteractiveState::Plain);
    // Clipped to the visible band, like every other control in this column.
    let (x0, y0) = (r.x.max(band.x), r.y.max(band.y));
    let (x1, y1) = (
        (r.x + r.w).min(band.x + band.w),
        (r.y + r.h).min(band.y + band.h),
    );
    if x1 > x0 && y1 > y0 {
        ctx.host
            .hit_index_mut()
            .register(id, Rect::new(x0, y0, x1 - x0, y1 - y0));
    }
}

/// **One gesture on a container's bar** — and the list of what it answers is the whole
/// specification: a DOUBLE-click enters, everything else is inert.
///
/// Inert is the feature, not an omission. A single click that did something would make the
/// first half of every double-click do it too; a drag would be the resize and the lane-cross
/// the bar must not have.
pub(crate) fn apply(state: &mut TimelinePanelState, index: usize, g: TimelineGesture) {
    if matches!(g.phase, GesturePhase::DoubleClick) {
        crate::state::open_container_root(state, index);
    }
}

#[cfg(test)]
#[path = "container_list_tests.rs"]
mod tests;
