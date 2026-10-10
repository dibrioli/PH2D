//! Behavioral seams — REAL `WidgetEvent`s through the panel's `apply_event`,
//! asserting the observable effect (the forwarded `ToolPanelEvent`s the
//! painter routes on the other side). The rows are loop-registered, which is
//! exactly why the sweep walks EVERY row: `architecture_panel_wiring_parity`
//! cannot see loop registrations, so nothing else would notice one going
//! dead under the mouse.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_wet_tuning::{WetTuningPanel, rows, set_current_brush, state};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn host() -> (MockPanelHost, state::WetTuningPanelState) {
    (
        MockPanelHost::with_panel::<WetTuningPanel>(),
        state::WetTuningPanelState,
    )
}

fn drained_setvalue(actions: &[EditorAction], id: ph2d_a11y::NodeId) -> Option<f64> {
    actions.iter().find_map(|a| match a {
        EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, v)) if *i == id => Some(*v),
        _ => None,
    })
}

fn drained_click(actions: &[EditorAction], id: ph2d_a11y::NodeId) -> bool {
    actions
        .iter()
        .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(i)) if *i == id))
}

/// EVERY row's slider forwards its REAL value (track → range mapping), and
/// every reset forwards its Click — the whole table, not a representative
/// (the fullest-card premise rots). Mutation: a row lost from the event
/// walk, or `value_of`/`track_of` ceasing to be inverses.
#[test]
fn every_row_slider_and_reset_forwards() {
    for row in rows::rows() {
        let (mut host, mut st) = host();
        // Put the slider's track at 1.0 — the forwarded value must be the
        // row's MAX (the def's own bound, from the registry).
        host.set_slider_value(row.slider, 1.0);
        let out = host
            .apply_panel_event::<WetTuningPanel>(&mut st, WidgetEvent::ValueChanged(row.slider));
        assert_eq!(out, EventOutcome::Consumed, "slider {} ignored", row.key);
        let actions = host.drained_actions();
        let v = drained_setvalue(&actions, row.slider)
            .unwrap_or_else(|| panic!("slider {} never reached the bus", row.key));
        assert!(
            (v - row.max).abs() <= row.step.max(1e-9),
            "slider {} at track 1.0 must forward ~max ({} vs {})",
            row.key,
            v,
            row.max
        );
        let (mut host, mut st) = self::host();
        let out = host.apply_panel_event::<WetTuningPanel>(&mut st, WidgetEvent::Click(row.reset));
        assert_eq!(out, EventOutcome::Consumed, "reset {} ignored", row.key);
        assert!(
            drained_click(&host.drained_actions(), row.reset),
            "reset {} never reached the bus",
            row.key
        );
    }
}

/// The chip's committed value forwards as-is (the tool clamps).
#[test]
fn a_chip_commit_forwards_its_number() {
    let row = &rows::rows()[0];
    let (mut host, mut st) = host();
    host.set_number_value(row.chip, 123.0);
    let out =
        host.apply_panel_event::<WetTuningPanel>(&mut st, WidgetEvent::ValueChanged(row.chip));
    assert_eq!(out, EventOutcome::Consumed);
    assert_eq!(
        drained_setvalue(&host.drained_actions(), row.chip),
        Some(123.0)
    );
}

/// Group resets, the K–M checkboxes and the close button all
/// forward — and close forwards the BASIC section's Tuning toggle (the one
/// authored fact; a panel-local hide would fight the bridge).
#[test]
fn commands_forward_and_close_is_the_tuning_toggle() {
    for id in ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS {
        let (mut host, mut st) = host();
        assert_eq!(
            host.apply_panel_event::<WetTuningPanel>(&mut st, WidgetEvent::Click(id)),
            EventOutcome::Consumed
        );
        assert!(drained_click(&host.drained_actions(), id));
    }
    for id in [
        ph2d_tool_painter::ids::WET_TUNING_KM_MIXING,
        ph2d_tool_painter::ids::WET_TUNING_KM_GLAZE,
    ] {
        let (mut host, mut st) = host();
        assert_eq!(
            host.apply_panel_event::<WetTuningPanel>(&mut st, WidgetEvent::Click(id)),
            EventOutcome::Consumed
        );
        assert!(drained_click(&host.drained_actions(), id));
    }
    let (mut host, mut st) = host();
    assert_eq!(
        host.apply_panel_event::<WetTuningPanel>(
            &mut st,
            WidgetEvent::Click(ph2d_tool_painter::ids::WET_TUNING_CLOSE)
        ),
        EventOutcome::Consumed
    );
    assert!(
        drained_click(
            &host.drained_actions(),
            ph2d_tool_painter::ids::PAINTER_WETPAINT_TUNING
        ),
        "close must forward the basic section's Tuning toggle"
    );
}

