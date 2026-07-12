//! Behavioral SEAM test for the Painter Layers panel ↔ tool — the test class
//! the 2026-06-20 diagnosis found missing (blindagem Fase 1).
//!
//! `PainterLayersPanel` is a thin FORWARDER (ADR-0040 TG-B, mirror of the
//! image panels): its `apply_event` classifies each `WidgetEvent` into a
//! tool-agnostic [`PanelEvent`] and pushes it on the action bus as
//! `EditorAction::ToolPanelEvent(..)` (or `CancelActiveTool` for the X). The
//! shell drains the bus each frame and calls `PainterTool::handle_panel_event`
//! on the active tool.
//!
//! Unit tests in `ph2d-tool-painter` exercise `handle_panel_event` / the
//! `LayerStack` API directly, and `populate.rs` registers the chrome buttons —
//! but NEITHER proves the wire BETWEEN them is intact. A forgotten `event.rs`
//! arm or a wrong `PanelEvent` shape leaves the "+ Layer" button painted,
//! clickable and SILENTLY DEAD while every unit test + `*_contract_surface`
//! gate stays green.
//!
//! These tests prove the PANEL→shell seam: the real click forwards the EXACT
//! `PanelEvent` the tool's matching arm consumes. (The tool-side effect —
//! `add_raster_layer` growing the stack — needs a canvas/source size, which a
//! headless seam test doesn't set up; that path is covered by
//! `ph2d-tool-painter`'s own unit tests. The seam this gate exists for is the
//! forward, which is asserted here on the exact inner id.)

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::PainterLayersPanelState;
use ph2d_ui_testkit::MockPanelHost;

/// Click the fixed "+ Layer" chrome button and prove it forwards as the
/// SPECIFIC `ToolPanelEvent(PanelEvent::Click(PAINTER_LAYERS_ADD))` — the exact
/// event `PainterTool::handle_panel_event`'s `PAINTER_LAYERS_ADD` arm consumes.
/// Exercises `populate` (button registered) → `event.rs` (chrome allowlist arm).
#[test]
fn add_layer_click_forwards_specific_id() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut panel_state = PainterLayersPanelState;

    // The exact id a real pointer click on the "+ Layer" button carries.
    let clicked = core_ids::PAINTER_LAYERS_ADD;

    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut panel_state, WidgetEvent::Click(clicked));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the '+ Layer' click — the `event.rs` chrome-button allowlist arm for \
         PAINTER_LAYERS_ADD is missing"
    );

    // Assert the bus carries the SPECIFIC inner PanelEvent::Click(clicked), not
    // merely "a ToolPanelEvent exists" — a wrong/forgotten arm changes the id.
    let actions = host.drained_actions();
    let forwarded = actions.iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
        )
    });
    assert!(
        forwarded,
        "'+ Layer' click never reached the bus as ToolPanelEvent(Click(PAINTER_LAYERS_ADD)) — \
         the panel→shell seam is dead. drained = {actions:?}"
    );
}

/// DEFECT REPRO (Vector falloff-handle menu does nothing): the right-click
/// "handle type" menu item `Click(CTX_MENU_FALLOFF_HANDLE_VECTOR)` is a CHROME
/// id, NOT a panel widget — it must fall through to `chrome::falloff_handle`.
/// The painter panel is in the registry whenever the Painter is active and runs
/// BEFORE chrome in `HeroScreen::apply_event`; if it returns `Consumed` (or
/// `Observed` that the host treats as handled) for this unknown id, the chrome
/// handler never runs and `pending_falloff_point_handle` is never set → the
/// curve stays smooth. This test pins the panel to `Ignored`.
#[test]
fn falloff_handle_menu_click_is_not_consumed_by_panel() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut panel_state = PainterLayersPanelState;

    for id in [
        core_ids::CTX_MENU_FALLOFF_HANDLE_VECTOR,
        core_ids::CTX_MENU_FALLOFF_HANDLE_AUTO,
    ] {
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut panel_state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Ignored,
            "painter panel ATE the falloff-handle menu click {id:?} — chrome::falloff_handle \
             never runs, so the Vector/Auto choice is silently dropped"
        );
        let actions = host.drained_actions();
        assert!(
            actions.is_empty(),
            "panel forwarded a spurious action for a chrome menu id: {actions:?}"
        );
    }
}

