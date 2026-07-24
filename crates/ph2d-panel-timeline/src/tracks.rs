//! Track list (W2.E4) — the left label column of the dope-sheet: the "+Track"
//! dropdown (header row, aligned with the ruler strip) and one row per binding
//! (property label + a short object tag).
//!
//! "+Track" is a dropdown button; opening it lists the six properties as a
//! popover overlay. Picking one raises `PanelEvent::Click(<prop id>)`; the shell
//! binds the *currently selected* sprite's matching property (it owns the
//! selection) and closes the dropdown. Each row's time area paints its key
//! **diamonds** (selected keys in the accent colour), culled to the visible
//! span, and registers their [`TimelineHitKind`] hit targets (+ a `Lane`
//! background) so the dope-sheet gestures reach `interact` (E5b: click-select,
//! drag-move, clear-on-empty).

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::text_elide::paint_text_elided;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{PropKind, SelectedKey, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Fill, Stroke};

use crate::ids;
use crate::state::TimelinePanelState;
use crate::{geom, graph, graph_paint, summary_paint};

/// Width of the left label column (property + object tag).
pub(crate) const LABEL_COL_W: f32 = 132.0; // LITERAL-PX-OK: track-label column width
/// Width of the expand/collapse twirl at the head of each row.
const TWIRL_W: f32 = 24.0; // LITERAL-PX-OK: row twirl hit column width
/// Half the twirl triangle's flat edge.
const TWIRL_BASE_HALF: f32 = 5.0; // LITERAL-PX-OK: row twirl triangle half-base
/// Height of the twirl triangle, flat edge to apex.
const TWIRL_TIP: f32 = 9.0; // LITERAL-PX-OK: row twirl triangle height
const DIAMOND_H: f32 = 4.5; // LITERAL-PX-OK: keyframe diamond half-size
/// A ROVING key draws as a small dot instead of the diamond (AE convention:
/// its time is derived, not authored).
const ROVE_DOT_R: f32 = 2.5; // LITERAL-PX-OK: roving key dot radius
/// Horizontal half-width of a key's clickable hit rect (larger than the visual
/// diamond so a small target is easy to grab).
pub(crate) const KEY_HIT_HW: f32 = 7.0; // LITERAL-PX-OK: keyframe grab half-width

/// Paint the "+ Track" dropdown button filling `header` (the label-column slice
/// aligned with the ruler strip). The property list opens as an overlay popover
/// (see [`paint_add_track_popover`], painted last).
pub(crate) fn paint_add_track(ctx: &mut PaintCtx, theme: Theme, header: Rect) {
    let state = ctx
        .host
        .store()
        .button_state(ids::TIMELINE_ADD_TRACK)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(
        ids::TIMELINE_ADD_TRACK,
        ph2d_i18n::tr("panel.timeline.add_track"),
    )
    .state(state);
    paint_button(&btn, header, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_ADD_TRACK, header);
}

