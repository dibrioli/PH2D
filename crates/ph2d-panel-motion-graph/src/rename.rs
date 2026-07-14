//! **Naming things** (doc 61) — F2 over the selected card, group or backdrop.
//!
//! A graph of eighty-eight node types and twenty cards, six of which say `Move`, is a graph you
//! read by *tracing wires*. Every node editor that expects to hold a real document lets you name
//! the cards (Blender F2, Houdini, Nuke, TouchDesigner, Cavalry), and they all bind it to the
//! same key.
//!
//! **One gesture, three targets.** A node, a collapsed group and a backdrop are three id spaces
//! and three storage places, but to the artist they are one verb: *call this thing something*.
//! So there is one box, one intent and one undo step, and [`RenameTarget`] says which of the
//! three the name is landing on — a bare id would be a coin toss, because a document routinely
//! has a node 3 *and* a subgraph 3 *and* a backdrop 3.
//!
//! Two of the three already had somewhere to put the name (`Subgraph::title`, `Backdrop::title`)
//! and **nothing in the editor could set either** — the backdrop's rename intent existed, was
//! handled by the shell, and had no emitter. The data model was not the gap. The box was.
//!
//! ## The buffer lives in the STORE
//!
//! Exactly like the add-menu's search field (doc 59): the `WidgetStore` owns the text, the caret
//! and the selection, and this module only ever *reads* it. Two copies of one string, edited on
//! both sides, is how a text field starts lying about what it holds.

use crate::hits::rename_id;
use crate::snapshot::{
    GraphIntent, GraphViewSnapshot, NodeViewKind, RenameTarget, SUBGRAPH_VIEW_TAG,
    is_subgraph_view, push_intent,
};
use crate::state::{MotionGraphPanelState, Rename};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::TextInputState;

/// **F2** — open the box over whatever single thing is selected.
///
/// Exactly one thing: renaming is a question about *one* name, and a multi-selection has no
/// answer to it. (Deleting many is meaningful; naming many is not.) A backdrop wins over an empty
/// node selection because the two selections are mutually exclusive by construction.
pub(crate) fn arm(state: &mut MotionGraphPanelState, snap: &GraphViewSnapshot) {
    let target = if let Some(b) = state.selected_backdrop {
        Some(RenameTarget::Backdrop(b))
    } else if state.selected.len() == 1 {
        let id = *state.selected.iter().next().expect("len == 1");
        if is_subgraph_view(id) {
            Some(RenameTarget::Subgraph(id & !SUBGRAPH_VIEW_TAG))
        } else {
            Some(RenameTarget::Node(id))
        }
    } else {
        None
    };
    let Some(target) = target else {
        return; // nothing, or many things: no one name to ask about
    };
    // The box opens holding **what the thing is called right now** — which is what the artist can
    // see, so it is what they expect to be editing. For an unnamed node that is its type's
    // display name, and typing over it is the fast path; clearing it is how you go back.
    state.rename = Some(Rename {
        target,
        seed: current_name(snap, target),
        opened: false,
    });
}

/// What the target is called at this moment, read off the SAME snapshot the paint draws from —
/// so the box seeds with the string the artist is looking at, never with a second derivation of
/// it ([[feedback_derived_coordinate_seed_must_match_sample]]).
fn current_name(snap: &GraphViewSnapshot, target: RenameTarget) -> String {
    let card = match target {
        RenameTarget::Node(id) => Some(id),
        RenameTarget::Subgraph(s) => Some(s | SUBGRAPH_VIEW_TAG),
        RenameTarget::Backdrop(_) => None,
    };
    match (card, target) {
        (Some(view), _) => snap
            .nodes
            .iter()
            .find(|n| n.id == view && n.kind != NodeViewKind::Ghost)
            .map(|n| n.display_name.clone())
            .unwrap_or_default(),
        (None, RenameTarget::Backdrop(id)) => snap
            .backdrops
            .iter()
            .find(|b| b.id == id)
            .map(|b| b.title.clone())
            .unwrap_or_default(),
        (None, _) => String::new(),
    }
}

