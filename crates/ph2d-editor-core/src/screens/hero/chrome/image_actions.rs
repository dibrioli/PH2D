//! Image-edit actions on the bus — Trim, Make Square, Bg Removal,
//! Undo. Wave 2.5 PR 11.8b migration from `pending_X` fields.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // Trim Transparency — push `EditorAction::Trim` with the current
    // gizmo selection. Empty selection still consumes the click.
    if id == ids::IMAGE_ACTION_TRIM {
        if let Some(entity_bits) = hero.gizmo.selection {
            hero.bus.push(EditorAction::Trim { entity_bits });
        }
        return true;
    }
    // Make Square — mirror of Trim. Same empty-selection semantics.
    if id == ids::IMAGE_ACTION_MAKE_SQUARE {
        if let Some(entity_bits) = hero.gizmo.selection {
            hero.bus.push(EditorAction::MakeSquare { entity_bits });
        }
        return true;
    }
    // Real Size — mirror of Trim / Make Square. Resets the selected
    // sprite's Transform scale to 1:1. Same empty-selection semantics.
    if id == ids::IMAGE_ACTION_REAL_SIZE {
        if let Some(entity_bits) = hero.gizmo.selection {
            hero.bus.push(EditorAction::RealSize { entity_bits });
        }
        return true;
    }
    // Bg Removal — ACTIVATES the stateful tool (panel opens with a live
    // preview) instead of running a one-shot algorithm. Apply trigger
    // lives in the tool's panel Toggle (drained as `EditorAction::Bgremoval`).
    if id == ids::IMAGE_ACTION_BGREMOVAL {
        hero.bus.push(EditorAction::ActivateBgRemoval);
        return true;
    }
    // Padding — ACTIVATES the stateful tool (panel opens with the four
    // signed per-edge fields) instead of running a one-shot algorithm.
    // Apply lives in the tool's panel (drained as `EditorAction::Padding`).
    if id == ids::IMAGE_ACTION_PADDING {
        hero.bus.push(EditorAction::ActivatePadding);
        return true;
    }
    // Image-edit Undo — TOOL_UNDO chip on the LeftRail (also bound to
    // Cmd+Z in the desktop shell). When no snapshot exists the shell's
    // drainer surfaces a "Nothing to undo" toast.
    if id == ids::TOOL_UNDO {
        hero.bus.push(EditorAction::UndoImageEdit);
        return true;
    }
    false
}
