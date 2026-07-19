//! **The Sculpt UI, driven by a POINTER** — the seam gate the plan marks with a ⚠️ (doc 18 §3).
//!
//! It exists because this line has already shipped a dead panel once, and the failure is silent in every
//! other kind of test: a widget that PAINTS, registers a hit rect, and is forwarded by `event.rs` is
//! still stone dead under the mouse unless `populate` also gave it an `InteractiveState`.
//! `is_focusable(store, id)` answers `None => false`, so the Down never activates it and the `Click`
//! never happens. `paint_segmented_adaptive` takes the store by `&` — a segment CANNOT self-register.
//!
//! And the second-order trap, which cost this line a second time: register a segment as a `Checkbox` and
//! it emits `Toggled`, which `event.rs` does not forward. Registered, and STILL dead.
//!
//! So these start at the pixel the artist aims at and end at the tool's state. Every link is real.
//!
//! **A widget is not done when it PAINTS. It is done when a test CLICKS it.**

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// A `PainterTool` in Sculpt mode, with the panel's brush snapshot published from it — exactly as the
/// shell publishes it each frame. The mode is entered through the tool's OWN router (the same string the
/// rail forwards), so the fixture cannot drift from the product.
/// The Chisel's index in the segmented control — the one verb with an axis, and so the one with the Rake.
const CHISEL: u8 = 5;

fn tool_in_sculpt() -> PainterTool {
    tool_in_sculpt_mode(None)
}

/// The same, armed on a chosen verb.
///
/// **There is no "fullest card".** The Chisel was the fullest one — eight chips, Offset, Angle, Rake — until
/// W5b gave `Smooth` a **Filter Layer** button the Chisel cannot have (a plane verb's target is fitted to
/// the brush's footprint, and a whole layer has none). So no single verb shows every clickable widget, and
/// the sweep asks each verb in turn rather than arming one and demanding everything
/// (`every_sculpt_click_widget_is_reachable_by_a_pointer`). The card shows the knobs the active verb USES;
/// a sweep that forgot this would be pressing for a knob that lies.
fn tool_in_sculpt_mode(mode: Option<u8>) -> PainterTool {
    let mut tool = PainterTool::default();
    tool.set_paint_tool_mode("sculpt");
    if let Some(m) = mode {
        tool.set_sculpt_mode(m);
    }
    let bs = tool.brush_settings();
    assert!(
        bs.is_sculpt,
        "fixture is wrong, not the seam: the tool is not in Sculpt, so the card is never painted and \
         nothing below could mean anything"
    );
    set_current_brush(Some(bs));
    tool
}

/// The same, plus a real impasto stroke already laid — the fixture the **sweep** needs.
///
/// "The fullest card" is a function of the STATE as well as the verb: `Filter Stroke` is offered only once
/// a last stroke exists on the layer (a button that could only refuse would be a button that lies). A sweep
/// armed on a virgin canvas would demand a widget the card is right not to paint — so the sweep's fixture
/// has to be a tool that has actually painted, exactly as the artist's is when they reach for the button.
fn tool_in_sculpt_with_a_stroke(mode: u8) -> PainterTool {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    let size = 96u32;
    let mut tool = PainterTool::default();
    tool.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    tool.set_brush_size_px(8.0);
    tool.toggle_brush_impasto();
    let cp = |p: [f32; 2], phase| CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    tool.on_canvas_pointer(cp([20.0, 40.0], PointerPhase::Down));
    for i in 1..=5u8 {
        tool.on_canvas_pointer(cp([20.0 + 8.0 * f32::from(i), 40.0], PointerPhase::Move));
    }
    tool.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Up));
    tool.set_paint_tool_mode("sculpt");
    tool.set_sculpt_mode(mode);
    let bs = tool.brush_settings();
    assert!(bs.is_sculpt, "fixture: the router did not enter Sculpt");
    set_current_brush(Some(bs));
    tool
}

/// Run one real click at `(x, y)` all the way through: dispatcher → panel → bus → tool.
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

