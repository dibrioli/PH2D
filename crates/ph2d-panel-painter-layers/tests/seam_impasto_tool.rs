//! **The Impasto tools are ONE list, in ONE place** (Enio, 2026-07-19).
//!
//! They used to be three, and the split was load-bearing on the paint mode: the Body/Material/Lighting
//! cards were painted only in `Paint`, the Plow row only in `Smear`, the Sculpt card only in `Sculpt`.
//! Read that down and the real defect appears — **the Lighting card was reachable in `Paint` and nowhere
//! else**, so entering Sculpt (the mode whose whole purpose is shaping relief) took away the controls
//! that make relief visible.
//!
//! These gates are pointer-driven for the reason the sibling `seam_sculpt.rs` states at length: a widget
//! that paints, registers a hit rect and is forwarded by `event.rs` is still stone dead under the mouse
//! unless `populate` gave it an `InteractiveState`. **A widget is not done when it PAINTS. It is done
//! when a test CLICKS it.**

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// The three modes that act on the paint's body, by the wire string the tool's own router parses — so
/// the fixture cannot drift from the product.
const RELIEF_MODES: [&str; 3] = ["brush", "smear", "sculpt"];

/// A painter in `mode`, with the panel's snapshot published from it exactly as the shell does each frame.
fn tool_in(mode: &str) -> PainterTool {
    let mut tool = PainterTool::default();
    // Impasto's master switch is OFF out of the box, and the Deposit's Body/Material cards live behind
    // it — so a fixture that skipped this would be asserting about a tool the artist has not switched on
    // yet, and the "no dead knobs" claims below would pass by everything being absent.
    tool.toggle_brush_impasto();
    tool.set_paint_tool_mode(mode);
    set_current_brush(Some(tool.brush_settings()));
    tool
}

/// Paint the Brush view and hand back everything the gates need: the host (to click through), the panel
/// state, and the id→rect list the paint produced.
fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

/// The rect a widget was painted at — `None` when it was not painted at all, which for this section is
/// the whole point: a control that does not apply is not painted, so it registers no hit and is inert.
/// Zero-area entries count as absent (a widget the layout collapsed is not on screen either).
fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// Run one real click all the way through: dispatcher → panel → bus → tool.
fn click_through(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    x: f32,
    y: f32,
) {
    for ev in host.click_at(x, y) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// **THE gate: the light is reachable from every mode that shapes relief.**
///
/// This is the defect the reorganisation exists to fix, stated so it can only be true one way. Before,
/// `Show Impasto` lived behind `impasto_applies` — which is `PaintMode::Paint` and nothing else — so the
/// artist entered Sculpt to shape the relief and lost the switch that lights it. Same in the Smear.
///
/// **Mutation that must bleed:** narrow `impasto_section_applies` back to `matches!(.., PaintMode::Paint)`.
/// Two thirds of this gate go red instantly, and no other test in the workspace notices.
#[test]
fn the_light_switch_is_reachable_from_every_mode_that_shapes_relief() {
    for mode in RELIEF_MODES {
        let tool = tool_in(mode);
        let (_host, _st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, core_ids::PAINTER_IMPASTO_SHOW).is_some(),
            "in mode {mode:?} the Lighting card's 'Show Impasto' is not on screen. Relief you cannot \
             light is relief you cannot see — which is what shaping it is FOR."
        );
        assert!(
            rect_of(&rects, core_ids::PAINTER_IMPASTO_LIGHT_ANGLE).is_some(),
            "in mode {mode:?} the lamp's Angle is missing: the Lighting card is per-CANVAS, so it \
             belongs to every tool that touches the body of the paint, not just the depositing one"
        );
    }
}

