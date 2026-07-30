//! The property-EXPRESSION seam (ADR-0144 · plano 10 W1), driven through the REAL
//! panel: the track menu's "Expression\u{2026}" row opens the **modal**, a gallery
//! card adds a row to the sheet, and Apply raises `SetBindingExpr` with the
//! formula the sheet projects.
//!
//! ⚠️ These gates replace the inline-field ones. The field was DELETED in W1 —
//! not because it was wrong, but because the modal subsumes it: the catalog's
//! `Custom Formula` recipe **is** a text field, and the card seeds itself from
//! whatever formula the track already carries. Keeping a routed, painted widget
//! with no opener is the rot this repo names; keeping a second menu row for it
//! would be two doors onto one question.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_ui_testkit::MockPanelHost;

/// Publish one track and return its raw `AnimTarget` (mirrors `extrapolation_seam`).
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

/// Open+park the track menu for `target`, then click the "Expression\u{2026}" row.
fn open_modal(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    target: u64,
) -> EventOutcome {
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 50.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu();
    host.apply_panel_event::<TimelinePanel>(state, WidgetEvent::Click(ids::CTX_MENU_TL_EXPR))
}

/// **The menu row opens the card.**
///
/// ⚠️ This is the gate that caught the row going dead during W1: the modal's
/// router guards on "is a card open?", and the menu click is the one event that
/// arrives when there is none — so routing it after the guard silently orphans
/// the row that opens the whole feature.
#[test]
fn the_expression_menu_row_opens_the_modal() {
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(7, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    assert!(
        state.expr_modal.is_none(),
        "no card before the row is clicked"
    );
    let out = open_modal(&mut host, &mut state, target);
    assert_eq!(out, EventOutcome::Consumed, "the row is consumed");
    let m = state
        .expr_modal
        .as_ref()
        .expect("the Expression row opens the card");
    assert_eq!(m.target, target);
    assert!(m.stack.rows.is_empty(), "a fresh card starts with no rows");
}

/// **A gallery card adds its recipe to the sheet, and Apply authors what the
/// sheet projects.**
#[test]
fn picking_a_recipe_and_applying_raises_the_projected_formula() {
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(8, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    let out = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("shake")),
    );
    assert_eq!(out, EventOutcome::Consumed, "a gallery card is consumed");
    let want = state
        .expr_modal
        .as_ref()
        .expect("still open")
        .stack
        .to_formula();
    assert!(
        want.contains("wiggle"),
        "the sheet holds a Shake row: {want}"
    );

    let out = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::EXPR_MODAL_APPLY));
    assert_eq!(out, EventOutcome::Consumed);
    assert!(state.expr_modal.is_none(), "Apply closes the card");
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::SetBindingExpr {
            target: ph2d_timeline::AnimTarget::new(target),
            expr: Some(want),
        }],
        "Apply authors exactly what the formula bar showed"
    );
}

/// **Opening and cancelling leaves the document untouched** (plano 10 §8, G14).
///
/// Both dismissals are checked, because a card you can leave two ways that leaves
/// differently is a card you cannot learn.
#[test]
fn opening_and_dismissing_the_modal_authors_nothing() {
    for dismiss in [ids::EXPR_MODAL_CANCEL, ids::EXPR_MODAL_CLOSE] {
        let _ = ph2d_panel_timeline::drain_intents();
        let target = publish_one_track(9, ph2d_timeline::PropKind::Rotation);
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        open_modal(&mut host, &mut state, target);
        host.apply_panel_event::<TimelinePanel>(
            &mut state,
            WidgetEvent::Click(ids::expr_gallery_id("shake")),
        );

        let out = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(dismiss));
        assert_eq!(out, EventOutcome::Consumed);
        assert!(state.expr_modal.is_none(), "{dismiss:?} closes the card");
        assert_eq!(
            ph2d_panel_timeline::drain_intents(),
            vec![],
            "{dismiss:?} must author NOTHING, even with rows on the sheet"
        );
    }
}

