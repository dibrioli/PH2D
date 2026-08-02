//! Guard for the LOCAL add-menu's one row source (`menu_rows`). `super` is `interact`.
//!
//! The node LIBRARY moved to the shell's full-screen palette (the `OpenLibrary` intent → the
//! palette's OWN search + scroll, gated in `ph2d-editor-core`). The scroll-geometry and fuzzy-search
//! tests that lived here were the library's, and went with it. What stays LOCAL — the card-ports /
//! node-actions / backdrop-tint popups — must still draw and hit-test through the ONE `menu_rows`, or
//! a row means one thing on screen and another under the cursor.

/// **The paint and the click enumerate ONE list.** Both `draw_menu` and `resolve_menu` read the popup
/// through `menu_rows`; neither reaches for the raw catalog. A second derivation of the same list was a
/// real defect once (the paint drew all 86 types while the click resolved a *filtered* list, so on a
/// wire-drop the artist read one row and pressed another), and the only way it regresses is by someone
/// reaching for the catalog again — right here — so the gate reads the source.
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
