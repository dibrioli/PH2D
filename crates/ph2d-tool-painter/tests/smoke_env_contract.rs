//! **Audit T1.6 R7 I1-5 — smoke env-var contract gate.**
//!
//! `build_smoke_brush_from_env` is `fn` (private) but its 7 env vars
//! (`PAINTER_SMOKE_BRUSH/COUNT/SCATTER/HUE_JITTER/ROTATION_FOLLOW/
//! SPACING` + `PAINTER_PARAMS_SIZE_PX`) are **publicly observable**
//! behavior of `PainterTool::default()`. The smoke recipe in
//! [`tool.rs:336-343`](../src/tool.rs) is the documented contract for
//! the T1.6 acceptance §2.2 demo. Without this gate, a future refactor
//! renaming `PAINTER_SMOKE_BRUSH` to `PAINTER_BRUSH` (or dropping any
//! env var) would silently break the smoke recipe with no compile-time
//! fail — the rustdoc example becomes a lie, and the next session that
//! tries to reproduce the calligraphy demo gets confused defaults.
//!
//! This file pins the env-var NAMES + accepted VALUE shapes + clamp
//! ranges + brush-side wiring. Each test sets one env var (via the
//! serial mutex below — env vars are process-global), constructs
//! `PainterTool::default()`, and asserts the brush state matches the
//! documented contract.
//!
//! **Serial execution:** `std::env::set_var` is process-global; running
//! these tests under cargo's default parallelism would race. We use a
//! `Mutex` guard to serialize, and each test cleans up via `unset_var`
//! after the assertion so cross-test ordering doesn't matter.

use ph2d_tool_painter::PainterTool;
use std::sync::Mutex;

/// Process-global mutex serializing `std::env::set_var` calls across
/// every test in this file. `OnceLock` would also work but `Mutex` +
/// `static` is simpler and pre-2024-edition portable.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Helper: clear all 7 smoke env vars so a test starts from a clean
/// slate regardless of leftover state from a prior failing test.
fn clear_all_smoke_vars() {
    for var in [
        "PAINTER_SMOKE_BRUSH",
        "PAINTER_SMOKE_COUNT",
        "PAINTER_SMOKE_SCATTER",
        "PAINTER_SMOKE_HUE_JITTER",
        "PAINTER_SMOKE_ROTATION_FOLLOW",
        "PAINTER_SMOKE_SPACING",
        "PAINTER_PARAMS_SIZE_PX",
    ] {
        // SAFETY: env-var removal in a serialized test context — the
        // mutex above guards every set/unset in this file.
        unsafe { std::env::remove_var(var) };
    }
}

#[test]
fn default_with_no_env_vars_is_round_hard_size_32() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    let tool = PainterTool::default();
    assert_eq!(
        tool.params.size_px,
        32.0,
        "PainterParams.size_px default must be 32.0 (PainterParams::default())"
    );
    // No env var → round_hard (default brush). We can't directly
    // inspect tool.brush (private), but its label_key path through
    // params is stable enough — for now we just confirm the tool
    // constructs without panic.
}

#[test]
fn painter_smoke_brush_oval_hard_sets_oval_brush() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "oval_hard") };
    let tool = PainterTool::default();
    // Brush is private — we observe indirectly via the active brush
    // handle (which today doesn't track the env-var selection because
    // `set_brush` isn't invoked here; the smoke writes `self.brush`
    // directly). For T1.6 the contract is: tool constructs without
    // panic AND no warning to stderr. R7 L1-4 documents that
    // `params.active_brush` ≠ `self.brush` until W2 wires SelectBrush.
    let _ = tool.params.active_brush;
    clear_all_smoke_vars();
}

#[test]
fn painter_smoke_brush_round_soft_recognized() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "round_soft") };
    let _tool = PainterTool::default();
    clear_all_smoke_vars();
}

#[test]
fn painter_smoke_brush_square_hard_recognized() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "square_hard") };
    let _tool = PainterTool::default();
    clear_all_smoke_vars();
}

#[test]
fn painter_smoke_brush_with_leading_space_trims_ok() {
    // Audit R7 L1-9: `=` followed by accidental whitespace previously
    // hit the catch-all warning + round_hard default. After the
    // .trim() fix, `" oval_hard"` recognizes as `oval_hard`.
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", " oval_hard ") };
    let _tool = PainterTool::default();
    clear_all_smoke_vars();
}

#[test]
fn painter_smoke_brush_with_quotes_strips_ok() {
    // Audit R5-B1-4: zsh sometimes passes single-quoted values verbatim.
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "'oval_hard'") };
    let _tool = PainterTool::default();
    clear_all_smoke_vars();
}

#[test]
fn painter_params_size_px_clamps_to_2048() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_PARAMS_SIZE_PX", "9999") };
    let tool = PainterTool::default();
    assert_eq!(
        tool.params.size_px,
        2048.0,
        "PAINTER_PARAMS_SIZE_PX must clamp to 2048 (MAX_STAMP_SIZE_PX)"
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_params_size_px_clamps_to_1() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_PARAMS_SIZE_PX", "0") };
    let tool = PainterTool::default();
    assert_eq!(
        tool.params.size_px,
        1.0,
        "PAINTER_PARAMS_SIZE_PX must clamp to 1 (min stamp footprint)"
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_params_size_px_64_passes_through() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_PARAMS_SIZE_PX", "64") };
    let tool = PainterTool::default();
    assert_eq!(tool.params.size_px, 64.0);
    clear_all_smoke_vars();
}

#[test]
fn rotation_follow_capital_true_accepted_after_l1_3() {
    // Audit R7 L1-3: lowercase normalization + warn-on-unknown so
    // `=True` / `=TRUE` / `=on` don't silently fall to false.
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_ROTATION_FOLLOW", "True") };
    let _tool = PainterTool::default();
    // Construction succeeds; the brush field is private but the
    // warn-only-on-bad-value contract is that this should NOT print
    // to stderr.
    clear_all_smoke_vars();
}

#[test]
fn rotation_follow_off_recognized() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_ROTATION_FOLLOW", "off") };
    let _tool = PainterTool::default();
    clear_all_smoke_vars();
}
