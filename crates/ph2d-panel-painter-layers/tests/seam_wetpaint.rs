//! **Wet Paint presence seams** — what the panel OFFERS in and around the wet mode.
//!
//! Two facts the 2026-07-21 smoke tripped on: the Enable checkbox has to be painted BOTH for the
//! plain brush (to arm) and inside the wet mode (to disarm — without it the artist cannot leave);
//! and the **Paper** section has to be offered in wet mode (W2.7 seeds the engine's tooth from the
//! Paper slot — with the section hidden there is no door to arm a paper, and the seam is
//! unreachable by hand). Presence AND absence are asserted: the plain brush shows no Paper (it
//! reads no substrate — Enio: "deve ser assim mesmo").

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn painted(tool: &PainterTool) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    host.paint::<PainterLayersPanel>(&mut st, viewport())
}

fn has(rects: &[(NodeId, Rect)], id: NodeId) -> bool {
    rects
        .iter()
        .any(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
}

/// The Enable checkbox exists on BOTH sides of the arm — offered to check
/// (plain brush) and to UNCHECK (wet mode; hiding it there is "não consigo
/// sair do modo wet" with different clothes). Mutation that bleeds it: the
/// section gated on `!brush.wetpaint` (or on the mode) instead of always.
#[test]
fn the_enable_checkbox_is_offered_to_arm_and_to_disarm() {
    let plain = PainterTool::default();
    assert!(
        has(&painted(&plain), core_ids::PAINTER_WETPAINT_ENABLE),
        "the plain brush has no Wet Paint checkbox to ARM"
    );
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    assert!(
        has(&painted(&wet), core_ids::PAINTER_WETPAINT_ENABLE),
        "the wet mode has no checkbox to DISARM — the artist cannot leave"
    );
}

/// W2.7's door: the **Paper** section is offered in wet mode (the slot seeds
/// the engine's tooth) and stays hidden for the plain brush (no substrate to
/// read). Mutation that bleeds it: `|| brush.wetpaint` dropped from the
/// Paper gate in `paint_brush_sections`.
#[test]
fn the_paper_section_is_offered_in_wet_mode_and_hidden_for_the_plain_brush() {
    let plain = PainterTool::default();
    assert!(
        !has(&painted(&plain), core_ids::PAINTER_WATERCOLOR_PAPER_SECTION),
        "the plain brush must NOT offer Paper (deve ser assim mesmo — Enio)"
    );
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    assert!(
        has(&painted(&wet), core_ids::PAINTER_WATERCOLOR_PAPER_SECTION),
        "wet mode offers no Paper section — the W2.7 seam is unreachable by hand"
    );
}

/// W3: the seven curated knob rows are offered ONLY while armed — presence
/// (armed shows all seven) and absence (a knob for an engine that is not
/// running is a dead control wearing a live one's clothes). Mutation that
/// bleeds it: dropping the `if brush.wetpaint` gate around the rows (absence
/// half) or a row lost from the table (presence half).
#[test]
fn the_knob_rows_are_offered_only_while_armed() {
    let plain = painted(&PainterTool::default());
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    let armed = painted(&wet);
    for id in core_ids::PAINTER_WETPAINT_FIELDS {
        assert!(
            has(&armed, id),
            "armed wet mode is missing a curated knob row ({id:?})"
        );
        assert!(
            !has(&plain, id),
            "the plain brush paints a wet knob for an engine that is not running ({id:?})"
        );
    }
}

/// W3 law #3: the **Watercolor section hides while Wet Paint is armed** —
/// its optics reinterpret the DIGITAL deposit and the fluid engine owns the
/// wet one; two wet-media switches over one brush would be two answers to
/// "what does this stroke do". Presence AND absence. Mutation that bleeds
/// it: dropping the `!brush.wetpaint` gate in `paint_brush_sections`.
#[test]
fn the_watercolor_section_hides_while_wet_is_armed() {
    let plain = PainterTool::default();
    assert!(
        has(&painted(&plain), core_ids::PAINTER_WATERCOLOR_SECTION),
        "positive control: the plain brush must offer the Watercolor section"
    );
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    assert!(
        !has(&painted(&wet), core_ids::PAINTER_WATERCOLOR_SECTION),
        "wet mode still offers the Watercolor section — two wet-media switches over one brush"
    );
}

/// W3: a wet knob edit forwards through the panel's REAL event path (the
/// `is_param_field` ValueChanged arm) — the synthetic-event blind spot that
/// let the dead Enable checkbox ship is exactly what this refuses to repeat.
/// Mutation that bleeds it: `PAINTER_WETPAINT_FIELDS` dropped from
/// `number_field::is_param_field`.
#[test]
fn a_wet_knob_edit_forwards_through_the_panel() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::EventOutcome;
    use ph2d_editor_core::tool::PanelEvent;
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    set_current_brush(Some(wet.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let _ = host.paint::<PainterLayersPanel>(&mut st, viewport()); // registers the number chips
    let id = core_ids::PAINTER_WETPAINT_WATER;
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "a wet knob edit died in the panel — the ValueChanged arm does not know the id"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
        )),
        "the wet knob edit never forwarded as SetValue — dead under the mouse. drained = {actions:?}"
    );
}

