//! Gates do dispatch do rail em modo Painter — irmão de `rail_painter_tools.rs`.
//!
//! Extraído em 2026-08-08 (teto de LOC). Segue sendo o módulo `tests` FILHO daquele arquivo — só o
//! arquivo se mudou —, então o `use super::*` continua alcançando o que é privado lá. ⚠️ E mora num
//! DIRETÓRIO de propósito: o `ph2d-chrome-sync` varre `chrome/*.rs` do topo e trataria um irmão
//! `rail_painter_tools_tests.rs` como um handler de chrome (o `command_palette_tests.rs` de 2026-08-02
//! outra vez).

use super::*;
use ph2d_a11y::NodeId as Aid;

fn pressed(hero: &HeroScreen, id: Aid) -> bool {
    matches!(hero.store.button_state(id), Some(ButtonState::Pressed))
}

#[test]
fn selecting_a_paint_tool_is_an_exclusive_radio() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    // Brush is the default selection.
    assert!(pressed(&hero, ids::PAINTER_RAIL_BRUSH));
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_ERASER)
    ));
    assert!(pressed(&hero, ids::PAINTER_RAIL_ERASER));
    assert!(!pressed(&hero, ids::PAINTER_RAIL_BRUSH));
}

#[test]
fn shapes_button_toggles_the_flyout() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    assert!(!hero.store.painter_shapes_flyout_open());
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_SHAPES)
    ));
    assert!(hero.store.painter_shapes_flyout_open());
    assert!(pressed(&hero, ids::PAINTER_RAIL_SHAPES));
    // Click again closes it.
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_SHAPES)
    ));
    assert!(!hero.store.painter_shapes_flyout_open());
}

#[test]
fn picking_a_shape_selects_shapes_tool_and_closes_flyout() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_SHAPES));
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_SHAPE_ELLIPSE)
    ));
    assert!(pressed(&hero, ids::PAINTER_RAIL_SHAPE_ELLIPSE));
    assert!(!pressed(&hero, ids::PAINTER_RAIL_SHAPE_FREEHAND));
    assert!(pressed(&hero, ids::PAINTER_RAIL_SHAPES));
    assert!(!hero.store.painter_shapes_flyout_open());
}

#[test]
fn selecting_another_tool_closes_the_flyout() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_SHAPES));
    assert!(hero.store.painter_shapes_flyout_open());
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_BRUSH));
    assert!(!hero.store.painter_shapes_flyout_open());
    assert!(pressed(&hero, ids::PAINTER_RAIL_BRUSH));
}

#[test]
fn eyedropper_arms_the_pick_without_opening_the_wheel() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    assert!(hero.store.picker_target().is_none());
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_EYEDROPPER)
    ));
    // The rail Eyedropper now arms an ON-CANVAS pick — it must NOT open the colour wheel.
    assert!(
        hero.store.picker_target().is_none(),
        "Eyedropper does not open the colour wheel — it arms an on-canvas pick"
    );
    assert!(
        pressed(&hero, ids::PAINTER_RAIL_EYEDROPPER),
        "Eyedropper is checked while the pick is armed"
    );
}

#[test]
fn reset_to_brush_snaps_the_rail_radio() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_EYEDROPPER));
    assert!(pressed(&hero, ids::PAINTER_RAIL_EYEDROPPER));
    // The shell calls this when the on-canvas pick completes → back to Brush.
    super::reset_to_brush(&mut hero.store);
    assert!(pressed(&hero, ids::PAINTER_RAIL_BRUSH));
    assert!(!pressed(&hero, ids::PAINTER_RAIL_EYEDROPPER));
}