/// The Close (X) button is the OTHER fixed-chrome seam: it must push
/// `CancelActiveTool` (canon BgRemoval/Painter sidebar), not a ToolPanelEvent.
#[test]
fn close_button_forwards_cancel_active_tool() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut panel_state = PainterLayersPanelState;

    let outcome = host.apply_panel_event::<PainterLayersPanel>(
        &mut panel_state,
        WidgetEvent::Click(core_ids::PAINTER_LAYERS_CLOSE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Close click was ignored — the `event.rs` arm for PAINTER_LAYERS_CLOSE is missing"
    );

    let actions = host.drained_actions();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::CancelActiveTool)),
        "Close click never reached the bus as CancelActiveTool — the panel→shell seam is dead. \
         drained = {actions:?}"
    );
}

// ── Stroke section seam (the new Blender "Stroke" controls) ──────────────────────
// One test per control SHAPE (toggle / slider / dropdown), proving the real
// WidgetEvent forwards the EXACT PanelEvent the tool's matching arm consumes. A
// forgotten `event.rs` drain or a wrong wire shape leaves the control painted,
// clickable, and silently dead while every unit test stays green (DIRETIVA §2).

/// Toggle shape: the Adjust-Strength toggle forwards `Click(PAINTER_BRUSH_SPACE_ATTEN)` (consumed
/// by the tool's `toggle_brush_space_attenuation` arm) — the surviving Stroke-section toggle.
#[test]
fn stroke_adjust_strength_toggle_forwards_click() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let id = core_ids::PAINTER_BRUSH_SPACE_ATTEN;
    let outcome = host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Adjust-Strength toggle click ignored — the event.rs toggle arm is missing the id"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(i)) if *i == id
        )),
        "Adjust-Strength click never forwarded as Click(PAINTER_BRUSH_SPACE_ATTEN) — seam dead. \
         drained = {actions:?}"
    );
}

/// Slider shape: a Spacing drag forwards `SetValue(PAINTER_BRUSH_SPACING, _)` (consumed by the
/// tool's `set_brush_spacing` arm). All Stroke sliders ride the same `event.rs` arm.
#[test]
fn stroke_spacing_slider_forwards_setvalue() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let id = core_ids::PAINTER_BRUSH_SPACING;
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Spacing slider drag ignored — the event.rs ValueChanged arm is missing the id"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
        )),
        "Spacing drag never forwarded as SetValue(PAINTER_BRUSH_SPACING) — seam dead. \
         drained = {actions:?}"
    );
}

/// Slider shape (the new Stabilizer knob): a drag forwards `SetValue(PAINTER_BRUSH_STABILIZE, _)`
/// (consumed by the tool's `set_brush_stabilizer` arm). Proves the populate registration (now a
/// slider, not the removed toggle) + the `event.rs` drain are both wired.
#[test]
fn stroke_stabilizer_slider_forwards_setvalue() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let id = core_ids::PAINTER_BRUSH_STABILIZE;
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Stabilizer slider drag ignored — the event.rs ValueChanged arm is missing the id"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
        )),
        "Stabilizer drag never forwarded as SetValue(PAINTER_BRUSH_STABILIZE) — seam dead. \
         drained = {actions:?}"
    );
}

/// Dropdown shape: clicking the "Drag Dot" (wire 4) Method option forwards
/// `SelectOption(PAINTER_BRUSH_STROKE_METHOD, "4")` (consumed by `set_brush_stroke_method`).
/// Pins the option-id ↔ decode round-trip for the Stroke Method chip.
#[test]
fn stroke_method_option_forwards_selectoption() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let opt = core_ids::painter_brush_stroke_method_option_id(4);
    let outcome = host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(opt));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Method option click ignored — decode_stroke_method_option is missing"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SelectOption(i, v))
                if *i == core_ids::PAINTER_BRUSH_STROKE_METHOD && v == "4"
        )),
        "Method pick never forwarded as SelectOption(PAINTER_BRUSH_STROKE_METHOD, \"4\") — \
         seam dead. drained = {actions:?}"
    );
}