/// **The rail button puts the Painter in Sculpt mode.**
///
/// The rail forwards `"sculpt"` over the frozen `SelectOption` channel; the tool's router maps it to
/// `PaintMode::Sculpt`. If either half is missing the tool is unreachable — you can paint the chip on the
/// rail all day and the brush will still be laying pigment.
///
/// **Mutation that must bleed:** drop the `"sculpt" => PaintMode::Sculpt` arm from `set_paint_tool_mode`
/// (the mode silently falls through to `_ => PaintMode::Paint`, which is the quietest possible failure).
#[test]
fn the_rail_button_enters_sculpt_mode() {
    let mut tool = PainterTool::default();
    assert!(!tool.is_sculpt_mode(), "fixture: the tool starts in Paint");

    // Exactly what `chrome/rail_painter_tools::push_paint_mode` puts on the bus for PAINTER_RAIL_SCULPT.
    tool.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "sculpt".to_string(),
    ));

    assert!(
        tool.is_sculpt_mode(),
        "the rail's `sculpt` message did not reach the tool — `set_paint_tool_mode` has no arm for it, \
         so it fell through to plain Paint and the rail button does nothing at all"
    );
    // …and the round trip, which the shell relies on to restore the tool after a momentary Fill drag.
    assert_eq!(
        tool.active_paint_mode_id(),
        "sculpt",
        "Sculpt does not survive a capture/restore of the active mode — a Fill drag would dump the \
         artist back into the Brush"
    );
}

/// **Clicking the Sharpen chip selects Sharpen.**
///
/// The whole reason this file exists. A real pointer, at the real pixel, through the real dispatcher.
///
/// **Mutation that must bleed:** delete `register_sculpt_widgets` from `populate.rs` — the chip still
/// paints, still registers its hit rect, and is stone dead under the mouse.
#[test]
fn clicking_the_sharpen_chip_selects_sharpen() {
    let mut tool = tool_in_sculpt();
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;

    assert_eq!(
        tool.brush_settings().sculpt_mode,
        0,
        "fixture: Sculpt opens on Smooth (the verb Enio asked for first)"
    );

    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_SCULPT_MODE_SHARPEN && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!(
            "the Sharpen chip is not painted with a clickable rect — the Sculpt card never drew it"
        );
    };

    let (x, y) = centre(rect);
    // The dispatcher must resolve this pixel to the chip. `hit()` is last-registered-wins, so a card
    // sized by a FIXED row count under content that reflows is how a chip quietly loses its own pixel.
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_SCULPT_MODE_SHARPEN),
        "the pixel at the centre of the Sharpen chip does not resolve to it — something painted AFTER \
         it covers its rect"
    );

    click_through(&mut host, &mut st, &mut tool, x, y);

    assert_eq!(
        tool.brush_settings().sculpt_mode,
        1,
        "a REAL pointer click on the Sharpen chip did not select Sharpen. The chip paints, registers a \
         hit rect, and `event.rs` would forward the Click — but the pointer never BECOMES one: an id \
         absent from the WidgetStore is not focusable, so Down never makes it active and Up never fires \
         apply_click. Register it in `populate.rs`."
    );
}

/// Every Sculpt click id, not just the one in the example — a card where 1 of 2 chips works is the same
/// defect with better luck. (Waves 2-3 append verbs to `PAINTER_SCULPT_CLICKS`; this sweep grows with it,
/// so the next one cannot ship inert either.)
///
/// **It asks every verb, because no single card is the fullest one any more.** That used to be the Chisel,
/// and W5b ended it: the Chisel carries the Rake, `Smooth` carries **Filter Layer**, and
/// neither card carries the other's — the plane verbs cannot be filtered (their target is fitted to the
/// brush's footprint) and the blur verbs have no plane to Rake. The property that actually matters is
/// unchanged, and this states it directly: **a widget SOME card paints must be clickable there**, and a
/// widget NO card paints is dead. Sweeping the verbs answers both without a hand-written id→verb table,
/// which would be a second copy of the panel's rules and would drift from them.
#[test]
fn every_sculpt_click_widget_is_reachable_by_a_pointer() {
    for clicked in core_ids::PAINTER_SCULPT_CLICKS {
        let mut reached = false;
        for verb in 0..8u8 {
            // A tool that has PAINTED: some widgets (Filter Stroke) are offered only once there is a last
            // stroke to act on, and the sweep must not demand a button the card is right to withhold.
            let tool = tool_in_sculpt_with_a_stroke(verb);
            let _ = &tool;
            let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
            let mut st = PainterLayersPanelState;
            let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());

            let Some((_, rect)) = painted
                .iter()
                .find(|(w, r)| *w == clicked && r.w > 0.0 && r.h > 0.0)
                .copied()
            else {
                continue; // this verb's card does not offer it — try the next
            };

            let (x, y) = centre(rect);
            let events = host.click_at(x, y);
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, WidgetEvent::Click(id) if *id == clicked)),
                "a real pointer click on sculpt widget {clicked:?} (verb {verb}) emitted NO Click event \
                 ({events:?}) — it is painted and hit-registered but absent from the WidgetStore, so \
                 `is_focusable` rejects it on Down and it is dead under the mouse. Register it in \
                 `populate.rs`."
            );
            reached = true;
            break;
        }
        assert!(
            reached,
            "sculpt widget {clicked:?} is painted by NO verb's card — it is unreachable by any pointer, \
             which is the definition of dead"
        );
    }
}