/// Doc 21 (deposit-at-commit): the Method dropdown offers the FULL list
/// while Wet Paint is armed — every method authors a flat static preview
/// and the fluid receives the final dab list once at commit, so no method
/// is incompatible (the W3 narrowing rested on a refuted premise). Mutation
/// that bleeds it: re-adding the `b.wetpaint` narrowing arm to
/// `stroke_method_offer::offered_stroke_methods`.
#[test]
fn the_method_menu_offers_every_method_while_wet_is_armed() {
    use ph2d_panel_painter_layers::stroke_method_offer::offered_stroke_methods;
    let plain = PainterTool::default().brush_settings();
    let full = offered_stroke_methods(Some(&plain));
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    let offered = offered_stroke_methods(Some(&wet.brush_settings()));
    assert_eq!(
        offered, full,
        "wet mode narrows the Method menu — deposit-at-commit made every method valid"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Doc 22 — the grown section: tools, tilt, canvas actions, Paper, Tuning.
// ─────────────────────────────────────────────────────────────────────────

/// Every doc-22 widget is offered while ARMED and absent for the plain brush
/// (presence AND absence — a control for an engine that is not running is a
/// dead control wearing a live one's clothes). Mutation that bleeds it: any
/// widget dropped from the armed branch, or painted outside it.
#[test]
fn every_doc22_wet_widget_is_offered_only_while_armed() {
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    let on = painted(&wet);
    let off = painted(&PainterTool::default());
    let mut ids: Vec<NodeId> = core_ids::PAINTER_WETPAINT_TOOL_IDS.to_vec();
    ids.extend([
        core_ids::PAINTER_WETPAINT_TILT_TOGGLE,
        core_ids::PAINTER_WETPAINT_TILT_PAD,
        core_ids::PAINTER_WETPAINT_WETCANVAS,
        core_ids::PAINTER_WETPAINT_DRYCANVAS,
        core_ids::PAINTER_WETPAINT_FASTDRY,
        core_ids::PAINTER_WETPAINT_SHOWWET,
        core_ids::PAINTER_WETPAINT_PAPER_VISUAL,
        core_ids::PAINTER_WETPAINT_TUNING,
    ]);
    for id in ids {
        assert!(has(&on, id), "armed wet section is missing {id:?}");
        assert!(!has(&off, id), "{id:?} painted for the PLAIN brush");
    }
}

/// Every doc-22 CLICK forwards through the REAL panel seam (the Enable
/// checkbox shipped dead under the mouse once — a synthetic tool-side event
/// skips this forward, which is exactly how it shipped). Mutation: an id
/// missing from `PAINTER_WETPAINT_CLICKS` (allowlist + populate share it).
#[test]
fn every_doc22_wet_click_forwards_through_the_panel() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::panel::EventOutcome;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_editor_core::interaction::WidgetEvent;
    for clicked in core_ids::PAINTER_WETPAINT_CLICKS {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(clicked));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "panel ignored the wet click {clicked:?} (allowlist arm missing)"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
            )),
            "wet click {clicked:?} never reached the bus. drained = {actions:?}"
        );
    }
}

/// The TILT pad drag snaps to the dial's grid and forwards ring+spoke as two
/// `SetValue`s (the shared conversion — a drag to the right edge is ring 8,
/// spoke 0). Mutation: the ValueChanged arm missing from `event.rs`, or the
/// conversion diverging from the paint's placement law.
#[test]
fn the_tilt_pad_drag_forwards_ring_and_spoke() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_editor_core::interaction::WidgetEvent;
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    host.store_mut().set_curve_point_drag(
        core_ids::PAINTER_WETPAINT_TILT_PAD,
        0,
        0,
        1.0, // right edge
        0.5, // vertical centre
    );
    let outcome = host.apply_panel_event::<PainterLayersPanel>(
        &mut st,
        WidgetEvent::ValueChanged(core_ids::PAINTER_WETPAINT_TILT_PAD),
    );
    assert_eq!(outcome, EventOutcome::Consumed, "the pad drain arm is missing");
    let actions = host.drained_actions();
    let val = |target: NodeId| {
        actions.iter().find_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)) if *id == target => Some(*v),
            _ => None,
        })
    };
    assert_eq!(
        val(core_ids::PAINTER_WETPAINT_TILT_RING),
        Some(8.0),
        "a drag to the rim must snap to the outer ring"
    );
    assert_eq!(
        val(core_ids::PAINTER_WETPAINT_TILT_SPOKE),
        Some(0.0),
        "a drag straight right is spoke 0"
    );
}