/// **`Enable` is the section's master, at the top — and the LIGHT survives it.**
///
/// Enio, 2026-07-19: *"Enable do Impasto deve ser colocado no topo da seção impasto já que ele é quem
/// habilita esse modo de pintura."* So it ranks first and gates what follows.
///
/// The exemption is the point, and it is the same law the whole reorganisation turns on: `Enable` says
/// whether *this brush* lays body, `Show Impasto` says whether the *canvas* reveals the body already in
/// the painting. Unticking your brush must not blind you to work you have already done. The engine agrees
/// — `impasto_visible()` reads `impasto_show` and whether any relief exists, and never `brush.impasto`.
///
/// **Mutation that must bleed:** return `y` instead of the Lighting card from the `!brush.impasto` branch.
/// That is the shape the section had for a year, and every other gate here stays green.
#[test]
fn enable_gates_the_tools_but_never_the_light() {
    let mut tool = tool_in("brush");
    tool.toggle_brush_impasto(); // …back OFF (the fixture turns it on)
    assert!(
        !tool.brush_settings().impasto,
        "fixture: Enable must be OFF for this gate to mean anything"
    );
    let (_host, _st, rects) = painted(&tool);

    assert!(
        rect_of(&rects, core_ids::PAINTER_IMPASTO_ENABLE).is_some(),
        "Enable must be painted even when it is off — it is the only way back on"
    );
    assert!(
        rect_of(&rects, core_ids::PAINTER_IMPASTO_SHOW).is_some(),
        "with Enable unticked the artist lost 'Show Impasto'. The relief already on the canvas is still \
         there and still lit; taking away the switch that reveals it is the same defect this section was \
         reorganised to remove, arriving through a different door."
    );
    assert!(
        rect_of(&rects, core_ids::PAINTER_IMPASTO_LIGHT_ANGLE).is_some(),
        "…and the lamp with it: the Lighting card is the canvas's, not the brush's"
    );
    // …while everything the switch DOES govern is gone.
    for (id, what) in [
        (core_ids::PAINTER_IMPASTO_TOOL_DEPOSIT, "the tool list"),
        (core_ids::PAINTER_IMPASTO_LIVE_EDIT, "Adjust Last Stroke"),
        (core_ids::PAINTER_IMPASTO_DEPTH, "the Body card"),
        (core_ids::PAINTER_IMPASTO_SHINE, "the Material card"),
    ] {
        assert!(
            rect_of(&rects, id).is_none(),
            "{what} is still painted with Enable off — the master switch would be decoration"
        );
    }
}

/// **Enable follows you across the tools** — it is the subject's switch, not one slot's.
///
/// Each paint mode keeps its own `BrushSpec`, and once Enable gates the tool list that per-slot flag is a
/// trap with a measured shape: tick it in the Deposit, click **Knife**, and `switch_brush_slot` loads the
/// Smear's own `impasto` (`false`) — which collapses the section to a lone checkbox **and takes away the
/// list you just clicked from**. Measured before the fix: `Deposit true → Knife false → Chisel false`.
///
/// **Mutation that must bleed:** drop the three-slot mirror from `toggle_brush_impasto`. Nothing else
/// notices; the artist just cannot reach the second tool.
#[test]
fn enabling_impasto_reaches_every_tool_not_just_the_one_in_hand() {
    let mut tool = tool_in("brush");
    assert!(tool.brush_settings().impasto, "fixture: Enable is on");
    for (t, name) in [(1u8, "Knife"), (7, "Chisel"), (2, "Smooth"), (0, "Deposit")] {
        tool.set_impasto_tool(t);
        assert!(
            tool.brush_settings().impasto,
            "picking {name} dropped Enable, so the TOOL card it was picked from disappears. Ticking the \
             box says 'I am working with body' — that cannot be true of the brush and false of the \
             knife in the same hand."
        );
        // …and the panel proves the consequence: the list is still there to pick the NEXT tool from.
        let (_host, _st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, core_ids::PAINTER_IMPASTO_TOOL_DEPOSIT).is_some(),
            "after picking {name} the tool list is gone from the panel"
        );
    }
}