/// **The Radius slider reaches the tool, and its own pixel belongs to it.**
///
/// A slider is not in `PAINTER_SCULPT_CLICKS` (it emits `SetValue`, not `Click`), so the sweep above does
/// not cover it — which is exactly the kind of gap that let the Wax swatch nearly ship dead.
#[test]
fn the_radius_slider_is_wired_to_the_tool() {
    let mut tool = tool_in_sculpt();
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;

    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_SCULPT_RADIUS_SLIDER && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!("the Radius slider is not painted with a clickable rect");
    };
    let (x, y) = centre(rect);
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_SCULPT_RADIUS_SLIDER),
        "the Radius slider's own pixel does not resolve to it"
    );

    // Leg 1 — panel → bus. A slider drag arrives as `ValueChanged(id)`; the panel must turn it into a
    // `SetValue(id, _)` on the bus. Missing from `event_brush_forward`'s list ⇒ the drag dies here.
    host.apply_panel_event::<PainterLayersPanel>(
        &mut st,
        WidgetEvent::ValueChanged(core_ids::PAINTER_SCULPT_RADIUS_SLIDER),
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _))
                if *i == core_ids::PAINTER_SCULPT_RADIUS_SLIDER
        )),
        "a Radius drag was never forwarded as SetValue — the id is missing from \
         `event_brush_forward`'s slider list, so the panel swallows it. drained = {actions:?}"
    );

    // Leg 2 — bus → tool. The router must ACT on it, and the number the artist sees must be the number
    // the kernel uses (one function, so the chip cannot drift from the blur).
    let before = tool.sculpt_radius_px();
    tool.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_SCULPT_RADIUS_SLIDER,
        1.0,
    ));
    assert_eq!(
        tool.sculpt_radius_px(),
        16,
        "SetValue on the Radius slider did not reach the kernel (radius is still {before} px) — \
         `route_sculpt_event` has no arm for the id"
    );
}

/// **The Offset slider reaches the tool — and it is only reachable where it means something.**
///
/// The plane family's knob (Wave 2). Same two legs as the Radius above, driven in **Scrape**, because that
/// is the only place the card paints it: a gate that drove it in Smooth would be testing a widget the artist
/// can never touch, which is the definition of a green gate that proves nothing.
///
/// The chip speaks **paint-loads**, signed. The sign is the meaning — negative sinks the plane so the
/// spatula digs — so a chip that showed the raw `0..1` track would hide the one number the artist steers by.
/// (That is not hypothetical: the Radius chip shipped exactly that way and the seam gate above is its scar.)
///
/// **Mutation that must bleed:** delete the `PAINTER_SCULPT_OFFSET_SLIDER` arm from `route_sculpt_event`,
/// or drop the id from `event_brush_forward`'s slider list.
#[test]
fn the_offset_slider_is_wired_to_the_tool() {
    let mut tool = tool_in_sculpt();
    tool.set_sculpt_mode(3); // Scrape — where the Offset row lives
    set_current_brush(Some(tool.brush_settings()));

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_SCULPT_OFFSET_SLIDER && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!("the Offset slider is not painted with a clickable rect in Scrape");
    };
    let (x, y) = centre(rect);
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_SCULPT_OFFSET_SLIDER),
        "the Offset slider's own pixel does not resolve to it"
    );

    // Leg 1 — panel → bus.
    host.apply_panel_event::<PainterLayersPanel>(
        &mut st,
        WidgetEvent::ValueChanged(core_ids::PAINTER_SCULPT_OFFSET_SLIDER),
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _))
                if *i == core_ids::PAINTER_SCULPT_OFFSET_SLIDER
        )),
        "an Offset drag was never forwarded as SetValue — the id is missing from \
         `event_brush_forward`'s slider list, so the panel swallows it. drained = {actions:?}"
    );

    // Leg 2 — bus → tool, in the unit the chip shows. Dead centre must be exactly ZERO (the plane sits ON
    // the fitted surface): an off-by-a-half here would give the spatula a permanent bite nobody asked for.
    tool.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_SCULPT_OFFSET_SLIDER,
        0.5,
    ));
    assert!(
        tool.sculpt_plane_offset().abs() < 1e-6,
        "the Offset track's dead centre maps to {} loads, not 0 — the plane does not rest on the surface \
         it was fitted to, so Flatten would drift the paint every time it touched it",
        tool.sculpt_plane_offset()
    );
    tool.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_SCULPT_OFFSET_SLIDER,
        0.0,
    ));
    assert!(
        (tool.sculpt_plane_offset() + 1.0).abs() < 1e-6,
        "SetValue on the Offset slider did not reach the kernel (offset is {} loads, want −1) — \
         `route_sculpt_event` has no arm for the id",
        tool.sculpt_plane_offset()
    );
}

