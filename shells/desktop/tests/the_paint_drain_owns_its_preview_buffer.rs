//! **Arch-gate: the CPU preview drain OWNS its buffer — it never stashes the tool's live canvas.**
//!
//! ## What this protects
//!
//! Painting a plain stroke is CPU-bound on a big canvas for one reason: the shell used to stash the
//! drained composite `Arc` (a clone of the tool's `canvas_rgba` on the trivial path) in
//! `painter_preview.rgba` and hold it across the frame. The tool's next `stamp_dabs` then called
//! `Arc::make_mut` on a two-owner buffer and copied the WHOLE canvas before blitting the dab —
//! measured 0.34 ms/move @ 2048², 10 ms/move @ 4096², flat across brush size (a 0.5 px brush pays a
//! 64 MiB copy to change one pixel). The fix routes the drain through `own_preview_buffer`, which
//! returns an INDEPENDENT shell-owned buffer, and lets the drained `Arc` drop — so the tool stays the
//! sole owner and its write is in place.
//!
//! ## Why a text gate, and not only the unit gate
//!
//! `painter_preview_pipeline_tests::the_shell_owns_its_preview_buffer_never_the_tools_canvas` proves
//! `own_preview_buffer` itself returns an independent buffer. It cannot see the day someone reverts
//! the DRAIN to `*painter_preview = Some(cache { rgba: drained })` — bypassing the helper entirely
//! and stashing the tool's Arc again. That regression is invisible to every unit gate (the helper is
//! still correct; nobody calls it) and to `ship.sh`'s timing-free run. This gate reads the product
//! source and asserts the drain routes through the helper and stashes its result.

const SRC: &str = include_str!("../src/render_loop/painter_bridge.rs");

#[test]
fn the_paint_drain_owns_its_preview_buffer() {
    let call = SRC.find("let mirror = own_preview_buffer(").unwrap_or_else(|| {
        panic!(
            "the CPU preview drain no longer routes the composite through `own_preview_buffer`. It \
             MUST: stashing the drained tool `Arc` directly makes `stamp_dabs` copy the whole canvas \
             every move (the CPU-bound FPS drop). If the helper was renamed, update this gate."
        )
    });
    let store = SRC.find("rgba: mirror,").unwrap_or_else(|| {
        panic!(
            "the drain's `PreviewCache` no longer stashes `own_preview_buffer`'s output \
             (`rgba: mirror`). If it went back to stashing the drained tool Arc, every move copies \
             the whole canvas again."
        )
    });
    assert!(
        call < store,
        "`own_preview_buffer` must run before its result is stashed in the preview cache"
    );
    assert!(
        !SRC.contains("rgba: drained,"),
        "the drain stashed the tool's canvas Arc directly (`rgba: drained`) — that reinstates the \
         per-move whole-canvas copy this fix removed. Stash `own_preview_buffer`'s buffer instead."
    );
}