/// **The rail radio FOLLOWS the painter's mode** — it is derived, not remembered.
///
/// The Painter panel's unified Impasto TOOL list can change the paint mode (picking "Chisel" there
/// enters Sculpt), so a rail that only learned about its own clicks would go on highlighting the
/// button the artist last pressed while the canvas is holding something else. Two answers to *"which
/// tool am I holding?"*, with the wrong one on screen.
///
/// The fixture starts from a rail that has been CLICKED, so the assertion is that the sync overrides
/// a stale pressed state — not merely that it can press a button on a fresh store.
///
/// **Mutation that must bleed:** make `sync_from_mode` return early for every mode. Nothing else in
/// the workspace notices: the tool is in the right mode, the panel paints the right card, and only
/// the rail lies.
#[test]
fn the_rail_radio_follows_the_published_paint_mode() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    // The artist clicked Brush on the rail…
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_BRUSH));
    assert!(pressed(&hero, ids::PAINTER_RAIL_BRUSH));
    // …then the painter ended up somewhere else by a door that is not this rail.
    super::sync_from_mode(&mut hero.store, "liquify");
    assert!(
        pressed(&hero, ids::PAINTER_RAIL_LIQUIFY),
        "the rail did not follow the painter into Liquify — it would keep highlighting Brush while \
         the artist warps"
    );
    assert!(
        !pressed(&hero, ids::PAINTER_RAIL_BRUSH),
        "…and it is a RADIO: the previous tool must let go"
    );
    // …and the two halves of the warp are two chips, so moving between them moves the radio.
    super::sync_from_mode(&mut hero.store, "transform");
    assert!(pressed(&hero, ids::PAINTER_RAIL_TRANSFORM));
    assert!(!pressed(&hero, ids::PAINTER_RAIL_LIQUIFY));
}

/// **A tool the rail does not OFFER leaves nothing pressed** — and that is a different answer from
/// the one below.
///
/// Two modes are picked from the Impasto TOOL list and have no chip here: the **Knife** (since
/// 2026-07-19) and **Sculpt** (since 2026-08-08, when Liquify took its slot — measured: in the
/// medium the Painter opens in, a Sculpt drag moves 0 pixels and 0 relief texels). Lighting up
/// their nearest relative would be the rail naming the wrong tool, so the honest rail is a blank
/// one.
///
/// ⚠️ Contrast `a_mode_with_no_rail_button_leaves_the_radio_alone`: `fill` / `eyedropper` are
/// MOMENTARY, so the chip the artist will return to must stay lit. These two are not momentary —
/// the artist is holding them — so the stale chip must go dark. Same shaped question, opposite
/// answers, which is why each has its own gate.
///
/// **Mutation that must bleed:** delete the `"knife" | "sculpt"` arm. It falls through to
/// `_ => return`, the radio is left ALONE, and the rail goes on highlighting Smear while the
/// artist carves.
#[test]
fn a_tool_the_rail_does_not_offer_leaves_nothing_pressed() {
    for held in ["knife", "sculpt"] {
        let mut hero = HeroScreen::new(NodeId(1));
        super::super::super::left_rail::populate(&mut hero.store);
        apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_SMEAR));
        assert!(
            pressed(&hero, ids::PAINTER_RAIL_SMEAR),
            "fixture: Smear is lit"
        );
        super::sync_from_mode(&mut hero.store, held);
        assert!(
            ids::PAINTER_RAIL_TOOL_IDS
                .iter()
                .all(|id| !pressed(&hero, *id)),
            "holding `{held}` left a chip lit — the rail is naming a tool the artist is not holding"
        );
    }
}

/// A mode with no rail button of its own leaves the radio ALONE.
///
/// `fill` is drag-activated (the C&F button is a colour well, not a mode radio — see
/// `clicking_c_and_f_is_a_colour_well_not_a_fill_mode_radio`) and `eyedropper` is momentary, owned by
/// `reset_to_brush`. Without this, the catch-all would have to guess, and guessing here means the
/// radio flickering to Brush every frame the artist is mid-ColorDrop.
#[test]
fn a_mode_with_no_rail_button_leaves_the_radio_alone() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_SMEAR));
    assert!(pressed(&hero, ids::PAINTER_RAIL_SMEAR));
    for unmapped in ["fill", "eyedropper", ""] {
        super::sync_from_mode(&mut hero.store, unmapped);
        assert!(
            pressed(&hero, ids::PAINTER_RAIL_SMEAR),
            "mode {unmapped:?} has no rail button, so it must not move the radio"
        );
    }
}

#[test]
fn ignores_non_rail_ids() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    assert!(!apply(&mut hero, WidgetEvent::Click(ids::TOOL_ROTATE)));
}

