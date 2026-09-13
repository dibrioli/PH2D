//! **Os testes da seta de redimensionar** (`input_dispatch::cursor_tests`) — o corpo do módulo mudou-se VERBATIM
//! (`line/input-dispatch`, 2026-09-13) para um ficheiro, declarado por `#[path]` no índice com o MESMO nome.

use super::resize_cursor_for_edges;
use ph2d_editor_core::interaction::{
    TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R, TIMELINE_EDGE_T,
};
use winit::window::CursorIcon;

#[test]
fn each_edge_points_across_the_side_it_moves() {
    assert_eq!(
        resize_cursor_for_edges(TIMELINE_EDGE_L),
        CursorIcon::EwResize
    );
    assert_eq!(
        resize_cursor_for_edges(TIMELINE_EDGE_R),
        CursorIcon::EwResize
    );
    assert_eq!(
        resize_cursor_for_edges(TIMELINE_EDGE_T),
        CursorIcon::NsResize
    );
    assert_eq!(
        resize_cursor_for_edges(TIMELINE_EDGE_B),
        CursorIcon::NsResize
    );
}

#[test]
fn each_corner_points_along_its_own_diagonal() {
    let tl = TIMELINE_EDGE_T | TIMELINE_EDGE_L;
    let br = TIMELINE_EDGE_B | TIMELINE_EDGE_R;
    let tr = TIMELINE_EDGE_T | TIMELINE_EDGE_R;
    let bl = TIMELINE_EDGE_B | TIMELINE_EDGE_L;
    assert_eq!(resize_cursor_for_edges(tl), CursorIcon::NwseResize);
    assert_eq!(resize_cursor_for_edges(br), CursorIcon::NwseResize);
    assert_eq!(resize_cursor_for_edges(tr), CursorIcon::NeswResize);
    assert_eq!(resize_cursor_for_edges(bl), CursorIcon::NeswResize);
}

#[test]
fn an_empty_mask_never_shows_a_vertical_arrow() {
    // Defensive: a mask with no bits is a horizontal edge by fallback, not a
    // panic and not a misleading up-down arrow on a left/right grip.
    assert_eq!(resize_cursor_for_edges(0), CursorIcon::EwResize);
}
