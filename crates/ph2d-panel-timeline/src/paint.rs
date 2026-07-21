//! Timeline panel paint.
//!
//! Visibility gate → resolve the panel rect (the user-resized one, else the
//! bottom dock from `ctx.layout.timeline`) → publish it for dispatch → canonical
//! dark-glass chrome → transport bar → dope sheet (label column + ruler + lanes
//! + scrollbar).
//!
//! **Two-pass geometry.** `interact::process` runs BEFORE the rect is finalized,
//! against the rect and `time_x` this frame started with, so a zoom/pan/resize
//! lands on the same frame it is dragged instead of one frame late. Then the rect
//! is re-resolved (a resize may have moved it) and everything paints from that.

use crate::geom::SPLIT_GRIP;
use crate::state::{self, TimelinePanelState, set_last_content_h, set_last_visible_h};
use crate::{TimelinePanel, geom, graph, ids, ruler, scrollbar, tracks, transport};
use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_title, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, paint_panel_close_button, paint_panel_corner_dot,
    paint_panel_corner_dot_bl, paint_panel_surface,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme, TypeToken};

/// Thickness of the hairline drawn on the label/time seam.
const SPLIT_LINE_W: f32 = 1.0; // LITERAL-PX-OK: splitter hairline width

/// **The two facts the panel mirrors back to the shell each frame** — which CLOCK the
/// timeline runs on, and which STACK is on screen.
///
/// One helper because they are one act (the reverse channel of the snapshot) and because
/// `paint` is at its 200-LOC cap. Both are a REFRESH of what `state::set_tab` already
/// published on the switch itself — the shell reads the host at the TOP of a frame, so a tab
/// that only took effect at paint time would route one frame of edits into the stack the
/// animator just left. This is what restores them when a hidden panel comes back.
fn publish_view(state: &TimelinePanelState, snapshot: &ph2d_timeline::TimelineViewSnapshot) {
    // On the Keys tab AND under a stack, the shell drives the CLIP playhead and solos the
    // active clip. **Without a stack there is nothing to solo** — the clip IS the timeline —
    // so keys_mode stays false and a fresh document behaves exactly as it always has (one
    // playhead, Motion and the timeline on the same clock).
    state::publish_keys_mode(state.tab.shows_keys() && snapshot.stacked());
    // Arrange is always the SCENE's stack, so the breadcrumb trail does not apply there
    // however deep the animator walked (`tab::Tab::scene_root`).
    state::publish_scene_root(state.tab.scene_root());
}