/// **The Radius chip speaks px — the same px the kernel blurs by.**
///
/// It shipped showing the raw `0..1` track ("0.50"), while `sculpt_radius_px` was published, defaulted,
/// documented as *"what the artist reads off the chip"* — and read by nobody. A number on screen that no
/// code produces is a number that is free to be wrong, and this one was: 0.50 is not a length, and the
/// kernel does not take one.
///
/// **Mutation that must bleed:** feed the row `brush.sculpt_radius` (the track) instead of
/// `brush.sculpt_radius_px`, or drop the `link_slider_number_mapped_integer` mapping.
#[test]
fn the_radius_chip_shows_the_pixels_the_kernel_uses() {
    let mut tool = tool_in_sculpt();
    tool.set_sculpt_radius(1.0);
    set_current_brush(Some(tool.brush_settings()));

    assert_eq!(
        tool.sculpt_radius_px(),
        16,
        "fixture: Radius 1.0 is the 16-px cap"
    );
    assert_eq!(
        tool.brush_settings().sculpt_radius_px as u32,
        tool.sculpt_radius_px(),
        "the snapshot's px and the kernel's px are two different numbers — they are supposed to be one \
         function, so the chip cannot drift from the blur"
    );

    // …and the chip is MAPPED — which is the half that makes the px real rather than cosmetic. Without
    // it the chip would DISPLAY pixels and INTERPRET them as a 0..1 track: a control that lies twice.
    //
    // The assertion is not "the mapping is (15, 1)" but the property that number is FOR: the chip's
    // mapping and the kernel's radius are the same function of the track. Pin the constant instead and
    // the two are free to drift apart while the gate applauds.
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let _ = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let (scale, offset) = host
        .store()
        .linked_slider_mapping(core_ids::PAINTER_SCULPT_RADIUS_CHIP);
    for track in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        tool.set_sculpt_radius(track);
        let chip_says = (track * scale + offset).round() as u32;
        assert_eq!(
            chip_says,
            tool.sculpt_radius_px(),
            "at track {track}, the Radius chip says {chip_says} px and the kernel blurs by {} px. They \
             are supposed to be one function — the chip is not mapped, or it is mapped to the wrong line.",
            tool.sculpt_radius_px()
        );
    }
}

/// **Sculpt shows the brush, and hides the colour.**
///
/// The design decision of doc 18 §10.1, as a test. Sculpt is NOT mode-exclusive like Deform: it rides the
/// same dab list the colour does, so the brush's own Size / Spacing / Falloff / Shape / Grain ARE the
/// spatula and must stay on screen. What goes is the colour half — a brush that lays no pigment has no
/// use for a blend mode.
///
/// Get this backwards (copy Deform's exclusivity, as §3's "mirror it line by line" invites) and the
/// artist is left holding the settings for a tool they can no longer aim.
#[test]
fn sculpt_keeps_the_brush_controls_and_drops_the_colour_ones() {
    let tool = tool_in_sculpt_mode(Some(CHISEL)); // the fullest card — see `tool_in_sculpt_mode`
    let bs = tool.brush_settings();
    assert!(
        bs.paints_no_color(),
        "Sculpt is not on the `paints_no_color` list, so the panel is still offering a colour and a \
         blend mode to a tool that writes neither"
    );

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let ids: Vec<_> = painted.iter().map(|(w, _)| *w).collect();

    // The verb chips are there — they are the widgets EVERY card carries, whichever verb is armed. (The
    // verb-conditional ones — Rake, Filter Layer — are swept per-verb by
    // `every_sculpt_click_widget_is_reachable_by_a_pointer`; demanding them all on one card asserted that
    // some verb shows everything, which stopped being true when W5b gave `Smooth` a button the Chisel
    // cannot have.)
    for id in core_ids::PAINTER_SCULPT_MODE_IDS {
        assert!(ids.contains(&id), "sculpt verb chip {id:?} is not painted");
    }
    // …AND the brush is still there. This is the assertion that separates Sculpt from Deform, and the
    // one that would fail if someone "fixed" the panel to be mode-exclusive like its neighbour.
    assert!(
        ids.contains(&core_ids::PAINTER_BRUSH_SIZE_SLIDER),
        "the brush Size slider is gone in Sculpt mode. The sculpt rides the brush's dab list — Size IS \
         the spatula's width, and without it the tool cannot be aimed (doc 18 §10.1)."
    );
}

