//! Painter-mode left-rail dispatch — the paint-tool radio group (Brush ·
//! Eyedropper · Eraser · Clone · Smear · Blur · Mask · Inpaint · Shapes) plus
//! the Shapes flyout (open/close + the shape sub-radio). Mirror of
//! `rail_tools.rs` for the Painter face of the rail.
//!
//! These ids are only painted + hit-registered while the Painter tool is active
//! ([`left_rail::paint_left_rail`](super::super::left_rail)), so this handler
//! never fires in object mode. Selecting a tool sets the rail's radio
//! selection + flyout state AND forwards the operating mode (Brush / Eraser /
//! Smear / Blur / Clone wired; Mask / Inpaint behaviour is a later step).

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
/// crate. The not-yet-wired tools (Inpaint / Shapes) map to "brush" for now, so selecting one always
/// leaves normal painting rather than a stuck Smear/Blur/Clone/Mask. Eyedropper also maps to "brush"
/// but additionally opens the rich colour picker (see the `apply` handler).
fn push_paint_mode(hero: &mut HeroScreen, tool_id: NodeId) {
    let mode = if tool_id == ids::PAINTER_RAIL_SMEAR {
        "smear"
    } else if tool_id == ids::PAINTER_RAIL_BLUR {
        "blur"
    } else if tool_id == ids::PAINTER_RAIL_CLONE {
        "clone"
    } else if tool_id == ids::PAINTER_RAIL_MASK {
        "mask"
    } else if tool_id == ids::PAINTER_RAIL_ERASER {
        "eraser"
    } else {
        "brush"
    };
    hero.bus
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            ids::PAINTER_PAINT_MODE,
            mode.to_string(),
        )));
}

/// Toggle the shared Blender colour picker targeting the brush colour swatch — the rail **Eyedropper**
/// IS the rich colour picker (with a built-in eyedropper). Seeds it with the brush colour, which the
/// panel keeps in `widget_color(PAINTER_COLOR_THUMB)` while the picker is closed; the panel's per-frame
/// read-back forwards the picked colour back to the brush. Same store ops the Color swatch uses.
fn toggle_color_picker(hero: &mut HeroScreen) {
    let thumb = ids::PAINTER_COLOR_THUMB;
    if hero.store.picker_target() == Some(thumb) {
        hero.store.set_picker_target(None);
    } else {
        let rgba = hero.store.widget_color(thumb).unwrap_or([0, 0, 0, 255]);
        hero.store.set_blender_value(
            ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
        );
        hero.store.set_picker_target(Some(thumb));
    }
}

/// Set `target` `Pressed` and every other id in `group` `Normal` (an exclusive
/// radio group, like the transform tools in `rail_tools.rs`).
fn set_radio(hero: &mut HeroScreen, group: &[NodeId], target: NodeId) {
    for id in group {
        if let Some(InteractiveState::Button { state }) = hero.store.get_mut(*id) {
            *state = if *id == target {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
    }
}

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // A shape option picked in the flyout: set the shape sub-radio, make Shapes
    // the active tool, and close the flyout.
    if ids::PAINTER_RAIL_SHAPE_IDS.contains(&id) {
        set_radio(hero, &ids::PAINTER_RAIL_SHAPE_IDS, id);
        set_radio(hero, &ids::PAINTER_RAIL_TOOL_IDS, ids::PAINTER_RAIL_SHAPES);
        hero.store.set_painter_shapes_flyout_open(false);
        // Shapes is a normal (paint) tool — exit any Smear/eraser mode. (Wiring the shape's
        // stroke method into the painter is a later step.)
        push_paint_mode(hero, ids::PAINTER_RAIL_SHAPES);
        return true;
    }
    // A paint tool (including Shapes): exclusive selection.
    if ids::PAINTER_RAIL_TOOL_IDS.contains(&id) {
        set_radio(hero, &ids::PAINTER_RAIL_TOOL_IDS, id);
        if id == ids::PAINTER_RAIL_SHAPES {
            // Shapes owns the flyout — toggle its reveal.
            let open = hero.store.painter_shapes_flyout_open();
            hero.store.set_painter_shapes_flyout_open(!open);
        } else {
            // Selecting any other tool closes a lingering flyout.
            hero.store.set_painter_shapes_flyout_open(false);
        }
        // Forward the operating mode to the active Painter (Smear / Eraser / Brush).
        push_paint_mode(hero, id);
        // The Eyedropper additionally opens the rich colour picker (it paints as Brush; the picker
        // floats so you sample/choose a colour then paint).
        if id == ids::PAINTER_RAIL_EYEDROPPER {
            toggle_color_picker(hero);
        }
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
            WidgetEvent::Click(ids::PAINTER_RAIL_SHAPE_CIRCLE)
        ));
        assert!(pressed(&hero, ids::PAINTER_RAIL_SHAPE_CIRCLE));
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
    fn eyedropper_toggles_the_colour_picker() {
        let mut hero = HeroScreen::new(NodeId(1));
        super::super::super::left_rail::populate(&mut hero.store);
        assert!(hero.store.picker_target().is_none());
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(ids::PAINTER_RAIL_EYEDROPPER)
        ));
        assert_eq!(
            hero.store.picker_target(),
            Some(ids::PAINTER_COLOR_THUMB),
            "Eyedropper opens the colour picker (targeting the brush swatch)"
        );
        // Click again → toggles it closed.
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(ids::PAINTER_RAIL_EYEDROPPER)
        ));
        assert!(
            hero.store.picker_target().is_none(),
            "Eyedropper toggles it closed"
        );
    }

    #[test]
    fn ignores_non_rail_ids() {
        let mut hero = HeroScreen::new(NodeId(1));
        super::super::super::left_rail::populate(&mut hero.store);
        assert!(!apply(&mut hero, WidgetEvent::Click(ids::TOOL_ROTATE)));
    }
}