/// A header click folds the section LOCALLY (no bus traffic — the shell has
/// no opinion about which sections are open).
#[test]
fn a_header_click_folds_locally() {
    let (mut host, mut st) = host();
    let header = rows::SECTIONS[0].header;
    assert!(!host.store().is_collapsed(header));
    assert_eq!(
        host.apply_panel_event::<WetTuningPanel>(&mut st, WidgetEvent::Click(header)),
        EventOutcome::Consumed
    );
    assert!(host.store().is_collapsed(header));
    assert!(
        host.drained_actions().is_empty(),
        "folding is panel-local view state"
    );
}

/// Paint offers every row (slider+chip+reset rects), the headers and the K–M
/// checkboxes — and NO paper row, armed or not (W1e, F2: o papel do Wet Paint é
/// o da secção Paper do painel; nenhum controlo de papel vive fora dela).
#[test]
fn paint_offers_the_table_and_no_paper_row_lives_here() {
    // ⚠️ Alto o bastante para o painel INTEIRO caber: desde a porta da rolagem o clique também é
    // recortado pelo corpo, e medido 2026-09-29 o corpo pede `1333` px contra `766` visíveis a
    // `900` de viewport — as linhas de baixo só passavam porque o registo não era recortado.
    let viewport = Rect::new(0.0, 0.0, 1600.0, 1800.0);
    let paint_with = |brush: ph2d_tool_painter::BrushSettings| {
        set_current_brush(Some(brush));
        let mut host = MockPanelHost::with_panel::<WetTuningPanel>();
        host.set_panel_visible(WetTuningPanel::ID, true);
        let mut st = state::WetTuningPanelState;
        host.paint::<WetTuningPanel>(&mut st, viewport)
    };
    let has = |rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId| {
        rects
            .iter()
            .any(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
    };
    let plain = paint_with(PainterTool::default().brush_settings());
    for row in rows::rows() {
        assert!(has(&plain, row.slider), "row {} slider missing", row.key);
        assert!(has(&plain, row.reset), "row {} reset missing", row.key);
    }
    for s in rows::SECTIONS {
        assert!(has(&plain, s.header), "header missing");
        assert!(has(&plain, s.reset), "group reset missing");
    }
    for id in [
        ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[4],
        ph2d_tool_painter::ids::WET_TUNING_KM_MIXING,
        ph2d_tool_painter::ids::WET_TUNING_KM_GLAZE,
        ph2d_tool_painter::ids::WET_TUNING_CLOSE,
    ] {
        assert!(has(&plain, id), "static widget {id:?} missing");
    }
    // Nenhuma linha de papel, com o papel do artista armado ou não.
    let mut armed = PainterTool::default().brush_settings();
    armed.paper_kind = 1;
    for brush in [PainterTool::default().brush_settings(), armed] {
        let rects = paint_with(brush);
        for def in ph2d_wet_paint::tuning::KNOB_DEFS
            .iter()
            .filter(|d| d.group == ph2d_wet_paint::tuning::KnobGroup::Paper)
        {
            let id = ph2d_tool_painter::ids::wet_tuning_slider_id(def.key);
            assert!(
                !has(&rects, id),
                "a linha de papel {} vive fora da secção Paper",
                def.key
            );
        }
    }
    set_current_brush(None);
}
