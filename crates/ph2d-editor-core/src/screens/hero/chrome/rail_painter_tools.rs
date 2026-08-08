// ph2d-chrome-sync:z=60 (dispatch priority, ADR-0107; lower = earlier)
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
    } else if tool_id == ids::PAINTER_RAIL_LIQUIFY {
        // ⚠️ NOT "deform": the two halves of the warp get one chip each, and each wire lands the artist
        // IN its temperament. The old single "deform" wire opened an antechamber where the canvas
        // consumed the drag and moved nothing (measured: 0 pixels — `measure_rail_chips`).
        "liquify"
    } else if tool_id == ids::PAINTER_RAIL_TRANSFORM {
        "transform"
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

/// **Snap the rail radio to the mode the painter is actually in** — the inverse of [`push_paint_mode`],
/// and it lives beside it so the two directions of one mapping are read together.
///
/// ⚠️ The rail is no longer the only thing that changes the paint mode. The Painter panel's unified
/// **Impasto TOOL** list (Enio, 2026-07-19) gathers the ten operations on the paint's body into one
/// place, and picking one USES it — so choosing "Chisel" there enters Sculpt. Without this, the rail
/// would go on highlighting "Brush" while the artist sculpts: two answers to *"which tool am I
/// holding?"*, and the wrong one is the one on screen.
///
/// So the rail's pressed state is **derived from the published mode**, not written by whoever was
/// clicked last. The shell calls this each frame with `PainterTool::paint_mode_wire()` — the SAME
/// vocabulary `set_paint_tool_mode` parses, so there is one set of strings and no third spelling to
/// drift. Modes with no rail button of their own (`fill`, `eyedropper`) leave the radio alone: they are
/// momentary or unwired, and `reset_to_brush` already owns the Eyedropper's return.
pub fn sync_from_mode(store: &mut crate::interaction::WidgetStore, mode: &str) {
    let target = match mode {
        "smear" => ids::PAINTER_RAIL_SMEAR,
        "blur" => ids::PAINTER_RAIL_BLUR,
        "clone" => ids::PAINTER_RAIL_CLONE,
        "mask" => ids::PAINTER_RAIL_MASK,
        "inpaint" => ids::PAINTER_RAIL_INPAINT,
        "selection" => ids::PAINTER_RAIL_SELECTION,
        "liquify" => ids::PAINTER_RAIL_LIQUIFY,
        "transform" => ids::PAINTER_RAIL_TRANSFORM,
        "eraser" => ids::PAINTER_RAIL_ERASER,
        "brush" => ids::PAINTER_RAIL_BRUSH,
        // ⚠️ The **Knife** deliberately has no rail button — the rail's Smear is the plain one, and the
        // knife is picked from the Impasto TOOL list (Enio, 2026-07-19). So the honest rail for it is a
        // rail with NOTHING pressed: the artist is holding a tool this strip does not offer, and lighting
        // up its nearest relative would be the rail naming the wrong tool.
        // …and **Sculpt** joined it in 2026-08-08, for the same reason and by the same measurement: it
        // reshapes the paint's BODY, so it belongs to the Impasto TOOL list where that medium is armed,
        // not to the universal rail where it moves nothing at all.
        "knife" | "sculpt" => {
            set_radio(store, &ids::PAINTER_RAIL_TOOL_IDS, NodeId(0));
            return;
        }
        _ => return,
    };
    // Only write when it actually moved: `set_radio` walks the whole group, and the Shapes / Mask group
    // buttons carry sub-tool state that a needless rewrite every frame would stamp over.
    if matches!(
        store.get(target),
        Some(InteractiveState::Button {
            state: ButtonState::Pressed
        })
    ) {
        return;
    }
    set_radio(store, &ids::PAINTER_RAIL_TOOL_IDS, target);
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
// ⚠️ Os gates moram em `rail_painter_tools/tests.rs`, num DIRETÓRIO — não num irmão
// `rail_painter_tools_tests.rs`. O `ph2d-chrome-sync` varre `chrome/*.rs` do topo e trata **todo**
// arquivo como um handler, então um irmão `*_tests.rs` vira um handler-fantasma no `dispatch_all` e no
// bloco de `mod` gerado (a cicatriz exata que o `command_palette_tests.rs` deixou em 2026-08-02, curada
// do mesmo jeito). Dentro do diretório ele é invisível para o gerador e segue sendo o módulo FILHO
// daqui, então o `use super::*` continua alcançando o que é privado.
#[cfg(test)]
mod tests;