/// The top-of-panel **Preset** dropdown option forwards `SelectOption(PAINTER_BRUSH_PRESET, idx)` —
/// the seam that reaches `PainterTool::apply_brush_preset`. A missing `decode_brush_preset_option` /
/// option-route row leaves the dropdown painted, clickable and silently dead.
#[test]
fn preset_option_forwards_selectoption() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let opt = core_ids::painter_brush_preset_option_id(1); // Watercolor Basic
    let outcome = host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(opt));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Preset option click ignored — decode_brush_preset_option / option-route row is missing"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SelectOption(i, v))
                if *i == core_ids::PAINTER_BRUSH_PRESET && v == "1"
        )),
        "Preset pick never forwarded as SelectOption(PAINTER_BRUSH_PRESET, \"1\") — seam dead. \
         drained = {actions:?}"
    );
}

/// The Watercolor **Paper** kind dropdown option forwards `SelectOption(PAINTER_WATERCOLOR_PAPER_KIND, k)`
/// (its own id namespace, distinct from the Grain picker) — the seam that reaches `set_brush_paper_kind`.
#[test]
fn paper_kind_option_forwards_selectoption() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let opt = core_ids::painter_paper_kind_option_id(26); // Paper Cold Press
    let outcome = host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(opt));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Paper kind option ignored — decoder/route missing"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SelectOption(i, v))
                if *i == core_ids::PAINTER_WATERCOLOR_PAPER_KIND && v == "26"
        )),
        "Paper kind pick never forwarded — seam dead. drained = {actions:?}"
    );
}

// ── Symmetry section seam (Use / Circular / X-Y-Custom axis / Segments / pick buttons / reset) ──
// Same discipline as the Stroke section above: one assertion per control SHAPE, proving the real
// WidgetEvent forwards the EXACT PanelEvent the tool's `route_brush_jitter_event` arm consumes. A
// forgotten `event.rs` membership leaves the new section painted, clickable and silently dead.

/// Every Symmetry button/checkbox/axis-segment forwards as `ToolPanelEvent(Click(id))` (the
/// `PAINTER_BRUSH_SYMMETRY_CLICKABLE` allowlist arm). Covers Use / Circular / X / Y / Custom /
/// Draw-Line / Pick-Center / Reset in one sweep.
#[test]
fn symmetry_click_controls_forward_their_click() {
    for clicked in core_ids::PAINTER_BRUSH_SYMMETRY_CLICKABLE {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(clicked));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "symmetry control {clicked:?} click ignored — the event.rs Click allowlist is missing the id"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
            )),
            "symmetry control {clicked:?} never forwarded as ToolPanelEvent(Click) — seam dead. \
             drained = {actions:?}"
        );
    }
}

/// Slider shape: a Segments drag forwards `SetValue(PAINTER_BRUSH_SYMMETRY_SEGMENTS, _)` (consumed by
/// the tool's `set_symmetry_segments` arm, which maps the `0..1` track onto the 3..12 count).
#[test]
fn symmetry_segments_slider_forwards_setvalue() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let id = core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS;
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Segments slider drag ignored — the event.rs ValueChanged arm is missing the id"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
        )),
        "Segments drag never forwarded as SetValue(PAINTER_BRUSH_SYMMETRY_SEGMENTS) — seam dead. \
         drained = {actions:?}"
    );
}

