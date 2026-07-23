//! Transport bar paint (W2.E2) — the row of Play/Pause · Prev/Next-frame ·
//! seconds+frame chips · Loop/Auto-key/Snap toggles, laid out left→right at the
//! top of the panel body.
//!
//! Each control is painted from the SHARED widget source of truth and its hit
//! rect registered so the generic dispatch routes clicks/edits into
//! `event::apply_event`. The display state (play vs pause glyph, chip values,
//! toggle on/off) is mirrored from the frame's [`TimelineViewSnapshot`] into the
//! store each frame (when not focused), so the panel stays a pure view while the
//! store still drives editing.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{
    Button, ButtonState, IconButtonStyle, IconGlyph, NumberInput, Toggle, ToggleState,
    paint_button, paint_icon_button, paint_number_input_with_buffer, paint_toggle,
};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{DEFAULT_FPS, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, Density, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::ids;
use crate::tab::Tab;

pub(crate) const BTN_W: f32 = 30.0; // LITERAL-PX-OK: square transport icon-button
const ADD_MARKER_W: f32 = 40.0; // LITERAL-PX-OK: "+M" add-marker button width
const CHIP_W: f32 = 72.0; // LITERAL-PX-OK: seconds/frame number chip width
const CHIP_LABEL_W: f32 = 48.0; // LITERAL-PX-OK: "Time(s)"/"Frames" chip-label column
const TOGGLE_LABEL_W: f32 = 52.0; // LITERAL-PX-OK: "AutoKey" label column
/// The Dur(s) chip's stepper increment — **0.2 s per click** (Enio, 2026-07-23:
/// *"faça cada clique subir ou descer o valor em 0.2 seg"*). The Time/Frame chips
/// step by a frame (`1/fps`); a DURATION is coarser, so `1/fps` (≈0.04 s) produced
/// the fiddly, "wrong-looking" values the report named (4.04, 4.08, …). A round
/// 0.2 s keeps the authored duration on clean tenths.
const DUR_STEP_SECONDS: f64 = 0.2;
/// Buttons in the transport cluster: go-start, step-back, play, step-forward, go-end.
const TRANSPORT_BTNS: usize = 5;

/// The controls, in the order Enio set (2026-07-12): the CLIP everything else
/// acts on, then the transport, then **+M** right after Go-to-End, then the
/// readouts, then the toggles.
///
/// [`Item::Tabs`] leads, because it decides what everything after it is talking
/// about. It rides the flow rather than taking a row of its own for a reason the
/// panel has already been told once — *"a timeline ficou apertada"* (Enio,
/// 2026-07-12): a dock this short cannot spend 28 px on two words, and as item
/// zero the strip always lands on the title row anyway.
#[derive(Clone, Copy, PartialEq)]
enum Item {
    Tabs,
    /// The breadcrumb — where in the nesting you ARE (ADR-0133 §5).
    ///
    /// Right after the tabs and before the clip, for the same reason the tabs lead: it decides
    /// what everything after it is talking about. It is **zero-width at the scene root**, so a
    /// document that never uses containers pays nothing and sees nothing — there is no trail
    /// to show until you are inside something.
    Crumbs,
    Clips,
    Transport,
    ReverseKeys,
    AddMarker,
    TimeChip,
    FrameChip,
    LengthChip,
    Loop,
    PingPong,
    Physics,
    AutoKey,
    Record,
    Snap,
    Speed,
}

// ⚠️ A contagem se CONTA, não se escolhe: a `main` trazia 13, a `line/physics`
// acrescentou `Physics` e a `line/anim-fixes` acrescentou `Crumbs` — as duas na
// MESMA jornada, cada uma declarando 14. O valor certo (15) não estava em nenhum
// dos dois lados do conflito.
const ITEMS: [Item; 16] = [
    Item::Tabs,
    Item::Crumbs,
    Item::Clips,
    Item::Transport,
    Item::ReverseKeys,
    Item::AddMarker,
    Item::TimeChip,
    Item::FrameChip,
    // The view's explicit DURATION (Enio, 2026-07-23) sits between the readouts
    // and the loop cluster on purpose: it is about the clock's extent — what
    // go-to-end reaches and what a fresh Loop brackets.
    Item::LengthChip,
    Item::Loop,
    Item::PingPong,
    // Physics sits with Loop/PingPong, not with AutoKey/Record: those two arm
    // AUTHORING (what a gesture records), while these three describe what the
    // clock DOES when it runs — the range it covers and who it drives.
    Item::Physics,
    Item::AutoKey,
    Item::Record,
    Item::Snap,
    Item::Speed,
];

/// The panel-local VIEW state the bar reflects — what the panel is SHOWING, as
/// opposed to what the document holds. Grouped so the painters take one argument
/// instead of trailing a widening tail of bools (HR-12).
/// Where the clip dropdown's chip landed this frame, and whether its list is open.
///
/// Reported ALWAYS, not only when open: two things anchor on this chip and both are
/// painted after the dope sheet (draw order — see [`paint_bar`]). The list only exists
/// while open; the RENAME FIELD anchors on it whenever it is open, because it renames
/// *that chip* and a field floating somewhere else is a field pointing at nothing
/// (Enio, 2026-07-16: *"renomear o clip deve abrir a caixa de texto sobre o dropdown"*).
#[derive(Clone, Copy)]
pub(crate) struct ClipChip {
    /// The chip's rect — where the dropdown painted, this frame, at this width.
    pub rect: Rect,
    /// Its option list is open.
    pub open: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct BarView {
    /// Which half of the document is on screen.
    pub tab: Tab,
    /// Expanded graph bands plot velocity instead of value.
    pub speed_view: bool,
    /// The container half of the source selection (`TimelinePanelState::source_container`).
    ///
    /// **What the lane's `+` would place**, never where the view is looking: those are two
    /// questions, and the second one is answered by the Containers list you double-click.
    pub source_container: Option<usize>,
}

/// Flow the transport controls beside the panel title, wrapping onto as many rows
/// as the panel's width needs.
///
/// Returns the `y` the dope sheet starts at, and — when the clip dropdown is OPEN
/// — the chip's rect. The caller paints the popover LAST, after the dope sheet, or
/// the list would be drawn under the rows it overlaps
/// ([[feedback_overlay_cut_at_boundary_check_draw_order]]).
///
/// **As many rows as it takes** (Enio, 2026-07-12). Row 1 is the strip beside the
/// title, and it is SHORT — the title and the close button both eat into it — so
/// the tail spills below. Each further row is the full body width. Wrapping only
/// ONCE was the bug in the first cut: everything past the second row's edge simply
/// painted outside the panel, where it could be seen and not clicked. A row is
/// only ever spent when an item needs it, so a wide panel still keeps the whole bar
/// on the title line and hands the dope sheet the space back.
pub(crate) fn paint_bar(
    ctx: &mut PaintCtx,
    theme: Theme,
    header: Rect,
    body: Rect,
    snap: &TimelineViewSnapshot,
    view: BarView,
) -> (f32, Option<ClipChip>) {
    let gap = Spacing::Sm.px();
    let row_step = ROW_H_PX + gap;
    let mut clip_chip = None;
    // Row 0 is the header strip; row 1+ are full-width rows in the body.
    let mut row = header;
    let mut body_rows = 0usize;
    let mut x = row.x;

    for item in ITEMS {
        let w = width(item, snap, view);
        // Does not fit? Take the next row. Never split a cluster across rows — a
        // short row reads better than a broken control.
        if x > row.x && x + w > row.x + row.w {
            body_rows += 1;
            row = Rect::new(
                body.x,
                body.y + (body_rows - 1) as f32 * row_step,
                body.w,
                ROW_H_PX,
            );
            x = row.x;
        }
        if let Some(chip) = paint_item(ctx, theme, item, x, row.y, snap, view) {
            clip_chip = Some(chip);
        }
        x += w + gap;
    }

    (body.y + body_rows as f32 * row_step, clip_chip)
}

/// How wide `item` paints. The single source the flow measures against — every
/// painter below lays out from these same constants, so the fit test and the
/// pixels cannot disagree.
fn width(item: Item, snap: &TimelineViewSnapshot, view: BarView) -> f32 {
    let gap = Spacing::Sm.px();
    let half = gap * 0.5;
    match item {
        // One cell per tab, side by side inside the segmented pill.
        Item::Tabs => crate::transport_tabs::width(),
        // [ Main v ] [+] [copy] [pencil] [trash] — the trash only exists above one
        // clip. Duplicate sits beside the `+` that made the clip (Enio, 2026-07-16):
        // they are the two ways to get a clip, and the difference between them is
        // whether it starts empty.
        Item::Crumbs => crate::breadcrumb::width(snap),
        Item::Clips => crate::transport_clips::width(snap, view.tab),
        // |< < >/|| > >| — five buttons, four gaps between them.
        Item::Transport => {
            let n = TRANSPORT_BTNS as f32;
            BTN_W * n + half * (n - 1.0)
        }
        Item::ReverseKeys => BTN_W,
        Item::AddMarker => ADD_MARKER_W,
        Item::TimeChip | Item::FrameChip | Item::LengthChip => CHIP_LABEL_W + half + CHIP_W,
        Item::Loop
        | Item::PingPong
        | Item::Physics
        | Item::AutoKey
        | Item::Record
        | Item::Snap
        | Item::Speed => toggle_w(),
    }
}

/// The outlined `[label | switch]` cell's width (mirrors [`toggle`]'s layout).
fn toggle_w() -> f32 {
    let pad = Spacing::Xs.px();
    pad + TOGGLE_LABEL_W + pad + TypeToken::Xl3.px() + pad
}

/// Paint one item at `(x, y)`. Returns the clip chip's rect when that item is the
/// clip dropdown AND it is open (the caller defers the popover — see [`paint_bar`]).
/// **Which duration the Dur(s) chip shows AND writes — THE VIEW you are looking
/// at, always** (Enio, 2026-07-23, smoke: *"nada escuro … o playhead ultrapassa"*).
/// One door for paint and for the event router, so what you read and what your
/// typing edits can never diverge.
///
/// The first cut let the source DROPDOWN hijack the scope, and the smoke showed
/// why that is a trap: you type a duration while looking at the Arrange, the
/// value lands on the container the dropdown happens to name, and the view you
/// are watching never closes — no shade, no clamp, and the box holds a number
/// that edited something off-screen. A write that goes somewhere invisible is a
/// control that lies. The container's own duration is edited INSIDE it (the
/// double-click), where its ruler, shade and clamp are on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LengthScope {
    /// The container open for editing (its interior is the view).
    Container(usize),
    /// The active clip (the Keys view).
    Clip,
    /// The scene (Arrange).
    Scene,
}

/// See [`LengthScope`].
pub(crate) fn length_scope(container_open: Option<usize>, tab: Tab) -> LengthScope {
    if let Some(c) = container_open {
        LengthScope::Container(c)
    } else if tab == Tab::Keys {
        LengthScope::Clip
    } else {
        LengthScope::Scene
    }
}

/// The Dur(s) chip: **the view's own duration** (`view_length_seconds` is
/// stamped per view — clip in Keys, scene in Arrange, the open container
/// inside one). The router writes the same scope (`length_scope`).
fn paint_length_chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    snap: &TimelineViewSnapshot,
) {
    labeled_chip(
        ctx,
        theme,
        x,
        y,
        "panel.timeline.length",
        ids::TIMELINE_LENGTH_NUM,
        snap.view_length_seconds,
        DUR_STEP_SECONDS,
        2,
    );
}