/// The value of the last `PAINTER_BRUSH_STROKE_METHOD` command pushed on the bus (drains it).
fn drained_stroke_method(hero: &mut HeroScreen) -> Option<String> {
    hero.bus.drain().find_map(|a| match a {
        EditorAction::ToolPanelEvent(PanelEvent::SelectOption(id, v))
            if id == ids::PAINTER_BRUSH_STROKE_METHOD =>
        {
            Some(v)
        }
        _ => None,
    })
}

#[test]
fn picking_a_shape_emits_its_stroke_method_over_the_frozen_channel() {
    // Forward seam: a flyout shape pick sends the shape's wire u8 on PAINTER_BRUSH_STROKE_METHOD
    // (Ellipse = 7), and selects the Shapes tool + the Ellipse sub-radio.
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_SHAPES));
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_SHAPE_ELLIPSE)
    ));
    assert!(pressed(&hero, ids::PAINTER_RAIL_SHAPES));
    assert!(pressed(&hero, ids::PAINTER_RAIL_SHAPE_ELLIPSE));
    assert_eq!(
        drained_stroke_method(&mut hero).as_deref(),
        Some("7"),
        "the Ellipse pick forwarded StrokeMethod::Ellipse (wire 7)"
    );
}

#[test]
fn clicking_brush_emits_the_restore_sentinel() {
    // Forward seam: the Brush button sends the "brush" sentinel → the tool restores the last
    // non-shape method (the rail can't know that value; the tool owns the memory).
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_BRUSH)
    ));
    assert_eq!(
        drained_stroke_method(&mut hero).as_deref(),
        Some("brush"),
        "the Brush button forwarded the restore sentinel"
    );
}

/// The value of the last `PAINTER_PAINT_MODE` command pushed on the bus (drains it).
fn drained_paint_mode(hero: &mut HeroScreen) -> Option<String> {
    hero.bus.drain().find_map(|a| match a {
        EditorAction::ToolPanelEvent(PanelEvent::SelectOption(id, v))
            if id == ids::PAINTER_PAINT_MODE =>
        {
            Some(v)
        }
        _ => None,
    })
}

#[test]
fn clicking_c_and_f_is_a_colour_well_not_a_fill_mode_radio() {
    // The **C&F** (Colour & Fill) rail button is a colour WELL: a plain click neither activates Fill nor
    // moves the tool radio (it only opens the picker, in the shell). Fill activates via the ColorDrop
    // DRAG onto the canvas, not this click (Enio 2026-07-02).
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    assert!(pressed(&hero, ids::PAINTER_RAIL_BRUSH));
    assert!(apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_FILL)));
    // The click is consumed but changes nothing: Brush stays selected, Fill never presses, no mode fires.
    assert!(!pressed(&hero, ids::PAINTER_RAIL_FILL));
    assert!(pressed(&hero, ids::PAINTER_RAIL_BRUSH));
    assert_eq!(
        drained_paint_mode(&mut hero).as_deref(),
        None,
        "a C&F click forwards NO operating mode (drag-to-canvas activates Fill)"
    );
}

#[test]
fn selecting_inpaint_forwards_the_inpaint_heal_mode() {
    // Forward seam: the Inpaint rail button forwards the "inpaint" operating mode over the frozen
    // PAINTER_PAINT_MODE channel → the tool's content-aware heal brush (ADR-0102).
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_INPAINT)
    ));
    assert!(pressed(&hero, ids::PAINTER_RAIL_INPAINT));
    assert_eq!(
        drained_paint_mode(&mut hero).as_deref(),
        Some("inpaint"),
        "the Inpaint button forwarded the heal mode, not the brush fallback"
    );
}

/// **The warp's two halves forward two DIFFERENT wires** — the forward seam of the 2026-08-08
/// promotion.
///
/// There used to be one `Deform` chip forwarding one `"deform"` wire, and it opened an antechamber:
/// the temperament came up unselected and the canvas router consumed the drag without acting.
/// Measured through `on_canvas_pointer` in the Digital medium, that chip moved **0** pixels, and the
/// same chip moved **26 964** after one more click in the panel (`measure_rail_chips`).
///
/// **Mutation that must bleed:** make both chips forward the same string. Whichever one loses,
/// clicking it puts the artist in the other half of the tool — silently, because both chips are
/// legitimate and the panel will happily paint the wrong body.
#[test]
fn the_two_halves_of_the_warp_forward_two_different_wires() {
    for (chip, wire) in [
        (ids::PAINTER_RAIL_LIQUIFY, "liquify"),
        (ids::PAINTER_RAIL_TRANSFORM, "transform"),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        super::super::super::left_rail::populate(&mut hero.store);
        assert!(apply(&mut hero, WidgetEvent::Click(chip)));
        assert!(pressed(&hero, chip));
        assert!(!pressed(&hero, ids::PAINTER_RAIL_BRUSH));
        assert_eq!(
            drained_paint_mode(&mut hero).as_deref(),
            Some(wire),
            "the {wire} chip forwarded the wrong mode — one of the two halves is unreachable"
        );
    }
}