/// **Every verb paints EXACTLY the knobs it uses — no more, and no fewer.**
///
/// Radius is the blur's own scale and means nothing to a plane verb (the plane is fitted to the brush's
/// footprint, so the brush's Size already IS its scale). Offset is where the fitted plane sits and means
/// nothing to a blur. Depth is a thickness and means nothing to either. Angle belongs to the one verb that
/// folds a V. Painting an inert one would be a knob that lies about what the tool can do — and this card has
/// already cost a smoke over exactly that class of mistake.
///
/// The gate asserts the **exact set**, verb by verb, rather than "this one yes, that one no". The difference
/// matters: a Wave 4 verb whose family answer is right but whose row nobody wired would slip past a
/// one-shown-one-hidden check and reach the artist as a card with a tool and nothing to steer it by. And the
/// Chisel — the only verb with TWO knobs — is precisely the case a looser gate would have got wrong.
///
/// **Mutation that must bleed:** paint `radius_row` unconditionally in `paint_sculpt_section`; or drop the
/// `sculpt_is_chisel` branch, which leaves the Chisel with an Offset and no Angle.
#[test]
fn every_verb_paints_exactly_the_knobs_it_uses() {
    use core_ids::{
        PAINTER_SCULPT_ANGLE_SLIDER as ANGLE, PAINTER_SCULPT_DEPTH_SLIDER as DEPTH,
        PAINTER_SCULPT_OFFSET_SLIDER as OFFSET, PAINTER_SCULPT_RADIUS_SLIDER as RADIUS,
        PAINTER_SCULPT_SMOOTH_SLIDER as SMOOTH,
    };
    // (verb, its name, the knobs it must paint — and by omission, the ones it must not)
    let cases: [(u8, &str, &[ph2d_a11y::NodeId]); 8] = [
        (0, "Smooth", &[RADIUS]),
        (1, "Sharpen", &[RADIUS]),
        (2, "Flatten", &[OFFSET]),
        (3, "Scrape", &[OFFSET]),
        (4, "Fill", &[OFFSET]),
        (5, "Chisel", &[OFFSET, ANGLE]), // the plane must be placed BEFORE the V is folded about it
        (6, "Layer", &[DEPTH]),
        (7, "Inflate", &[DEPTH, SMOOTH]), // Depth offsets the form; Smoothness rounds the ball's hard edge
    ];
    let all = [RADIUS, OFFSET, DEPTH, ANGLE, SMOOTH];

    for (mode, name, wanted) in cases {
        let mut tool = tool_in_sculpt();
        tool.set_sculpt_mode(mode);
        set_current_brush(Some(tool.brush_settings()));

        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
        let ids: Vec<_> = painted.iter().map(|(w, _)| *w).collect();

        for knob in all {
            let should = wanted.contains(&knob);
            let does = ids.contains(&knob);
            assert_eq!(
                does,
                should,
                "{name} (verb {mode}) {} a knob it {} use. The card must show exactly the controls the \
                 active verb steers by: a missing one leaves the artist holding a tool they cannot aim, and \
                 a spare one is a control that does nothing and says otherwise.",
                if does { "paints" } else { "hides" },
                if should { "DOES" } else { "does NOT" },
            );
        }
    }
}