/// **An empty sheet clears the expression** (back to keyframes).
///
/// The empty stack projects to `"value"` — the identity — and authoring that
/// would pin a formula that says "leave it alone", which is what having no
/// formula already means.
#[test]
fn applying_an_empty_sheet_clears_the_expression() {
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(10, ph2d_timeline::PropKind::ScaleX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::EXPR_MODAL_APPLY));
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::SetBindingExpr {
            target: ph2d_timeline::AnimTarget::new(target),
            expr: None,
        }],
        "an empty sheet clears the expression"
    );
}

/// **A family card walks INTO the family, and `..` walks back out.**
///
/// The gallery shows either the nine families or ONE family's recipes — that is
/// what makes 55 cards fit a card that does not scroll.
#[test]
fn the_gallery_walks_into_a_family_and_back_out() {
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(11, ph2d_timeline::PropKind::ScaleY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    let page = |s: &TimelinePanelState| s.expr_modal.as_ref().unwrap().page;
    assert_eq!(
        page(&state),
        ph2d_panel_timeline::expr_modal::GalleryPage::Families,
        "the card opens on the families"
    );
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("Life")),
    );
    assert_eq!(
        page(&state),
        ph2d_panel_timeline::expr_modal::GalleryPage::Family(ph2d_expr_recipes::Family::Life),
    );
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("..")),
    );
    assert_eq!(
        page(&state),
        ph2d_panel_timeline::expr_modal::GalleryPage::Families,
        "`..` walks back out"
    );
}

/// **A pair recipe inserts BOTH halves.** Half a circle is not a feature.
#[test]
fn a_pair_recipe_inserts_both_of_its_rows() {
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(12, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("orbit-x")),
    );
    let rows: Vec<_> = state
        .expr_modal
        .as_ref()
        .unwrap()
        .stack
        .rows
        .iter()
        .map(|r| r.recipe)
        .collect();
    assert_eq!(rows, vec!["orbit-x", "orbit-y"], "one click, both halves");
}

/// **The bypass eye and the remove button reach the sheet.**
#[test]
fn a_row_can_be_bypassed_and_removed() {
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(13, ph2d_timeline::PropKind::Opacity);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("shake")),
    );

    let with = state.expr_modal.as_ref().unwrap().stack.to_formula();
    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::expr_bypass_id(0)));
    let bypassed = state.expr_modal.as_ref().unwrap().stack.to_formula();
    assert_ne!(with, bypassed, "the eye mutes the row");
    assert_eq!(bypassed, "value", "a muted row contributes nothing");

    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::expr_remove_id(0)));
    assert!(
        state.expr_modal.as_ref().unwrap().stack.rows.is_empty(),
        "the X removes the row"
    );
}

