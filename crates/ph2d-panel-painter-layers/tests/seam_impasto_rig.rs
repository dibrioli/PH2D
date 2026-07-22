//! **The Impasto light rig, driven by a POINTER** — the seam gate that was missing when the rig
//! shipped dead (Enio 2026-07-12: *"UI não funciona, nem o checkbox nem se pode selecionar outra
//! luz"*).
//!
//! ## Why the other gates all stayed green
//!
//! `tests/seam.rs` proves the panel FORWARDS an event it is HANDED
//! (`impasto_click_controls_forward_their_click` injects `WidgetEvent::Click(id)` straight into
//! `apply_event`). The tool's unit tests prove the router ACTS on that event. The contract gates
//! count symbols. Every one of them was green — and the chips were stone dead under the mouse,
//! because **nothing in between was ever asked to turn a POINTER into that event.**
//!
//! It cannot, unless the id also carries an `InteractiveState` in the `WidgetStore`:
//! `dispatch_pointer`'s Down only makes a hit `active` when it is *focusable*, and
//! `is_focusable(store, id)` is `false` for an id the store never heard of (`None => false`).
//! `paint_segmented_group_adaptive` takes the store by `&`, so a segment CANNOT self-register —
//! only `populate` can. The lamp chips were not in it.
//!
//! So these gates start at the pixel the artist clicks and end at the tool's state. Every link is
//! real: the paint that registers the rect, the dispatcher that resolves the hit, the panel that
//! forwards, the bus the shell drains, the tool that routes.
//!
//! **A widget is not done when it PAINTS. It is done when a test CLICKS it.**

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

/// A desktop viewport — the panel docks in the right rail of this layout.
fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// A `PainterTool` with the Impasto section LIVE, and the panel's brush snapshot published from
/// it — exactly as the shell publishes it each frame (`set_current_brush`). The medium is picked
/// through the tool's own router (the **Paint Mode** dropdown's `SelectOption`, 2026-07-22), so the
/// fixture cannot drift from the product.
fn tool_with_impasto_on() -> PainterTool {
    let mut tool = PainterTool::default();
    tool.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_MEDIA,
        "2".into(),
    ));
    let bs = tool.brush_settings();
    assert!(
        bs.impasto && bs.impasto_applies,
        "fixture is wrong, not the seam: the Impasto section is not live, so the Lighting card is \
         never painted and no gate below could mean anything (impasto={}, applies={})",
        bs.impasto,
        bs.impasto_applies
    );
    set_current_brush(Some(bs));
    tool
}

/// Republish the tool's live brush to the panel — the shell does this every frame, and the card
/// re-paints from it (the Enable row only EXISTS once a lamp ≠ 1 is selected).
fn republish(tool: &PainterTool) {
    set_current_brush(Some(tool.brush_settings()));
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

/// The centre of a widget's painted rect — the pixel an artist aims at.
fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// **Clicking lamp chip 2 selects lamp 2.** The bug Enio reported, as a test.
///
/// Mutation that must bleed: drop `PAINTER_IMPASTO_LIGHT_2` from `populate.rs` and this goes red
/// while every other gate in the repo stays green — which is precisely how it shipped.
#[test]
fn clicking_a_lamp_chip_selects_that_lamp() {
    let mut tool = tool_with_impasto_on();
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;

    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_IMPASTO_LIGHT_2 && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!("lamp chip 2 is not painted with a clickable rect — the card never drew it");
    };

    let (x, y) = centre(rect);
    // The dispatcher must resolve this pixel to the chip. If some later-painted widget overlaps it,
    // `hit_at` names the thief — a card sized by a FIXED row count under content that reflows is how
    // that happens, so say so rather than making the reader guess.
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_IMPASTO_LIGHT_2),
        "the pixel at the centre of lamp chip 2 does not resolve to the chip — a widget painted \
         AFTER it covers that rect (hit() is last-registered-wins)"
    );

    click_through(&mut host, &mut st, &mut tool, x, y);

    assert_eq!(
        tool.brush_settings().impasto_rig.selected,
        1,
        "a REAL pointer click on lamp chip 2 did not select lamp 2. The chip paints, registers a \
         hit rect, and `event.rs` would forward the Click — but the pointer never becomes one: an \
         id absent from the WidgetStore is not focusable, so Down never makes it active and Up \
         never fires apply_click. Register it in `populate.rs`."
    );
}