/// Paint the property dropdown as an overlay below `anchor` (the +Track button)
/// when open, and register each option's hit. Call LAST so it sits on top.
pub(crate) fn paint_add_track_popover(ctx: &mut PaintCtx, theme: Theme, anchor: Rect, open: bool) {
    if !open {
        return;
    }
    let n = ids::ADDPROP_BUTTONS.len() as f32;
    let list = Rect::new(anchor.x, anchor.y + anchor.h, anchor.w, ROW_H_PX * n);
    fill_rounded_rect(
        ctx.scene,
        list,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        list,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    let mut y = list.y;
    for (id, prop) in ids::ADDPROP_BUTTONS {
        let r = Rect::new(list.x, y, list.w, ROW_H_PX);
        let state = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, prop_label(prop)).state(state);
        paint_button(&btn, r, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(id, r);
        y += ROW_H_PX;
    }
}

/// Paint one row per binding in `region`: the label (left `label_w` column) and
/// the key diamonds in the time area (mapped by `time_x`/`view_start`/`px_per_s`,
/// culled to the visible span). Selected keys paint in the accent colour, shifted
/// by `preview_dx` px while a key drag is in flight.
///
/// Registers a [`TimelineHitKind::Lane`] background over the time area (click =
/// clear selection) and one [`TimelineHitKind::Key`] hit per visible diamond
/// (click = select, drag = move), so `dispatch` streams gestures the panel's
/// `interact` step drains. Hits are keyed by the key's *identity* (stable across
/// frames), not its position.
pub(crate) fn paint_rows(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: graph::TimeView,
    preview_dx: f32,
    state: &mut TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    let (region, label_w) = (g.rows, g.label_w);
    let right = region.x + region.w;
    let bottom = region.y + region.h;
    let time_x = view.time_x;
    // Lane background over the time area — a click here clears the selection or
    // rubber-bands a box-select.
    let lane = Rect::new(time_x, region.y, (right - time_x).max(0.0), region.h);
    ctx.host.store_mut().register(
        ids::TIMELINE_LANES,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::Lane,
            canvas: lane,
        },
    );
    ctx.host.hit_index_mut().register(ids::TIMELINE_LANES, lane);

    // Rows overflow the band while scrolling and an expanded one is tall; clip
    // so a half-visible graph never bleeds over the ruler or the scrollbar.
    ctx.scene.push_clip(&rect_to_vello(region));
    // The stack band, above everything, scrolling with it. Two painters over ONE band, each
    // refusing on the same question (`tab::rows`): lanes on Arrange and inside a container,
    // the container LIST at the Containers tab's root.
    crate::stack_lane_paint::paint(ctx, theme, g, view, state, snap);
    crate::container_list::paint(ctx, theme, g, view, state, snap);
    // The master row, above track 0 and scrolling with it.
    summary_paint::paint(ctx, theme, g, view, preview_dx, state, snap);
    let bands: Vec<(usize, f32, f32)> = geom::row_bands(
        snap,
        state.tab,
        &state.expanded,
        state.graph_h,
        region.y,
        state.scroll_y,
    )
    .collect();
    for (i, y, h) in bands {
        // Cull the rows outside the band entirely: a long track list then neither
        // paints nor registers off-screen hits.
        if y + h <= region.y || y >= bottom {
            continue;
        }
        let track = &snap.tracks[i];
        // Zebra so lanes read as discrete rows.
        if i % 2 == 1 {
            fill_row(
                ctx,
                theme,
                Rect::new(region.x, y, region.w, ROW_H_PX),
                ColorToken::TimelineRowAlt,
            );
        }
        paint_twirl(ctx, theme, state, region.x, y, track.target.get());
        // The label slice (after the twirl) is a right-click surface: dispatch
        // opens the track-row context menu on it (Delete Track). Inert to
        // primary gestures — see the `Row` arm in `interact`.
        let row_id = ids::timeline_row_id(track.target.get());
        let row_hit = Rect::new(
            region.x + TWIRL_W,
            y,
            (label_w - TWIRL_W).max(0.0),
            ROW_H_PX,
        );
        ctx.host.store_mut().register(
            row_id,
            InteractiveState::TimelineSurface {
                parent: ids::TIMELINE_PANEL,
                kind: TimelineHitKind::Row {
                    target: track.target.get(),
                },
                canvas: row_hit,
            },
        );
        ctx.host.hit_index_mut().register(row_id, row_hit);
        let font = TypeToken::Sm.px();
        let text = format!("{}  #{}", prop_label(track.prop), track.entity % 10_000);
        let color = if track.missing {
            ColorToken::TimelineMissing
        } else {
            ColorToken::Text1
        };
        let label_x = region.x + TWIRL_W;
        // Elided, never wrapped. In `paint_text`, `max_width` is a WRAP budget:
        // a name one pixel too long silently became two lines and spilled over
        // the row below it.
        // NOTE the wording above avoids a lone apostrophe on purpose — the
        // panel LOC gate parser treats one inside a comment as a char literal
        // and stops counting braces (see FN_OVERAGE_OK in that test).
        paint_text_elided(
            ctx.text_system,
            ctx.scene,
            &text,
            label_x,
            y + (ROW_H_PX - font) * 0.5,
            font,
            (label_w - TWIRL_W - Spacing::Xs.px()).max(0.0),
            resolve(color, theme),
        );
        paint_lane_keys(ctx, theme, track, view, preview_dx, y, lane);
        if h > ROW_H_PX {
            let band = Rect::new(region.x, y + ROW_H_PX, region.w, h - ROW_H_PX);
            graph_paint::paint_track(ctx, theme, state, band, label_w, view, track);
        }
    }
    ctx.scene.pop_layer();
}

