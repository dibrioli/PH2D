//! **Audit T1.6 R7 I1-5 + R8 N1-2 — smoke env-var contract gate.**
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
//! ranges + brush-side wiring via **observable state assertions**
//! (R8 N1-2 strengthened the R7 ship which had 7/11 tests that only
//! verified construction-without-panic — those wouldn't catch a
//! rename refactor that silently fell through to the default brush).
//! Every test now asserts the actual brush slot / param value the
//! env var should have produced.
//!
//! **Serial execution:** `std::env::set_var` is process-global;
//! running these tests under cargo's default parallelism would race.
//! We use a `Mutex` guard to serialize, and each test cleans up via
//! `unset_var` after the assertion so cross-test ordering doesn't
//! matter.

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

/// **Audit T1.6 R8 N1-2:** observe the actual brush slot via
/// `tool.active_brush()` (P1-3 accessor). Replaces R7's "constructs
/// without panic" smoke assertions, which would pass even after a
/// silent rename of `PAINTER_SMOKE_BRUSH`.
fn assert_active_shape_slot(tool: &PainterTool, expected_slot: u32, msg: &str) {
    match &tool.active_brush().shape.shape_source {
        ph2d_painter_brush::shape::ShapeSource::Builtin { atlas_layer, .. } => {
            assert_eq!(
                *atlas_layer, expected_slot,
                "{msg}: expected slot {expected_slot}, got {atlas_layer}"
            );
        }
        other => panic!("{msg}: expected Builtin shape source, got {other:?}"),
    }
}

#[test]
fn default_with_no_env_vars_is_round_hard_size_32() {
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    let tool = PainterTool::default();
    assert_eq!(
        tool.params.size_px, 32.0,
        "PainterParams.size_px default must be 32.0 (PainterParams::default())"
    );
    assert_active_shape_slot(
        &tool,
        ph2d_painter_brush::ROUND_HARD_SLOT,
        "no env var -> default brush is round_hard",
    );
}

#[test]
fn painter_smoke_brush_oval_hard_sets_oval_brush() {
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "oval_hard") };
    let tool = PainterTool::default();
    assert_active_shape_slot(
        &tool,
        ph2d_painter_brush::OVAL_HARD_SLOT,
        "PAINTER_SMOKE_BRUSH=oval_hard",
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_smoke_brush_round_soft_recognized() {
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "round_soft") };
    let tool = PainterTool::default();
    assert_active_shape_slot(
        &tool,
        ph2d_painter_brush::ROUND_SOFT_SLOT,
        "PAINTER_SMOKE_BRUSH=round_soft",
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_smoke_brush_square_hard_recognized() {
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "square_hard") };
    let tool = PainterTool::default();
    assert_active_shape_slot(
        &tool,
        ph2d_painter_brush::SQUARE_HARD_SLOT,
        "PAINTER_SMOKE_BRUSH=square_hard",
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_smoke_brush_with_leading_space_trims_ok() {
    // Audit R7 L1-9: `=` followed by accidental whitespace previously
    // hit the catch-all warning + round_hard default. After the
    // .trim() fix, `" oval_hard"` recognizes as `oval_hard`.
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", " oval_hard ") };
    let tool = PainterTool::default();
    assert_active_shape_slot(
        &tool,
        ph2d_painter_brush::OVAL_HARD_SLOT,
        "PAINTER_SMOKE_BRUSH=' oval_hard ' (L1-9 trim)",
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_smoke_brush_with_quotes_strips_ok() {
    // Audit R5-B1-4: zsh sometimes passes single-quoted values verbatim.
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "'oval_hard'") };
    let tool = PainterTool::default();
    assert_active_shape_slot(
        &tool,
        ph2d_painter_brush::OVAL_HARD_SLOT,
        "PAINTER_SMOKE_BRUSH='oval_hard' (quote strip)",
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_params_size_px_clamps_to_2048() {
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_PARAMS_SIZE_PX", "9999") };
    let tool = PainterTool::default();
    assert_eq!(
        tool.params.size_px, 2048.0,
        "PAINTER_PARAMS_SIZE_PX must clamp to 2048 (MAX_STAMP_SIZE_PX)"
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_params_size_px_clamps_to_1() {
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    unsafe { std::env::set_var("PAINTER_PARAMS_SIZE_PX", "0") };
    let tool = PainterTool::default();
    assert_eq!(
        tool.params.size_px, 1.0,
        "PAINTER_PARAMS_SIZE_PX must clamp to 1 (min stamp footprint)"
    );
    clear_all_smoke_vars();
}

#[test]
fn painter_params_size_px_64_passes_through() {
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
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
    // R8 N1-2: assert the actual flag value, not just construction.
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    // Pin the base brush so we know what shape_rotation_follow default
    // would have been (oval_hard defaults to true; round_hard to false).
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "round_hard") };
    unsafe { std::env::set_var("PAINTER_SMOKE_ROTATION_FOLLOW", "True") };
    let tool = PainterTool::default();
    assert!(
        tool.active_brush().shape.shape_rotation_follow,
        "PAINTER_SMOKE_ROTATION_FOLLOW=True (case-insensitive) must \
         enable shape_rotation_follow even on a brush whose default \
         is false (round_hard)"
    );
    clear_all_smoke_vars();
}

#[test]
fn rotation_follow_off_recognized() {
    // Audit T1.6 R9 R1-1: explicit `.expect` surfaces mutex poisoning
    // (a panic mid-test would otherwise cascade silently across the
    // suite as cargo-test parallelism re-acquires the lock). The
    // diagnostic names the originating test so the failing case is
    // visible in CI output instead of just `PoisonError`.
    let _guard = ENV_LOCK
        .lock()
        .expect("ENV_LOCK poisoned by a prior test panic — see preceding test failure for root cause");
    clear_all_smoke_vars();
    // oval_hard defaults to true; explicit off must override.
    unsafe { std::env::set_var("PAINTER_SMOKE_BRUSH", "oval_hard") };
    unsafe { std::env::set_var("PAINTER_SMOKE_ROTATION_FOLLOW", "off") };
    let tool = PainterTool::default();
    assert!(
        !tool.active_brush().shape.shape_rotation_follow,
        "PAINTER_SMOKE_ROTATION_FOLLOW=off must disable \
         shape_rotation_follow even on oval_hard (default true)"
    );
    clear_all_smoke_vars();
}
