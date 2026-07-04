//! Painter-mode left-rail dispatch — the paint-tool radio group (Brush ·
//! Eyedropper · Eraser · Clone · Smear · Blur · Mask · Inpaint · Shapes) plus
//! the Shapes flyout (open/close + the shape sub-radio). Mirror of
//! `rail_tools.rs` for the Painter face of the rail.
//!
//! These ids are only painted + hit-registered while the Painter tool is active
//! ([`left_rail::paint_left_rail`](super::super::left_rail)), so this handler
//! never fires in object mode. Selecting a tool sets the rail's radio
//! selection + flyout state AND forwards the operating mode (Brush / Eraser /
//! Smear / Blur / Clone / Mask / Inpaint wired; Shapes is a later step).

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::tool::PanelEvent;
use crate::widget::ButtonState;
use ph2d_a11y::NodeId;

/// Forward the selected paint tool's operating mode to the active Painter tool over the frozen
/// `PanelEvent` channel: Smear → the smear drag, Blur → the soften, Eraser → paint with Erase-Alpha,
/// everything else → normal Brush paint. The shell drains `ToolPanelEvent` into `handle_panel_event`,
/// so this reaches `PainterTool::set_paint_tool_mode` without any dependency on the concrete painter
/// crate. Inpaint → the content-aware heal brush (mark a defect → reconstruct on pen-up); the still
/// not-yet-wired Shapes maps to "brush". Eyedropper maps to "eyedropper" — the tool arms an on-canvas
/// colour pick that samples the composite, then reverts to Brush.
fn push_paint_mode(hero: &mut HeroScreen, tool_id: NodeId) {
    let mode = if tool_id == ids::PAINTER_RAIL_SMEAR {
        "smear"
    } else if tool_id == ids::PAINTER_RAIL_BLUR {
        "blur"
    } else if tool_id == ids::PAINTER_RAIL_CLONE {
        "clone"
    } else if tool_id == ids::PAINTER_RAIL_MASK {
        "mask"
    } else if tool_id == ids::PAINTER_RAIL_SELECTION {
        "selection"
    } else if tool_id == ids::PAINTER_RAIL_INPAINT {
        "inpaint"
    } else if tool_id == ids::PAINTER_RAIL_DEFORM {
        "deform"
    } else if tool_id == ids::PAINTER_RAIL_FILL {
        // Placeholder: the Fill (Bucket) behaviour + colour-picker wiring lands in a follow-up; for now
        // selecting it just marks the rail radio (the painter defaults an unknown mode to Brush paint).
        "fill"
    } else if tool_id == ids::PAINTER_RAIL_ERASER {
        "eraser"
    } else if tool_id == ids::PAINTER_RAIL_EYEDROPPER {
        "eyedropper"
    } else {
        "brush"
    };
    hero.bus
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            ids::PAINTER_PAINT_MODE,
            mode.to_string(),
        )));
}

/// Set `target` `Pressed` and every other id in `group` `Normal` (an exclusive
/// radio group, like the transform tools in `rail_tools.rs`).
fn set_radio(store: &mut crate::interaction::WidgetStore, group: &[NodeId], target: NodeId) {
    for id in group {
        if let Some(InteractiveState::Button { state }) = store.get_mut(*id) {
            *state = if *id == target {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
    }
}

/// The wire `StrokeMethod` discriminant (as a string for the frozen `SelectOption` channel) for a rail
/// Shapes-flyout id — Free Hand 9 / Line 5 / Curve 6 / Ellipse 7 / Polygon 8 (mirrors
/// `ph2d_painter_brush::StrokeMethod::to_u8`; editor-core must not depend on the brush crate, so the
/// values are inlined against that frozen wire contract). `None` for a non-shape id.
fn shape_method_wire(id: NodeId) -> Option<&'static str> {
    if id == ids::PAINTER_RAIL_SHAPE_FREEHAND {
        Some("9")
    } else if id == ids::PAINTER_RAIL_SHAPE_LINE {
        Some("5")
    } else if id == ids::PAINTER_RAIL_SHAPE_CURVE {
        Some("6")
    } else if id == ids::PAINTER_RAIL_SHAPE_ELLIPSE {
        Some("7")
    } else if id == ids::PAINTER_RAIL_SHAPE_POLYGON {
        Some("8")
    } else {
        None
    }
}

/// Forward a stroke-method command to the active Painter over the frozen `PanelEvent` channel (the SAME
/// id the Brush panel's Method dropdown uses, so no new channel): a shape's wire u8 to select it, or the
/// sentinel `"brush"` to restore the last non-shape method (the tool owns that memory). Drained by the
/// shell into `PainterTool::handle_panel_event`.
fn push_stroke_method(hero: &mut HeroScreen, value: &str) {
    hero.bus
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            ids::PAINTER_BRUSH_STROKE_METHOD,
            value.to_string(),
        )));
}

