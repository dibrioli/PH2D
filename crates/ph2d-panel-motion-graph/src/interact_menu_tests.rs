//! Guards for the add-menu's SCROLL (doc 54). `super` is `interact`.
//!
//! The library has 86 node types. A popup as tall as its list runs off the bottom of the screen
//! and the last forty are unreachable — which is exactly what Enio's screenshot showed.

use super::tests::{CENTER, RECT, gesture, two_node_snapshot};
use super::*;
// `MenuBody` used to reach here via `interact`'s re-export; the smart-connect menu that imported
// it moved to `interact_drop`, so this test now names it directly.
use crate::snapshot::{GraphIntent, NodeChoice, drain_intents, set_current_node_catalog};
use crate::state::MenuBody;
use ph2d_node_registry::NodeUiCategory;

/// A catalog of `n` node types — enough of them to overflow any screen.
fn catalog(n: usize) {
    set_current_node_catalog(
        (0..n)
            .map(|_| NodeChoice {
                type_name: "motion.grid",
                display: "Grid",
                category: NodeUiCategory::Source,
                inputs: &[],
            })
            .collect(),
    );
}

fn open_menu(scroll: f32) -> MotionGraphPanelState {
    MotionGraphPanelState {
        menu: Some(Menu {
            scroll,
            screen: (RECT.x + 20.0, RECT.y + 20.0),
            spawn: (0.0, 0.0),
            query: String::new(),
            opened: false,
            body: MenuBody::Library {
                connect_from: None,
                splice: None,
            },
        }),
        ..Default::default()
    }
}

fn panel_of(st: &MotionGraphPanelState, n: usize) -> Rect {
    geom::menu_panel(st.menu.as_ref().unwrap(), n, RECT)
}

/// **The popup never runs off the canvas.** However many node types the library grows, the menu is
/// capped to the panel it is drawn in — and what does not fit SCROLLS.
///
/// FALSIFIED by the old geometry (height = rows x row-height): with 86 types the list ran hundreds
/// of pixels past the bottom edge and the last forty entries could not be reached at all.
#[test]
fn the_popup_is_capped_to_the_canvas_and_the_rest_scrolls() {
    catalog(86);
    let st = open_menu(0.0);
    let panel = panel_of(&st, 86);
    assert!(
        panel.y >= RECT.y && panel.y + panel.h <= RECT.y + RECT.h + 0.01,
        "the popup fits on the canvas: {panel:?} in {RECT:?}"
    );
    assert!(
        geom::menu_max_scroll(st.menu.as_ref().unwrap(), panel, 86) > 0.0,
        "…and what did not fit is reachable by scrolling"
    );
    // A short list needs no scrollbar at all — and does not get one: a scrollbar on a list that
    // fits is a control that lies about there being more.
    let short = panel_of(&st, 3);
    assert_eq!(
        geom::menu_max_scroll(st.menu.as_ref().unwrap(), short, 3),
        0.0
    );
    assert!(
        geom::menu_track(st.menu.as_ref().unwrap(), short, 3).is_none(),
        "no false bar"
    );
    set_current_node_catalog(Vec::new());
}

/// **The wheel over an open menu SCROLLS it — it does not zoom the canvas.** A wheel that zoomed
/// the graph out from under a list you are reading would be the most annoying thing in the editor.
///
/// Off the menu, the very same wheel zooms as it always did.
#[test]
fn the_wheel_scrolls_the_menu_and_leaves_the_canvas_alone() {
    catalog(86);
    let mut st = open_menu(0.0);
    let panel = panel_of(&st, 86);
    let zoom_before = st.view.zoom;

    let over_menu = GraphZoom {
        delta: -120.0,
        anchor_x: panel.x + panel.w * 0.5,
        anchor_y: panel.y + panel.h * 0.5,
    };
    assert!(
        scroll_menu(&mut st, RECT, &two_node_snapshot(), &over_menu),
        "the menu ate it"
    );
    assert!(st.menu.unwrap().scroll > 0.0, "…and the list moved down");
    assert_eq!(st.view.zoom, zoom_before, "the canvas did NOT zoom");

    // …and it stops at the end: scrolling forever past the last row is a list you cannot read.
    let mut st = open_menu(0.0);
    for _ in 0..200 {
        scroll_menu(&mut st, RECT, &two_node_snapshot(), &over_menu);
    }
    let max = geom::menu_max_scroll(st.menu.as_ref().unwrap(), panel, 86);
    assert!((st.menu.unwrap().scroll - max).abs() < 0.01, "clamped");

    // Away from the menu the wheel is the canvas's again.
    let mut st = open_menu(0.0);
    let away = GraphZoom {
        delta: -120.0,
        anchor_x: RECT.x + RECT.w - 5.0,
        anchor_y: RECT.y + RECT.h - 5.0,
    };
    assert!(
        !scroll_menu(&mut st, RECT, &two_node_snapshot(), &away),
        "not the menu's wheel"
    );
    assert_eq!(st.menu.unwrap().scroll, 0.0);
    set_current_node_catalog(Vec::new());
}