/// Every lamp chip, not just the one in the bug report — a rig where 3 of 4 chips work is the same
/// defect with better luck.
#[test]
fn every_lamp_chip_selects_its_lamp() {
    for (i, chip) in [
        core_ids::PAINTER_IMPASTO_LIGHT_1,
        core_ids::PAINTER_IMPASTO_LIGHT_2,
        core_ids::PAINTER_IMPASTO_LIGHT_3,
        core_ids::PAINTER_IMPASTO_LIGHT_4,
    ]
    .into_iter()
    .enumerate()
    {
        let mut tool = tool_with_impasto_on();
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;

        // Start from a lamp that is NOT the target, or "it selected lamp i" would be vacuous for
        // whichever lamp happens to be selected already.
        tool.select_impasto_light(if i == 0 { 3 } else { 0 });
        republish(&tool);

        let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
        let Some((_, rect)) = painted
            .iter()
            .find(|(w, r)| *w == chip && r.w > 0.0 && r.h > 0.0)
            .copied()
        else {
            panic!("lamp chip {} is not painted with a clickable rect", i + 1);
        };

        let (x, y) = centre(rect);
        click_through(&mut host, &mut st, &mut tool, x, y);

        assert_eq!(
            tool.brush_settings().impasto_rig.selected,
            i as u8,
            "a real pointer click on lamp chip {} did not select it — the chip is dead under the \
             mouse",
            i + 1
        );
    }
}

/// **The Enable checkbox turns the selected lamp on.** It is painted only for lamps 2-4 (lamp 1 is
/// the key and cannot be switched off), so this also pins the row's existence: select a lamp, and
/// the checkbox must both APPEAR and WORK.
#[test]
fn the_enable_checkbox_switches_the_selected_lamp_on() {
    let mut tool = tool_with_impasto_on();
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;

    tool.select_impasto_light(1);
    republish(&tool);
    assert!(
        !tool.brush_settings().impasto_rig.lights[1].on,
        "fixture: lamp 2 must start OFF (lamps 2-4 are off by default — that is what keeps an \
         untouched canvas byte-identical to the one-lamp build)"
    );

    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_IMPASTO_LIGHT_ON && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!(
            "the Enable checkbox is not painted even with lamp 2 selected — the row that only \
             exists for lamps 2-4 never drew"
        );
    };

    let (x, y) = centre(rect);
    click_through(&mut host, &mut st, &mut tool, x, y);

    assert!(
        tool.brush_settings().impasto_rig.lights[1].on,
        "a REAL pointer click on Enable did not switch lamp 2 on — the checkbox registers a hit \
         rect (`paint_checkbox_row`) but no InteractiveState, so the dispatcher never makes it \
         active. Register it in `populate.rs`."
    );
}

/// The Body card's segments ride the SAME dead wire — `paint_segmented_group_adaptive` cannot
/// self-register either, and neither Depth Source nor Draw To was in `populate.rs`. They were
/// assumed working during the diagnosis; they were not. Sweep every Impasto click id so the next
/// knob cannot ship inert.
#[test]
fn every_impasto_click_widget_is_reachable_by_a_pointer() {
    for clicked in core_ids::PAINTER_IMPASTO_CLICKS {
        let mut tool = tool_with_impasto_on();
        // The Enable row only exists for lamps 2-4; select one so the sweep can reach it.
        tool.select_impasto_light(1);
        republish(&tool);

        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());

        let Some((_, rect)) = painted
            .iter()
            .find(|(w, r)| *w == clicked && r.w > 0.0 && r.h > 0.0)
            .copied()
        else {
            panic!("impasto widget {clicked:?} is not painted with a clickable rect");
        };

        let (x, y) = centre(rect);
        let events = host.click_at(x, y);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, ph2d_editor_core::interaction::WidgetEvent::Click(id) if *id == clicked)),
            "a real pointer click on impasto widget {clicked:?} emitted NO Click event ({events:?}) \
             — it is painted and hit-registered but absent from the WidgetStore, so `is_focusable` \
             rejects it on Down and it is dead under the mouse. Register it in `populate.rs`."
        );
    }
}