/// Snap the painter tool-rail radio back to **Brush**. The shell calls this when a MOMENTARY tool
/// completes — the Eyedropper, whose on-canvas colour pick auto-returns to Brush after sampling — so the
/// rail button stops looking "checked" once the pick is done (matching the tool's actual mode).
pub fn reset_to_brush(store: &mut crate::interaction::WidgetStore) {
    for id in ids::PAINTER_RAIL_TOOL_IDS {
        if let Some(InteractiveState::Button { state }) = store.get_mut(id) {
            *state = if id == ids::PAINTER_RAIL_BRUSH {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
    }
}

/// Reflect the painter tool's current `StrokeMethod` (wire discriminant) on the rail. A **shape** method
/// (Line 5 / Curve 6 / Ellipse 7 / Polygon 8 / Free Hand 9) selects the Shapes tool + its matching
/// shape sub-radio; a **non-shape** method leaves the tool radio untouched (returning to Brush is the
/// Brush button's job, not this sync — so this never stomps Eraser/Smear/… which also run a non-shape
/// method). The shell calls this when the tool's stroke method changes, so choosing a shape in the Brush
/// panel's Method dropdown moves the rail to the matching Shapes button automatically.
pub fn sync_rail_to_stroke_method(store: &mut crate::interaction::WidgetStore, method_u8: u8) {
    let shape_id = match method_u8 {
        5 => ids::PAINTER_RAIL_SHAPE_LINE,
        6 => ids::PAINTER_RAIL_SHAPE_CURVE,
        7 => ids::PAINTER_RAIL_SHAPE_ELLIPSE,
        8 => ids::PAINTER_RAIL_SHAPE_POLYGON,
        9 => ids::PAINTER_RAIL_SHAPE_FREEHAND,
        _ => return, // non-shape → leave the tool radio (the Brush button restores it)
    };
    set_radio(store, &ids::PAINTER_RAIL_SHAPE_IDS, shape_id);
    set_radio(store, &ids::PAINTER_RAIL_TOOL_IDS, ids::PAINTER_RAIL_SHAPES);
}

/// The active Mask-group sub-tool id (the Pressed one in the sub-radio), defaulting to Mask when none is
/// pressed. Drives the mode the Mask group button forwards when clicked.
fn active_mask_sub_id(store: &crate::interaction::WidgetStore) -> NodeId {
    ids::PAINTER_RAIL_MASK_SUB_IDS
        .into_iter()
        .find(|id| matches!(store.button_state(*id), Some(ButtonState::Pressed)))
        .unwrap_or(ids::PAINTER_RAIL_MASK)
}

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // The **C&F** (Colour & Fill) rail button is a COLOUR WELL, not a paint-mode radio: a plain click only
    // opens the colour picker (the picked colour is shared with the Brush — handled in the shell's
    // `fill_drag`), and the ColorDrop DRAG onto the canvas is what activates Fill. So the rail click itself
    // changes neither the operating mode nor the radio selection (Enio 2026-07-02). Consume it.
    if id == ids::PAINTER_RAIL_FILL {
        return true;
    }
    // A Mask-group sub-tool picked in the flyout (Mask / Selection): set the sub-radio, make the Mask
    // group the active tool, close the flyout, and forward that sub's paint mode. Sub-tools paint with
    // the normal (non-shape) stroke method, so restore it like the "other tool" branch below.
    if ids::PAINTER_RAIL_MASK_SUB_IDS.contains(&id) {
        set_radio(&mut hero.store, &ids::PAINTER_RAIL_MASK_SUB_IDS, id);
        set_radio(
            &mut hero.store,
            &ids::PAINTER_RAIL_TOOL_IDS,
            ids::PAINTER_RAIL_MASK_GROUP,
        );
        hero.store.set_painter_mask_flyout_open(false);
        hero.store.set_painter_shapes_flyout_open(false);
        push_stroke_method(hero, "brush");
        push_paint_mode(hero, id);
        return true;
    }
    // A shape option picked in the flyout: set the shape sub-radio, make Shapes the active tool, close
    // the flyout, and set the painter's Stroke:Method TO that shape (nothing else in the Brush panel
    // changes).
    if ids::PAINTER_RAIL_SHAPE_IDS.contains(&id) {
        set_radio(&mut hero.store, &ids::PAINTER_RAIL_SHAPE_IDS, id);
        set_radio(
            &mut hero.store,
            &ids::PAINTER_RAIL_TOOL_IDS,
            ids::PAINTER_RAIL_SHAPES,
        );
        hero.store.set_painter_shapes_flyout_open(false);
        // Shapes draws with the normal Brush paint mode (exit any Smear/eraser)...
        push_paint_mode(hero, ids::PAINTER_RAIL_SHAPES);
        // ...and the picked shape becomes the Stroke:Method.
        if let Some(m) = shape_method_wire(id) {
            push_stroke_method(hero, m);
        }
        return true;
    }
    // A paint tool (including Shapes): exclusive selection.
    if ids::PAINTER_RAIL_TOOL_IDS.contains(&id) {
        set_radio(&mut hero.store, &ids::PAINTER_RAIL_TOOL_IDS, id);
        if id == ids::PAINTER_RAIL_SHAPES {
            // Shapes owns the flyout — toggle its reveal. The flyout PICK sets the method; opening it
            // leaves the current shape sub-radio + method as-is.
            hero.store.set_painter_mask_flyout_open(false);
            let open = hero.store.painter_shapes_flyout_open();
            hero.store.set_painter_shapes_flyout_open(!open);
        } else if id == ids::PAINTER_RAIL_MASK_GROUP {
            // Mask group owns the Mask flyout — toggle its reveal and forward the ACTIVE sub-tool's mode
            // (Mask by default) so clicking the group activates the shown sub-tool, Photoshop-style.
            hero.store.set_painter_shapes_flyout_open(false);
            let open = hero.store.painter_mask_flyout_open();
            hero.store.set_painter_mask_flyout_open(!open);
            push_stroke_method(hero, "brush");
            push_paint_mode(hero, active_mask_sub_id(&hero.store));
            return true;
        } else {
            // Any other tool closes a lingering flyout AND returns Stroke:Method to the last non-shape
            // method — so leaving a shape via Brush/Eraser/… reverts to normal painting (and the reverse
            // sync, which only forces Shapes for a shape method, never bounces this selection back).
            hero.store.set_painter_shapes_flyout_open(false);
            hero.store.set_painter_mask_flyout_open(false);
            push_stroke_method(hero, "brush");
        }
        // Forward the operating mode to the active Painter (Smear / Eraser / Brush / Eyedropper). The
        // Eyedropper arms an ON-CANVAS colour pick (mode "eyedropper") — no wheel; the button stays
        // checked until the next canvas click samples a colour, then the shell snaps the rail back to
        // Brush (see `reset_to_brush`). Same forward path as every other tool.
        push_paint_mode(hero, id);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn selecting_deform_forwards_the_deform_mode() {
        // Forward seam: the Deform rail button forwards the "deform" operating mode over the frozen
        // PAINTER_PAINT_MODE channel → the painter's reshape/liquify kernel (Deform Wave 1). It is a plain
        // radio tool (no flyout), so the generic tool branch drives it.
        let mut hero = HeroScreen::new(NodeId(1));
        super::super::super::left_rail::populate(&mut hero.store);
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(ids::PAINTER_RAIL_DEFORM)
        ));
        assert!(pressed(&hero, ids::PAINTER_RAIL_DEFORM));
        assert!(!pressed(&hero, ids::PAINTER_RAIL_BRUSH));
        assert_eq!(
            drained_paint_mode(&mut hero).as_deref(),
            Some("deform"),
            "the Deform button forwarded the reshape mode, not the brush fallback"
        );
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
}