/// **The gallery cards are alive under a REAL pointer.**
///
/// ⚠️ Every gate above dispatches a synthetic `WidgetEvent::Click`, which skips
/// the store's focusability check — so all of them stay green over a card that is
/// painted, hit-registered and **dead under the mouse**. That pair is the exact
/// failure the physics panel paid for with its 36 collision-matrix cells, and it
/// is why this one paints the card and then clicks a PIXEL.
#[test]
fn a_gallery_card_is_reachable_by_a_real_pointer() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(14, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    // Paint once so the card registers its hit rects AND its store entries.
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let life = ids::expr_gallery_id("Life");
    let rect = regs
        .iter()
        .find(|(id, _)| *id == life)
        .map(|(_, r)| *r)
        .expect("the Life family card is painted");

    let evs = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(id) if *id == life)),
        "a pointer over the Life card must produce its Click; got {evs:?}"
    );
    for ev in evs {
        host.apply_panel_event::<TimelinePanel>(&mut state, ev);
    }
    assert_eq!(
        state.expr_modal.as_ref().unwrap().page,
        ph2d_panel_timeline::expr_modal::GalleryPage::Family(ph2d_expr_recipes::Family::Life),
        "and the click walks the gallery into that family"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// **A knob's NUMBER BOX is alive under a real pointer, and what it commits
/// reaches the formula.**
///
/// ⚠️ Written because the slider→box swap (Enio, smoke de 2026-07-29: *"no lugar
/// de sliders, melhor apenas caixas de input numérico"*) passed **eighteen green
/// gates that never touched a knob**. Everything above drives the gallery, the
/// footer and the title band; the widget the artist spends all their time in had no
/// gate at all, so replacing it was indistinguishable from deleting it.
///
/// The gesture is the stepper ARROW at a real pixel, which is the shortest path
/// that crosses every seam at once: hit rect → dispatch → the registered range's
/// `step` → the store's committed value → `sync_from_store` → `to_formula`. It also
/// pins the step: without a REGISTERED range the dispatch falls back to a buffer
/// heuristic and one click moves `1.0` — three canvases on an Amount of `0.3`.
#[test]
fn a_knob_box_steps_by_its_own_increment_under_a_real_pointer() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(21, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("shake")),
    );

    // Knob 1 of row 0 is Shake's Amount.
    let knob = ids::expr_knob_id(0, 1);
    let amount = ph2d_expr_recipes::by_id("shake").unwrap().knobs[1];
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let rect = regs
        .iter()
        .find(|(id, _)| *id == knob)
        .map(|(_, r)| *r)
        .expect("the Amount knob paints a box");

    let before = state.expr_modal.as_ref().unwrap().stack.to_formula();
    // The UP arrow: asked of the widget itself, which is the same door the
    // dispatch's hit-test uses — a rect derived here would be a second answer.
    let up = ph2d_editor_core::widget::NumberInput::new(knob, "", 0.0).up_rect(rect);
    for ev in host.click_at(up.x + up.w * 0.5, up.y + up.h * 0.5) {
        host.apply_panel_event::<TimelinePanel>(&mut state, ev);
    }
    // Repaint: the read-back into the stack happens at the top of the paint.
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let after = state.expr_modal.as_ref().unwrap().stack.to_formula();

    assert_ne!(
        before, after,
        "one click on the stepper must reach the formula"
    );
    let want = ph2d_expr_recipes::fmt_num(amount.default + amount.step_value());
    assert!(
        after.contains(&want),
        "the box steps by the knob's own increment ({}): wanted {want} in {after}",
        amount.step_value()
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

// ─────────────────────────── W2 — the live preview ───────────────────────────

/// **The preview evaluates the window through the PRODUCT's evaluator** (P3), and
/// what the curve strip plots is what the GHOST stands on.
///
/// ⚠️ ONE door, checked as one: the ghost indexes the SAME vector the strip draws.
/// A ghost that sampled on its own would drift from the curve beside it, and the
/// artist would have no way to tell which of the two lies — which matters MORE now
/// that the two live on separate cards.
#[test]
fn the_preview_samples_the_window_once_and_both_views_read_it() {
    use ph2d_expr_recipes::RecipeStack;
    use ph2d_panel_timeline::expr_modal_preview as pv;

    // A Sway is a sine: over a 2 s window it must leave the baseline in BOTH
    // directions, which a constant or a clamped-to-zero evaluation cannot do.
    //
    // ⚠️ **And `sway` is exactly why this gate was blind to D-J**: a pure sine never
    // reads `__seed`, so the ribbon could feed the wrong seed — or none — with this
    // green. The seed is gated by `the_ribbon_draws_what_the_object_does`, whose
    // fixture is a `jitter`; the fixture has to contain the phenomenon.
    let stack = RecipeStack::of(&["sway"]);
    let base = pv::preview_value(ph2d_timeline::PropKind::TranslationX);
    let s = pv::sample_window(&stack, base, 3);
    assert_eq!(s.len(), pv::PREVIEW_SAMPLES);
    assert!(
        s.iter().any(|v| *v > base) && s.iter().any(|v| *v < base),
        "a Sway crosses the baseline both ways: {:?}..{:?}",
        s.iter().cloned().fold(f32::INFINITY, f32::min),
        s.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
    );
    assert!(s.iter().all(|v| v.is_finite()), "no sample is non-finite");
}

/// **A flat curve still has a span.** The empty sheet projects to `value`, which
/// is the most common thing the column will ever be asked to draw — and
/// normalising by a zero extent is how a preview goes blank on it.
#[test]
fn a_flat_curve_still_has_a_span_to_draw_in() {
    use ph2d_expr_recipes::RecipeStack;
    use ph2d_panel_timeline::expr_modal_preview as pv;

    for prop in [
        ph2d_timeline::PropKind::TranslationX,
        ph2d_timeline::PropKind::ScaleX,
    ] {
        let base = pv::preview_value(prop);
        let flat = pv::sample_window(&RecipeStack::new(), base, 3);
        let (lo, hi) = pv::extent(&flat, base);
        assert!(hi > lo, "a flat curve must still have a non-zero span");
        assert!(
            lo <= base && base <= hi,
            "the dashed baseline has to sit INSIDE the span it is drawn in"
        );
    }
}

/// **A resting property is not one number.**
///
/// ⚠️ A translation rests at `0` and a SCALE rests at `1`. Before the stage existed
/// the preview declared a single `0`, which is invisible for a scale and for an
/// opacity — the ghost would have opened at nothing and stayed there, and the
/// artist would have read that as a broken stage rather than as a wrong baseline.
#[test]
fn a_resting_property_is_zero_or_one_depending_which_property_it_is() {
    use ph2d_panel_timeline::expr_modal_preview::preview_value;
    use ph2d_timeline::PropKind;
    for p in [
        PropKind::TranslationX,
        PropKind::TranslationY,
        PropKind::Rotation,
    ] {
        assert_eq!(preview_value(p), 0.0, "{p:?} rests at zero");
    }
    for p in [PropKind::ScaleX, PropKind::ScaleY, PropKind::Opacity] {
        assert_eq!(
            preview_value(p),
            1.0,
            "{p:?} rests at ONE, or it is invisible"
        );
    }
}

/// **The preview animates, and it loops.**
///
/// ⚠️ A frame counter, not wall-clock: that is the house convention for chrome
/// (`ToastQueue::tick`), and it is what makes this assertion possible at all.
#[test]
fn the_preview_phase_advances_and_wraps() {
    use ph2d_panel_timeline::expr_modal_preview as pv;
    assert_eq!(pv::phase(0), 0.0);
    assert!(
        pv::phase(1) > pv::phase(0),
        "the phase advances with the frame"
    );
    assert_eq!(pv::phase(0), pv::phase(120), "and the window loops");
}

/// **The card advances its own preview while it is open**, driven by the real
/// paint pass rather than by a test poking the counter.
#[test]
fn painting_the_card_advances_its_preview() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(15, ph2d_timeline::PropKind::Rotation);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let first = state.expr_modal.as_ref().unwrap().preview_frame;
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let second = state.expr_modal.as_ref().unwrap().preview_frame;
    assert!(
        second > first,
        "each paint advances the preview: {first} -> {second}"
    );
    assert_eq!(
        state.expr_modal.as_ref().unwrap().prop,
        ph2d_timeline::PropKind::Rotation,
        "and the card learns which property it is previewing"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// **The card actually PAINTS the wave strip.**
///
/// ⚠️ This gate exists because its absence bit during W2: the wiring edit silently
/// missed its anchor, the column was never drawn, and every preview gate above
/// stayed green — they exercise the preview MODULE, and a module can be perfect
/// while nothing calls it. The two failures that did surface were an unused
/// binding and an unused constant, which is a compiler warning, not a gate.
///
/// It asserts the PROPERTY (each painter is handed the window this frame sampled),
/// never a byte offset — the proxy that expired twice on the Vector line.
#[test]
fn the_card_paints_the_wave_strip() {
    let src = include_str!("../src/expr_modal_paint.rs");
    // ⚠️ ONE door, asserted straight — this was a loop over two while the card also
    // hosted a stage; the stage is gone, and a loop over one element is a loop that
    // reads like it is guarding a family it no longer has.
    let call = src
        .find("expr_modal_preview::paint_strip(")
        .expect("the card must call the wave strip");
    let body = &src[call..];
    let end = body.find(");").expect("the call terminates");
    let args = &body[..end];
    assert!(
        args.contains("&samples"),
        "the wave strip must be handed THIS frame's samples, not its own: {args}"
    );
    assert!(
        args.contains("base"),
        "…against the property's own resting value: {args}"
    );
    // Positive control: the scanner finds the real thing, not any old text.
    assert!(
        !src.contains("expr_modal_preview::paint_strip(/* unwired */"),
        "control: the scanner is looking at the real call"
    );
}

// ─────────────────── the first smoke: reachable, and movable ───────────────────

const SMOKE_VIEWPORT: ph2d_editor_core::zones::Rect =
    ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1249.0, 709.0);

/// **The card opens fully INSIDE the viewport, wherever the menu was clicked.**
///
/// ⚠️ Red-first against the first smoke (Enio): the card opened at the click, the
/// click is by construction down in the timeline, and the bottom two thirds of it
/// hung off the screen — *"o painel está fixo embaixo da tela, não posso vê-lo"*.
/// Opening at the pointer is the wrong rule for a card this size; it centres.
#[test]
fn the_card_opens_fully_inside_the_viewport() {
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(20, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    let regs = host.paint::<TimelinePanel>(&mut state, SMOKE_VIEWPORT);
    let band = regs
        .iter()
        .find(|(id, _)| *id == ids::EXPR_MODAL_HANDLE)
        .map(|(_, r)| *r)
        .expect("the title band is painted");
    let (x, y) = state.expr_modal.as_ref().unwrap().pos.expect("placed");
    let (w, h) = (
        ph2d_panel_timeline::expr_modal_paint::card_w(),
        ph2d_panel_timeline::expr_modal_paint::card_h(),
    );
    assert!(
        x >= SMOKE_VIEWPORT.x
            && y >= SMOKE_VIEWPORT.y
            && x + w <= SMOKE_VIEWPORT.x + SMOKE_VIEWPORT.w + 0.5
            && y + h <= SMOKE_VIEWPORT.y + SMOKE_VIEWPORT.h + 0.5,
        "the whole card must be on screen: ({x}, {y}) + ({w} x {h}) in {SMOKE_VIEWPORT:?}"
    );
    assert!(band.y >= SMOKE_VIEWPORT.y, "…including its title band");

    // ⚠️ And it opens CENTRED, which the clamp alone does not give: the clamp
    // rescues any bad position, so "on screen" stays true for a card that opens
    // jammed in a corner. Centring is the product decision the smoke asked for,
    // so it is asserted on its own.
    let (ccx, ccy) = (x + w * 0.5, y + h * 0.5);
    let (vcx, vcy) = (
        SMOKE_VIEWPORT.x + SMOKE_VIEWPORT.w * 0.5,
        SMOKE_VIEWPORT.y + SMOKE_VIEWPORT.h * 0.5,
    );
    assert!(
        (ccx - vcx).abs() < 1.0 && (ccy - vcy).abs() < 1.0,
        "the card opens centred in the viewport: ({ccx}, {ccy}) vs ({vcx}, {vcy})"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// **The title band MOVES the card, and cannot push it off screen.**
///
/// The delta is taken against the position captured at Begin — never accumulated
/// per Update, which is how a drag drifts away from the pointer.
#[test]
fn the_title_band_drags_the_card_and_the_clamp_keeps_it_reachable() {
    use ph2d_editor_core::interaction::{
        GestureMods, GesturePhase, TimelineGesture, TimelineHitKind,
    };
    use ph2d_host::PointerButton;
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(21, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);
    host.paint::<TimelinePanel>(&mut state, SMOKE_VIEWPORT);
    let (x0, y0) = state.expr_modal.as_ref().unwrap().pos.unwrap();

    let g = |phase, x: f32, y: f32| TimelineGesture {
        surface: ids::EXPR_MODAL_HANDLE,
        kind: TimelineHitKind::ExprModalHandle,
        phase,
        x,
        y,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    };
    ph2d_panel_timeline::interact_for_test(&mut state, g(GesturePhase::Begin, x0, y0));
    // ⚠️ TWO updates, and the second is what matters: with one, "delta from the
    // Begin position" and "accumulate each step" give the SAME answer, so a
    // one-update fixture stays green over a drag that drifts. The second step
    // separates them — accumulation would land at 2x the offset.
    for (dx, dy) in [(-40.0_f32, -30.0_f32), (-60.0, -45.0)] {
        ph2d_panel_timeline::interact_for_test(
            &mut state,
            g(GesturePhase::Update, x0 + dx, y0 + dy),
        );
        let moved = state.expr_modal.as_ref().unwrap().pos.unwrap();
        assert!(
            (moved.0 - (x0 + dx)).abs() < 0.5 && (moved.1 - (y0 + dy)).abs() < 0.5,
            "the card follows the pointer 1:1, every step: {moved:?} vs {:?}",
            (x0 + dx, y0 + dy)
        );
    }

    // Shove it far past the corner; the paint's clamp has to bring it back.
    ph2d_panel_timeline::interact_for_test(
        &mut state,
        g(GesturePhase::Update, x0 + 9_000.0, y0 + 9_000.0),
    );
    host.paint::<TimelinePanel>(&mut state, SMOKE_VIEWPORT);
    let (x, y) = state.expr_modal.as_ref().unwrap().pos.unwrap();
    let (w, h) = (
        ph2d_panel_timeline::expr_modal_paint::card_w(),
        ph2d_panel_timeline::expr_modal_paint::card_h(),
    );
    assert!(
        x + w <= SMOKE_VIEWPORT.w + 0.5 && y + h <= SMOKE_VIEWPORT.h + 0.5,
        "a card dragged past the corner is pulled back on screen, not lost: ({x}, {y})"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

// ────────────── the second smoke: the REAL object runs the effect ──────────────

/// **An open card publishes what it projects, so the REAL object can run it.**
///
/// ⚠️ Red-first against the smoke: *"vamos tirar o fantasma. Vamos fazer o efeito
/// correr no objeto selecionado em tempo real mesmo que o clip esteja pausado, desde
/// que o painel esteja aberto"* (Enio, 2026-07-29). The ghost is gone; what replaces
/// it is the property itself, driven by `ph2d_timeline::expr_live` — and the ONLY
/// thing the panel owes that channel is *what formula, on whose binding*. The clock
/// is the shell's, in wall-clock seconds, because the transport is precisely what is
/// not moving.
///
/// The publish happens in the PAINT, where the sheet has just been read back from the
/// widgets; published anywhere else it would be a frame stale, and the artist would
/// be watching the knob they turned one frame ago.
#[test]
fn an_open_card_publishes_its_formula_for_the_real_object_to_run() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(40, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    open_modal(&mut host, &mut state, target);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("shake")),
    );
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);

    let (t, formula) =
        ph2d_panel_timeline::expr_live_target().expect("an open card publishes a live preview");
    assert_eq!(t, target, "…on the binding the card was opened from");
    assert_eq!(
        formula,
        state.expr_modal.as_ref().unwrap().stack.to_formula(),
        "…and it is EXACTLY what the formula bar shows, never a stale copy"
    );

    // Cancel: the scene must stop running it.
    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::EXPR_MODAL_CANCEL));
    assert!(
        ph2d_panel_timeline::expr_live_target().is_none(),
        "closing the card stops the preview — a formula nobody can see or stop is worse \
         than no preview at all"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// **A card that stops being painted stops previewing**, even by a route that never
/// runs `cancel`.
///
/// ⚠️ The panel can go away without the card being dismissed — hidden panel, timeline
/// closed, the track deleted under it. Every one of those is a door out, and a
/// preview that survived one would drive the scene with no card left to stop it. So
/// the clear does NOT live only in `cancel`: the paint clears it whenever there is no
/// card, which is the condition that is true for all of them at once.
#[test]
fn a_card_that_stops_being_painted_stops_driving_the_scene() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(41, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(ph2d_panel_timeline::expr_live_target().is_some());

    // The card vanishes without anyone calling `cancel`.
    state.expr_modal = None;
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        ph2d_panel_timeline::expr_live_target().is_none(),
        "a paint with no card must leave nothing driving the scene"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// **The combine chip is alive under a real pointer, it cycles, and the formula follows.**
///
/// ⚠️ Red-first against a report: *"Expressões não podem ser somadas, multiplicadas,
/// etc."* Measured before the fix, a sheet of `Sway` then `Blink` projected to
/// `select(fract(time*4) < 0.5, 1, 0)` — the Sway **silently gone**, because 29 of the
/// 55 recipes ignored the value above them.
///
/// The pointer is a real pixel and not a synthetic `WidgetEvent`: the physics panel's
/// 36 collision cells were painted, hit-registered and dead under the mouse while a
/// synthetic-event gate stayed green.
#[test]
fn the_combine_chip_cycles_under_a_real_pointer_and_the_formula_follows() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(21, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    // Two SOURCES: the second one used to eat the first.
    for r in ["sway", "blink"] {
        host.apply_panel_event::<TimelinePanel>(
            &mut state,
            WidgetEvent::Click(ids::expr_gallery_id(r)),
        );
    }
    let before = state.expr_modal.as_ref().unwrap().stack.to_formula();
    assert!(
        !before.contains("sin("),
        "Blink's own default is Replace, so the Sway is dropped — that is the recipe's \
         honest default and the chip is what changes it: {before}"
    );

    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let chip = ids::expr_combine_id(1);
    let rect = regs
        .iter()
        .find(|(id, _)| *id == chip)
        .map(|(_, r)| *r)
        .expect("row 1 is a SOURCE, so its combine chip is painted and hit-registered");

    let evs = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(id) if *id == chip)),
        "a pointer over the chip must produce its Click; got {evs:?}"
    );
    for ev in evs {
        host.apply_panel_event::<TimelinePanel>(&mut state, ev);
    }
    let after = state.expr_modal.as_ref().unwrap().stack.to_formula();
    assert_ne!(before, after, "the click must reach the fold");
    assert!(
        after.contains("sin(") && after.contains("select("),
        "and now BOTH rows are in the answer — the report's fix: {after}"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// **A MODIFIER row is offered no chip** — presence and ABSENCE, in one gate.
///
/// ⚠️ The absent half is the one that rots: `Limit` folds the value itself, so a mode
/// on it would be a control that means nothing, and the day someone paints the chip
/// unconditionally every other gate here stays green.
#[test]
fn a_modifier_row_has_no_combine_chip() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(22, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    // Row 0 a source, row 1 a modifier — so the same paint proves both halves.
    for r in ["shake", "limit"] {
        host.apply_panel_event::<TimelinePanel>(
            &mut state,
            WidgetEvent::Click(ids::expr_gallery_id(r)),
        );
    }
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        regs.iter().any(|(id, _)| *id == ids::expr_combine_id(0)),
        "the SOURCE row keeps its chip"
    );
    assert!(
        !regs.iter().any(|(id, _)| *id == ids::expr_combine_id(1)),
        "the MODIFIER row must have none"
    );

    // …and a synthetic click on the chip it does not have changes nothing, so the mode
    // of a modifier cannot be moved by something the artist never sees.
    let before = state.expr_modal.as_ref().unwrap().stack.to_formula();
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_combine_id(1)),
    );
    assert_eq!(
        before,
        state.expr_modal.as_ref().unwrap().stack.to_formula(),
        "a modifier has no mode to cycle"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// Publish TWO objects bound on the same property, and set who is selected. Returns the
/// two raw targets.
fn publish_two_tracks(
    a: u64,
    b: u64,
    prop: ph2d_timeline::PropKind,
    selected: Option<u64>,
) -> (u64, u64) {
    use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    for e in [a, b] {
        apply_intent(&mut st, &mut ph, TimelineIntent::Bind { entity: e, prop });
    }
    let ta = st.doc.binding_for(a, prop).unwrap().target.get();
    let tb = st.doc.binding_for(b, prop).unwrap().target.get();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    snap.selected_entity = selected;
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    (ta, tb)
}

/// **The card follows the scene selection.**
///
/// ⚠️ Red-first against a report: *"se eu seleciono outro objeto na cena, o painel de
/// expressões não atualiza para o novo objeto."* The card captured one target when it
/// opened and nothing revisited it — and it COULD not, because the snapshot carried no
/// selection at all.
///
/// ⚠️ The gate moves `selected_entity` BETWEEN frames and repaints, which is the only
/// shape that can fail: a test that asserts "the card follows" while never changing that
/// field is green by construction.
#[test]
fn the_card_follows_the_scene_selection() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let prop = ph2d_timeline::PropKind::TranslationX;
    let (ta, tb) = publish_two_tracks(31, 32, prop, Some(31));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, ta);
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert_eq!(
        state.expr_modal.as_ref().unwrap().target,
        ta,
        "the card opens on the track it was opened from"
    );
    // ⚠️ A row IN FLIGHT, and it is load-bearing: the first version of this gate switched
    // selection over an EMPTY sheet, where clearing it and not clearing it are
    // indistinguishable — the mutation that drops the clear survived.
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("shake")),
    );
    assert_eq!(state.expr_modal.as_ref().unwrap().stack.rows.len(), 1);

    // The artist clicks the other object.
    publish_two_tracks(31, 32, prop, Some(32));
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let m = state
        .expr_modal
        .as_ref()
        .expect("the card must SURVIVE the move");
    assert_eq!(
        m.target, tb,
        "…and re-point at the same property on the new object"
    );
    assert_eq!(m.entity, 32);
    assert!(
        m.stack.rows.is_empty(),
        "the sheet describes the OLD object's formula, so it is cleared rather than \
         carried over or auto-committed"
    );
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![],
        "and following the selection authors NOTHING"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// **Selecting an object with no track for this property does not DISMISS the card.**
