//! **The Expression card is MODAL over its own frame** (FASE 0.1 do plano 12).
//!
//! The card is painted last, so it is on top — and it registered nothing for its own
//! background, so it was **transparent to the pointer**. Measured by the audit of
//! 2026-07-29 (§4-bis U1/U3):
//!
//! * **18 named transport widgets** were live inside the card's footprint, and clicking
//!   the centre of the formula bar returned `TIMELINE_LENGTH_NUM` — typing there edited
//!   the composition's `Dur(s)`;
//! * a wheel over the card zoomed the timeline behind it, `px_per_s` **120 → 326**.
//!
//! ⚠️ **Why the 23 existing seam gates were all green over this:** every one of them
//! looks an id up BY NAME (`regs.iter().find(|(id, _)| *id == target)`) and asserts it is
//! there. Not one asks *"what ELSE is alive here?"* — which is the only question that can
//! see a widget losing the hit to something painted over it, or winning a hit it should
//! have lost. These two gates ask that question instead, and they take the answer from
//! the paint the artist gets, never from a list written by hand.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_ui_testkit::MockPanelHost;

/// The audit's viewport — big enough that the centred card lands squarely on the
/// transport, which is the situation being gated.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// Publish one track and return its raw `AnimTarget`.
fn publish_one_track(entity: u64, prop: ph2d_timeline::PropKind) -> u64 {
    use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, TimelineIntent::Bind { entity, prop });
    let target = st.doc.binding_for(entity, prop).unwrap().target.get();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    target
}

/// Open the card the way the artist does — through the track menu's row.
fn open_modal(host: &mut MockPanelHost, state: &mut TimelinePanelState, target: u64) {
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 50.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu();
    host.apply_panel_event::<TimelinePanel>(state, WidgetEvent::Click(ids::CTX_MENU_TL_EXPR));
}

/// A grid of probe points inside `r`, inset so the frame's own border is not sampled.
fn probes(r: Rect) -> Vec<(f32, f32)> {
    const STEPS: usize = 12;
    let mut out = Vec::new();
    for iy in 0..STEPS {
        for ix in 0..STEPS {
            let fx = (ix as f32 + 0.5) / STEPS as f32;
            let fy = (iy as f32 + 0.5) / STEPS as f32;
            out.push((r.x + r.w * fx, r.y + r.h * fy));
        }
    }
    out
}

/// **Every pointer inside the card's frame lands on the card.**
///
/// The oracle is a DIFFERENCE between two paints, not a hardcoded list of ids: whatever
/// the panel registers with the card closed is, by construction, "the panel"; with the
/// card open, none of those may win a hit inside the frame. A list written here would
/// drift from the transport the day someone adds a button to it.
#[test]
fn the_card_swallows_every_pointer_inside_its_frame() {
    let target = publish_one_track(11, ph2d_timeline::PropKind::TranslationX);

    // Pass 1 — no card. These are the panel's own widgets.
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let panel_regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let panel_ids: Vec<NodeId> = panel_regs.iter().map(|(id, _)| *id).collect();

    // Pass 2 — the card, opened through the menu, painted twice so its remembered
    // position is the one the second paint reports (the first centres it).
    open_modal(&mut host, &mut state, target);
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let frame = regs
        .iter()
        .find(|(id, _)| *id == ids::EXPR_MODAL_SCRIM)
        .map(|(_, r)| *r)
        .expect("the open card registers its frame");

    // ⚠️ **The fixture has to CONTAIN the phenomenon.** If the card did not overlap the
    // transport there would be nothing to swallow, and a card floating over empty space
    // would make this gate pass while saying nothing.
    let underneath: Vec<NodeId> = panel_regs
        .iter()
        .filter(|(_, r)| {
            r.x < frame.x + frame.w
                && r.x + r.w > frame.x
                && r.y < frame.y + frame.h
                && r.y + r.h > frame.y
        })
        .map(|(id, _)| *id)
        .collect();
    assert!(
        underneath.len() >= 10,
        "the card must land ON the transport for this gate to mean anything; \
         only {} panel widgets overlap its frame",
        underneath.len()
    );

    for (x, y) in probes(frame) {
        let hit = host.hit_at(x, y).unwrap_or_else(|| {
            panic!("({x}, {y}) is inside the card and hit NOTHING — the frame is not registered")
        });
        assert!(
            !panel_ids.contains(&hit),
            "a pointer at ({x}, {y}) — inside the card — landed on a PANEL widget \
             ({hit:?}). The card is painted on top; it must be opaque to the pointer."
        );
    }
}

/// **The open card refuses the wheel inside its frame, and only inside it.**
///
/// ⚠️ Two halves in one gate on purpose: refusing everywhere would be a card that breaks
/// the dope sheet's zoom, which is the same defect wearing the opposite sign. The control
/// (a wheel just outside the frame still zooms) is what makes the refusal a statement
/// about the FRAME rather than about the wheel.
#[test]
fn the_open_card_refuses_the_wheel_inside_its_frame_but_not_outside() {
    let target = publish_one_track(12, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let frame = regs
        .iter()
        .find(|(id, _)| *id == ids::EXPR_MODAL_SCRIM)
        .map(|(_, r)| *r)
        .expect("the open card registers its frame");

    let zoom_at = |host: &mut MockPanelHost, state: &mut TimelinePanelState, x: f32, y: f32| {
        let before = state.px_per_s;
        host.store_mut()
            .add_timeline_wheel(ids::TIMELINE_PANEL, 120.0, 0.0, 0.0, x, y);
        host.paint::<TimelinePanel>(state, VIEWPORT);
        (before, state.px_per_s)
    };

    let centre = (frame.x + frame.w * 0.5, frame.y + frame.h * 0.5);
    let (before, after) = zoom_at(&mut host, &mut state, centre.0, centre.1);
    assert_eq!(
        before, after,
        "a wheel at the centre of the open card zoomed the timeline behind it \
         ({before} -> {after} px/s)"
    );

    // The control: the same wheel, one pixel-and-a-bit LEFT of the frame.
    let outside = (frame.x - 8.0, centre.1);
    assert!(
        outside.0 > VIEWPORT.x,
        "the control point must be inside the viewport"
    );
    let (before, after) = zoom_at(&mut host, &mut state, outside.0, outside.1);
    assert_ne!(
        before, after,
        "a wheel OUTSIDE the card must still zoom the dope sheet — refusing everywhere \
         is the same bug with the opposite sign"
    );
}