#[test]
fn mask_group_button_toggles_its_flyout_and_forwards_the_active_sub() {
    // The Mask group button (shared with Selection) toggles the Mask flyout on click and forwards
    // the active sub-tool's mode — Mask by default (populate presses the Mask sub).
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    assert!(!hero.store.painter_mask_flyout_open());
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_MASK_GROUP)
    ));
    assert!(hero.store.painter_mask_flyout_open());
    assert!(pressed(&hero, ids::PAINTER_RAIL_MASK_GROUP));
    assert_eq!(
        drained_paint_mode(&mut hero).as_deref(),
        Some("mask"),
        "the Mask group forwards its default sub-tool (Mask) mode"
    );
    // Click again closes it.
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_MASK_GROUP)
    ));
    assert!(!hero.store.painter_mask_flyout_open());
}

#[test]
fn picking_selection_activates_the_mask_group_and_forwards_selection() {
    // Forward seam: picking Selection in the Mask flyout sets the sub-radio, makes the Mask group the
    // active tool, closes the flyout, and forwards the "selection" paint mode over PAINTER_PAINT_MODE.
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    // Flyout already open (the group was pressed); pick Selection from it. Pick directly so the bus
    // holds only the pick's forwarded mode (clicking the group first would enqueue a stale "mask").
    hero.store.set_painter_mask_flyout_open(true);
    assert!(apply(
        &mut hero,
        WidgetEvent::Click(ids::PAINTER_RAIL_SELECTION)
    ));
    assert!(pressed(&hero, ids::PAINTER_RAIL_SELECTION));
    assert!(!pressed(&hero, ids::PAINTER_RAIL_MASK));
    assert!(pressed(&hero, ids::PAINTER_RAIL_MASK_GROUP));
    assert!(!hero.store.painter_mask_flyout_open());
    assert_eq!(
        drained_paint_mode(&mut hero).as_deref(),
        Some("selection"),
        "the Selection pick forwarded the selection mode"
    );
}

#[test]
fn mask_and_shapes_flyouts_are_mutually_exclusive() {
    // Opening one group flyout closes the other — they anchor to different chips and must never both
    // be open.
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_SHAPES));
    assert!(hero.store.painter_shapes_flyout_open());
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_MASK_GROUP));
    assert!(hero.store.painter_mask_flyout_open());
    assert!(
        !hero.store.painter_shapes_flyout_open(),
        "opening the Mask flyout closes the Shapes flyout"
    );
}

#[test]
fn sync_reflects_a_shape_method_and_leaves_a_non_shape_alone() {
    // Reverse seam: a shape method (from the Method dropdown) moves the rail to Shapes + its sub-radio;
    // a non-shape method must NOT stomp a non-Brush tool selection (e.g. Eraser).
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    // Ellipse (7) → Shapes + Ellipse sub-radio active.
    super::sync_rail_to_stroke_method(&mut hero.store, 7);
    assert!(pressed(&hero, ids::PAINTER_RAIL_SHAPES));
    assert!(pressed(&hero, ids::PAINTER_RAIL_SHAPE_ELLIPSE));
    // Now select Eraser, then a non-shape method (Space = 3) must leave Eraser alone (no bounce).
    apply(&mut hero, WidgetEvent::Click(ids::PAINTER_RAIL_ERASER));
    super::sync_rail_to_stroke_method(&mut hero.store, 3);
    assert!(
        pressed(&hero, ids::PAINTER_RAIL_ERASER),
        "a non-shape method must not force the rail off Eraser"
    );
    assert!(!pressed(&hero, ids::PAINTER_RAIL_SHAPES));
}
