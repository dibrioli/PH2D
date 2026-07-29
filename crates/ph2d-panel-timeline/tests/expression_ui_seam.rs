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

// ─────────────────────────── W2 — the live preview ───────────────────────────

/// **The preview evaluates the window through the PRODUCT's evaluator** (P3), and
/// what the curve strip plots is what the puppet stands on.
///
/// ⚠️ ONE door, checked as one: the puppet indexes the SAME vector the strip
/// draws. A puppet that sampled on its own would drift from the curve beside it,
/// and the artist would have no way to tell which of the two lies.
#[test]
fn the_preview_samples_the_window_once_and_both_views_read_it() {
    use ph2d_expr_recipes::RecipeStack;
    use ph2d_panel_timeline::expr_modal_preview as pv;

    // A Sway is a sine: over a 2 s window it must leave the baseline in BOTH
    // directions, which a constant or a clamped-to-zero evaluation cannot do.
    let stack = RecipeStack::of(&["sway"]);
    let s = pv::sample_window(&stack);
    assert_eq!(s.len(), pv::PREVIEW_SAMPLES);
    assert!(
        s.iter().any(|v| *v > pv::PREVIEW_VALUE) && s.iter().any(|v| *v < pv::PREVIEW_VALUE),
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

    let flat = pv::sample_window(&RecipeStack::new());
    let (lo, hi) = pv::extent(&flat);
    assert!(hi > lo, "a flat curve must still have a non-zero span");
    assert!(
        lo <= pv::PREVIEW_VALUE && pv::PREVIEW_VALUE <= hi,
        "the dashed baseline has to sit INSIDE the span it is drawn in"
    );
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

/// **The puppet follows the PROPERTY.** A rotation gets a needle, an opacity a
/// fading square — one function answers it, so the frame and the strip can never
/// disagree about what is being previewed.
#[test]
fn the_puppet_is_chosen_by_the_property_being_driven() {
    use ph2d_panel_timeline::expr_modal_preview::{Puppet, puppet_for};
    use ph2d_timeline::PropKind;
    assert_eq!(puppet_for(PropKind::Rotation), Puppet::Needle);
    assert_eq!(puppet_for(PropKind::Opacity), Puppet::Fade);
    assert_eq!(puppet_for(PropKind::TranslationX), Puppet::SlideX);
    assert_eq!(puppet_for(PropKind::TranslationY), Puppet::SlideY);
    assert_ne!(
        puppet_for(PropKind::ScaleX),
        puppet_for(PropKind::ScaleY),
        "the two scale axes are different figures, or the preview lies about which"
    );
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

/// **The card actually PAINTS the preview column.**
///
/// ⚠️ This gate exists because its absence bit during W2: the wiring edit silently
/// missed its anchor, the column was never drawn, and every preview gate above
/// stayed green — they exercise the preview MODULE, and a module can be perfect
/// while nothing calls it. The two failures that did surface were an unused
/// binding and an unused constant, which is a compiler warning, not a gate.
///
/// It asserts the PROPERTY (the painter is handed the window this frame sampled
/// and the puppet this property chose), never a byte offset — the proxy that
/// expired twice on the Vector line.
#[test]
fn the_card_paints_the_preview_column() {
    let src = include_str!("../src/expr_modal_paint.rs");
    let call = src
        .find("expr_modal_preview::paint(")
        .expect("the card must call the preview painter");
    let body = &src[call..];
    let end = body.find(");").expect("the call terminates");
    let args = &body[..end];
    assert!(
        args.contains("&samples"),
        "the preview must be handed THIS frame's samples, not its own: {args}"
    );
    assert!(
        args.contains("puppet_for(m.prop)"),
        "…and the puppet the driven property chose: {args}"
    );
    assert!(
        args.contains("m.preview_frame"),
        "…at the card's own phase, or it never animates: {args}"
    );
    // Positive control: the scanner finds the real thing, not any old text.
    assert!(
        !src.contains("expr_modal_preview::paint(/* unwired */"),
        "control: the scanner is looking at the real call"
    );
}