pub(crate) fn paint(state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(TimelinePanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning the panel
        // (and `dispatch_wheel` stops zooming its time axis) once it is hidden.
        ctx.host.store_mut().clear_panel_rect(ids::TIMELINE_PANEL);
        ctx.host.store_mut().clear_timeline_canvas();
        // Drop any in-flight gesture: hiding the panel mid-drag must not leave a
        // marquee to resolve (or repaint) when it comes back — nor an undo
        // bracket open, which would swallow the next atomic edit.
        state::drop_row_gestures(state);
        // A hidden panel is not editing keys: the timeline runs on its normal
        // (timeline) clock, not a soloed clip one.
        state::publish_keys_mode(false);
        // ...nor is it inside a container. The trail survives (it comes back where it was),
        // but a hidden panel must not leave the shell driving a container's interior.
        state::publish_scene_root(true);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();
    publish_view(state, &snapshot);
    let viewport = ctx.layout.viewport;
    let docked = ctx.layout.timeline;
    if state.px_per_s <= 0.0 {
        state.px_per_s = state::DEFAULT_PX_PER_S;
    }

    // Pass 1: drain this frame's wheel + gestures against the rect we start with.
    let rect0 = geom::clamp_to(state.rect.unwrap_or(docked), viewport);
    // The label column's floor depends on what it HOLDS — which is what the TAB
    // shows: a lane row carries controls a track row does not (`geom::min_label_w`).
    let min_label = geom::min_label_w(&snapshot, state.tab);
    let time_x0 = geom::time_x(rect0, state.label_w, min_label);
    crate::interact::process(state, ctx, rect0, time_x0, viewport, &snapshot);

    // Pass 2: a resize may have just moved the panel — paint from the new rect.
    let rect = geom::clamp_to(state.rect.unwrap_or(docked), viewport);
    ctx.host
        .store_mut()
        .set_panel_rect(ids::TIMELINE_PANEL, rect);
    // Resize grippers FIRST, so every later hit (close button, lanes, keys) wins
    // over the border strips where they overlap.
    register_resize_grips(ctx, rect);

    let title_size = paint_chrome(ctx, theme, rect);
    let body = geom::body(rect, title_size);
    // The transport now flows BESIDE the title and only spills to a row of its own
    // when the panel is too narrow to hold it (Enio: "a timeline ficou apertada").
    let head_strip = geom::header_controls(rect, title_size);
    let (after_transport, clip_dd_chip) = transport::paint_bar(
        ctx,
        theme,
        head_strip,
        body,
        &snapshot,
        transport::BarView {
            tab: state.tab,
            speed_view: state.speed_view,
            source_container: state.source_container,
        },
    );
    let g = geom::resolve(rect, after_transport, state.label_w, min_label);
    // Write the clamped width back, so a drag that ran past the bounds does not
    // have to be dragged all the way back before the column moves again.
    state.label_w = g.label_w;

    // The WHOLE dope sheet takes the wheel (not just the time column) so a wheel
    // over the label names still scrolls the rows.
    ctx.host
        .store_mut()
        .set_timeline_canvas(ids::TIMELINE_PANEL, g.region);

    // Measure the scroll range, then clamp the model into it.
    state.graph_h = graph::clamp_graph_h(state.graph_h);
    let content_h = geom::content_h(&snapshot, state.tab, &state.expanded, state.graph_h);
    state.scroll_max = geom::scroll_max(content_h, g.rows.h);
    state.scroll_y = state.scroll_y.clamp(0.0, state.scroll_max); // CLAMP-OK: measured bounds, min<=max

    // `F` fits the time axis to the keys; it needs the time area's pixel width,
    // known only now. Runs before the view is read, so the fit lands this frame.
    if state::take_fit_request() {
        crate::view::apply_fit(state, g.time_area.w, &snapshot);
    }
    // A transport jump (go-to-start/end, frame step, typed time) pans the view
    // after it, so the playhead never lands off-screen while paused.
    if state::take_reveal_request() {
        crate::view::reveal_time(state, g.time_area.w, snapshot.time_seconds);
    }

    // Resolve the time view (after the wheel landed) and share it with the ruler
    // + lanes so ticks and key diamonds align.
    let px_per_s = state.px_per_s;
    let span = f64::from(g.time_area.w) / px_per_s;
    let mut view_start = state.view_start_s;
    // Page-follow ONLY while playing: the view chases the playhead during
    // playback, but a manual pan/zoom while paused is never yanked back.
    if snapshot.playing
        && span > 0.0
        && (snapshot.time_seconds < view_start || snapshot.time_seconds >= view_start + span)
    {
        view_start = snapshot.time_seconds.max(0.0);
    }
    state.view_start_s = view_start;
    state.view_span_s = span;

    // The time axis, shared by the ruler, the lanes and every expanded graph.
    let view = graph::TimeView {
        time_x: g.time_area.x,
        right: g.rows.x + g.rows.w,
        view_start,
        px_per_s,
    };
    // A box-select that ended this frame resolves HERE, where the rows' `y` is
    // finally known (the gesture only recorded the marquee).
    crate::box_select::commit(state, g.rows, view, &snapshot);

    let preview_dx = crate::key_drag::preview_dx(state);

    // The label column's header strip, aligned with the ruler: whichever ADD this
    // tab's half is made of, and only that one. Sharing the strip was right while
    // both halves were on screen; now a "+Lane" in the Keys tab would add a lane
    // the tab cannot show — a button whose result is invisible is worse than a
    // button that is not there ([[feedback_disabled_button_still_dispatches]]).
    let header = Rect::new(g.region.x, g.region.y, g.label_w, ruler::RULER_H);
    if state.tab.shows_keys() {
        tracks::paint_add_track(ctx, theme, header);
    } else {
        crate::stack_add_header::paint_add_lane(ctx, theme, header, state.tab);
    }
    // Track rows (labels + key diamonds + expanded graph bands) below the ruler.
    tracks::paint_rows(ctx, theme, &g, view, preview_dx, state, &snapshot);
    // A handle or anchor drag whose row got culled (scrolled away, or its track
    // unbound) never reached its `resolve_drag`: close the undo bracket here so
    // the next atomic edit is not silently swallowed into it.
    if state.handle_drag.is_some_and(|d| d.ending) {
        state.handle_drag = None;
        state::push_intent(ph2d_timeline::TimelineIntent::EndEdit);
    }
    if state.anchor_drag.as_ref().is_some_and(|d| d.ending) {
        state.anchor_drag = None;
        state::push_intent(ph2d_timeline::TimelineIntent::EndEdit);
    }
    // The live box-select rubber band rides over the diamonds it is picking.
    if let Some(b) = state.box_drag {
        tracks::paint_marquee(ctx, theme, b.rect());
    }
    scrollbar::paint(ctx, theme, g.scrollbar, state, content_h);
    // Time axis last, so ticks + playhead overlay the rows. **What it measures is
    // the tab's** (`RulerClock`): the Arrange tab rules the timeline, the Keys tab
    // rules the clip — and without a stack those are one clock, which is why a
    // document that never touches the feature sees no change at all.
    let clock = crate::ruler_clock::clock_for(state.tab, &snapshot);
    ruler::paint(
        ctx,
        theme,
        g.time_area,
        view_start,
        px_per_s,
        &snapshot,
        clock,
    );

    // The label/time splitter sits over the lanes AND the ruler, so it is
    // registered after both — a grab on the seam must not scrub the ruler.
    paint_label_splitter(
        ctx,
        theme,
        g.region,
        g.region.x + g.label_w,
        state.label_drag.is_some(),
    );

    paint_overlays(
        ctx,
        theme,
        state,
        &snapshot,
        Overlays {
            body,
            header,
            time_area: g.time_area,
            clip_dd_chip,
            view_start,
            px_per_s,
        },
    );

    set_last_content_h(content_h);
    set_last_visible_h(g.rows.h);
}

/// The seam between the track-name column and the time area: a hairline, plus a
/// wider invisible strip to grab it by. Highlights while dragging.
fn paint_label_splitter(ctx: &mut PaintCtx, theme: Theme, region: Rect, x: f32, active: bool) {
    let line = Rect::new(x - SPLIT_LINE_W * 0.5, region.y, SPLIT_LINE_W, region.h);
    let tok = if active {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    fill_rounded_rect(ctx.scene, line, 0.0, resolve(tok, theme));

    let grip = Rect::new(x - SPLIT_GRIP, region.y, SPLIT_GRIP * 2.0, region.h);
    ctx.host.store_mut().register(
        ids::TIMELINE_LABEL_SPLIT,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::LabelSplitter,
            canvas: grip,
        },
    );
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_LABEL_SPLIT, grip);
}