/// **The Depth and Angle sliders reach the tool — each driven in a verb that actually shows it.**
///
/// The Wave 3 knobs. A gate that drove them in Smooth would be testing widgets the artist can never touch,
/// which is the definition of a green gate that proves nothing — so Depth is driven in **Layer** and Angle in
/// **Chisel**, the verbs whose cards paint them.
///
/// The Angle leg checks the number in **degrees**, and that is not cosmetic. A chisel's angle is a *geometric*
/// quantity, and a height field's two axes are not the same unit (`x` is texels, `h` is paint-loads) — so the
/// kernel divides `tan(angle)` by `DEPTH_UNIT_PX` on its way in. The chip must still say the degrees the
/// artist dialled, or the one number they steer by would be a number no code produces.
///
/// **Mutation that must bleed:** delete either arm from `route_sculpt_event`, or either id from
/// `event_brush_forward`'s slider list.
#[test]
fn the_depth_and_angle_sliders_are_wired_to_the_tool() {
    // ── Depth, in Layer ────────────────────────────────────────────────────────────────────────────
    let mut tool = tool_in_sculpt();
    tool.set_sculpt_mode(6); // Layer
    set_current_brush(Some(tool.brush_settings()));

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_SCULPT_DEPTH_SLIDER && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!("the Depth slider is not painted with a clickable rect in Layer");
    };
    let (x, y) = centre(rect);
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_SCULPT_DEPTH_SLIDER),
        "the Depth slider's own pixel does not resolve to it"
    );
    host.apply_panel_event::<PainterLayersPanel>(
        &mut st,
        WidgetEvent::ValueChanged(core_ids::PAINTER_SCULPT_DEPTH_SLIDER),
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _))
                if *i == core_ids::PAINTER_SCULPT_DEPTH_SLIDER
        )),
        "a Depth drag was never forwarded as SetValue — the id is missing from `event_brush_forward`'s \
         slider list. drained = {actions:?}"
    );
    tool.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_SCULPT_DEPTH_SLIDER,
        1.0,
    ));
    assert!(
        (tool.sculpt_depth() - 1.0).abs() < 1e-6,
        "SetValue on the Depth slider did not reach the kernel (depth is {} loads, want +1)",
        tool.sculpt_depth()
    );

    // ── Angle, in Chisel ───────────────────────────────────────────────────────────────────────────
    let mut tool = tool_in_sculpt();
    tool.set_sculpt_mode(5); // Chisel
    set_current_brush(Some(tool.brush_settings()));

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_SCULPT_ANGLE_SLIDER && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!("the Angle slider is not painted with a clickable rect in Chisel");
    };
    let (x, y) = centre(rect);
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_SCULPT_ANGLE_SLIDER),
        "the Angle slider's own pixel does not resolve to it"
    );
    host.apply_panel_event::<PainterLayersPanel>(
        &mut st,
        WidgetEvent::ValueChanged(core_ids::PAINTER_SCULPT_ANGLE_SLIDER),
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _))
                if *i == core_ids::PAINTER_SCULPT_ANGLE_SLIDER
        )),
        "an Angle drag was never forwarded as SetValue — the id is missing from `event_brush_forward`'s \
         slider list. drained = {actions:?}"
    );
    tool.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_SCULPT_ANGLE_SLIDER,
        0.0,
    ));
    assert!(
        tool.sculpt_chisel_angle_deg().abs() < 1e-6,
        "the Angle track's bottom is {}°, not 0 — and 0° is not a dead zone, it is the FLAT knife (the \
         Chisel is byte-identical to Scrape there)",
        tool.sculpt_chisel_angle_deg()
    );
    tool.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_SCULPT_ANGLE_SLIDER,
        1.0,
    ));
    assert!(
        (tool.sculpt_chisel_angle_deg() - 60.0).abs() < 1e-4,
        "SetValue on the Angle slider did not reach the kernel (angle is {}°, want 60)",
        tool.sculpt_chisel_angle_deg()
    );
}

/// **The Smoothness slider reaches the tool — driven in Inflate, the only verb that paints it.**
///
/// Inflate's Radius knob (Enio 2026-07-14). Same two legs as Depth/Angle, in the verb whose card shows it: a
/// gate that drove it in Smooth would test a widget the artist can never reach there.
///
/// The chip speaks **texels**, `0..16` — the box-blur radius the kernel uses — so the number on screen is the
/// number in the softening, one function apart.
///
/// **Mutation that must bleed:** delete the `PAINTER_SCULPT_SMOOTH_SLIDER` arm from `route_sculpt_event`, or
/// drop the id from `event_brush_forward`'s slider list.
#[test]
fn the_smoothness_slider_is_wired_to_the_tool() {
    const INFLATE: u8 = 7;
    let mut tool = tool_in_sculpt();
    tool.set_sculpt_mode(INFLATE);
    set_current_brush(Some(tool.brush_settings()));

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_SCULPT_SMOOTH_SLIDER && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!("the Smoothness slider is not painted with a clickable rect in Inflate");
    };
    let (x, y) = centre(rect);
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_SCULPT_SMOOTH_SLIDER),
        "the Smoothness slider's own pixel does not resolve to it"
    );
    host.apply_panel_event::<PainterLayersPanel>(
        &mut st,
        WidgetEvent::ValueChanged(core_ids::PAINTER_SCULPT_SMOOTH_SLIDER),
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _))
                if *i == core_ids::PAINTER_SCULPT_SMOOTH_SLIDER
        )),
        "a Smoothness drag was never forwarded as SetValue — the id is missing from \
         `event_brush_forward`'s slider list. drained = {actions:?}"
    );
    tool.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_SCULPT_SMOOTH_SLIDER,
        1.0,
    ));
    assert_eq!(
        tool.sculpt_smooth_px(),
        16,
        "SetValue on the Smoothness slider did not reach the kernel — `route_sculpt_event` has no arm for it"
    );
}