/// **Dragging the thumb scrolls the list, and the menu STAYS OPEN.**
///
/// FALSIFIED by the menu's own dismissal rule (a press anywhere closes it): reaching for the
/// scrollbar would shut the very menu you were trying to scroll — a control that destroys the
/// thing it controls.
#[test]
fn dragging_the_thumb_scrolls_and_does_not_close_the_menu() {
    catalog(86);
    let snap = two_node_snapshot();
    let mut st = open_menu(0.0);
    let panel = panel_of(&st, 86);
    let thumb =
        geom::menu_thumb(st.menu.as_ref().unwrap(), panel, 86, 0.0).expect("there is a thumb");
    let bg = GraphHitKind::Background;

    // Press ON the thumb…
    apply_gesture(
        &mut st,
        gesture(bg, GesturePhase::Begin, thumb.x + 2.0, thumb.y + 4.0),
        RECT,
        CENTER,
        &snap,
    );
    assert!(
        matches!(st.interaction, Interaction::MenuScroll { .. }),
        "the scrollbar took the press, not the dismissal"
    );

    // …drag it to the bottom of the track…
    let track = geom::menu_track(st.menu.as_ref().unwrap(), panel, 86).unwrap();
    apply_gesture(
        &mut st,
        gesture(bg, GesturePhase::Update, thumb.x + 2.0, track.y + track.h),
        RECT,
        CENTER,
        &snap,
    );
    let scrolled = st.menu.as_ref().expect("the menu is still open").scroll;
    assert!(
        (scrolled - geom::menu_max_scroll(st.menu.as_ref().unwrap(), panel, 86)).abs() < 0.5,
        "dragged to the end of the list: {scrolled}"
    );

    // …and let go: the menu is still open, still scrolled.
    apply_gesture(
        &mut st,
        gesture(bg, GesturePhase::End, thumb.x + 2.0, track.y + track.h),
        RECT,
        CENTER,
        &snap,
    );
    assert!(st.menu.is_some(), "the menu SURVIVED its own scrollbar");
    assert!(matches!(st.interaction, Interaction::Idle));
    set_current_node_catalog(Vec::new());
}

/// **The row you can SEE is the row you can click.** After scrolling, the row under the cursor is
/// the one drawn there — and a row scrolled out of the band is not clickable at all, however much
/// of the canvas its (unclipped) rect would cover.
#[test]
fn the_hit_follows_the_scroll_and_stops_at_the_band() {
    catalog(86);
    let st = open_menu(0.0);
    let panel = panel_of(&st, 86);
    let list = geom::menu_list(st.menu.as_ref().unwrap(), panel);

    // Unscrolled: row 0 sits at the top of the band.
    let r0 = geom::menu_row(st.menu.as_ref().unwrap(), panel, 0, 0.0);
    assert!(list.contains(r0.x + 1.0, r0.y + 1.0));

    // Scrolled by exactly three rows, row 3 is where row 0 was — and row 0 is ABOVE the band.
    let three = 3.0 * geom::MENU_ROW_H;
    let r3 = geom::menu_row(st.menu.as_ref().unwrap(), panel, 3, three);
    assert!((r3.y - r0.y).abs() < 0.01, "row 3 took row 0's place");
    let gone = geom::menu_row(st.menu.as_ref().unwrap(), panel, 0, three);
    assert!(
        !list.contains(gone.x + 1.0, gone.y + 1.0),
        "row 0 scrolled out of the band, so it is out of reach"
    );
    set_current_node_catalog(Vec::new());
}

/// **The paint and the click enumerate ONE list.**
///
/// They did not, and it was a real defect: `draw_menu` drew `current_catalog()` — all 86
/// types — while `resolve_menu` resolved the click against the *filtered* smart-connect
/// list. So on a wire-drop the artist read one row and pressed another, and the popup's
/// height and scrollbar were sized for a list nobody was looking at.
///
/// The fix is structural (both call `menu_rows`), so the gate is structural too: it reads
/// the source. A second derivation of the same list is the bug — one is the fix — and the
/// only way that regresses is by someone reaching for the catalog again, right here.
#[test]
fn the_paint_and_the_hit_enumerate_one_list() {
    let paint = include_str!("paint_menu.rs");
    let hit = include_str!("interact_menu.rs");
    for (who, src) in [("the paint", paint), ("the hit", hit)] {
        assert!(
            src.contains("menu_rows"),
            "{who} must enumerate the popup through `menu_rows`"
        );
        assert!(
            !src.contains("current_catalog()"),
            "{who} reached for the raw catalog again - that is the divergence coming back"
        );
    }
}

