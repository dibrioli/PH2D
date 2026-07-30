//! **The add-node menu's gestures** — scrolling the list, dragging its scrollbar, and resolving
//! the row that was picked (doc 54). Split from `interact` for the panel LOC cap; `super` is
//! `interact`.
//!
//! The library has **86 node types**, and a popup as tall as its list runs off the bottom of the
//! screen with the last forty entries unreachable (Enio's screenshot). So the panel is capped to
//! the canvas, and the list scrolls inside it — by the wheel, or by a scrollbar you can drag.

use super::{GraphViewSnapshot, Menu, MotionGraphPanelState, geom};
use crate::snapshot::ChoiceTarget;
use crate::snapshot::{GraphIntent, menu_matches, menu_rows, push_intent};
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
    let max = geom::menu_max_scroll(menu, panel, count);
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
    let thumb = geom::menu_thumb(menu, panel, count, menu.scroll)?;
    let track = geom::menu_track(menu, panel, count)?;
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
    menu.scroll = geom::menu_scroll_at(menu, panel, count, y, grab);
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
        let row = geom::menu_row(menu, panel, *i, menu.scroll);
        geom::menu_list(menu, panel).contains(x, y) && row.contains(x, y)
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
            // A PARAMETER (doc 58): it has no port, so it cannot be a `Connect`. Driving it
            // IS the promotion — the socket appears because the wire exists.
            if let ChoiceTarget::Param(param) = p.target {
                // A pulled-off end that lands on a param: the shell unplugs the old socket
                // and drives the param in one step (it knows both halves).
                push_intent(GraphIntent::DriveParam {
                    from_node: other.0,
                    from_port: other.1,
                    to_node: p.node,
                    param,
                });
                return;
            }
            // A wire whose end was pulled off MOVES; it does not copy (doc 45). The port
            // just named is where it moves TO.
            if let (Some((old_to_node, old_to_port)), ChoiceTarget::Port(port)) =
                (*detach, p.target)
            {
                push_intent(GraphIntent::MoveWireEnd {
                    from_node: other.0,
                    from_port: other.1,
                    old_to_node,
                    old_to_port,
                    new_to: Some((p.node, port)),
                });
                return;
            }
            let ChoiceTarget::Port(port) = p.target else {
                unreachable!("the param arm returned above");
            };
            let (from, to) = if *forward {
                (*other, (p.node, port))
            } else {
                ((p.node, port), *other)
            };
            push_intent(GraphIntent::Connect {
                from_node: from.0,
                from_port: from.1,
                to_node: to.0,
                to_port: to.1,
            });
        }
        // **The palette** (doc 62): the row IS the tint index — the paint enumerates
        // `TINT_NAMES` in order and so does this, which is the whole reason there is one
        // `menu_rows`. One undo step, and no re-cook: a colour is decoration, and a cook that
        // depended on it would be a cook that depended on taste.
        MenuBody::BackdropTints { backdrop, .. } => {
            push_intent(GraphIntent::SetBackdropColor {
                id: *backdrop,
                color: i as u8,
            });
        }
        MenuBody::Library {
            connect_from,
            splice,
        } => {
            // The SAME list the paint drew: filtered by the query AND ranked by it. Reading
            // the raw catalog here is exactly the bug this popup already had once.
            let c = menu_matches(snap, menu, *connect_from)[i];
            // ONE undo step whatever the shape (and the shell, which mints the id, is the only
            // one that can name the node it just created). `library_pick` is the shared door the
            // Enter pick (`menu_search::pick_first`) also uses — one rule, two callers.
            push_intent(crate::snapshot::library_pick(
                *connect_from,
                *splice,
                c.type_name,
                menu.spawn,
            ));
        }
    }
}

/// **What a right-press opens** (doc 62). On the PRESS (Begin), over ANY hit — doing it on the
/// press rather than the release makes it movement-independent: a right-click that drifts a pixel
/// is classified `End` by the dispatch, and would otherwise never open the menu (or would dismiss
/// it the instant it appeared). All secondary phases are absorbed by the caller, so a right-drag /
/// right-release never pans, selects or dismisses.
///
/// The BODY depends on what is under the cursor. Over the canvas, a node, a socket, a wire: the
/// node library. Over a **backdrop's header**: its eight tints — because over a backdrop, a list of
/// 88 node types is an answer to a question nobody asked, and the colour of the thing you are
/// pointing at is the one you want.
pub(super) fn open_on_right_press(
    state: &mut MotionGraphPanelState,
    g: ph2d_editor_core::interaction::GraphGesture,
    rect: Rect,
    snap: &GraphViewSnapshot,
) {
    use ph2d_editor_core::interaction::{GesturePhase, GraphHitKind};
    if g.phase != GesturePhase::Begin {
        return;
    }
    let spawn = super::View::new(rect, state.view).graph(g.x, g.y);
    let body = match g.kind {
        GraphHitKind::Backdrop { id } => {
            let id = id as u32;
            // A right-press selects what it is asking about, like every other press does.
            state.selected.clear();
            state.selected_backdrop = Some(id);
            MenuBody::BackdropTints {
                backdrop: id,
                current: snap
                    .backdrops
                    .iter()
                    .find(|b| b.id == id)
                    .map_or(0, |b| b.color),
            }
        }
        // R-press ON a wire arms a SPLICE: picking a node inserts it INTO this wire
        // (its unique target `(to_node, to_port)` names it) — source → new → target. The
        // generalisation of the double-click reroute (doc 45) to any type the artist chooses.
        GraphHitKind::Wire { edge } => MenuBody::Library {
            connect_from: None,
            splice: Some(crate::paint::wire_target(edge)),
        },
        _ => MenuBody::Library {
            connect_from: None,
            splice: None,
        },
    };
    state.menu = Some(super::Menu {
        scroll: 0.0,
        screen: (g.x, g.y),
        spawn,
        query: String::new(),
        opened: false,
        body,
    });
    state.interaction = super::Interaction::Idle;
}