/// Apply / Apply & Keep / Delete (the Stroke shape-editor buttons) MUST forward as
/// `ToolPanelEvent(Click(id))` — the `try_apply_brush_event` allowlist was missing them, so the
/// buttons painted, registered hit rects and were SILENTLY DEAD (Enio 2026-06-27 "Apply não funciona").
/// This seam is the exact layer that was broken: paint + populate + tool-route were all fine.
#[test]
fn stroke_apply_buttons_forward_their_click() {
    for clicked in [
        core_ids::PAINTER_BRUSH_STROKE_APPLY,
        core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP,
        core_ids::PAINTER_BRUSH_STROKE_DELETE,
        core_ids::PAINTER_BRUSH_STROKE_EDIT,
        // Simplify + Merge (Enio 2026-07-05): both are momentary curve verbs; the forward allowlist collapsed
        // to `PAINTER_BRUSH_STROKE_BUTTONS.contains()`, so this guards that the array carries them.
        core_ids::PAINTER_BRUSH_STROKE_SIMPLIFY,
        core_ids::PAINTER_BRUSH_STROKE_MERGE,
        // Multi-shape OPERATION segments (Enio 2026-07-04): these were NOT in the forward allowlist, so a
        // real click never reached the tool and the segment never activated. Regression guard.
        core_ids::PAINTER_STROKE_OP_OVERLAY,
        core_ids::PAINTER_STROKE_OP_ADD,
        core_ids::PAINTER_STROKE_OP_REMOVE,
    ] {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(clicked));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "stroke button {clicked:?} click ignored — the try_apply_brush_event allowlist arm is missing"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
            )),
            "stroke button {clicked:?} never forwarded as ToolPanelEvent(Click) — seam dead \
             (the original 'Apply does nothing' bug). drained = {actions:?}"
        );
    }
}

/// The selection curve buttons (Convert / **Merge** / Simplify) MUST forward as `ToolPanelEvent(Click(id))`.
/// Merge was added 2026-07-05 and, like the stroke Apply buttons before it, is registered + painted but would
/// be SILENTLY DEAD if the `try_apply_brush_event` allowlist arm is missing (the populate+forward gotcha).
#[test]
fn selection_curve_buttons_forward_their_click() {
    for clicked in [
        core_ids::PAINTER_SEL_CONVERT,
        core_ids::PAINTER_SEL_MERGE,
        core_ids::PAINTER_SEL_SIMPLIFY,
    ] {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(clicked));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "selection curve button {clicked:?} click ignored — try_apply_brush_event allowlist arm missing"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
            )),
            "selection curve button {clicked:?} never forwarded as ToolPanelEvent(Click) — seam dead. \
             drained = {actions:?}"
        );
    }
}

// ── Watercolor section seam (Wet-edges / Pigment toggles + Edge / Spread / Granulation / Mix) ──────
// Same discipline as the Stroke/Symmetry sweeps: prove the real WidgetEvent forwards the EXACT
// PanelEvent the tool's `route_brush_watercolor_event` arm consumes. A forgotten `event.rs` membership
// (Click allowlist / `is_param_field`) leaves the new section painted, clickable and silently dead.

/// The two Watercolor toggles + the section reset forward as `ToolPanelEvent(Click(id))` (the
/// `PAINTER_WATERCOLOR_CLICKS` allowlist arm). Covers Wet-edges / Pigment / Reset in one sweep.
#[test]
fn watercolor_click_controls_forward_their_click() {
    for clicked in core_ids::PAINTER_WATERCOLOR_CLICKS {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(clicked));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "watercolor control {clicked:?} click ignored — the event.rs Click allowlist is missing the id"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
            )),
            "watercolor control {clicked:?} never forwarded as ToolPanelEvent(Click) — seam dead. \
             drained = {actions:?}"
        );
    }
}

/// Slider shape: each Watercolor number-field (Edge / Spread / Granulation / Mix) drag forwards
/// `SetValue(id, _)` (consumed by the tool's `route_brush_watercolor_event` SetValue arm). Rides the
/// `event.rs` `is_param_field` route — a forgotten membership leaves the slider painted and dead.
#[test]
fn watercolor_sliders_forward_setvalue() {
    for id in core_ids::PAINTER_WATERCOLOR_FIELDS {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "watercolor slider {id:?} drag ignored — the event.rs `is_param_field` arm is missing the id"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
            )),
            "watercolor slider {id:?} never forwarded as SetValue — seam dead. drained = {actions:?}"
        );
    }
}