// ── The search (doc 59) ─────────────────────────────────────────────────────

/// A catalog whose FIRST entry is a decoy: "Mirror Alpha Pass" matches the query `map` as a
/// scattered subsequence (**M**irror **A**lpha **P**ass), so a search that filters without
/// RANKING would list it above the node the artist actually wants — in the very first row.
fn library() {
    use crate::snapshot::NodeChoice;
    set_current_node_catalog(vec![
        NodeChoice {
            type_name: "paint.mirror_alpha_pass",
            display: "Mirror Alpha Pass",
            category: NodeUiCategory::Utility,
            inputs: &[],
        },
        NodeChoice {
            type_name: "value.lfo",
            display: "LFO",
            category: NodeUiCategory::Source,
            inputs: &[],
        },
        NodeChoice {
            type_name: "value.map_range",
            display: "Map Range",
            category: NodeUiCategory::Utility,
            inputs: &[],
        },
    ]);
}

fn with_query(q: &str) -> MotionGraphPanelState {
    let mut st = open_menu(0.0);
    st.menu.as_mut().unwrap().query = q.to_string();
    st
}

/// **Typing finds the node, and the best match is FIRST** — and the row the artist clicks is
/// the row they were shown, because the paint, the hit and the ranking are one derivation.
///
/// This is the smoke that prompted it (Enio, 2026-07-13: *"não encontrei value.lfo"*): 86
/// types in a flat list, and he knew the name.
#[test]
fn typing_filters_the_library_and_the_first_row_is_the_best_match() {
    library();
    let snap = two_node_snapshot();
    let mut st = with_query("map");
    let menu = st.menu.clone().unwrap();

    let rows = crate::snapshot::menu_rows(&snap, &menu);
    assert!(
        rows.len() >= 2,
        "the decoy matches too - that is the point of it"
    );
    assert_eq!(
        rows[0].label, "Map Range",
        "the node the artist meant is FIRST. 'Mirror Alpha Pass' matches `map` as a scattered \
         subsequence and sits earlier in the catalog, so a filter that does not RANK would \
         hand them that one - and the whole difference between a search that works and a list \
         that happens to contain the answer is which row comes first"
    );

    // Clicking that first row adds THAT type: the pick resolves against the same filtered,
    // ranked list the paint drew.
    let panel = geom::menu_panel(&menu, rows.len(), RECT);
    let row = geom::menu_row(st.menu.as_ref().unwrap(), panel, 0, 0.0);
    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Background,
            GesturePhase::Click,
            row.x + 2.0,
            row.y + 2.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::AddNode {
            type_name: "value.map_range",
            x: 0.0,
            y: 0.0,
        }],
        "the row that was drawn first is the row the click adds"
    );
}

/// **The canonical name is a name too.** I told Enio to look for `value.lfo` — a string the
/// menu never showed him. Now it finds it, and so does the domain prefix.
#[test]
fn the_canonical_name_and_the_domain_are_both_searchable() {
    library();
    let snap = two_node_snapshot();

    let menu = with_query("value.lfo").menu.unwrap();
    assert_eq!(crate::snapshot::menu_rows(&snap, &menu)[0].label, "LFO");

    // Typing a DOMAIN lists that domain — the structure the flat list never exposed.
    let menu = with_query("value").menu.unwrap();
    let rows = crate::snapshot::menu_rows(&snap, &menu);
    let labels: Vec<&str> = rows.iter().map(|r| r.label).collect();
    assert!(labels.contains(&"LFO") && labels.contains(&"Map Range"));
    assert!(
        !labels.contains(&"Mirror Alpha Pass"),
        "a paint.* node is not a value.* node"
    );

    // A query that matches nothing lists nothing — and picking in an empty list does nothing.
    let menu = with_query("zzzz").menu.unwrap();
    assert!(crate::snapshot::menu_rows(&snap, &menu).is_empty());
    crate::menu_search::pick_first(&menu);
    assert!(drain_intents().is_empty());
}

/// **Enter takes the top match** — the row you are already looking at. A search box where the
/// obvious key does nothing sends you back to the mouse.
#[test]
fn enter_picks_the_top_match() {
    library();
    crate::snapshot::set_current_motion_graph(Some(two_node_snapshot()));
    let menu = with_query("mr").menu.unwrap(); // initials of "Map Range"
    crate::menu_search::pick_first(&menu);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::AddNode {
            type_name: "value.map_range",
            x: 0.0,
            y: 0.0,
        }]
    );
}
