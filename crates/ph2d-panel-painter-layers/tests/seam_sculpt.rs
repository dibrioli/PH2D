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
fn tool_in_sculpt() -> PainterTool {
    let mut tool = PainterTool::default();
    tool.set_paint_tool_mode("sculpt");
    let bs = tool.brush_settings();
    assert!(
        bs.is_sculpt,
        "fixture is wrong, not the seam: the tool is not in Sculpt, so the card is never painted and \
         nothing below could mean anything"
    );
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
#[test]
fn every_sculpt_click_widget_is_reachable_by_a_pointer() {
    for clicked in core_ids::PAINTER_SCULPT_CLICKS {
        let tool = tool_in_sculpt();
        let _ = &tool;
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());

        let Some((_, rect)) = painted
            .iter()
            .find(|(w, r)| *w == clicked && r.w > 0.0 && r.h > 0.0)
            .copied()
        else {
            panic!("sculpt widget {clicked:?} is not painted with a clickable rect");
        };

        let (x, y) = centre(rect);
        let events = host.click_at(x, y);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(id) if *id == clicked)),
            "a real pointer click on sculpt widget {clicked:?} emitted NO Click event ({events:?}) — it \
             is painted and hit-registered but absent from the WidgetStore, so `is_focusable` rejects it \
             on Down and it is dead under the mouse. Register it in `populate.rs`."
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
    let tool = tool_in_sculpt();
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

    // Every verb chip is there…
    for id in core_ids::PAINTER_SCULPT_CLICKS {
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

/// **The knob row swaps with the verb's family — one knob, never both, never neither.**
///
/// Radius is the blur's own scale and means nothing to a plane verb (the plane is fitted to the brush's
/// footprint, so the brush's Size already IS its scale). Offset is where the fitted plane sits and means
/// nothing to a blur, which has no plane. Painting the inert one would be a knob that lies — and this card
/// has already cost a smoke over exactly that class of mistake.
///
/// The "never neither" half is not padding: it is what catches a family added in Wave 3 whose `is_plane`
/// answer is right but whose row nobody wired, leaving the card with a verb and no knob at all.
///
/// **Mutation that must bleed:** paint `radius_row` unconditionally in `paint_sculpt_section` (the Offset
/// disappears in the plane verbs), or swap the branch (each verb then shows the other's knob).
#[test]
fn the_knob_row_swaps_with_the_verb_family() {
    // (mode index, the id the card must paint, the id it must NOT)
    let cases = [
        (
            0u8,
            core_ids::PAINTER_SCULPT_RADIUS_SLIDER,
            core_ids::PAINTER_SCULPT_OFFSET_SLIDER,
        ), // Smooth
        (
            1,
            core_ids::PAINTER_SCULPT_RADIUS_SLIDER,
            core_ids::PAINTER_SCULPT_OFFSET_SLIDER,
        ), // Sharpen
        (
            2,
            core_ids::PAINTER_SCULPT_OFFSET_SLIDER,
            core_ids::PAINTER_SCULPT_RADIUS_SLIDER,
        ), // Flatten
        (
            3,
            core_ids::PAINTER_SCULPT_OFFSET_SLIDER,
            core_ids::PAINTER_SCULPT_RADIUS_SLIDER,
        ), // Scrape
        (
            4,
            core_ids::PAINTER_SCULPT_OFFSET_SLIDER,
            core_ids::PAINTER_SCULPT_RADIUS_SLIDER,
        ), // Fill
    ];
    for (mode, shown, hidden) in cases {
        let mut tool = tool_in_sculpt();
        tool.set_sculpt_mode(mode);
        set_current_brush(Some(tool.brush_settings()));

        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
        let ids: Vec<_> = painted.iter().map(|(w, _)| *w).collect();

        assert!(
            ids.contains(&shown),
            "verb {mode} paints NO knob of its own — the card shows a spatula with nothing to steer it by"
        );
        assert!(
            !ids.contains(&hidden),
            "verb {mode} paints the OTHER family's knob. It does nothing to this verb, and a control that \
             does nothing is a control that lies about what the tool can do."
        );
    }
}
