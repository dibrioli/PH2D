//! Paint of the **Summary** channel — the master row above the tracks. Split
//! from `summary` (the columns + the gesture) the way `graph_paint` is split
//! from `graph`; that module owns every fact this one draws.
//!
//! It reads as a header rather than a track: its own background, a rule under it,
//! no twirl, and diamonds a size larger than the ones they stand for. A column
//! only takes the accent when **every** key beneath it is selected — a half-
//! selected column that looked grabbed would lie about what a drag will move.

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};

use crate::geom;
use crate::graph::TimeView;
use crate::ids;
use crate::summary::{Column, columns};
use crate::tracks::paint_diamond;

/// Half-size of a Summary diamond. Larger than a track's ([`crate::tracks`]'s
/// `DIAMOND_H`) so the master row reads as the one you grab to move a column.
const SUMMARY_DIAMOND_H: f32 = 6.0; // LITERAL-PX-OK: summary column diamond half-size
/// Grab half-width of a Summary column, a touch wider than its diamond.
const SUMMARY_HIT_HW: f32 = 8.0; // LITERAL-PX-OK: summary column grab half-width
/// The rule that separates the master row from the tracks it summarises.
const RULE_H: f32 = 1.0; // LITERAL-PX-OK: summary row bottom rule

/// The row's label. English by canon (`feedback_app_ui_english_only`); it joins
/// the `panel.timeline.*` i18n sweep with the rest of this panel's strings.
const SUMMARY_LABEL: &str = "Summary";

/// Paint the Summary channel and register one grab target per column. A no-op
/// when nothing is bound — an empty timeline shows no master row.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    preview_dx: f32,
    scroll_y: f32,
    snap: &TimelineViewSnapshot,
) {
    let region = g.rows;
    let Some((y, h)) = geom::summary_band(snap, region.y, scroll_y) else {
        return;
    };
    // Scrolled out from under the ruler: neither paint nor leave hits behind.
    if y + h <= region.y || y >= region.y + region.h {
        return;
    }
    let row = Rect::new(region.x, y, region.w, h);
    fill_rounded_rect(
        ctx.scene,
        row,
        Radius::Xs.px(),
        resolve(ColorToken::Bg3, theme),
    );
    fill_rounded_rect(
        ctx.scene,
        Rect::new(region.x, y + h - RULE_H, region.w, RULE_H),
        Radius::Xs.px(),
        resolve(ColorToken::BorderEmph, theme),
    );
    let font = TypeToken::Sm.px();
    ph2d_editor_core::text_elide::paint_text_elided(
        ctx.text_system,
        ctx.scene,
        SUMMARY_LABEL,
        region.x + Spacing::Sm.px(),
        y + (ROW_H_PX - font) * 0.5,
        font,
        (g.label_w - Spacing::Sm.px() * 2.0).max(0.0),
        resolve(ColorToken::Text2, theme),
    );

    let lane = Rect::new(
        view.time_x,
        row.y,
        (region.x + region.w - view.time_x).max(0.0),
        row.h,
    );
    for c in columns(snap) {
        paint_column(ctx, theme, &c, view, preview_dx, row, lane);
    }
}

/// One column: its diamond at the shared time, and the grab target under it.
fn paint_column(
    ctx: &mut PaintCtx,
    theme: Theme,
    c: &Column,
    view: TimeView,
    preview_dx: f32,
    row: Rect,
    lane: Rect,
) {
    // A grabbed column rides the same one-frame move preview its keys do, so the
    // master diamond never drifts from the diamonds under it mid-drag.
    let base_x = view.x(c.t_seconds);
    let x = if c.all_selected {
        base_x + preview_dx
    } else {
        base_x
    };
    let right = row.x + row.w;
    if x < view.time_x - SUMMARY_DIAMOND_H || x > right + SUMMARY_DIAMOND_H {
        return;
    }
    let cy = row.y + ROW_H_PX * 0.5;
    let tok = if c.all_selected {
        ColorToken::Accent
    } else {
        ColorToken::Text2
    };
    paint_diamond(ctx, x, cy, SUMMARY_DIAMOND_H, resolve(tok, theme));

    let id = ids::timeline_summary_hit_id(c.t_bits());
    let hit = Rect::new(x - SUMMARY_HIT_HW, row.y, SUMMARY_HIT_HW * 2.0, row.h);
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::SummaryKey { t_bits: c.t_bits() },
            canvas: lane,
        },
    );
    ctx.host.hit_index_mut().register(id, hit);
}