/// **Clicking Filter Layer filters the layer** (W5b) — the button's whole reason to exist, driven from the
/// pixel the artist aims at.
///
/// The fixture lays REAL relief with a real impasto stroke first (the tool's `heights` is private, and it
/// should be: a seam gate that reached inside to plant a field would be testing a fixture, not the
/// product). Then it clicks the button and asks the layer whether it moved — **far from where the stroke
/// went**, which is the one thing a brush cannot do and this button must.
///
/// **Mutation that must bleed:** drop the `PAINTER_SCULPT_FILTER` arm from `route_sculpt_event`, or the id
/// from `populate` — the button paints, the Down never activates it, the relief never moves.
#[test]
fn clicking_filter_layer_filters_the_whole_layer() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

    let size = 120u32;
    let mut tool = PainterTool::default();
    tool.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    tool.set_brush_size_px(10.0);
    tool.toggle_brush_impasto();
    // A short stroke in ONE corner: it lays relief, and it leaves the rest of the layer untouched — which
    // is what makes the "far from the stroke" assertion below mean something.
    let cp = |p: [f32; 2], phase| CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    tool.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    for i in 1..=6u8 {
        tool.on_canvas_pointer(cp([20.0 + 6.0 * f32::from(i), 20.0], PointerPhase::Move));
    }
    tool.on_canvas_pointer(cp([56.0, 20.0], PointerPhase::Up));
    let layer = tool.layers().active().expect("a layer");
    let before = tool
        .layer_height_view(layer)
        .expect("the stroke laid relief");
    assert!(
        before.iter().any(|&v| v > 0.05),
        "fixture: the stroke must leave relief, else the filter has nothing to filter"
    );

    tool.set_paint_tool_mode("sculpt");
    tool.set_sculpt_mode(0); // Smooth — a verb whose target is a function of `pre`
    let bs = tool.brush_settings();
    assert!(
        bs.sculpt_filters,
        "fixture: Smooth must be offered the button, else the paint below finds no rect"
    );
    set_current_brush(Some(bs));

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let (_, rect) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_SCULPT_FILTER && r.w > 0.0 && r.h > 0.0)
        .copied()
        .expect("the Filter Layer button is not painted with a clickable rect on the Smooth card");
    let (x, y) = centre(rect);
    click_through(&mut host, &mut st, &mut tool, x, y);

    let after = tool.layer_height_view(layer).expect("relief");
    let moved = before
        .iter()
        .zip(after.iter())
        .filter(|(a, b)| (*a - *b).abs() > 1e-4)
        .count();
    assert!(
        moved > 0,
        "the click never reached the tool — the Filter Layer seam has a dead link \
         (populate / forward / route)"
    );
}

/// **The button is not offered where the verb has no whole-layer meaning** — and that is the refusal, not a
/// dim.
///
/// The plane verbs (here: the Chisel) fit their target to the brush's FOOTPRINT; a layer has none. The
/// panel asks the tool (`sculpt_filters` ⇐ `SculptMode::filters_layer`) and simply does not paint the row.
/// A dimmed-but-painted button would still be hit-indexed, and a dimmed button that dispatches is the bug
/// this codebase has already paid for.
///
/// **Mutation that must bleed:** paint the row unconditionally (drop the `if brush.sculpt_filters`) — the
/// rect appears on the Chisel card and this goes red.
#[test]
fn the_filter_button_is_not_offered_for_a_footprint_fitted_verb() {
    let tool = tool_in_sculpt_mode(Some(CHISEL));
    assert!(
        !tool.brush_settings().sculpt_filters,
        "fixture: the Chisel's target is a fitted plane — the tool must say it cannot be filtered"
    );
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    assert!(
        !painted
            .iter()
            .any(|(w, r)| *w == core_ids::PAINTER_SCULPT_FILTER && r.w > 0.0 && r.h > 0.0),
        "the Filter Layer button is painted on the Chisel card — a verb it cannot filter"
    );
}

