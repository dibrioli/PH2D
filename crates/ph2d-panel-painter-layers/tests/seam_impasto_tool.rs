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
const RELIEF_MODES: [&str; 3] = ["brush", "knife", "sculpt"];

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

/// **`Enable` is the section's master, at the top, and the Lighting card hides with everything else.**
///
/// Enio, 2026-07-19: *"Enable do Impasto deve ser colocado no topo da seção impasto já que ele é quem
/// habilita esse modo de pintura"* and *"o card Lighting é próprio de Impasto. só deve aparecer se impasto
/// estiver ativo"*. So Enable ranks first and gates the WHOLE section — the Lighting card included.
///
/// ⚠️ This reverses an earlier exemption: I had kept the light reachable with Enable off, on the theory
/// that its controls belong to the canvas rather than the brush. Enio's call is that the Lighting card is
/// part of the Impasto subject and hides with it. The light PASS is a separate matter and untouched —
/// `impasto_visible()` reads `impasto_show` and whether relief exists, never `brush.impasto`, so relief
/// already painted stays LIT; only its controls go away until Impasto is switched back on. This gate is
/// about the CARD's visibility, which is what Enio spoke to.
///
/// **Mutation that must bleed:** paint the Lighting card from the `!brush.impasto` branch anyway. That was
/// the shape a day earlier, and it is exactly what this reversal removes.
#[test]
fn enable_off_hides_the_whole_section_but_leaves_the_way_back_on() {
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
    // Everything the master switch governs is gone — the Lighting card among it (Enio: the card is
    // Impasto's own, so it only appears when Impasto is active).
    for (id, what) in [
        (core_ids::PAINTER_IMPASTO_TOOL_DEPOSIT, "the tool list"),
        (core_ids::PAINTER_IMPASTO_LIVE_EDIT, "Adjust Last Stroke"),
        (core_ids::PAINTER_IMPASTO_DEPTH, "the Body card"),
        (core_ids::PAINTER_IMPASTO_SHINE, "the Material card"),
        (
            core_ids::PAINTER_IMPASTO_SHOW,
            "the Lighting card's Show toggle",
        ),
        (core_ids::PAINTER_IMPASTO_LIGHT_ANGLE, "the lamp Angle"),
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

/// **The Knife and the rail's Smear are two tools, and they disagree about the VOLUME.**
///
/// Enio, 2026-07-19: *"o modo Smear do Impasto (knife) deve ser único e não compartilhado com o smear dos
/// outros tipos de pintura já que ele afeta o Volume do impasto. Smear com botão no painel lateral é o
/// smear dos outros modos de pintura."*
///
/// They ran as one `PaintMode` until then, which meant one `BrushSpec` slot: the **Plow** that makes a
/// knife a knife was also on the ordinary smear, and dialling either moved the other. The separation is
/// only real if their settings are, so that is what this asserts — and `Plow` is the number the whole
/// distinction rests on. (While the knife *was* the Smear, `impasto_plow` was defaulted to `1.0` — *"a
/// faca leva a massa"*, and that measurement stands; split apart it belongs to the Knife, and the plain
/// smear goes back to dragging colour and leaving the body where it is.)
///
/// **Mutation that must bleed:** map `IMPASTO_TOOL_KNIFE` back to `PaintMode::Smear`. The panel looks
/// identical and the two tools silently share one set of settings again.
#[test]
fn the_knife_and_the_plain_smear_are_two_tools_with_two_plows() {
    let knife = tool_in("knife").brush_settings();
    let smear = tool_in("smear").brush_settings();
    assert!(
        knife.impasto_plow > 0.0,
        "the Knife carries no body (Plow {}) — that IS the knife",
        knife.impasto_plow
    );
    assert_eq!(
        smear.impasto_plow, 0.0,
        "the rail's Smear is ploughing the impasto. It is the smear of the OTHER painting modes: it \
         drags the colour and leaves the body where it is."
    );
    // …and editing one must not reach the other: separate modes mean separate brush slots.
    let mut tool = tool_in("knife");
    tool.set_brush_impasto_plow(0.25);
    assert!(
        (tool.brush_settings().impasto_plow - 0.25).abs() < 1e-6,
        "fixture: the Knife's Plow took the edit"
    );
    tool.set_paint_tool_mode("smear");
    assert_eq!(
        tool.brush_settings().impasto_plow,
        0.0,
        "dialling the Knife's Plow moved the plain Smear's too — they are sharing a slot, which is the \
         thing that was supposed to stop"
    );
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

/// **The impasto cards, exactly as Enio refined them across the 2026-07-22 smoke** — and none of them
/// with Enable off:
///
/// - **Body, Material and Lighting are PERMANENT** (their edits fan out to the relief slots on the
///   tool side, so none is a dead knob under any tool);
/// - the two tool-SPECIFIC cards follow their tools: **Sculpt only with a verb in hand** (directly
///   below TOOL — order pinned by y), **Knife only while the Knife is selected**;
/// - the **Filter buttons follow the verb in hand**: Smooth/Sharpen/Inflate offer them, a plane verb
///   (Flatten) does not, and no other tool sees them at all.
///
/// One representative knob per card; the OFF half keeps the gate honest — presence alone would stay
/// green if the Enable stopped gating anything.
#[test]
fn every_card_is_painted_when_impasto_is_on_and_none_when_off() {
    let permanent: [(ph2d_a11y::NodeId, &str); 3] = [
        (core_ids::PAINTER_IMPASTO_DEPTH, "Body/Depth"),
        (core_ids::PAINTER_IMPASTO_SHINE, "Material/Shine"),
        (core_ids::PAINTER_IMPASTO_SHOW, "Lighting/Show"),
    ];
    for mode in RELIEF_MODES {
        let mut tool = tool_in(mode);
        tool.set_sculpt_mode(0); // Smooth ⇒ the Sculpt card (in hand) shows the Radius row
        set_current_brush(Some(tool.brush_settings()));
        let (_host, _st, rects) = painted(&tool);
        for (id, name) in permanent {
            assert!(
                rect_of(&rects, id).is_some(),
                "with Impasto ON in {mode:?}, the {name} card is missing"
            );
        }
        // The tool-specific pair follows its tools.
        let plow = rect_of(&rects, core_ids::PAINTER_IMPASTO_PLOW);
        let radius = rect_of(&rects, core_ids::PAINTER_SCULPT_RADIUS_SLIDER);
        let filter = rect_of(&rects, core_ids::PAINTER_SCULPT_FILTER);
        match mode {
            "knife" => {
                assert!(plow.is_some(), "the Knife in hand must show its Plow card");
                assert!(radius.is_none(), "the Sculpt card needs a verb in hand");
            }
            "sculpt" => {
                assert!(plow.is_none(), "sculpt must not offer the Knife card");
                // Order: the Sculpt card sits directly below TOOL, Body after it.
                let tool_y = rect_of(&rects, core_ids::PAINTER_IMPASTO_TOOL_DEPOSIT)
                    .expect("TOOL chips painted")
                    .y;
                let sculpt_y = radius.expect("a verb in hand must show its card").y;
                let body_y = rect_of(&rects, core_ids::PAINTER_IMPASTO_DEPTH)
                    .expect("asserted above")
                    .y;
                assert!(
                    tool_y < sculpt_y && sculpt_y < body_y,
                    "card order must be TOOL < Sculpt < Body (got {tool_y} / {sculpt_y} / {body_y})"
                );
                assert!(filter.is_some(), "Smooth in hand must offer Filter Layer");
            }
            _ => {
                assert!(
                    plow.is_none() && radius.is_none(),
                    "mode {mode:?} is offering a tool-specific card it does not hold"
                );
            }
        }
        if mode != "sculpt" {
            assert!(
                filter.is_none(),
                "mode {mode:?} is offering Filter Layer — the buttons need the verb IN HAND"
            );
        }
        // A plane verb in hand shows its card but not the Filter buttons.
        tool.set_sculpt_mode(2); // Flatten
        set_current_brush(Some(tool.brush_settings()));
        let (_host, _st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, core_ids::PAINTER_SCULPT_FILTER).is_none(),
            "Flatten is offering Filter Layer in {mode:?} — the buttons follow the verbs that use them"
        );
        tool.set_sculpt_mode(0);
        // …and the OFF half: unticking Enable takes every card with it.
        tool.toggle_brush_impasto();
        set_current_brush(Some(tool.brush_settings()));
        let (_host, _st, rects) = painted(&tool);
        for (id, name) in permanent {
            assert!(
                rect_of(&rects, id).is_none(),
                "with Impasto OFF in {mode:?}, the {name} card must not paint"
            );
        }
        for (id, name) in [
            (core_ids::PAINTER_IMPASTO_PLOW, "Knife"),
            (core_ids::PAINTER_SCULPT_RADIUS_SLIDER, "Sculpt"),
        ] {
            assert!(
                rect_of(&rects, id).is_none(),
                "with Impasto OFF in {mode:?}, the {name} card must not paint either"
            );
        }
    }
}

/// **A Material edit reaches the DEPOSIT slot from every tool.**
///
/// The Material card is visible under every impasto tool (all-cards rule), but only the Deposit's brush
/// slot is ever baked — so the tool fans a material write out to the three relief slots
/// (`set_material_field`, the `toggle_brush_impasto` pattern). Without the fan-out, dialling Shine while
/// holding the Knife writes a slot nothing reads: the knob would be alive on screen and dead in the
/// paint. Mutation that bleeds: reverting the setters to the active-slot-only write.
#[test]
fn a_material_edit_under_any_tool_reaches_the_deposit_slot() {
    for mode in ["knife", "sculpt"] {
        let mut tool = tool_in(mode);
        tool.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SetValue(
            core_ids::PAINTER_IMPASTO_SHINE,
            0.91,
        ));
        // Switch back to the Deposit: ITS slot must carry the edit.
        tool.set_paint_tool_mode("brush");
        let shine = tool.brush_settings().impasto_shine;
        assert!(
            (shine - 0.91).abs() < 1e-6,
            "Shine dialled under {mode:?} never reached the deposit slot (reads {shine})"
        );
    }
}

/// **The Impasto section stays away from the modes that have no verb on the list.**
///
/// The negative half of `impasto_section_applies`. Without it, "the section applies to three modes" is
/// only half-asserted, and widening the predicate to *every* mode would pass every gate above.
#[test]
fn the_section_does_not_follow_modes_that_have_no_relief_verb() {
    // ⚠️ the plain **Smear** is on this list now: since the Knife became its own mode it is the
    // smear "dos outros modos de pintura" and has no operation on the paint's body.
    for mode in ["smear", "blur", "clone", "mask", "inpaint"] {
        let tool = tool_in(mode);
        let (_host, _st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, core_ids::PAINTER_IMPASTO_TOOL_DEPOSIT).is_none(),
            "the Impasto tool list is painted in {mode:?}, which has no operation on the paint's body — \
             a section offered where it does not apply is the other half of the same dishonesty"
        );
    }
}