/// **All ten tools are on screen, and a POINTER can pick every one of them.**
///
/// ⚠️ Asserting a hit rect is not enough, and the first draft of this gate made exactly that mistake: it
/// found rects for all ten and passed while the chips were **dead under the mouse**, because the fixture
/// built its host with `MockPanelHost::new()` (which skips `populate`, so nothing is focusable). Painted,
/// hit-registered, and inert — the precise failure this file's header warns about, reproduced by the gate
/// meant to catch it. So it clicks: ten pointer presses, ten tools.
#[test]
fn every_impasto_tool_is_reachable_by_a_pointer() {
    for (i, id) in core_ids::PAINTER_IMPASTO_TOOL_IDS.iter().enumerate() {
        // Start each leg from the Deposit, so landing on tool `i` cannot be the tool already being there.
        let mut tool = tool_in("brush");
        let (mut host, mut st, rects) = painted(&tool);
        let rect = rect_of(&rects, *id)
            .unwrap_or_else(|| panic!("tool {i} of the Impasto list was not painted at all"));
        let (x, y) = centre(rect);
        click_through(&mut host, &mut st, &mut tool, x, y);
        assert_eq!(
            tool.brush_settings().impasto_tool,
            i as u8,
            "clicking tool {i} left the painter on tool {}. A chip that paints and hit-registers is \
             still stone dead unless `populate` gave it an InteractiveState.",
            tool.brush_settings().impasto_tool
        );
    }
}

/// **Picking a tool USES it** — the list drives the paint mode, through the tool's own doors.
///
/// A list of tools you cannot pick is a legend, not a tool list. Each leg starts from a DIFFERENT mode so
/// the assertion cannot pass by the tool happening to be there already.
///
/// **Mutation that must bleed:** make `set_impasto_tool` set only the sculpt verb and skip `enter_mode`.
/// The chips still highlight (the panel reads `impasto_tool`, which reads the verb) and nothing paints —
/// so the artist picks the Chisel, strokes the canvas, and lays pigment.
#[test]
fn picking_a_tool_puts_the_painter_in_that_mode() {
    // Deposit → Knife.
    let mut tool = tool_in("brush");
    let (mut host, mut st, rects) = painted(&tool);
    let knife =
        rect_of(&rects, core_ids::PAINTER_IMPASTO_TOOL_KNIFE).expect("the Knife chip is painted");
    let (x, y) = centre(knife);
    click_through(&mut host, &mut st, &mut tool, x, y);
    assert!(
        tool.brush_settings().is_smear,
        "clicking Knife did not put the painter in the Smear: the tool list would be a picture of tools"
    );

    // Knife → a sculpt verb (Chisel, index 5 ⇒ tool 7).
    let (mut host, mut st, rects) = painted(&tool);
    let chisel =
        rect_of(&rects, core_ids::PAINTER_SCULPT_MODE_CHISEL).expect("the Chisel chip is painted");
    let (x, y) = centre(chisel);
    click_through(&mut host, &mut st, &mut tool, x, y);
    let bs = tool.brush_settings();
    assert!(
        bs.is_sculpt,
        "clicking Chisel from the Knife did not enter Sculpt — picking a verb has to select the MODE \
         too, or the chip highlights a tool the canvas is not holding"
    );
    assert_eq!(bs.sculpt_mode, 5, "…and it must be the Chisel, not verb 0");
    assert_eq!(
        bs.impasto_tool, 7,
        "the published tool index must agree with the pair of modes it is derived from"
    );

    // …and back to Deposit.
    let (mut host, mut st, rects) = painted(&tool);
    let deposit = rect_of(&rects, core_ids::PAINTER_IMPASTO_TOOL_DEPOSIT)
        .expect("the Deposit chip is painted");
    let (x, y) = centre(deposit);
    click_through(&mut host, &mut st, &mut tool, x, y);
    let bs = tool.brush_settings();
    assert!(
        !bs.is_sculpt && !bs.is_smear,
        "clicking Deposit did not return the painter to plain Paint"
    );
    assert_eq!(bs.impasto_tool, 0, "…and Deposit is tool 0");
}

