//! The clip cluster in the transport bar: `[ Main ▾ ] [+] [copy] [✎] [🗑]`.
//!
//! Its own module rather than more arms of `transport.rs`, which is at the 600-line cap
//! (HR-18): the cluster is one coherent thing — the chip, the four buttons that act on the
//! clip it names, and the option list — so it is the honest seam to cut on.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, DropdownState, paint_dropdown_chip};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_timeline::TimelineIntent;

use crate::ids;
use crate::state::{self, TimelinePanelState};
use crate::transport::{BTN_W, ClipChip, icon_button};

const CLIP_DD_W: f32 = 108.0; // LITERAL-PX-OK: clip dropdown chip width

/// How wide the cluster paints — the single source `transport`'s flow measures against.
pub(crate) fn width(snap: &TimelineViewSnapshot) -> f32 {
    let half = Spacing::Sm.px() * 0.5;
    // [ Main v ] [+] [copy] [pencil] [trash] — the trash only exists above one clip.
    // Duplicate sits beside the `+` that made the clip (Enio, 2026-07-16): they are the
    // two ways to get a clip, and the difference is only whether it starts empty.
    let trash = if snap.clips.len() > 1 {
        BTN_W + half
    } else {
        0.0
    };
    CLIP_DD_W + half + BTN_W + half + BTN_W + half + BTN_W + half + trash
}

/// The clip cluster: `[ Main ▾ ] [+] [✎] [🗑]`.
///
/// Returns the `x` after it, and the chip rect when the dropdown is open (the
/// caller defers the popover paint — see [`paint_bar`]).
///
/// The TRASH is not painted, and — the part that matters —  **not hit-registered**,
/// while the document holds a single clip: a document must always have one to edit,
/// and a dimmed button that still dispatches is a click that silently does nothing
/// ([[feedback_disabled_button_still_dispatches]]).
pub(crate) fn cluster(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    snap: &TimelineViewSnapshot,
) -> Option<ClipChip> {
    let gap = Spacing::Sm.px();
    let mut x = x;

    let chip = Rect::new(x, y, CLIP_DD_W, ROW_H_PX);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_CLIP_DD, chip);
    let (state, open) = match ctx.host.store().get(ids::TIMELINE_CLIP_DD) {
        Some(InteractiveState::Dropdown { state, open, .. }) => (*state, *open),
        _ => (DropdownState::Normal, false),
    };
    let dd = Dropdown::new(ids::TIMELINE_CLIP_DD, "", clip_options(snap))
        .selected(snap.active_clip)
        .open(open)
        .state(state);
    paint_dropdown_chip(&dd, chip, ctx.scene, ctx.text_system, theme);
    x += CLIP_DD_W + gap * 0.5;

    x = icon_button(ctx, theme, x, y, ids::TIMELINE_ADD_CLIP, IconId::Plus) + gap * 0.5;
    x = icon_button(ctx, theme, x, y, ids::TIMELINE_DUP_CLIP, IconId::Duplicate) + gap * 0.5;
    x = icon_button(ctx, theme, x, y, ids::TIMELINE_RENAME_CLIP, IconId::Text) + gap * 0.5;
    if snap.clips.len() > 1 {
        icon_button(ctx, theme, x, y, ids::TIMELINE_DELETE_CLIP, IconId::Trash);
    }

    Some(ClipChip { rect: chip, open })
}

/// One dropdown option per clip: the VALUE is the clip's index (what the dispatch
/// needs) and the label is its name (what the animator reads). Truncated at the id
/// array — a clip past it could be painted but never clicked, so
/// `ph2d_timeline::MAX_CLIPS` refuses to create one and a gate holds the two equal.
pub(crate) fn clip_options(snap: &TimelineViewSnapshot) -> Vec<DropdownOption<usize>> {
    snap.clips
        .iter()
        .enumerate()
        .take(ids::TIMELINE_CLIP_OPT.len())
        .map(|(i, name)| DropdownOption::new(ids::TIMELINE_CLIP_OPT[i], i, name.clone()))
        .collect()
}

/// Whether `ev` is the clip cluster's to answer.
///
/// Every id here is one this module's [`apply_event`] has an arm for. A router that
/// enumerated them separately from the arms would drift, and the drift is silent in the
/// direction that matters: an event CLAIMED and not handled is a control that clicks and
/// does nothing. The seam gate clicks each of them for exactly that reason.
pub(crate) fn owns(ev: &WidgetEvent) -> bool {
    let id = match *ev {
        WidgetEvent::Click(id)
        | WidgetEvent::Submit(id)
        | WidgetEvent::Blur(id)
        | WidgetEvent::Cancel(id) => id,
        _ => return false,
    };
    id == ids::TIMELINE_CLIP_DD
        || id == ids::TIMELINE_ADD_CLIP
        || id == ids::TIMELINE_DUP_CLIP
        || id == ids::TIMELINE_REVERSE_CLIP
        || id == ids::TIMELINE_RENAME_CLIP
        || id == ids::TIMELINE_DELETE_CLIP
        || id == ids::TIMELINE_CLIP_RENAME_INPUT
        || ids::TIMELINE_CLIP_OPT.contains(&id)
}

/// Answer one clip-cluster event. Only reached for events [`owns`] claimed.
pub(crate) fn apply_event(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        // ── Clip selector (W5) ──────────────────────────────────────────────
        // Picking a clip from the open list. The store's `selected_index` is set
        // too, so the chip reads right on the SAME frame — the document round-trip
        // only lands on the next one.
        WidgetEvent::Click(id) if ids::TIMELINE_CLIP_OPT.contains(&id) => {
            if let Some(index) = ids::TIMELINE_CLIP_OPT.iter().position(|&o| o == id) {
                state::push_intent(TimelineIntent::SetActiveClip { index });
                if let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(ids::TIMELINE_CLIP_DD)
                {
                    *open = false;
                    *selected_index = Some(index);
                }
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_ADD_CLIP => {
            state::push_intent(TimelineIntent::AddClip);
            EventOutcome::Consumed
        }
        // Duplicate the ACTIVE clip — the sibling of `+`, and the difference is only
        // whether the clip starts empty. Refused past `MAX_CLIPS` by the document.
        WidgetEvent::Click(id) if id == ids::TIMELINE_DUP_CLIP => {
            state::push_intent(TimelineIntent::DuplicateClip {
                index: state::current_snapshot().active_clip,
            });
            EventOutcome::Consumed
        }
        // **I** — play the ACTIVE clip backwards.
        WidgetEvent::Click(id) if id == ids::TIMELINE_REVERSE_CLIP => {
            state::push_intent(TimelineIntent::ReverseClip {
                index: state::current_snapshot().active_clip,
            });
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_RENAME_CLIP => {
            crate::clip_rename::open(state, &state::current_snapshot());
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_DELETE_CLIP => {
            // The SECOND barrier: the paint does not even hit-register the trash
            // while a single clip remains, but a dimmed control that still
            // dispatches is precisely the bug that guard is for — so refuse here
            // too, and let the document refuse a third time
            // ([[feedback_disabled_button_still_dispatches]]).
            let snap = state::current_snapshot();
            if snap.clips.len() > 1 {
                state::push_intent(TimelineIntent::DeleteClip {
                    index: snap.active_clip,
                });
            }
            EventOutcome::Consumed
        }
        // Clip rename field — same Enter/click-away/Esc contract as the marker's.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id)
            if id == ids::TIMELINE_CLIP_RENAME_INPUT =>
        {
            crate::clip_rename::commit(state, host.store());
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::TIMELINE_CLIP_RENAME_INPUT => {
            crate::clip_rename::cancel(state);
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