/// One labeled numeric chip — the label at `x`, the chip a half-gap after it.
/// The three transport chips (Time / Frame / Dur) are this one drawing, and the
/// measure arm (`CHIP_LABEL_W + half + CHIP_W`) describes exactly this layout.
#[allow(clippy::too_many_arguments)]
fn labeled_chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    key: &str,
    id: ph2d_a11y::NodeId,
    value: f64,
    step: f64,
    decimals: usize,
) {
    let half = Spacing::Sm.px() * 0.5;
    label(ctx, theme, ph2d_i18n::tr(key), x, y, CHIP_LABEL_W);
    chip(
        ctx,
        theme,
        x + CHIP_LABEL_W + half,
        y,
        id,
        value,
        step,
        decimals,
    );
}

fn paint_item(
    ctx: &mut PaintCtx,
    theme: Theme,
    item: Item,
    x: f32,
    y: f32,
    snap: &TimelineViewSnapshot,
    view: BarView,
) -> Option<ClipChip> {
    let gap = Spacing::Sm.px();
    let half = gap * 0.5;
    let fps = if snap.fps > 0.0 {
        snap.fps
    } else {
        DEFAULT_FPS
    };
    match item {
        Item::Tabs => crate::transport_tabs::paint(ctx, theme, x, y, view.tab),
        Item::Crumbs => crate::breadcrumb::paint(ctx, theme, x, y, snap),
        Item::Clips => return crate::transport_clips::cluster(ctx, theme, x, y, snap, view),
        Item::Transport => {
            // |< < >/|| > >| — jump to start, step back, play/pause, step forward,
            // jump to end. The skip glyphs bracket the frame-steppers, as every
            // transport does.
            let mut x =
                icon_button(ctx, theme, x, y, ids::TIMELINE_GO_START, IconId::SkipBack) + half;
            x = icon_button(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_PREV_FRAME,
                IconId::ChevronLeft,
            ) + half;
            let play_glyph = if snap.playing {
                IconId::Pause
            } else {
                IconId::Play
            };
            // **The Containers LIST has no playback mode** (Enio, 2026-07-22): the
            // list is a library of assets, not a view of time, so play/pause paints
            // DEAD there — disabled AND unhittable, because a dimmed control that
            // still dispatches lies ([[feedback_widget_is_done_when_a_test_clicks_it]]).
            // It comes back inside a container, on Keys and on Arrange, where the
            // same door (`tab::rows`) stops answering `Containers`.
            x = if crate::tab::rows(view.tab, snap) == crate::tab::Rows::Containers {
                widgets::dead_icon_button(ctx, theme, x, y, play_glyph)
            } else {
                icon_button(ctx, theme, x, y, ids::TIMELINE_PLAY, play_glyph)
            } + half;
            x = icon_button(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_NEXT_FRAME,
                IconId::ChevronRight,
            ) + half;
            icon_button(ctx, theme, x, y, ids::TIMELINE_GO_END, IconId::SkipForward);
        }
        // **I** — the active clip's keys, played backwards. Beside the transport it
        // reads as what it is: a thing you do to the whole clip, not to a selection.
        Item::ReverseKeys => {
            icon_button(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_REVERSE_CLIP,
                IconId::LetterI,
            );
        }
        Item::AddMarker => add_marker_button(ctx, theme, x, y),
        // The three chips share one drawing (label + numeric), so they share one
        // painter — a fourth copy of the pair is how the label and the chip come
        // to disagree about the half-gap between them. `LengthChip` is the view's
        // EFFECTIVE duration (Enio, 2026-07-23): derived from content, or the
        // authored override when one is set; typing writes the scope on screen
        // (clip in Keys, the open container inside one, the scene in Arrange)
        // and 0 clears back to derived.
        Item::TimeChip => labeled_chip(
            ctx,
            theme,
            x,
            y,
            "panel.timeline.time_seconds",
            ids::TIMELINE_TIME_NUM,
            snap.time_seconds,
            1.0 / fps,
            2,
        ),
        Item::FrameChip => labeled_chip(
            ctx,
            theme,
            x,
            y,
            "panel.timeline.frame",
            ids::TIMELINE_FRAME_NUM,
            snap.frame as f64,
            1.0,
            0,
        ),
        Item::LengthChip => paint_length_chip(ctx, theme, x, y, snap),
        // Loop and PingPong are the SAME loop seen two ways, so exactly one can
        // read as on — the snapshot carries a range plus a mode, and there is no
        // value that is both.
        Item::Loop => {
            toggle(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_LOOP,
                ph2d_i18n::tr("panel.timeline.loop"),
                snap.loop_range.is_some() && !snap.loop_ping_pong,
            );
        }
        Item::PingPong => {
            toggle(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_PINGPONG,
                ph2d_i18n::tr("panel.timeline.ping_pong"),
                snap.loop_range.is_some() && snap.loop_ping_pong,
            );
        }
        // Physics (ADR-0131) — one transport, two consumers. Reads from the
        // snapshot like every other document-backed toggle, so the painted
        // switch cannot disagree with what the clock is actually driving.
        Item::Physics => {
            toggle(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_PHYSICS,
                ph2d_i18n::tr("panel.timeline.physics"),
                snap.simulate_physics,
            );
        }
        Item::AutoKey => {
            toggle(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_AUTOKEY,
                ph2d_i18n::tr("panel.timeline.autokey"),
                snap.auto_key,
            );
        }
        // Record / performing (W5) — records the pose LIVE while playing +
        // dragging, sits next to AutoKey (its paused counterpart).
        Item::Record => {
            toggle(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_RECORD,
                ph2d_i18n::tr("panel.timeline.record"),
                snap.performing,
            );
        }
        Item::Snap => {
            toggle(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_SNAP,
                ph2d_i18n::tr("panel.timeline.snap"),
                snap.frame_snap,
            );
        }
        // Speed-graph view (W5) — panel-local, NOT a document command: flips every
        // expanded graph band between the value curve and the velocity curve.
        Item::Speed => {
            toggle(
                ctx,
                theme,
                x,
                y,
                ids::TIMELINE_SPEED,
                ph2d_i18n::tr("panel.timeline.speed"),
                view.speed_view,
            );
        }
    }
    None
}

/// Paint the "+M" add-marker button + register its hit. `+` matches the
/// "+Track" convention (the affordance that ADDS something).
fn add_marker_button(ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32) {
    let rect = Rect::new(x, y, ADD_MARKER_W, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(ids::TIMELINE_ADD_MARKER)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(
        ids::TIMELINE_ADD_MARKER,
        ph2d_i18n::tr("panel.timeline.add_marker"),
    )
    .state(state);
    paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_ADD_MARKER, rect);
}

/// The bar's reusable pieces (icon-button, number chip, toggle, label) — a sibling module
/// under the 600-LOC panel cap. They are the transport's *vocabulary*: nothing in here knows
/// what a playhead is, which is exactly why they were the honest slice to cut.
#[path = "transport_widgets.rs"]
mod widgets;
pub(crate) use widgets::{chip, icon_button, label, mirror_number, toggle};

#[cfg(test)]
#[path = "transport_playback_tests.rs"]
mod playback_tests;