/// One row's key diamonds + their hit targets, on the dope-sheet strip.
fn paint_lane_keys(
    ctx: &mut PaintCtx,
    theme: Theme,
    track: &ph2d_timeline::TrackView,
    view: graph::TimeView,
    preview_dx: f32,
    y: f32,
    lane: Rect,
) {
    let (time_x, right) = (view.time_x, view.right);
    let cy = y + ROW_H_PX * 0.5;
    for k in &track.keys {
        let base_x = time_x + ((k.t_seconds - view.view_start) * view.px_per_s) as f32;
        // Selected keys ride the live drag preview.
        let kx = if k.selected {
            base_x + preview_dx
        } else {
            base_x
        };
        if kx < time_x - DIAMOND_H || kx > right + DIAMOND_H {
            continue;
        }
        let tok = if k.selected {
            ColorToken::TimelineKeySelected
        } else {
            ColorToken::TimelineKey
        };
        if k.roving {
            // AE convention: a roving key is a small circle, not a diamond.
            ph2d_editor_core::paint::fill_circle(
                ctx.scene,
                kx,
                cy,
                ROVE_DOT_R,
                resolve(tok, theme),
            );
        } else {
            paint_diamond(ctx, kx, cy, DIAMOND_H, resolve(tok, theme));
        }
        // Hit target keyed by identity (stable across frames/reorders). The grab
        // rect follows the drawn (previewed) position.
        let id = ids::timeline_key_hit_id(track.target.get(), k.id.get());
        let hit = Rect::new(kx - KEY_HIT_HW, y, KEY_HIT_HW * 2.0, ROW_H_PX);
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ids::TIMELINE_PANEL,
                kind: TimelineHitKind::Key {
                    target: track.target.get(),
                    key: k.id.get(),
                },
                canvas: lane,
            },
        );
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// The expand/collapse twirl at the head of a row: a right-pointing triangle
/// that turns down when the track's graph editor is open.
fn paint_twirl(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    x: f32,
    y: f32,
    target: u64,
) {
    let open = state.is_expanded(target);
    let (cx, cy) = (x + TWIRL_W * 0.5, y + ROW_H_PX * 0.5);
    // An isosceles triangle: `BASE` across the flat edge, `TIP` from that edge to
    // the apex. Rotated a quarter turn when the row opens (right → down).
    let (b, t) = (TWIRL_BASE_HALF, TWIRL_TIP);
    let corners = if open {
        [
            (cx - b, cy - t * 0.5),
            (cx + b, cy - t * 0.5),
            (cx, cy + t * 0.5),
        ]
    } else {
        [
            (cx - t * 0.5, cy - b),
            (cx + t * 0.5, cy),
            (cx - t * 0.5, cy + b),
        ]
    };
    let mut p = BezPath::new();
    p.move_to((f64::from(corners[0].0), f64::from(corners[0].1)));
    for (px, py) in &corners[1..] {
        p.line_to((f64::from(*px), f64::from(*py)));
    }
    p.close_path();
    let tok = if open {
        ColorToken::Accent
    } else {
        ColorToken::Text2
    };
    ctx.scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(tok, theme)),
        None,
        &p,
    );
    let id = ids::timeline_twirl_id(target);
    let hit = Rect::new(x, y, TWIRL_W, ROW_H_PX);
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::Twirl { target },
            canvas: hit,
        },
    );
    ctx.host.hit_index_mut().register(id, hit);
}