/// Where the box paints and hit-tests: **over the thing's title**, so the name you are typing is
/// in the place the name lives. (A dialog in the middle of the screen would be a second place to
/// look for the same string.)
pub(crate) fn box_rect(
    target: RenameTarget,
    snap: &GraphViewSnapshot,
    view: &crate::geom::View,
) -> Option<ph2d_editor_core::zones::Rect> {
    match target {
        RenameTarget::Backdrop(id) => snap
            .backdrops
            .iter()
            .find(|b| b.id == id)
            .map(|b| crate::backdrop::header_rect(b, view)),
        RenameTarget::Node(_) | RenameTarget::Subgraph(_) => {
            let want = match target {
                RenameTarget::Subgraph(s) => s | SUBGRAPH_VIEW_TAG,
                RenameTarget::Node(id) => id,
                RenameTarget::Backdrop(_) => unreachable!("handled above"),
            };
            let n = snap.nodes.iter().find(|n| n.id == want)?;
            let card = crate::geom::card_rect(n, view);
            Some(ph2d_editor_core::zones::Rect::new(
                card.x,
                card.y,
                card.w,
                crate::geom::HEADER_H * view.zoom,
            ))
        }
    }
}

/// **Who owns the keyboard** — settled AFTER the frame's gestures, since a gesture (F2, a click
/// away) is what opens and closes the box. The same shape as the menu's `settle_focus`, and for
/// the same reason: the box closes down four paths (Enter, Esc, a click elsewhere, the thing
/// being deleted under it), and chasing the blur through all four is how one gets forgotten.
///
/// A field that kept focus after the box closed would swallow every shortcut in the editor: `A`
/// would not open the add-menu, it would type an "a" into a buffer nobody can see.
pub(crate) fn settle_focus(
    state: &mut MotionGraphPanelState,
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
) {
    match state.rename.as_mut() {
        Some(r) if !r.opened => {
            open_box(ctx.host.store_mut(), &r.seed);
            r.opened = true;
        }
        None if ctx.host.store().focus_id() == Some(rename_id()) => {
            ctx.host.store_mut().set_focus(None);
        }
        _ => {}
    }
}

/// Register the box, seed it with the current name **selected whole**, and give it the keyboard —
/// once, on the frame it opens.
///
/// Seeded-and-selected is the rename convention everywhere (Finder, Explorer, Blender): the first
/// character you type replaces the old name, and yet the old name is right there if you only meant
/// to append to it.
pub(crate) fn open_box(store: &mut WidgetStore, seed: &str) {
    let id = rename_id();
    store.register(
        id,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: seed.to_string(),
            caret: seed.chars().count(),
            selection_anchor: Some(0),
        },
    );
    store.set_focus(Some(id));
    // Esc closes the BOX (and keeps the old name), rather than merely blurring the field and
    // leaving an editor on screen that no longer answers the keyboard.
    store.mark_cancel_on_escape(id);
}

/// Draw the box over the target's title — and, when the target is **gone** (deleted, ungrouped,
/// or simply not on this canvas any more because the artist walked into another level), close it.
///
/// A box floating over nothing, still holding the keyboard, is the shape of a bug that eats every
/// shortcut in the editor. So the paint is also where the box learns it has outlived its subject:
/// the snapshot is the truth about what exists, and this is the frame that reads it.
pub(crate) fn paint(
    state: &mut MotionGraphPanelState,
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    _rect: ph2d_editor_core::zones::Rect,
    snap: &GraphViewSnapshot,
    view: &crate::geom::View,
) {
    let Some(r) = state.rename.as_ref() else {
        return;
    };
    let Some(field) = box_rect(r.target, snap, view) else {
        state.rename = None; // its subject is gone; `settle_focus` hands the keyboard back
        return;
    };
    let theme = ctx.host.theme();
    let (fstate, text, caret, anchor) = match ctx.host.store().get(rename_id()) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        // The box opens on the NEXT frame's `settle_focus` (a gesture opened it, and the store is
        // written after the gestures), so on the very first frame it is not registered yet: draw
        // it seeded, so it never flashes empty.
        _ => (TextInputState::Focused, r.seed.clone(), 0, None),
    };
    let input = ph2d_editor_core::widget::TextInput::new(rename_id(), "").state(fstate);
    ph2d_editor_core::widget::paint_text_input_with_buffer(
        &input,
        Some(&text),
        Some(caret),
        anchor,
        field,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(rename_id(), field);
}

/// **Enter** — the name in the box becomes the name of the thing. An empty box clears the name
/// (the graph/doc side treats empty as "call it what it is").
pub(crate) fn commit(state: &mut MotionGraphPanelState, store: &WidgetStore) {
    let Some(r) = state.rename.take() else {
        return;
    };
    let name = store.text(rename_id()).unwrap_or_default().to_string();
    push_intent(GraphIntent::Rename {
        target: r.target,
        name,
    });
}