/// **The section header collapses, and its colour dot opens the picker.**
///
/// Found by asking the same question of every other widget the section paints (the lamp chips were
/// the reported symptom, not the extent of it): the Impasto header was never
/// `mark_collapsible_section`'d and its dot was never `register_picker_swatch`'d, so the chevron
/// was decoration and the dot did nothing. Every sibling section (Shape, Stroke, Watercolor, …) has
/// both; Impasto simply was not added to the list.
#[test]
fn the_section_header_collapses_and_its_colour_dot_opens_the_picker() {
    let tool = tool_with_impasto_on();
    let _ = &tool;
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;

    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let rect_of = |id| {
        painted
            .iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    };

    // The chevron: a click must actually FLIP the collapsed state, not merely be delivered.
    let header = rect_of(core_ids::PAINTER_IMPASTO_SECTION).expect("the section header is painted");
    assert!(
        !host.store().is_collapsed(core_ids::PAINTER_IMPASTO_SECTION),
        "fixture: the section starts expanded"
    );
    let (hx, hy) = centre(header);
    let _ = host.click_at(hx, hy);
    assert!(
        host.store().is_collapsed(core_ids::PAINTER_IMPASTO_SECTION),
        "clicking the Impasto section header did not collapse it — the id was never \
         `mark_collapsible_section`'d, so `apply_click`'s collapse branch never sees it and the \
         chevron is decoration"
    );

    // The colour dot: Down on a picker swatch targets the shared picker (its own dispatch path).
    let dot = rect_of(core_ids::PAINTER_IMPASTO_SECTION_COLOR)
        .expect("the section colour dot is painted");
    let (dx, dy) = centre(dot);
    let _ = host.click_at(dx, dy);
    assert_eq!(
        host.store().picker_target(),
        Some(core_ids::PAINTER_IMPASTO_SECTION_COLOR),
        "clicking the Impasto colour dot did not open the shared picker — the id was never \
         `register_picker_swatch`'d, so Down falls through to the focus path and does nothing"
    );
}

/// **No widget in the Impasto section loses its own hit** — the sliders included.
///
/// The card is sized by a FIXED row count (`rows = if selected > 0 { 7 } else { 6 }`) while its
/// content is free to grow: `paint_checkbox_row` advances by a wider gap than the card budgets,
/// and the lamp chips REFLOW onto a second row on a narrow panel (`measure_segmented_adaptive`
/// exists for exactly that). Today the two errors cancel and the content lands on the card's last
/// pixel — measured, not assumed. But `hit()` is last-registered-wins, so the day a row is added
/// or a label widens, the card under-sizes, the NEXT section paints over the bottom of it, and
/// these rows go quietly dead — the same silence as the bug this file exists for, from a different
/// direction. This pins the symptom, so the arithmetic cannot rot in the dark.
#[test]
fn no_impasto_widget_loses_its_hit_to_the_section_below() {
    let mut tool = tool_with_impasto_on();
    tool.select_impasto_light(1); // the widest the card ever gets: the Enable row exists
    republish(&tool);

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());

    for id in core_ids::PAINTER_IMPASTO_CLICKS
        .into_iter()
        .chain(core_ids::PAINTER_IMPASTO_FIELDS)
    {
        let Some((_, rect)) = painted
            .iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
            .copied()
        else {
            continue; // Plow is Smear-only; Push/Body live in the Body card — absence is not the bug here.
        };
        let (x, y) = centre(rect);
        assert_eq!(
            host.hit_at(x, y),
            Some(id),
            "the centre of impasto widget {id:?} resolves to {:?}, not to itself — something \
             painted AFTER it covers its rect. The card's fixed row count no longer matches the \
             height its rows actually take.",
            host.hit_at(x, y)
        );
    }
}

/// **The Wax-colour swatch opens the picker when a pointer clicks it.**
///
/// It is a swatch, not a checkbox, so it is not in `PAINTER_IMPASTO_CLICKS` and the sweep above does not
/// reach it — which is exactly the gap that let the lamp chips ship dead. A swatch has the same four
/// silent links as any other widget (paint · hit rect · `WidgetStore` · the `event.rs` arm), and the one
/// that kills it is the third: `paint_color_swatch` does not register anything, so the panel must call
/// `register_button` at paint time or the pointer never becomes a Click.
///
/// (Enio, 2026-07-13: *"não deveria ter um seletor de cor ao lado de Wax como é com Intensity?"* — it
/// should, and this is the test that says it actually does.)
#[test]
fn the_wax_colour_swatch_opens_the_picker() {
    let tool = tool_with_impasto_on();
    let _ = &tool; // the fixture publishes the brush snapshot the panel paints from
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;

    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_IMPASTO_WAX_COLOR && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!(
            "the Wax-colour swatch is not painted with a clickable rect — the Material card's Wax row never drew it"
        );
    };

    let (x, y) = centre(rect);
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_IMPASTO_WAX_COLOR),
        "the swatch's own pixel does not resolve to it — the Wax slider beside it is overlapping the square"
    );
    for ev in host.click_at(x, y) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    assert_eq!(
        host.store().picker_target(),
        Some(core_ids::PAINTER_IMPASTO_WAX_COLOR),
        "clicking the Wax-colour swatch did not open the shared picker targeting it — the swatch is \
         painted, hit-registered and stone dead under the mouse"
    );
}