/// **Only the selected tool's knobs are painted.**
///
/// The house rule, and this section has already paid a smoke for breaking it: a knob that does nothing to
/// the tool in your hand is a knob that lies about what the tool can do. Dimming is not the answer — a
/// dimmed control still hit-registers, so it is cosmetic.
///
/// The two directions are both asserted, because each alone is a plausible bug: the knob missing where it
/// belongs (the tool is unusable) and the knob present where it does not (the tool lies).
#[test]
fn a_tool_shows_its_own_knobs_and_no_others() {
    // Deposit: Depth yes, Plow no.
    let tool = tool_in("brush");
    let (_host, _st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_IMPASTO_DEPTH).is_some(),
        "the Deposit has no Depth slider — it is the one tool that lays body down"
    );
    assert!(
        rect_of(&rects, core_ids::PAINTER_IMPASTO_PLOW).is_none(),
        "the Deposit is offering Plow, which belongs to the Knife: the brush does not drag existing \
         relief, it lays new"
    );

    // Knife: Plow yes, Depth no.
    let tool = tool_in("smear");
    let (_host, _st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_IMPASTO_PLOW).is_some(),
        "the Knife has no Plow — its only knob"
    );
    assert!(
        rect_of(&rects, core_ids::PAINTER_IMPASTO_DEPTH).is_none(),
        "the Knife is offering Depth. There is no depth to set when nothing is being laid down."
    );

    // A sculpt verb: its own knob, and neither of the two above.
    let mut tool = tool_in("sculpt");
    tool.set_sculpt_mode(0); // Smooth ⇒ the Radius row
    set_current_brush(Some(tool.brush_settings()));
    let (_host, _st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_SCULPT_RADIUS_SLIDER).is_some(),
        "Smooth has no Radius — the kernel's own scale"
    );
    for (id, name) in [
        (core_ids::PAINTER_IMPASTO_DEPTH, "Depth"),
        (core_ids::PAINTER_IMPASTO_PLOW, "Plow"),
    ] {
        assert!(
            rect_of(&rects, id).is_none(),
            "a sculpt verb is offering {name}: the verbs reshape the relief that is there, they neither \
             deposit it nor drag it"
        );
    }
}

/// **Material follows the Deposit, and only the Deposit.**
///
/// The material is per-BRUSH and is baked into the canvas *with the deposit*. Each mode keeps its own
/// brush slot, so a Shine slider under the Knife or a sculpt verb would be editing a slot nothing ever
/// reads — the definition of a dead knob, painted in the section that has been bitten by them most.
///
/// Its sibling claim is in `the_light_switch_is_reachable_from_every_mode_that_shapes_relief`: Material is
/// the brush's and narrows, Lighting is the canvas's and does not. Getting those two backwards is exactly
/// the mistake that produced the layout this replaces.
#[test]
fn material_is_the_deposits_and_lighting_is_everyones() {
    let tool = tool_in("brush");
    let (_host, _st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_IMPASTO_SHINE).is_some(),
        "fixture: the Deposit should show the Material card (else the negatives below are vacuous)"
    );
    for mode in ["smear", "sculpt"] {
        let tool = tool_in(mode);
        let (_host, _st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, core_ids::PAINTER_IMPASTO_SHINE).is_none(),
            "mode {mode:?} is offering the Material card. It is baked with the DEPOSIT, and this mode \
             deposits nothing — the slider would edit a brush slot no pixel ever reads."
        );
        assert!(
            rect_of(&rects, core_ids::PAINTER_IMPASTO_SHOW).is_some(),
            "…while Lighting must still be there in {mode:?}: it is the canvas's, not the brush's"
        );
    }
}

/// **The Impasto section stays away from the modes that have no verb on the list.**
///
/// The negative half of `impasto_section_applies`. Without it, "the section applies to three modes" is
/// only half-asserted, and widening the predicate to *every* mode would pass every gate above.
#[test]
fn the_section_does_not_follow_modes_that_have_no_relief_verb() {
    for mode in ["blur", "clone", "mask", "inpaint"] {
        let tool = tool_in(mode);
        let (_host, _st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, core_ids::PAINTER_IMPASTO_TOOL_DEPOSIT).is_none(),
            "the Impasto tool list is painted in {mode:?}, which has no operation on the paint's body — \
             a section offered where it does not apply is the other half of the same dishonesty"
        );
    }
}