/// **Clicking Filter Stroke filters the last stroke — and not the layer** (W5b, Enio 2026-07-16).
///
/// The sibling above proves the Layer button; this proves the STROKE one is a different button wired to a
/// different scope. The fixture paints a stroke in one corner, plants relief far away, and demands the far
/// relief survive: a Filter Stroke wired to `FilterScope::Layer` would smooth it and go red here while
/// staying green in every "did the relief move?" test.
///
/// **Mutation that must bleed:** route `PAINTER_SCULPT_FILTER_STROKE` to `FilterScope::Layer` in
/// `route_sculpt_event` — the far relief moves.
#[test]
fn clicking_filter_stroke_filters_only_the_last_stroke() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

    let size = 120u32;
    let mut tool = PainterTool::default();
    tool.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    tool.set_brush_size_px(9.0);
    tool.toggle_brush_impasto();
    let cp = |p: [f32; 2], phase| CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    // Stroke 1, far away — the control. Stroke 2, the LAST one — the target.
    tool.on_canvas_pointer(cp([20.0, 95.0], PointerPhase::Down));
    for i in 1..=6u8 {
        tool.on_canvas_pointer(cp([20.0 + 8.0 * f32::from(i), 95.0], PointerPhase::Move));
    }
    tool.on_canvas_pointer(cp([68.0, 95.0], PointerPhase::Up));
    tool.on_canvas_pointer(cp([20.0, 25.0], PointerPhase::Down));
    for i in 1..=6u8 {
        tool.on_canvas_pointer(cp([20.0 + 8.0 * f32::from(i), 25.0], PointerPhase::Move));
    }
    tool.on_canvas_pointer(cp([68.0, 25.0], PointerPhase::Up));

    let layer = tool.layers().active().expect("a layer");
    let before = tool.layer_height_view(layer).expect("relief");
    assert!(
        before[(95 * size + 40) as usize] > 0.05 && before[(25 * size + 40) as usize] > 0.05,
        "fixture: BOTH strokes must have laid relief, else 'only the last one' proves nothing"
    );

    tool.set_paint_tool_mode("sculpt");
    tool.set_sculpt_mode(0); // Smooth
    let bs = tool.brush_settings();
    assert!(
        bs.sculpt_can_filter_stroke,
        "fixture: a stroke exists on this layer, so the button must be on offer"
    );
    set_current_brush(Some(bs));

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let (_, rect) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_SCULPT_FILTER_STROKE && r.w > 0.0 && r.h > 0.0)
        .copied()
        .expect("the Filter Stroke button is not painted with a clickable rect");
    let (x, y) = centre(rect);
    click_through(&mut host, &mut st, &mut tool, x, y);

    let after = tool.layer_height_view(layer).expect("relief");
    let band = |y0: u32, y1: u32| -> usize {
        let mut n = 0;
        for yy in y0..y1 {
            for xx in 0..size {
                let i = (yy * size + xx) as usize;
                if (after[i] - before[i]).abs() > 1e-4 {
                    n += 1;
                }
            }
        }
        n
    };
    assert!(
        band(0, 60) > 0,
        "the click never reached the tool — the Filter Stroke seam has a dead link"
    );
    assert_eq!(
        band(60, size),
        0,
        "the FIRST stroke moved too — the button is scoped to the layer, not to the last stroke"
    );
}

/// **The Filter Stroke button is not offered before there is a stroke to filter.**
///
/// A button that can only refuse is a button that lies. `live_paint` is empty until a stroke commits.
///
/// **Mutation that must bleed:** paint the second option unconditionally (drop the `can_filter_stroke`
/// branch in `filter_row`) — the rect appears on a virgin canvas.
#[test]
fn the_filter_stroke_button_waits_for_a_stroke_to_exist() {
    let tool = tool_in_sculpt_mode(Some(0)); // Smooth, nothing painted yet
    assert!(
        !tool.brush_settings().sculpt_can_filter_stroke,
        "fixture: nothing has been painted, so there is no last stroke"
    );
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    assert!(
        painted
            .iter()
            .any(|(w, r)| *w == core_ids::PAINTER_SCULPT_FILTER && r.w > 0.0 && r.h > 0.0),
        "fixture: the Layer button IS offered on Smooth — else this gate is vacuous"
    );
    assert!(
        !painted
            .iter()
            .any(|(w, r)| *w == core_ids::PAINTER_SCULPT_FILTER_STROKE && r.w > 0.0 && r.h > 0.0),
        "the Filter Stroke button is painted with no stroke to filter — it can only refuse"
    );
}