// ── Per-Layer Color row seam (the "B" blend dropdown + opacity box, Enio 2026-06-29) ──────────────
// One assertion per control SHAPE, proving the real WidgetEvent forwards the EXACT PanelEvent the tool's
// `route_shape_layer_event` arm consumes. These factory-id widgets are paint-time registered, so a
// forgotten event.rs arm would leave them painted, clickable and silently dead.

/// Dropdown shape: clicking blend option (layer 0, Multiply = wire 1) forwards
/// `SelectOption(shape_layer_blend_id(0), "1")` (consumed by `set_brush_shape_layer_blend`).
#[test]
fn shape_layer_blend_option_forwards_selectoption() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let opt = core_ids::painter_shape_layer_blend_option_id(0, 1);
    let outcome = host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(opt));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Shape-layer blend option click ignored — the event.rs shape_layer_picker::blend_option arm is missing"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SelectOption(i, v))
                if *i == core_ids::painter_shape_layer_blend_id(0) && v == "1"
        )),
        "blend pick never forwarded as SelectOption(shape_layer_blend_id(0), \"1\") — seam dead. \
         drained = {actions:?}"
    );
}

/// NumberInput shape: an opacity scrub forwards `SetValue(opacity_id(0), _)` (consumed by the tool's
/// `set_brush_shape_layer_opacity` arm, which edits the source document layer's opacity).
#[test]
fn shape_layer_opacity_forwards_setvalue() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let id = core_ids::painter_shape_layer_opacity_id(0);
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "opacity ValueChanged ignored — the event.rs shape_layer_picker::opacity_index arm is missing"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
        )),
        "opacity scrub never forwarded as SetValue(opacity_id(0)) — seam dead. drained = {actions:?}"
    );
}

/// The Shape section's watercolor **Automatic** checkbox (doc 13 #1) forwards its click as
/// `ToolPanelEvent(Click(id))` — the `event.rs` shape-arm membership. A forgotten arm leaves the
/// checkbox painted, clickable and silently dead (the dead-seam class this suite exists for).
#[test]
fn shape_watercolor_automatic_forwards_its_click() {
    let clicked = core_ids::PAINTER_SHAPE_WATERCOLOR_AUTO;
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(clicked));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Automatic click ignored — event.rs shape Click arm missing the id"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
        )),
        "Automatic never forwarded as ToolPanelEvent(Click) — seam dead. drained = {actions:?}"
    );
}

// ── Impasto section seam (#16) ────────────────────────────────────────────────────────────────────
// The panel can paint a card, register its hits and still be completely dead if the event.rs
// membership is missing — the card looks alive and nothing happens. These two prove the REAL widget
// event forwards the EXACT PanelEvent the tool's `route_brush_impasto_event` arm consumes, for every
// id in the section, so a knob added later cannot ship silently disconnected.

/// Every Impasto **Click** widget (Enable / Reset / the Depth-Source + Draw-To segments / Show Impasto)
/// forwards as `ToolPanelEvent(Click(id))` — the `PAINTER_IMPASTO_CLICKS` allowlist arm in `event.rs`.
#[test]
fn impasto_click_controls_forward_their_click() {
    for clicked in core_ids::PAINTER_IMPASTO_CLICKS {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(clicked));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "impasto control {clicked:?} click ignored — the event.rs Click allowlist is missing the id"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
            )),
            "impasto control {clicked:?} never forwarded as ToolPanelEvent(Click) — seam dead. \
             drained = {actions:?}"
        );
    }
}

/// Every Impasto number-field (Depth / Smoothing / Angle / Elevation / Amount / Shine) forwards
/// `SetValue(id, _)` — the `is_param_field` route.
#[test]
fn impasto_sliders_forward_setvalue() {
    for id in core_ids::PAINTER_IMPASTO_FIELDS {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "impasto slider {id:?} drag ignored — the event.rs `is_param_field` arm is missing the id"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
            )),
            "impasto slider {id:?} never forwarded as SetValue — seam dead. drained = {actions:?}"
        );
    }
}