/// The rects + view scalars the overlay pass needs (grouped so the helper takes
/// one argument instead of six — HR-12).
struct Overlays {
    body: Rect,
    header: Rect,
    time_area: Rect,
    /// Where the clip dropdown's chip landed, and whether its list is open. The
    /// option popover and the rename field both hang off it.
    clip_dd_chip: Option<transport::ClipChip>,
    view_start: f64,
    px_per_s: f64,
}

/// Everything that floats ON TOP, painted after the dope sheet — the "+Track"
/// list, the clip list, and the two inline rename fields.
///
/// They are deferred here for one reason: painted where they are AUTHORED (inside
/// the header, inside the transport bar) each would be drawn under the ruler and
/// the rows it hangs over. Same cause as the overlay-clipping bug this project
/// already paid for once — the fix is the draw ORDER, not a clamp
/// ([[feedback_overlay_cut_at_boundary_check_draw_order]]).
fn paint_overlays(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &mut TimelinePanelState,
    snapshot: &ph2d_timeline::TimelineViewSnapshot,
    o: Overlays,
) {
    tracks::paint_add_track_popover(ctx, theme, o.header, state.add_track_open);

    if let Some(chip) = o.clip_dd_chip.filter(|c| c.open).map(|c| c.rect) {
        let dd = ph2d_editor_core::widget::Dropdown::new(
            ids::TIMELINE_CLIP_DD,
            "",
            crate::transport_clips::source_options(snapshot, state.tab),
        )
        // The SAME two doors the chip paints from (`source_options`/`selected_source`), so
        // the open list and the collapsed chip cannot name different things.
        .selected(crate::transport_clips::selected_source(
            snapshot,
            crate::transport::BarView {
                tab: state.tab,
                speed_view: state.speed_view,
                source_container: state.source_container,
            },
        ))
        .open(true);
        ph2d_editor_core::widget::paint_dropdown_popover(
            &dd,
            chip,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        // Each option's hit rect is only knowable HERE, from the OPEN popover's
        // geometry — register them now, or the list paints and nothing clicks.
        for (i, opt) in dd.options.iter().enumerate() {
            ctx.host
                .hit_index_mut()
                .register(opt.id, dd.option_rect(chip, i));
        }
        // Publish the popover so a Down that lands OUTSIDE it closes the list
        // (`dispatch::pointer_down`'s light-dismiss). Without this the list stays
        // open over the dope sheet, which is the bug Enio hit — and the reason is
        // that the dismiss reads a slot no painter had ever written.
        ctx.host
            .store_mut()
            .set_dropdown_popover(ids::TIMELINE_CLIP_DD, dd.popover_rect(chip));
    }

    crate::marker_rename::paint(
        state,
        ctx,
        theme,
        o.time_area,
        o.view_start,
        o.px_per_s,
        snapshot,
    );
    // Over the CHIP it renames — not at the corner of the body, where it used to
    // float with nothing to say what it was for (Enio, 2026-07-16). The chip's rect
    // is only knowable from the bar's flow, which is why it is reported back rather
    // than re-derived here: two answers to "where is the chip" would drift the moment
    // the bar wraps to a second row.
    crate::clip_rename::paint(
        state,
        ctx,
        theme,
        o.clip_dd_chip.map_or(o.body, |c| c.rect),
        snapshot,
    );
}

/// Register the eight edge/corner grippers as `TimelineSurface` hits so dispatch
/// streams their drag to `interact::apply_resize`. Invisible by design — the
/// panel border is the affordance.
/// The panel's shell: glass, corner dots, title, close button. Returns the title's
/// size, which the body's geometry hangs off.
///
/// The title is painted HERE rather than through `paint_panel_title`, because it
/// has to sit ON the control row: the header IS the transport's first row now, and
/// a title floating on a line above it read as a stray label (Enio, 2026-07-12).
/// The shared chrome helper hard-codes its own baseline — and it is at its LOC cap,
/// so this is the one panel that owns its title placement rather than growing the
/// chrome for everyone.
fn paint_chrome(ctx: &mut PaintCtx, theme: Theme, rect: Rect) -> f32 {
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    let title_size = TypeToken::Lg.px();
    paint_text_title(
        ctx.text_system,
        ctx.scene,
        ph2d_i18n::tr("panel.timeline.title"),
        rect.x + PANEL_HEAD_PAD,
        geom::title_baseline(rect),
        title_size,
        (rect.w - PANEL_HEAD_PAD * 2.0 - PANEL_HEADER_CLOSE_RESERVE).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
    paint_panel_close_button(
        rect,
        ids::TIMELINE_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );
    title_size
}

fn register_resize_grips(ctx: &mut PaintCtx, rect: Rect) {
    for (id, edges, r) in geom::resize_grips(rect) {
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ids::TIMELINE_PANEL,
                kind: TimelineHitKind::ResizeEdge { edges },
                canvas: rect,
            },
        );
        ctx.host.hit_index_mut().register(id, r);
    }
}
