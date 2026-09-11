//! Arch gate: the Repeat Image 3×3 tile preview must be drawn **under** the Painter editing
//! chrome — `draw_repeat_image` is called BEFORE `draw_selection_overlay` / `draw_overlays`
//! in `render_loop/painter_bridge.rs`. All three write into the SAME overlay `VectorScene`,
//! where later calls paint on top.
//!
//! This pins the fix for the "overlay stops at the seam" bug (Enio 2026-07-11): with
//! Tiling + Repeat Image, a shape crossing the sprite border kept its raw geometry past the
//! edge and the overlay drew it un-clipped — but the 8 neighbour tiles (opaque full-canvas
//! blits) were drawn AFTER the chrome, covering everything beyond the border. The editor
//! overlay looked cut at the sprite edge, the brush ring and the selection ants vanished
//! over the neighbour tiles. Re-ordering the calls back turns this gate red.

use std::fs;
use std::path::Path;

#[test]
fn repeat_image_tiles_draw_under_the_editing_chrome() {
    let bridge = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop/painter_bridge.rs");
    let src =
        fs::read_to_string(&bridge).unwrap_or_else(|e| panic!("read {}: {e}", bridge.display()));
    // Byte offset of the first CALL site (skip comment lines so prose can name the fns freely).
    let call_offset = |needle: &str| -> usize {
        let mut offset = 0;
        for line in src.lines() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("//") && trimmed.contains(needle) {
                return offset;
            }
            offset += line.len() + 1;
        }
        panic!(
            "call `{needle}` not found in {} — if the Repeat Image or the overlay dispatch \
             moved, point this gate at the new site (the tiles-under-chrome z-order must \
             stay provable)",
            bridge.display()
        );
    };
    let tiles = call_offset("draw_repeat_image(");
    let selection = call_offset("draw_selection_overlay(");
    let chrome = call_offset("draw_overlays(");
    assert!(
        tiles < selection && tiles < chrome,
        "`draw_repeat_image` must be called BEFORE `draw_selection_overlay` and \
         `draw_overlays` in painter_bridge.rs: the three share one overlay VectorScene \
         (later = on top), and the tiles are opaque full-canvas blits — drawn after the \
         chrome they cover every overlay past the sprite border (the 'overlay stops at \
         the seam' bug, Enio 2026-07-11). offsets: tiles={tiles} selection={selection} \
         chrome={chrome}"
    );
}
