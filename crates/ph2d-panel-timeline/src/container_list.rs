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
//! # Exactly three verbs — rename, delete, enter — and why enter is the double-click
//!
//! *"As funções normais de renomear, deletar e entrar"* (Enio, 2026-07-21), and nothing
//! else: *"ela não pode ser redimensionada e nem pode sofrer nenhuma outra operação"*. Enter
//! is the double-click, so the other two cannot be — they are the pencil and the trash, the
//! clip cluster's own `[✎][🗑]` pair one bar down. Delete cascades (the asset AND its
//! instances, one undo step): a strip whose source is gone can never be re-pointed, so
//! leaving it would be leaving a corpse.
//!
//! The bar is **strip-sized** — `[0, length]` in the container's own seconds, drawn in the
//! strip's visual language, because a strip is what it stands for. The length comes doored
//! through [`ph2d_timeline::container_bar_seconds`], so an EMPTY container is born 2 s wide
//! and the double-click always has somewhere to land. ⚠️ The first cut spanned the whole
//! time area in a background tone (reasoning the empty case would be 0 px) — on screen that
//! read as an empty lane, and the report was immediate: *"a strip que representa o container
//! Jump não apareceu"*.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, TimelineHitKind,
};
use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello, resolve, stroke_rounded_rect};
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

/// One container: `name … ✎ 🗑 | ▮▮▮ (a strip-sized bar, from 0 to its length)`.
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

    // ── the label column: the name, then the pencil and the trash ──
    //
    // Right-aligned like the lane row's controls, and the trash takes the far edge — the
    // clip cluster's `[✎][🗑]` order, one bar down.
    let btn_w = ROW_H_PX - STRIP_PAD_Y * 2.0;
    let btn_y = row.y + STRIP_PAD_Y;
    let trash = Rect::new(
        row.x + g.label_w - btn_w - Spacing::Xs.px(),
        btn_y,
        btn_w,
        btn_w,
    );
    let pencil = Rect::new(trash.x - btn_w - Spacing::Xs.px(), btn_y, btn_w, btn_w);
    let font = TypeToken::Sm.px();
    // The name's budget stops where the buttons start: text that elides UNDER a button reads
    // as a button with a word behind it (the lane row learned this first).
    let name_w = (pencil.x - row.x - Spacing::Sm.px() * 2.0).max(0.0);
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
    // its buttons where "+ Container" is painted.
    let band = Rect::new(g.rows.x, g.rows.y, g.label_w, g.rows.h);
    paint_row_button(
        ctx,
        theme,
        ids::TIMELINE_CONT_RENAME[index],
        pencil,
        band,
        IconId::Text,
    );
    paint_row_button(
        ctx,
        theme,
        ids::TIMELINE_CONT_DELETE[index],
        trash,
        band,
        IconId::Trash,
    );

    // ── the bar: a STRIP-SIZED box, the container's own length from 0 ──
    //
    // Drawn in the strip's visual language (same fill, same corner, name inside) because that
    // is what it stands for — *"uma barra como uma Strip que ainda está em branco"* (Enio,
    // 2026-07-21). ⚠️ The first cut spanned the whole time area in a background tone,
    // reasoning an empty container would be 0 px wide; on screen that read as an empty lane —
    // *"a strip que representa o container Jump não apareceu"*. The width is `c.length`,
    // which comes doored through `container_bar_seconds`: content answers for itself and an
    // EMPTY container is born 2 s long, so the double-click always has somewhere to land.
    //
    // What it does NOT get is any of a strip's grips: no trim edges, no stretch corners, no
    // fades — ONE rect, and the only gesture is the double-click that enters.
    let time_band = Rect::new(
        view.time_x,
        g.rows.y,
        (g.rows.x + g.rows.w - view.time_x).max(0.0),
        g.rows.h,
    );
    let (x0, x1) = (view.x(0.0), view.x(c.length));
    let bar = Rect::new(
        x0,
        row.y + STRIP_PAD_Y,
        (x1 - x0).max(0.0),
        (row.h - STRIP_PAD_Y * 2.0).max(0.0),
    );
    let Some(hit) = clipped(bar, time_band) else {
        return; // panned fully out of view: no ink, no hit
    };
    ctx.scene.push_clip(&rect_to_vello(time_band));
    fill_rounded_rect(
        ctx.scene,
        bar,
        Radius::Sm.px(),
        resolve(ColorToken::TimelineKey, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        bar,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::BorderStrong, theme),
    );
    // The name inside, centred — the strip's own idiom, in ACCENT for the strip's own reason
    // (`TimelineKey` is `Text1` in every theme: `Text1` here would be white on white).
    let budget = (bar.w - Spacing::Xs.px() * 2.0).max(0.0);
    let text_w = ctx.text_system.prefix_width(&c.name, font).min(budget);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &c.name,
        bar.x + (bar.w - text_w) * 0.5,
        bar.y + (bar.h - font) * 0.5,
        font,
        budget,
        resolve(ColorToken::Accent, theme),
    );
    ctx.scene.pop_layer();
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
        .register(ids::TIMELINE_CONT_ROW[index], hit);
}

/// The part of `r` inside `band`, or `None` — the lane row's rule, for the same reason: a
/// hit that reaches outside the band is a control you cannot see and can still click.
fn clipped(r: Rect, band: Rect) -> Option<Rect> {
    let (x0, y0) = (r.x.max(band.x), r.y.max(band.y));
    let (x1, y1) = (
        (r.x + r.w).min(band.x + band.w),
        (r.y + r.h).min(band.y + band.h),
    );
    (x1 > x0 && y1 > y0).then(|| Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// One of the row's two buttons (pencil / trash) — the transport chip's pair, one bar down.
fn paint_row_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    r: Rect,
    band: Rect,
    icon: IconId,
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
        icon,
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
    if let Some(hit) = clipped(r, band) {
        ctx.host.hit_index_mut().register(id, hit);
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