///
/// ⚠️ This is the half that makes the fix safe rather than a different bug. The guard
/// below the re-point abandons the card when its target has vanished, so a naive
/// "re-point to whatever is selected" would make clicking an unbound object close the
/// panel — worse than not following at all. Authoring the binding instead would write to
/// the document as a side effect of *selecting*, which nothing else in this app does.
#[test]
fn selecting_an_object_with_no_track_leaves_the_card_alone() {
    const VIEWPORT: ph2d_editor_core::zones::Rect =
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = ph2d_panel_timeline::drain_intents();
    let prop = ph2d_timeline::PropKind::TranslationX;
    let (ta, _tb) = publish_two_tracks(41, 42, prop, Some(41));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, ta);
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("shake")),
    );
    let rows = state.expr_modal.as_ref().unwrap().stack.rows.len();
    assert_eq!(rows, 1, "a row is in flight");

    // Select an object that has NO binding at all.
    publish_two_tracks(41, 42, prop, Some(9_999));
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let m = state
        .expr_modal
        .as_ref()
        .expect("the card must NOT be dismissed by selecting an unbound object");
    assert_eq!(m.target, ta, "it stays on the object it was authoring");
    assert_eq!(
        m.stack.rows.len(),
        rows,
        "…and the row the artist was editing survives"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}
