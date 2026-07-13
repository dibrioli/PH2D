//! **The add-node menu's gestures** — scrolling the list, dragging its scrollbar, and resolving
//! the row that was picked (doc 54). Split from `interact` for the panel LOC cap; `super` is
//! `interact`.
//!
//! The library has **86 node types**, and a popup as tall as its list runs off the bottom of the
//! screen with the last forty entries unreachable (Enio's screenshot). So the panel is capped to
//! the canvas, and the list scrolls inside it — by the wheel, or by a scrollbar you can drag.

use super::{GraphViewSnapshot, Menu, MotionGraphPanelState, geom};
use crate::snapshot::{GraphIntent, menu_catalog, menu_rows, push_intent};
use crate::state::MenuBody;
use ph2d_editor_core::interaction::GraphZoom;
use ph2d_editor_core::zones::Rect;

/// How far one wheel notch scrolls the list.
const MENU_WHEEL_STEP: f32 = 0.6; // LITERAL-PX-OK: wheel-notch -> menu scroll px

/// How many rows the open popup has — its panel height, its scroll extent and its
/// thumb are all functions of THIS, and so is the hit-test. It used to be read off the
/// unfiltered catalog here while the paint used a different list, which is how a menu
/// can be scrollable to rows that are not there.
fn row_count(snap: &GraphViewSnapshot, menu: &Menu) -> usize {
    menu_rows(snap, menu).len()
}

/// **The wheel over an OPEN menu scrolls it, and does not zoom the canvas.** Returns whether the
/// menu ate the notch. A wheel that zoomed the graph out from under a list you are reading would
/// be the most annoying thing in the editor.
pub(super) fn scroll_menu(
    state: &mut MotionGraphPanelState,
    rect: Rect,
    snap: &GraphViewSnapshot,
    z: &GraphZoom,
) -> bool {
    let Some(menu) = state.menu.as_mut() else {
        return false;
    };
    let count = row_count(snap, menu);
    let panel = geom::menu_panel(menu, count, rect);
    if !panel.contains(z.anchor_x, z.anchor_y) {
        return false;
    }
    let max = geom::menu_max_scroll(panel, count);
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
    snap: &GraphViewSnapshot,
    x: f32,
    y: f32,
) -> Option<f32> {
    let menu = state.menu.as_ref()?;
    let count = row_count(snap, menu);
    let panel = geom::menu_panel(menu, count, rect);
    let thumb = geom::menu_thumb(panel, count, menu.scroll)?;
    let track = geom::menu_track(panel, count)?;
    if thumb.contains(x, y) {
        Some(y - thumb.y)
    } else if track.contains(x, y) {
        Some(thumb.h * 0.5)
    } else {
        None
    }
}

/// Drag the thumb: the list follows the cursor, and the thumb stays under the point that seized it.
pub(super) fn drag_menu_thumb(
    state: &mut MotionGraphPanelState,
    rect: Rect,
    snap: &GraphViewSnapshot,
    y: f32,
    grab: f32,
) {
    let Some(menu) = state.menu.as_mut() else {
        return;
    };
    let count = row_count(snap, menu);
    let panel = geom::menu_panel(menu, count, rect);
    menu.scroll = geom::menu_scroll_at(panel, count, y, grab);
}

/// Resolve a primary click at `(x, y)` against the open popup — WHICH row was hit, and
/// then what that row means:
///
/// - **library row** → `AddNode` at the menu's graph-space spawn point, and when the menu
///   was opened by a wire dropped in space (**smart-connect**), a `Connect` right behind
///   it, so the node arrives already wired to what asked for it.
/// - **card-port row** (doc 57 §5) → an ordinary `Connect` to the REAL port inside the
///   group. The card grows its socket by derivation, from the edge that now crosses it.
///
/// The row index is resolved against [`menu_rows`] — the same list the paint drew. Anything
/// else and the artist clicks one row and gets another.
pub(super) fn resolve_menu(menu: &Menu, rect: Rect, snap: &GraphViewSnapshot, x: f32, y: f32) {
    let rows = menu_rows(snap, menu);
    let panel = geom::menu_panel(menu, rows.len(), rect);
    let hit = (0..rows.len()).find(|i| {
        // A row half-scrolled out of the band is half-clickable: the hit is the row INTERSECTED
        // with the list's viewport — the same rect the paint clips to, because the row you can see
        // is the row you can click.
        let row = geom::menu_row(panel, *i, menu.scroll);
        geom::menu_list(panel).contains(x, y) && row.contains(x, y)
    });
    let Some(i) = hit else { return };
    match &menu.body {
        // The wire lands on the port the artist named. Nothing about the group is edited —
        // the graph is flat, so this is the same `Connect` any two nodes would make.
        MenuBody::CardPorts {
            rows,
            other,
            forward,
            detach,
        } => {
            let p = &rows[i];
            // A wire whose end was pulled off MOVES; it does not copy (doc 45). The port
            // just named is where it moves TO.
            if let Some((old_to_node, old_to_port)) = *detach {
                push_intent(GraphIntent::MoveWireEnd {
                    from_node: other.0,
                    from_port: other.1,
                    old_to_node,
                    old_to_port,
                    new_to: Some((p.node, p.port)),
                });
                return;
            }
            let (from, to) = if *forward {
                (*other, (p.node, p.port))
            } else {
                ((p.node, p.port), *other)
            };
            push_intent(GraphIntent::Connect {
                from_node: from.0,
                from_port: from.1,
                to_node: to.0,
                to_port: to.1,
            });
        }
        MenuBody::Library { connect_from } => {
            let c = menu_catalog(snap, *connect_from)[i];
            // Smart-connect is ONE intent, not an add followed by a connect: it was
            // one gesture, so it must be one undo step (and the shell, which mints
            // the id, is the only one that can name the node it just created).
            match connect_from {
                Some((from_node, from_port)) => push_intent(GraphIntent::SmartConnect {
                    from_node: *from_node,
                    from_port: *from_port,
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
        }
    }
}