/// Every key whose diamond **centre** falls inside `sel` (a marquee in global
/// px). Mirrors the row/column math of [`paint_rows`] exactly — a key the user
/// can see inside the box is a key that gets selected.
///
/// Rows scrolled out of `rows` never match: `sel` is intersected with the band
/// first, so a marquee dragged past the bottom edge cannot reach into the
/// clipped rows below it.
pub(crate) fn keys_in_rect(
    rows: Rect,
    view: graph::TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
    sel: Rect,
) -> Vec<SelectedKey> {
    let bottom = rows.y + rows.h;
    let (top_y, bot_y) = (sel.y.max(rows.y), (sel.y + sel.h).min(bottom));
    let mut out = Vec::new();
    if top_y > bot_y {
        return out;
    }
    for (i, y, _h) in geom::row_bands(
        snap,
        state.tab,
        &state.expanded,
        state.graph_h,
        rows.y,
        state.scroll_y,
    ) {
        // The diamonds sit on the row's dope-sheet STRIP, never in its graph
        // band — a marquee over the curve must not grab the keys under it.
        let cy = y + ROW_H_PX * 0.5;
        if cy < top_y || cy > bot_y {
            continue;
        }
        let track = &snap.tracks[i];
        for k in &track.keys {
            let kx = view.time_x + ((k.t_seconds - view.view_start) * view.px_per_s) as f32;
            if kx >= sel.x && kx <= sel.x + sel.w {
                out.push(SelectedKey::new(track.target.get(), k.id.get()));
            }
        }
    }
    out
}

/// The live box-select rubber band: marching-ants accent outline, no fill (so
/// the diamonds underneath stay readable). Painted over the rows.
pub(crate) fn paint_marquee(ctx: &mut PaintCtx, theme: Theme, rect: Rect) {
    let stroke = Stroke::new(StrokeToken::Thin.px() as f64).with_dashes(0.0, [4.0, 3.0]); // LITERAL-PX-OK: marching-ants dash pattern
    ctx.scene.inner_mut().stroke(
        &stroke,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &rect_to_vello(rect),
    );
}

/// Fill a keyframe diamond centred at `(cx, cy)` with half-size `h`.
pub(crate) fn paint_diamond(
    ctx: &mut PaintCtx,
    cx: f32,
    cy: f32,
    h: f32,
    color: ph2d_vector::Color,
) {
    let (cx, cy, h) = (cx as f64, cy as f64, h as f64);
    let mut p = BezPath::new();
    p.move_to((cx, cy - h));
    p.line_to((cx + h, cy));
    p.line_to((cx, cy + h));
    p.line_to((cx - h, cy));
    p.close_path();
    ctx.scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &p,
    );
}

/// The display label for a property (the panel's presentation of `PropKind`).
fn prop_label(p: PropKind) -> &'static str {
    match p {
        PropKind::TranslationX => ph2d_i18n::tr("panel.timeline.prop.translate_x"),
        PropKind::TranslationY => ph2d_i18n::tr("panel.timeline.prop.translate_y"),
        PropKind::Rotation => ph2d_i18n::tr("panel.timeline.prop.rotation"),
        PropKind::ScaleX => ph2d_i18n::tr("panel.timeline.prop.scale_x"),
        PropKind::ScaleY => ph2d_i18n::tr("panel.timeline.prop.scale_y"),
        PropKind::Opacity => ph2d_i18n::tr("panel.timeline.prop.opacity"),
        PropKind::TimeRemap => ph2d_i18n::tr("panel.timeline.prop.time"),
        PropKind::Morph => ph2d_i18n::tr("panel.timeline.prop.morph"),
        PropKind::Position => ph2d_i18n::tr("panel.timeline.prop.position"),
    }
}

/// A subtle zebra fill for alternate rows.
fn fill_row(ctx: &mut PaintCtx, theme: Theme, rect: Rect, tok: ColorToken) {
    fill_rounded_rect(ctx.scene, rect, Radius::Xs.px(), resolve(tok, theme));
}
