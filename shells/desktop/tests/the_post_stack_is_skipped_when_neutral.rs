//! ADR-0145 arch-gate: the app HDR post-stack (Pass 1d in `present.rs`) must run ONLY
//! when the grade is non-neutral — the byte-identity guarantee.
//!
//! The neutral point is bit-exact because the SHELL SKIPS the pass (the glow's
//! discipline, `intensity > 0`), NOT because the grade shader is a perfect identity — it
//! is only a near-identity. So if this guard were removed (or cravado em `true`), a
//! "neutral" frame would be graded by that near-identity shader and drift from the
//! byte-identical frame it must produce. This gate pins the guard on the product source.
//!
//! Lives in `shells/desktop/tests/` (an arch-gate over `present.rs`), so a `cargo test
//! -p` closure does NOT reach it — run it in the closing sweep.

const PRESENT: &str = include_str!("../src/render_loop/present.rs");

#[test]
fn the_post_stack_grade_is_gated_on_a_non_neutral_grade() {
    let call = PRESENT
        .find("post_stack.grade(")
        .expect("Pass 1d `post_stack.grade(` call must exist in present.rs");
    // The guard is the nearest `if` before the call.
    let before = &PRESENT[..call];
    let guard_at = before
        .rfind("if ")
        .expect("`post_stack.grade` must be guarded by an `if`");
    let guard = &before[guard_at..call];
    assert!(
        guard.contains("!grade.is_neutral()"),
        "the post-stack must run ONLY when `!grade.is_neutral()` (byte-identical at neutral, \
         ADR-0145) — found guard: {:?}",
        &guard[..guard.len().min(120)]
    );
    // Refuse the `true` / always-run mutation explicitly.
    assert!(
        !guard.contains("if true"),
        "the post-stack guard must not be cravado em `true`"
    );
}
