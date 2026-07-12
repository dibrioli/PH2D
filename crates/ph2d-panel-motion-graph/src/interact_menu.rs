//! **The add-node menu's gestures** — scrolling the list, dragging its scrollbar, and resolving
//! the row that was picked (doc 54). Split from `interact` for the panel LOC cap; `super` is
//! `interact`.
//!
//! The library has **86 node types**, and a popup as tall as its list runs off the bottom of the
//! screen with the last forty entries unreachable (Enio's screenshot). So the panel is capped to
//! the canvas, and the list scrolls inside it — by the wheel, or by a scrollbar you can drag.

use super::{AddMenu, GraphViewSnapshot, MotionGraphPanelState, geom};
use crate::snapshot::{GraphIntent, current_catalog, push_intent};
use ph2d_editor_core::interaction::GraphZoom;
use ph2d_editor_core::zones::Rect;

/// How far one wheel notch scrolls the list.
const MENU_WHEEL_STEP: f32 = 0.6; // LITERAL-PX-OK: wheel-notch -> menu scroll px

/// **The wheel over an OPEN menu scrolls it, and does not zoom the canvas.** Returns whether the
/// menu ate the notch. A wheel that zoomed the graph out from under a list you are reading would
/// be the most annoying thing in the editor.
pub(super) fn scroll_menu(state: &mut MotionGraphPanelState, rect: Rect, z: &GraphZoom) -> bool {
    let count = current_catalog().len();
    let Some(menu) = state.add_menu.as_mut() else {
        return false;
    };
    let panel = geom::add_menu_panel(menu, count, rect);
    if !panel.contains(z.anchor_x, z.anchor_y) {
        return false;
    }
    let max = geom::add_menu_max_scroll(panel, count);
    menu.scroll = (menu.scroll - z.delta * MENU_WHEEL_STEP).clamp(0.0, max); // CLAMP-OK: 0..max
    true
}

/// The scrollbar under `(x, y)`, and where INSIDE the thumb the cursor took hold.
///
/// The whole TRACK is grabbable, not just the thumb: a press on the empty track jumps the thumb to
/// the cursor and drags from there, which is what every scrollbar in the world does.
pub(super) fn grab_menu_thumb(
    state: &MotionGraphPanelState,
    rect: Rect,
    x: f32,
    y: f32,
) -> Option<f32> {
    let menu = state.add_menu.as_ref()?;
    let count = current_catalog().len();
    let panel = geom::add_menu_panel(menu, count, rect);
    let thumb = geom::add_menu_thumb(panel, count, menu.scroll)?;
    let track = geom::add_menu_track(panel, count)?;
    if thumb.contains(x, y) {
        Some(y - thumb.y)
    } else if track.contains(x, y) {
        Some(thumb.h * 0.5)
    } else {
        None
    }
}

/// Drag the thumb: the list follows the cursor, and the thumb stays under the point that seized it.
pub(super) fn drag_menu_thumb(state: &mut MotionGraphPanelState, rect: Rect, y: f32, grab: f32) {
    let count = current_catalog().len();
    let Some(menu) = state.add_menu.as_mut() else {
        return;
    };
    let panel = geom::add_menu_panel(menu, count, rect);
    menu.scroll = geom::add_menu_scroll_at(panel, count, y, grab);
}

/// Resolve a primary click at `(x, y)` against the open add-menu: if it lands on a
/// catalog row, emit `AddNode` at the menu's graph-space spawn point — and, when
/// the menu was opened by a wire dropped in space (**smart-connect**), a `Connect`
/// right behind it, so the node arrives already wired to what asked for it.
pub(super) fn resolve_add_menu(
    menu: &AddMenu,
    rect: Rect,
    snap: &GraphViewSnapshot,
    x: f32,
    y: f32,
) {
    let catalog = crate::snapshot::menu_catalog(snap, menu.connect_from);
    let panel = geom::add_menu_panel(menu, catalog.len(), rect);
    for (i, c) in catalog.iter().enumerate() {
        // A row half-scrolled out of the band is half-clickable: the hit is the row INTERSECTED
        // with the list's viewport — the same rect the paint clips to, because the row you can see
        // is the row you can click.
        let row = geom::add_menu_row(panel, i, menu.scroll);
        if geom::add_menu_list(panel).contains(x, y) && row.contains(x, y) {
            // Smart-connect is ONE intent, not an add followed by a connect: it was
            // one gesture, so it must be one undo step (and the shell, which mints
            // the id, is the only one that can name the node it just created).
            match menu.connect_from {
                Some((from_node, from_port)) => push_intent(GraphIntent::SmartConnect {
                    from_node,
                    from_port,
                    to_type: c.type_name,
                    x: menu.spawn.0,
                    y: menu.spawn.1,
                }),
                None => push_intent(GraphIntent::AddNode {
                    type_name: c.type_name,
                    x: menu.spawn.0,
                    y: menu.spawn.1,
                }),
            }
            return;
        }
    }
}
