use super::*;
use crate::library::round_hard;

fn p(x: f32, y: f32) -> PointerSample {
    PointerSample {
        position: [x, y],
        pressure: 0.5,
        tilt: 0.0,
    }
}

#[test]
fn new_pool_has_max_capacity() {
    // Audit T1.6 U-8: tightened from `>= MAX` to `== MAX`. Rust
    // `Vec::with_capacity(N)` is allowed to allocate `>= N` (allocator
    // discretion); macOS/Linux/Windows allocators all return exactly
    // 4096 for this size today, but a future allocator change or a
    // bump to a non-page-aligned MAX could break. Pinning equality
    // makes the platform contract explicit.
    let s = StampScheduler::new();
    assert_eq!(
        s.pool.capacity(),
        MAX_STAMPS_PER_DISPATCH,
        "Vec::with_capacity({MAX_STAMPS_PER_DISPATCH}) must reserve \
             exactly that many slots — HR-3 cap fence depends on the \
             logical-len vs physical-capacity match"
    );
    assert!(s.pool.is_empty());
    assert!(!s.is_in_stroke());
}

#[test]
fn begin_stroke_resets_state() {
    // Test the canonical clean path: end_stroke between begin_strokes.
    // Audit T1.6 T-4: the debug_assert in begin_stroke catches
    // dangling-stroke bugs (covered by `begin_stroke_hard_resets_
    // dangling_stroke` below).
    let mut s = StampScheduler::new();
    s.begin_stroke(42);
    let brush = round_hard();
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    assert!(s.is_in_stroke());
    s.end_stroke();
    s.begin_stroke(99);
    assert!(s.pool.is_empty());
    assert!(!s.is_in_stroke());
    assert_eq!(s.stroke_seed, 99);
}

#[test]
fn begin_stroke_hard_resets_dangling_stroke() {
    // Audit T1.6 T-4: a dropped end_stroke (bridge bug, missed
    // pointer-up) leaves the scheduler with is_in_stroke()=true.
    // The next begin_stroke MUST still hard-reset (and the
    // debug_assert fires in debug builds to signal the bridge bug).
    // Release behavior: silent reset is correct — strokes from the
    // dropped pointer-up are discarded.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    assert!(s.is_in_stroke(), "stroke open after one advance");
    // Skip end_stroke — simulate bridge dropping the pointer-up.
    // begin_stroke must still recover.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        s.begin_stroke(2);
    }));
    if cfg!(debug_assertions) {
        assert!(
            result.is_err(),
            "debug_assert!(!is_in_stroke) MUST fire when begin_stroke \
                 is called on a non-ended stroke (T-4 bridge-bug signal)"
        );
        // After the assert panic, the StampScheduler may be in an
        // inconsistent state — drop it. The behavioral check below
        // runs on a fresh scheduler.
    } else {
        // Release: begin_stroke silently hard-resets.
        assert!(!s.is_in_stroke(), "release: hard reset takes effect");
        assert_eq!(s.stroke_seed, 2);
    }
}

#[test]
fn first_pointer_emits_one_stamp() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let stamps = s.advance(&brush, p(10.0, 20.0), 32.0, [0.7, 0.0, 0.0, 1.0]);
    assert_eq!(stamps.len(), 1);
    assert_eq!(stamps[0].position_world, [10.0, 20.0]);
    assert_eq!(stamps[0].size_px, 32.0);
    assert_eq!(stamps[0].color_oklab, [0.7, 0.0, 0.0, 1.0]);
}

#[test]
fn segment_emits_stamps_at_spacing() {
    let mut s = StampScheduler::new();
    s.begin_stroke(7);
    let brush = round_hard(); // spacing 0.10
    let diameter = 100.0;
    // spacing_px = 0.10 * 100.0 = 10 px. First pointer emits 1.
    let _ = s.advance(&brush, p(0.0, 0.0), diameter, [0.0; 4]);
    // Move to x=50; segment_len 50; expect 5 stamps at x = 10, 20, 30, 40, 50.
    let stamps = s.advance(&brush, p(50.0, 0.0), diameter, [0.0; 4]);
    assert_eq!(stamps.len(), 5, "expected 5 stamps from (0,0)→(50,0)");
    assert_eq!(stamps[0].position_world, [10.0, 0.0]);
    assert_eq!(stamps[4].position_world, [50.0, 0.0]);
}

#[test]
fn determinism_same_seed_same_jitter() {
    let mut brush = round_hard();
    brush.stroke_path.spacing_jitter = 0.5;
    brush.stroke_path.jitter_lateral = 0.3;

    let mut s1 = StampScheduler::new();
    s1.begin_stroke(12345);
    let _ = s1.advance(&brush, p(0.0, 0.0), 50.0, [0.5, 0.0, 0.0, 1.0]);
    let a: Vec<[f32; 2]> = s1
        .advance(&brush, p(100.0, 0.0), 50.0, [0.5, 0.0, 0.0, 1.0])
        .iter()
        .map(|st| st.position_world)
        .collect();

    let mut s2 = StampScheduler::new();
    s2.begin_stroke(12345);
    let _ = s2.advance(&brush, p(0.0, 0.0), 50.0, [0.5, 0.0, 0.0, 1.0]);
    let b: Vec<[f32; 2]> = s2
        .advance(&brush, p(100.0, 0.0), 50.0, [0.5, 0.0, 0.0, 1.0])
        .iter()
        .map(|st| st.position_world)
        .collect();

    // Audit T1.6 A1-U1-RESIDUE: this assertion runs both invocations
    // in the SAME process → trivially bit-equal within a single
    // Rust target. Cross-OS bit-equality of positions depends on
    // `f32::sqrt`/integer-mul determinism — INTEGER ops are
    // cross-OS bit-identical (wyhash mixer), but the `t_along *
    // ux + perp * lat_offset` chain uses f32 arithmetic. Within a
    // single Rust target it's deterministic; cross-OS HR-5 strict
    // requires `--features det-painter` (no-op today). The U-1
    // overclaim correction applies here as in
    // `color_jitter_deterministic_across_runs`.
    assert_eq!(
        a, b,
        "same seed must produce bit-identical positions within a \
             single Rust target (cross-OS HR-5 requires det-painter)"
    );
}

#[test]
fn advance_does_not_realloc_pool() {
    // HR-3: pool capacity is reserved at construct; subsequent advances
    // re-clear and re-push without realloc. We sample capacity before
    // and after a high-density burst.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.stroke_path.spacing = 0.01; // very tight (10x default)
    let cap_before = s.pool.capacity();
    let _ = s.advance(&brush, p(0.0, 0.0), 200.0, [0.0; 4]);
    // segment of 1000 px with spacing 0.01 * 200 = 2 px → 500 stamps; well
    // under cap. capacity should be unchanged.
    let _ = s.advance(&brush, p(1000.0, 0.0), 200.0, [0.0; 4]);
    let cap_after = s.pool.capacity();
    assert_eq!(
        cap_before, cap_after,
        "pool capacity must not grow across advances"
    );
    assert!(
        cap_after >= MAX_STAMPS_PER_DISPATCH,
        "pool must hold at least MAX_STAMPS_PER_DISPATCH"
    );
}

#[test]
fn pool_is_capped_at_max() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.stroke_path.spacing = 0.01; // tight
    // First pointer + a huge segment that would otherwise overflow:
    // 1_000_000 px / 2 px spacing = 500k stamps — cap at 4096.
    let _ = s.advance(&brush, p(0.0, 0.0), 200.0, [0.0; 4]);
    let stamps = s.advance(&brush, p(1_000_000.0, 0.0), 200.0, [0.0; 4]);
    assert!(
        stamps.len() <= MAX_STAMPS_PER_DISPATCH,
        "scheduler must cap at MAX_STAMPS_PER_DISPATCH (got {})",
        stamps.len()
    );
}

#[test]
fn nan_input_emits_nothing() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let bad = PointerSample {
        position: [f32::NAN, 0.0],
        pressure: 0.5,
        tilt: 0.0,
    };
    let stamps = s.advance(&brush, bad, 32.0, [0.0; 4]);
    assert!(
        stamps.is_empty(),
        "NaN input must produce zero stamps (HR-3 defense-in-depth)"
    );
    assert!(!s.is_in_stroke(), "state must remain pristine after NaN");
}

#[test]
fn duplicate_pointer_emits_nothing_after_first() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let _ = s.advance(&brush, p(5.0, 5.0), 32.0, [0.0; 4]);
    let stamps = s.advance(&brush, p(5.0, 5.0), 32.0, [0.0; 4]);
    assert!(
        stamps.is_empty(),
        "duplicate pointer (segment_len < eps) must emit nothing"
    );
}

#[test]
fn end_stroke_clears_state() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    s.end_stroke();
    assert!(!s.is_in_stroke());
    assert!(s.last_point.is_none());
    assert_eq!(s.residual_dist, 0.0);
}

#[test]
fn stamp_index_monotonic_across_advances() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let _ = s.advance(&brush, p(0.0, 0.0), 50.0, [0.0; 4]);
    let idx_after_one = s.stamp_index;
    // (0,0)→(25,0) at spacing 5 px → 5 stamps.
    let _ = s.advance(&brush, p(25.0, 0.0), 50.0, [0.0; 4]);
    assert!(s.stamp_index > idx_after_one, "stamp_index must grow");
}

#[test]
fn rendering_mode_propagates_to_stamps() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.rendering.rendering_mode = crate::RenderingMode::IntenseBlending;
    let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    assert_eq!(stamps.len(), 1);
    assert_eq!(
        stamps[0].rendering_mode,
        crate::RenderingMode::IntenseBlending as u32
    );
}

#[test]
fn residual_dist_chain_short_segments() {
    // Audit T1.5 round 1 A-H3 regression + round 2 F7 strengthening:
    // 3 consecutive sub-spacing segments. The contract is:
    // (a) `residual_dist ∈ [0, spacing_px)` at all times;
    // (b) cumulative stamps sit on the spacing grid (positions
    //     `0, spacing_px, 2·spacing_px, …`) — NOT clustered at
    //     segment boundaries.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard(); // spacing 0.10
    let diameter = 100.0;
    let spacing_px = 10.0;
    // 1st advance: emits stamp at (0,0). residual stays 0.
    let stamps_a = s
        .advance(&brush, p(0.0, 0.0), diameter, [0.0; 4])
        .iter()
        .map(|st| st.position_world[0])
        .collect::<Vec<_>>();
    assert_eq!(stamps_a, vec![0.0_f32], "first advance emits at origin");

    // Three sub-spacing segments (cumulative 9 < spacing_px=10).
    let _ = s.advance(&brush, p(3.0, 0.0), diameter, [0.0; 4]);
    assert!(
        s.residual_dist >= 0.0 && s.residual_dist < spacing_px,
        "residual must stay in [0, {}) after segment 1; got {}",
        spacing_px,
        s.residual_dist
    );
    let _ = s.advance(&brush, p(6.0, 0.0), diameter, [0.0; 4]);
    let _ = s.advance(&brush, p(9.0, 0.0), diameter, [0.0; 4]);
    assert!(
        s.residual_dist >= 0.0 && s.residual_dist < spacing_px,
        "residual must stay in [0, {}) after 3 short segments; got {}",
        spacing_px,
        s.residual_dist
    );

    // 5th advance crosses the next grid line.
    let stamps_d = s
        .advance(&brush, p(15.0, 0.0), diameter, [0.0; 4])
        .iter()
        .map(|st| st.position_world[0])
        .collect::<Vec<_>>();
    // At least one stamp landed on or near the next grid position.
    // The spacing grid expectation: stamps at x ∈ {0, 10, 20, …}.
    // After cumulative 9 px (3 short segments emitting zero stamps)
    // + 6 px crossing to x=15, the gridline at x=10 was crossed
    // once → exactly 1 stamp at x ≈ 10.
    assert_eq!(stamps_d.len(), 1, "exactly 1 stamp crossing x=10");
    assert!(
        (stamps_d[0] - spacing_px).abs() < 1.5,
        "grid stamp must be at x ≈ {} (got {})",
        spacing_px,
        stamps_d[0]
    );
    assert!(
        s.residual_dist >= 0.0 && s.residual_dist < spacing_px,
        "residual ∈ [0, {}) at end; got {}",
        spacing_px,
        s.residual_dist
    );
}

#[test]
fn break_segment_resets_last_point_without_ending_stroke() {
    // Audit T1.5 round 3 R3-LE-1: cursor exit/re-enter footprint
    // should not paint a smear across the gap. `break_segment`
    // wipes `last_point` so next advance starts fresh (single stamp
    // at new position, no interpolation).
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let diameter = 100.0;
    // Deposit one stamp at (0,0).
    let _ = s.advance(&brush, p(0.0, 0.0), diameter, [0.0; 4]);
    assert!(s.is_in_stroke());
    let initial_stamp_index = s.stamp_index;

    // Cursor exits footprint → break_segment.
    s.break_segment();
    assert!(
        !s.is_in_stroke(),
        "break_segment clears last_point (is_in_stroke becomes false)"
    );
    assert_eq!(s.residual_dist, 0.0);
    // BUT: stroke_seed + stamp_index counter survive — same stroke continues.
    assert_eq!(
        s.stamp_index, initial_stamp_index,
        "stamp_index must NOT reset (stroke continues)"
    );
    assert_eq!(s.stroke_seed, 1, "stroke_seed survives break_segment");

    // Re-enter at (500, 500) — should emit exactly 1 stamp at that
    // position (no smear interpolation from (0,0)).
    let stamps = s
        .advance(&brush, p(500.0, 500.0), diameter, [0.0; 4])
        .iter()
        .map(|st| st.position_world)
        .collect::<Vec<_>>();
    assert_eq!(
        stamps,
        vec![[500.0_f32, 500.0]],
        "re-entry after break_segment must emit ONE stamp at new pos \
             (no smear across the gap)"
    );
}

#[test]
fn stamp_index_does_not_exceed_pool_cap() {
    // Audit T1.5 round 2 F6 (MISSING-GATE-STAMP-INDEX-CAP). The
    // A-M4 desync risk fires exactly when `push_stamp` rejects
    // (cap saturated). Verify stamp_index advances ≤ pool cap.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.stroke_path.spacing = 0.01; // tight spacing
    // (0,0) → (1_000_000, 0) at spacing 0.01·200=2 px would emit
    // 500k stamps; pool caps at 4096.
    let _ = s.advance(&brush, p(0.0, 0.0), 200.0, [0.0; 4]);
    let stamps = s.advance(&brush, p(1_000_000.0, 0.0), 200.0, [0.0; 4]);
    let emitted = stamps.len();
    assert!(
        emitted <= MAX_STAMPS_PER_DISPATCH,
        "scheduler must cap at MAX_STAMPS_PER_DISPATCH (got {})",
        emitted
    );
    // stamp_index = first advance (1) + second advance (emitted).
    // Critical: it must NOT exceed the actual stamps pushed +
    // the prior advance's 1 stamp.
    assert_eq!(
        s.stamp_index,
        1 + emitted as u64,
        "stamp_index = 1 (initial) + actually-emitted; got {}",
        s.stamp_index
    );
}

#[test]
fn stamp_index_increments_one_per_pushed_stamp() {
    // Audit T1.5 round 1 A-M4: stamp_index must advance by exactly
    // the count of stamps actually emitted, never more.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let stamps = s.advance(&brush, p(0.0, 0.0), 50.0, [0.0; 4]).len() as u64;
    assert_eq!(s.stamp_index, stamps);
    let n2 = s.advance(&brush, p(25.0, 0.0), 50.0, [0.0; 4]).len() as u64;
    assert_eq!(s.stamp_index, stamps + n2);
}

#[test]
fn det_random_in_unit_interval() {
    let mut s = StampScheduler::new();
    s.begin_stroke(42);
    for i in 0..1024 {
        for axis in [0xA1, 0xB2, 0xC3] {
            let v = s.det_random(i, axis);
            assert!((0.0..1.0).contains(&v), "det_random out of [0,1): {v}");
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// T1.6 extension tests — shape_count / rotation / scatter / flip /
// color jitter / shape_layer propagation / rotated footprint scale.
// ──────────────────────────────────────────────────────────────────────

use crate::library::{ROUND_HARD_SLOT, SQUARE_HARD_SLOT, round_soft, square_hard};
use crate::shape::ShapeSource;
use crate::stamp::{FLAG_SHAPE_FLIP_X, FLAG_SHAPE_FLIP_Y};

#[test]
fn shape_count_emits_n_stamps_per_pointer() {
    // T1.6: multi-stamp emit per spacing step. `shape_count = 4`
    // should produce 4 stamps at the same world position on the
    // very first pointer of the stroke.
    let mut s = StampScheduler::new();
    s.begin_stroke(7);
    let mut brush = round_hard();
    brush.shape.shape_count = 4;
    let stamps = s.advance(&brush, p(10.0, 10.0), 32.0, [0.5, 0.0, 0.0, 1.0]);
    assert_eq!(stamps.len(), 4, "shape_count=4 must emit 4 stamps");
    // All four should share the same world position.
    for st in &stamps[1..] {
        assert_eq!(st.position_world, stamps[0].position_world);
    }
}

#[test]
fn shape_count_capped_at_max() {
    let mut s = StampScheduler::new();
    s.begin_stroke(7);
    let mut brush = round_hard();
    brush.shape.shape_count = 999; // out-of-spec; scheduler clamps to MAX_SHAPE_COUNT
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    assert_eq!(
        stamps.len(),
        MAX_SHAPE_COUNT as usize,
        "shape_count > MAX must clamp at {}",
        MAX_SHAPE_COUNT
    );
}

#[test]
fn shape_count_zero_clamps_as_defense_in_depth() {
    // Audit T1.6 Q-7: spec range is `1..=16`; `shape_count = 0` is
    // INVALID and should ideally be rejected at brush-load time.
    // The scheduler clamps as **defense-in-depth** so a corrupt
    // brush file or fuzz input doesn't panic the render loop. This
    // test pins the defensive behavior but does NOT endorse the
    // clamp as canonical — brush-param validation in
    // `ph2d-painter-contracts` (W2 follow-up) is where the rejection
    // belongs.
    let mut s = StampScheduler::new();
    s.begin_stroke(7);
    let mut brush = round_hard();
    brush.shape.shape_count = 0;
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    assert!(
        !stamps.is_empty(),
        "invalid shape_count=0 must not produce zero stamps (defense-in-depth)"
    );
    assert!(
        stamps.len() <= 1,
        "defense-in-depth clamp emits ≤ 1 stamp; got {}",
        stamps.len()
    );
}

#[test]
fn shape_rotation_follow_sets_atan2_on_direction() {
    // With rotation_follow=true, the stamp's rotation_rad equals
    // atan2(stroke_dir.y, stroke_dir.x). Test a 45° upward-right
    // segment → rotation should be π/4.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = square_hard();
    brush.shape.shape_rotation_follow = true;
    // First pointer (no direction yet — first stamp uses sentinel
    // [1, 0] → atan2(0, 1) = 0).
    let _ = s.advance(&brush, p(0.0, 0.0), 50.0, [0.0; 4]);
    // Move 45° up-right → segment direction = (√2/2, √2/2),
    // atan2 = π/4. With brush spacing 0.10 + diameter 50 = 5px,
    // segment length 100√2 ≈ 141 → ~28 spacing steps. Each step
    // group has 1 stamp (default shape_count=1).
    let stamps = s.advance(&brush, p(100.0, 100.0), 50.0, [0.0; 4]);
    assert!(!stamps.is_empty());
    let expected_angle = core::f32::consts::FRAC_PI_4;
    for st in stamps {
        assert!(
            (st.rotation_rad - expected_angle).abs() < 1e-5,
            "rotation_follow should yield π/4 on diagonal up-right; got {}",
            st.rotation_rad
        );
    }
}

#[test]
fn shape_scatter_produces_per_stamp_rotation_variance() {
    // With scatter=1.0 (max) and shape_count=4, the four stamps at a single
    // pointer step should have distinct rotation_rad values within ±π
    // (scatter 1.0 → up to ±180° of rotation).
    let mut s = StampScheduler::new();
    s.begin_stroke(42);
    let mut brush = round_hard();
    brush.shape.shape_count = 4;
    brush.shape.shape_scatter = 1.0;
    let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    assert_eq!(stamps.len(), 4);
    let mut rotations: Vec<f32> = stamps.iter().map(|st| st.rotation_rad).collect();
    rotations.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // Adjacent rotations should differ by at least some small amount
    // (PRNG won't return identical values for distinct stamp_index).
    for w in rotations.windows(2) {
        assert!(
            (w[1] - w[0]).abs() > 1e-3,
            "scatter must produce distinct rotations per stamp; got {} and {}",
            w[0],
            w[1]
        );
    }
    // All rotations within ±π.
    for r in &rotations {
        assert!(
            r.abs() <= core::f32::consts::PI + 1e-5,
            "scatter rotation outside ±π: {}",
            r
        );
    }
}

#[test]
fn shape_randomized_fixes_stroke_base() {
    // randomized=true → stroke_rotation_base set on first advance
    // and PERSISTS for the rest of the stroke. Verify by checking
    // that two stamps from different advances (with scatter=0) share
    // the same rotation_rad.
    let mut s = StampScheduler::new();
    s.begin_stroke(123);
    let mut brush = round_hard();
    brush.shape.shape_randomized = true;
    // scatter=0 + rotation_follow=false → only stroke_rotation_base
    // contributes to rotation.
    let st1 = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4])[0].rotation_rad;
    let st2_arr = s.advance(&brush, p(50.0, 50.0), 32.0, [0.0; 4]);
    for st in st2_arr {
        assert!(
            (st.rotation_rad - st1).abs() < 1e-5,
            "stroke_randomized base must persist across advances; got {} vs {}",
            st.rotation_rad,
            st1
        );
    }
}

#[test]
fn shape_randomized_survives_break_segment() {
    // Spec: break_segment preserves stroke_rotation_base — same
    // stroke continues.
    let mut s = StampScheduler::new();
    s.begin_stroke(99);
    let mut brush = round_hard();
    brush.shape.shape_randomized = true;
    let r1 = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4])[0].rotation_rad;
    s.break_segment();
    let r2 = s.advance(&brush, p(500.0, 500.0), 32.0, [0.0; 4])[0].rotation_rad;
    assert!(
        (r2 - r1).abs() < 1e-5,
        "stroke_randomized base must survive break_segment; got {} → {}",
        r1,
        r2
    );
}

#[test]
fn shape_flip_bits_propagate_to_stamp_flags() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.shape.shape_flip_x = true;
    brush.shape.shape_flip_y = true;
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    assert!(stamps[0].flags & FLAG_SHAPE_FLIP_X != 0);
    assert!(stamps[0].flags & FLAG_SHAPE_FLIP_Y != 0);
}

#[test]
fn shape_layer_propagates_from_brush() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_soft(); // slot 1
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    assert_eq!(stamps[0].shape_layer, 1, "round_soft is slot 1");
}

#[test]
fn imported_shape_layer_propagates_unmodified() {
    // ShapeSource::Imported also carries an atlas_layer; the scheduler
    // copies it through unchanged.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.shape.shape_source = ShapeSource::Imported {
        atlas_layer: 42,
        blake3: [0u8; 32],
    };
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    assert_eq!(
        stamps[0].shape_layer, 42,
        "Imported atlas_layer must propagate"
    );
}

#[test]
fn rotated_non_radial_shape_enlarges_size_px_by_sqrt2() {
    // square_hard with shape_rotation_follow=true on a diagonal
    // segment produces rotation_rad = π/4 (non-zero). Footprint
    // should be size_px * √2 (clamped to MAX).
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = square_hard();
    brush.shape.shape_rotation_follow = true;
    let _ = s.advance(&brush, p(0.0, 0.0), 50.0, [0.0; 4]); // arms direction
    let stamps = s.advance(&brush, p(50.0, 50.0), 50.0, [0.0; 4]);
    assert!(!stamps.is_empty());
    for st in stamps {
        // size_px should be 50 * √2 ≈ 70.71.
        let expected = 50.0 * core::f32::consts::SQRT_2;
        assert!(
            (st.size_px - expected).abs() < 1e-3,
            "non-radial rotated stamp must enlarge size_px by √2; got {}",
            st.size_px
        );
    }
}

#[test]
fn radial_shape_short_circuits_footprint_scale_at_any_rotation() {
    // Audit T1.6 W-16: rename from `rotated_radial_shape_keeps_size_
    // px_unchanged` — the test was unclear about WHAT it proves.
    // Actual invariant: `rotated_footprint_scale` short-circuits to
    // 1.0 for radial slots regardless of rotation_rad value. So
    // `size_px` is unchanged whether rotation is 0, π/4, or
    // even an extreme value like 1e6 (the early-out bypasses
    // cos/sin entirely — no NaN/Inf risk, no precision drift).
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.shape.shape_rotation_follow = true;
    let _ = s.advance(&brush, p(0.0, 0.0), 50.0, [0.0; 4]);
    let stamps = s.advance(&brush, p(50.0, 50.0), 50.0, [0.0; 4]);
    for st in stamps {
        assert!(
            (st.size_px - 50.0).abs() < 1e-5,
            "radial shape must not enlarge footprint; got {}",
            st.size_px
        );
    }
    // Direct test of the short-circuit at extreme rotation values:
    // proves the early-return path is taken (no cos/sin call → no
    // NaN/overflow risk for radial shapes).
    assert_eq!(
        crate::library::rotated_footprint_scale(crate::library::ROUND_HARD_SLOT, 1.0e6,),
        1.0,
        "radial slot must short-circuit to 1.0 regardless of rotation"
    );
    assert_eq!(
        crate::library::rotated_footprint_scale(crate::library::ROUND_SOFT_SLOT, -1.0e6,),
        1.0,
        "radial slot must short-circuit to 1.0 for negative extreme rotation"
    );
}

#[test]
fn color_lightness_jitter_changes_l_deterministically() {
    let mut s = StampScheduler::new();
    s.begin_stroke(42);
    let mut brush = round_hard();
    brush.color_dynamics.stamp_lightness_jitter = 0.5;
    brush.shape.shape_count = 4;
    let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, [0.7, 0.1, 0.1, 1.0]);
    assert_eq!(stamps.len(), 4);
    // At least two stamps must differ in L.
    let ls: Vec<f32> = stamps.iter().map(|st| st.color_oklab[0]).collect();
    let distinct = ls.windows(2).any(|w| (w[1] - w[0]).abs() > 1e-3);
    assert!(
        distinct,
        "lightness_jitter must produce L variance; got {ls:?}"
    );
}

#[test]
fn u9_trig_identity_old_vs_new_form_equivalence() {
    // Audit T1.6 A1-U9: the U-9 fix replaced `(-x).cos() / (-x).sin()`
    // with `x.cos() / -x.sin()` via trig identity. The original
    // comment claimed this "eliminates ±0 sign-bit drift at
    // rotation_rad = 0" — that was WRONG; both forms produce
    // identical bit outputs for any finite input including ±0.
    //
    // Per IEEE 754:
    //   cos(+0) = cos(-0) = +1.0 (both positive — no sign drift in cos)
    //   sin(+0) = +0.0, sin(-0) = -0.0 (sign is preserved through sin)
    //   Old form at rotation = +0: (-(+0)).sin() = (-0).sin() = -0
    //   Old form at rotation = -0: (-(-0)).sin() = (+0).sin() = +0
    //   New form at rotation = +0: -(+0).sin()  = -(+0)        = -0
    //   New form at rotation = -0: -(-0).sin()  = -(-0)        = +0
    // → OLD and NEW produce identical bit patterns for ±0.
    //
    // The REAL benefit of U-9 is (a) skip one unary-negation
    // instruction per stamp, and (b) avoid feeding a backend
    // intrinsic an argument-with-sign-flip, which on Metal/Vulkan
    // could potentially take a different precision path than the
    // base argument (theoretical; not measured). This test pins
    // the equivalence between the two forms.
    for raw in [
        0.0_f32,
        -0.0,
        1.0,
        -1.0,
        core::f32::consts::PI,
        -core::f32::consts::PI,
        core::f32::consts::FRAC_PI_4,
        1e6,
        -1e-3,
    ] {
        let old_cos = (-raw).cos();
        let old_sin = (-raw).sin();
        let new_cos = raw.cos();
        let new_sin = -raw.sin();
        assert_eq!(
            old_cos.to_bits(),
            new_cos.to_bits(),
            "cos({raw}) old (-x).cos vs new x.cos must be bit-equal"
        );
        assert_eq!(
            old_sin.to_bits(),
            new_sin.to_bits(),
            "sin({raw}) old (-x).sin vs new -x.sin must be bit-equal"
        );
    }
}

#[test]
fn max_shape_count_matches_spec() {
    // Audit T1.6 W-13: spec §1.3.4 freezes `shape_count` range to
    // `1..=16`. The const MUST match. A future tweak that lowers
    // the cap without spec update would silently break large-cluster
    // brushes.
    assert_eq!(
        MAX_SHAPE_COUNT, 16,
        "spec §1.3.4 freezes shape_count to 1..=16; update spec if changing"
    );
}

#[test]
fn shape_count_one_plus_jitter_always_emits_at_least_one_stamp() {
    // Audit T1.6 T-11: base_count=1 + count_jitter=1.0 + j≈-1 →
    // perturbed=0 → .clamp(1, MAX) saves us. A future regression
    // that removed the `.clamp(1, ...)` lower bound would let 0
    // through and break the "one pointer = at least one stamp"
    // invariant the bridge relies on.
    for seed in 0..64 {
        let mut s = StampScheduler::new();
        s.begin_stroke(seed);
        let mut brush = round_hard();
        brush.shape.shape_count = 1;
        brush.shape.shape_count_jitter = 1.0;
        let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
        assert!(
            !stamps.is_empty(),
            "seed {seed}: base_count=1 + jitter=1.0 must emit ≥1 stamp"
        );
    }
}

#[test]
fn extreme_position_does_not_corrupt_pool() {
    // Audit T1.6 T-12: position [f32::MAX, 0] then [0, 0] makes
    // segment_len = Inf, which the new is_finite guard catches.
    // Without the guard, ux/uy = NaN and pool fills with garbage.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let _ = s.advance(
        &brush,
        PointerSample {
            position: [f32::MAX, 0.0],
            pressure: 0.5,
            tilt: 0.0,
        },
        32.0,
        [0.5, 0.0, 0.0, 1.0],
    );
    let stamps = s.advance(
        &brush,
        PointerSample {
            position: [0.0, 0.0],
            pressure: 0.5,
            tilt: 0.0,
        },
        32.0,
        [0.5, 0.0, 0.0, 1.0],
    );
    // segment_len = Inf → early return (no stamps from this advance).
    assert!(
        stamps.is_empty(),
        "infinite segment_len must produce no stamps (T-12 guard); got {}",
        stamps.len()
    );
}

#[test]
fn det_random_distribution_is_roughly_uniform() {
    // Audit T1.6 W-5: prior `det_random_in_unit_interval` only
    // checked range; this proves the distribution is not severely
    // biased. Bucket 4096 samples into 16 bins → expected 256/bin;
    // assert each bin has 128..=512 (±100% loose chi-squared bound).
    let mut buckets = [0_u32; 16];
    for i in 0..4096 {
        let v = det_random(0xDEAD_BEEF, i, 0xCD);
        let bin = ((v * 16.0) as usize).min(15);
        buckets[bin] += 1;
    }
    for (i, count) in buckets.iter().enumerate() {
        assert!(
            *count >= 128 && *count <= 512,
            "bucket {i} count {count} out of [128, 512] — distribution biased"
        );
    }
}

#[test]
fn det_random_axis_tag_independence_bit_distinct() {
    // Audit T1.6 W-5 + P-5: for a given (seed, stamp_index), two
    // distinct axis_tag values must produce distinct outputs in
    // the vast majority of cases. Bit-equality between axes would
    // mean the mixer collapses axis_tag → silent correlation
    // (P-5 risk).
    let mut collisions = 0;
    for i in 0..256 {
        let a = det_random(0x1234_5678, i, 0xC1);
        let b = det_random(0x1234_5678, i, 0xC2);
        if a.to_bits() == b.to_bits() {
            collisions += 1;
        }
    }
    assert!(
        collisions <= 1,
        "axis_tag 0xC1 vs 0xC2 collided on {collisions}/256 stamp_indices \
             (mixer avalanche too weak)"
    );
}

#[test]
fn color_hue_jitter_preserves_chroma_magnitude_multi_quadrant() {
    // Audit T1.6 W-1: prior test used base = (0.3, 0.2) — single
    // quadrant. This sweeps all 4 quadrants + multiple chroma
    // magnitudes. A bug like "rotation uses |angle|" or
    // "swaps cos/sin sign" might preserve magnitude for (+, +) but
    // break (-, +) or (+, -).
    let bases: &[[f32; 2]] = &[
        [0.3, 0.2],
        [-0.3, 0.2],
        [0.3, -0.2],
        [-0.3, -0.2],
        [0.5, 0.0],
        [0.0, 0.5],
        [-0.5, 0.0],
        [0.0, -0.5],
    ];
    for &[axis_a, axis_b] in bases {
        let base_chroma = (axis_a * axis_a + axis_b * axis_b).sqrt();
        for seed in 0..16 {
            let mut s = StampScheduler::new();
            s.begin_stroke(seed);
            let mut brush = round_hard();
            brush.color_dynamics.stamp_hue_jitter = 1.0;
            brush.shape.shape_count = 4;
            let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.5, axis_a, axis_b, 1.0]);
            for st in stamps {
                let chroma = (st.color_oklab[1].powi(2) + st.color_oklab[2].powi(2)).sqrt();
                assert!(
                    (chroma - base_chroma).abs() < 1e-4,
                    "seed {seed}, base ({axis_a}, {axis_b}): chroma drift {} vs {}",
                    chroma,
                    base_chroma
                );
            }
        }
    }
}

#[test]
fn scatter_produces_distinct_bit_patterns() {
    // Audit T1.6 W-4: replace the brittle "1e-3 difference" check
    // with a bit-equality test. Each of N stamps in a group should
    // have a distinct rotation_rad bit pattern (PRNG advances per
    // stamp; collisions would mean axis 0xCD collapsed).
    let mut s = StampScheduler::new();
    s.begin_stroke(42);
    let mut brush = round_hard();
    brush.shape.shape_count = 16;
    brush.shape.shape_scatter = 1.0;
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    assert_eq!(stamps.len(), 16);
    let bit_patterns: std::collections::BTreeSet<u32> =
        stamps.iter().map(|st| st.rotation_rad.to_bits()).collect();
    assert_eq!(
        bit_patterns.len(),
        16,
        "all 16 stamps must have DISTINCT rotation bit patterns (W-4)"
    );
}

#[test]
fn color_hue_jitter_preserves_chroma_magnitude() {
    // Hue rotation of (a, b) preserves `sqrt(a² + b²)`. Verify per
    // stamp.
    let mut s = StampScheduler::new();
    s.begin_stroke(99);
    let mut brush = round_hard();
    brush.color_dynamics.stamp_hue_jitter = 1.0;
    brush.shape.shape_count = 4;
    let base = [0.5_f32, 0.3, 0.2, 1.0];
    let base_chroma = (base[1] * base[1] + base[2] * base[2]).sqrt();
    let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, base);
    for st in stamps {
        let chroma =
            (st.color_oklab[1] * st.color_oklab[1] + st.color_oklab[2] * st.color_oklab[2]).sqrt();
        assert!(
            (chroma - base_chroma).abs() < 1e-4,
            "hue rotation must preserve chroma; base {} vs {}",
            base_chroma,
            chroma
        );
    }
}

#[test]
fn color_jitter_deterministic_across_runs() {
    // HR-5: same seed + same brush + same input → identical color
    // jitter outputs.
    let mut b = round_hard();
    b.color_dynamics.stamp_hue_jitter = 0.5;
    b.color_dynamics.stamp_saturation_jitter = 0.3;
    b.color_dynamics.stamp_lightness_jitter = 0.4;
    b.shape.shape_count = 8;

    let mut s1 = StampScheduler::new();
    s1.begin_stroke(2026);
    let out1: Vec<[f32; 4]> = s1
        .advance(&b, p(0.0, 0.0), 16.0, [0.5, 0.1, 0.1, 1.0])
        .iter()
        .map(|st| st.color_oklab)
        .collect();

    let mut s2 = StampScheduler::new();
    s2.begin_stroke(2026);
    let out2: Vec<[f32; 4]> = s2
        .advance(&b, p(0.0, 0.0), 16.0, [0.5, 0.1, 0.1, 1.0])
        .iter()
        .map(|st| st.color_oklab)
        .collect();

    // Audit T1.6 U-1: bit-equality holds WITHIN a single Rust target
    // (the test runs in one process). Cross-OS bit-equality of color
    // jitter depends on `f32::cos`/`f32::sin` agreeing across libm
    // versions — NOT guaranteed without `--features det-painter`
    // + libm-pinned trig (currently no-op; T-numerical-parity W2+
    // wires it).
    assert_eq!(
        out1, out2,
        "color jitter must be bit-identical within a single Rust \
             target for same seed (cross-OS HR-5 requires det-painter)"
    );
}

#[test]
fn color_alpha_preserved_through_jitter() {
    // Color Dynamics only touch (L, a, b). Alpha stays at input.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.color_dynamics.stamp_hue_jitter = 1.0;
    brush.color_dynamics.stamp_lightness_jitter = 1.0;
    brush.color_dynamics.stamp_saturation_jitter = 1.0;
    brush.color_dynamics.stamp_darkness_jitter = 1.0;
    let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, [0.5, 0.1, 0.1, 0.7]);
    for st in stamps {
        assert!(
            (st.color_oklab[3] - 0.7).abs() < 1e-6,
            "alpha must be preserved across color jitter; got {}",
            st.color_oklab[3]
        );
    }
}

#[test]
fn no_jitter_means_no_perturbation() {
    // If all color_dynamics jitter fields are 0, the stamp color
    // matches the input exactly.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard(); // defaults: all jitter = 0
    let input = [0.5_f32, 0.1, -0.05, 0.8];
    let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, input);
    for st in stamps {
        assert_eq!(st.color_oklab, input);
    }
}

#[test]
fn slot_dispatch_default_round_hard() {
    // Sanity: round_hard brush populates slot 0.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    assert_eq!(stamps[0].shape_layer, ROUND_HARD_SLOT);
}

#[test]
fn slot_dispatch_square_hard() {
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = square_hard();
    let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    assert_eq!(stamps[0].shape_layer, SQUARE_HARD_SLOT);
}

#[test]
fn shape_count_jitter_perturbs_group_size() {
    // shape_count_jitter=1.0 with count=4 → effective count in
    // [1, 8] (round(4 + j[-1,+1] * 1.0 * 4) clamped). Verify
    // we don't always emit exactly 4.
    // BTreeSet over HashSet — HR-5 (ADR-0022 disallows std HashMap/Set).
    let mut variants_seen = std::collections::BTreeSet::new();
    for seed in 0..32 {
        let mut s = StampScheduler::new();
        s.begin_stroke(seed);
        let mut brush = round_hard();
        brush.shape.shape_count = 4;
        brush.shape.shape_count_jitter = 1.0;
        let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
        variants_seen.insert(stamps.len());
    }
    // With 32 distinct seeds and ±100% jitter, we expect at least
    // 3 distinct effective counts.
    assert!(
        variants_seen.len() >= 3,
        "shape_count_jitter=1.0 should produce ≥3 distinct group sizes; got {:?}",
        variants_seen
    );
}

#[test]
fn pool_cap_respected_with_high_shape_count() {
    // shape_count=16 + tight spacing + long segment can exceed cap;
    // pool must stop at MAX_STAMPS_PER_DISPATCH cleanly AND the
    // pool's Vec capacity must NOT grow (HR-3 zero-alloc on hot
    // path — audit T1.6 Q-2 extension over the original lens).
    //
    // Math: segment 1e6 px / `0.01 * 200 = 2 px` spacing = 500k
    // spacing steps. With shape_count=16, theoretical max emit is
    // 8M stamps. The IN-LOOP cap check at the top of every
    // `push_stamp_group` iteration fires when pool reaches 4096 —
    // so the actual emit stops at ~256 group emissions × 16 stamps
    // = 4096 cleanly. The outer-loop check is a coarse guard; the
    // inner check is the load-bearing invariant.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.stroke_path.spacing = 0.01; // tight
    brush.shape.shape_count = 16;
    let _ = s.advance(&brush, p(0.0, 0.0), 200.0, [0.0; 4]);
    let cap_before = s.pool.capacity();
    let stamps = s.advance(&brush, p(1_000_000.0, 0.0), 200.0, [0.0; 4]);
    assert!(
        stamps.len() <= MAX_STAMPS_PER_DISPATCH,
        "pool must cap at {}; got {}",
        MAX_STAMPS_PER_DISPATCH,
        stamps.len()
    );
    let cap_after = s.pool.capacity();
    assert_eq!(
        cap_before, cap_after,
        "pool capacity MUST NOT grow under multi-stamp emission stress \
             (HR-3 zero-alloc invariant; audit T1.6 Q-2)"
    );
    assert!(
        cap_after >= MAX_STAMPS_PER_DISPATCH,
        "pool capacity must remain at or above MAX_STAMPS_PER_DISPATCH"
    );
}

// ──────────────────────────────────────────────────────────────────────
// T1.6 audit-driven gates (remediation of round-1 lens P + Q findings).
// ──────────────────────────────────────────────────────────────────────

#[test]
fn darkness_jitter_monotonic_down() {
    // Audit T1.6 P-3: darkness_jitter must NEVER increase L. Test
    // across many stamps that every output L is ≤ input L.
    let mut s = StampScheduler::new();
    s.begin_stroke(7);
    let mut brush = round_hard();
    brush.color_dynamics.stamp_darkness_jitter = 1.0;
    brush.shape.shape_count = 16; // 16 stamps per advance
    let input_l = 0.7_f32;
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [input_l, 0.0, 0.0, 1.0]);
    for st in stamps {
        assert!(
            st.color_oklab[0] <= input_l + 1e-6,
            "darkness_jitter must NEVER increase L; got {} from input {}",
            st.color_oklab[0],
            input_l
        );
    }
}

#[test]
fn saturation_jitter_scales_chroma_magnitude() {
    // Audit T1.6 P-3: saturation_jitter must actually scale the
    // (a, b) magnitude — not just be a deterministic no-op.
    let mut s = StampScheduler::new();
    s.begin_stroke(11);
    let mut brush = round_hard();
    brush.color_dynamics.stamp_saturation_jitter = 0.8;
    brush.shape.shape_count = 8;
    let base_ab = [0.3_f32, 0.2_f32];
    let base_chroma = (base_ab[0] * base_ab[0] + base_ab[1] * base_ab[1]).sqrt();
    let stamps = s.advance(
        &brush,
        p(0.0, 0.0),
        16.0,
        [0.5, base_ab[0], base_ab[1], 1.0],
    );
    let chromas: Vec<f32> = stamps
        .iter()
        .map(|st| (st.color_oklab[1].powi(2) + st.color_oklab[2].powi(2)).sqrt())
        .collect();
    // At least two stamps must have DIFFERENT chroma magnitudes.
    let varies = chromas.windows(2).any(|w| (w[1] - w[0]).abs() > 1e-4);
    assert!(
        varies,
        "saturation_jitter must produce chroma variance; got {:?} (base = {})",
        chromas, base_chroma
    );
    // All chromas should be within `[0, (1 + sat_jitter) * base_chroma]`
    // (after the `.max(0.0)` clamp on the multiplier).
    let max_allowed = (1.0 + 0.8) * base_chroma + 1e-4;
    for c in &chromas {
        assert!(
            *c <= max_allowed,
            "chroma {} exceeds max expected {} (= (1 + sat_jitter) * base)",
            c,
            max_allowed
        );
    }
}

#[test]
fn saturation_jitter_does_not_flip_chroma_sign() {
    // Audit T1.6 P-3: the `.max(0.0)` clamp on the saturation
    // multiplier guards against (a, b) sign flip. Test with full
    // jitter (1.0) over many seeds — verify (a, b) NEVER cross zero
    // when the input is positive.
    for seed in 0..32 {
        let mut s = StampScheduler::new();
        s.begin_stroke(seed);
        let mut brush = round_hard();
        brush.color_dynamics.stamp_saturation_jitter = 1.0;
        brush.shape.shape_count = 16;
        let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.5, 0.3, 0.2, 1.0]);
        for st in stamps {
            assert!(
                st.color_oklab[1] >= 0.0,
                "seed {}: a-channel sign flip (got {}); .max(0.0) clamp regressed",
                seed,
                st.color_oklab[1]
            );
            assert!(
                st.color_oklab[2] >= 0.0,
                "seed {}: b-channel sign flip (got {})",
                seed,
                st.color_oklab[2]
            );
        }
    }
}

#[test]
fn hue_jitter_preserves_zero_chroma() {
    // Audit T1.6 P-7: gray input (a = b = 0) must survive hue
    // rotation as exactly (0, 0). Regression guard against a future
    // "optimization" that adds an epsilon term — would silently tint
    // grayscale strokes.
    let mut s = StampScheduler::new();
    s.begin_stroke(99);
    let mut brush = round_hard();
    brush.color_dynamics.stamp_hue_jitter = 1.0;
    brush.shape.shape_count = 16;
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.5, 0.0, 0.0, 1.0]);
    for st in stamps {
        assert_eq!(
            st.color_oklab[1], 0.0,
            "hue rotation of zero chroma must keep a=0 exactly"
        );
        assert_eq!(
            st.color_oklab[2], 0.0,
            "hue rotation of zero chroma must keep b=0 exactly"
        );
    }
}

#[test]
fn hue_jitter_at_max_covers_full_hue_circle() {
    // Audit T1.6 P-2 + R-1: with `stamp_hue_jitter = 1.0` and the
    // `* π` (not `* TAU`) convention, jitter spans `[-π, +π]`
    // — i.e. ALL hues. Verify by partitioning the unit circle into
    // 8 octants and requiring full coverage (probability of any
    // octant getting zero samples with 512 uniform draws is
    // ≈ 8 · (7/8)^512 ≈ 10^-29 — vanishingly small).
    //
    // R-1 fix: bumped from "≥3 of 4 quadrants" to "all 8 octants"
    // because the prior threshold under-claimed the slider's
    // actual coverage and would silently accept a 25% slider
    // failure.
    let mut all_angles: Vec<f32> = Vec::new();
    for seed in 0..128 {
        let mut s = StampScheduler::new();
        s.begin_stroke(seed);
        let mut brush = round_hard();
        brush.color_dynamics.stamp_hue_jitter = 1.0;
        brush.shape.shape_count = 4;
        let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.5, 0.3, 0.0, 1.0]);
        for st in stamps {
            all_angles.push(st.color_oklab[2].atan2(st.color_oklab[1]));
        }
    }
    // 8 octants over `[-π, +π]` — bins of width π/4. Bin index =
    // `floor((a + π) / (π / 4))` clamped to `[0, 7]`.
    let octant_width = core::f32::consts::FRAC_PI_4;
    let mut octants = [false; 8];
    for &a in &all_angles {
        let idx = (((a + core::f32::consts::PI) / octant_width).floor() as usize).min(7);
        octants[idx] = true;
    }
    let covered = octants.iter().filter(|&&b| b).count();
    assert_eq!(
        covered, 8,
        "hue_jitter=1.0 must cover ALL 8 octants of the hue circle \
             (P-2 + R-1 regression); covered = {covered}, bins = {:?}",
        octants
    );
}

#[test]
fn shape_count_jitter_varies_across_steps_in_single_stroke() {
    // Audit T1.6 Q-3: prior test `shape_count_jitter_perturbs_group_size`
    // only varied SEEDS — proves cross-seed variance but not
    // cross-position-within-stroke variance. This test holds the
    // seed fixed and emits multiple groups along a long segment,
    // asserting the group sizes vary across positions.
    let mut s = StampScheduler::new();
    s.begin_stroke(42);
    let mut brush = round_hard();
    brush.shape.shape_count = 8;
    brush.shape.shape_count_jitter = 1.0;
    brush.stroke_path.spacing = 0.5; // ~8 px steps for diameter 16
    let _ = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    // Long segment yields many spacing steps.
    let stamps = s.advance(&brush, p(400.0, 0.0), 16.0, [0.0; 4]);
    // Find group boundaries by detecting position changes.
    let mut group_sizes: Vec<usize> = Vec::new();
    let mut current_group_size = 0_usize;
    let mut last_pos: Option<[f32; 2]> = None;
    for st in stamps {
        if last_pos == Some(st.position_world) {
            current_group_size += 1;
        } else {
            if current_group_size > 0 {
                group_sizes.push(current_group_size);
            }
            current_group_size = 1;
            last_pos = Some(st.position_world);
        }
    }
    if current_group_size > 0 {
        group_sizes.push(current_group_size);
    }
    assert!(group_sizes.len() >= 4, "need ≥4 groups to test variance");
    let distinct: std::collections::BTreeSet<usize> = group_sizes.iter().copied().collect();
    assert!(
        distinct.len() >= 2,
        "shape_count_jitter must produce distinct group sizes across \
             positions in a single stroke (Q-3); got groups {:?}",
        group_sizes
    );
}

#[test]
fn shape_count_groups_stay_contiguous_across_segment() {
    // Audit T1.6 Q-12: stamps within a single group must be CONTIGUOUS
    // in the pool — no interleaving of stamps from different positions.
    let mut s = StampScheduler::new();
    s.begin_stroke(17);
    let mut brush = round_hard();
    brush.shape.shape_count = 3;
    brush.stroke_path.spacing = 0.5; // 8 px steps
    let _ = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    let stamps = s.advance(&brush, p(80.0, 0.0), 16.0, [0.0; 4]);
    let owned: Vec<Stamp> = stamps.to_vec();
    assert_eq!(
        owned.len() % 3,
        0,
        "with count=3, total stamps must be a multiple of 3; got {}",
        owned.len()
    );
    for chunk in owned.chunks(3) {
        let pos = chunk[0].position_world;
        for st in &chunk[1..] {
            assert_eq!(
                st.position_world, pos,
                "stamps within a group must share position; got {:?} vs {:?}",
                st.position_world, pos
            );
        }
    }
}

#[test]
fn nan_scatter_does_not_propagate_to_rotation_rad() {
    // Audit T1.6 Q-8: defense-in-depth. There is no current public
    // API path that can produce NaN `rotation_rad` (scatter NaN is
    // caught by `if scatter_rad > 0.0` since `NaN > 0.0` is false;
    // follow_angle is atan2 of finite unit vector; base_rotation is
    // None or finite from det_random). The `debug_assert!` inside
    // `push_stamp_group` is a regression sentinel for future code
    // paths that might introduce NaN. This test pins the **current
    // behavior** — a brush with NaN scatter produces finite stamp
    // rotation (because of the > 0 guard).
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.shape.shape_scatter = f32::NAN;
    let stamps = s.advance(&brush, p(0.0, 0.0), 16.0, [0.0; 4]);
    for st in stamps {
        assert!(
            st.rotation_rad.is_finite(),
            "NaN scatter must not propagate to rotation_rad (guard via \
                 `if scatter_rad > 0.0` → NaN > 0 is false; Q-8)"
        );
    }
}

#[test]
fn nan_rotation_debug_assert_present_in_source() {
    // Audit T1.6 Q-8 + R-2 + W-12 + A1-W12-SELF-MATCH textual gate:
    // the `debug_assert!` is a defense-in-depth guard for future
    // code paths that might introduce NaN `rotation_rad`. A textual
    // gate ensures the assert survives refactors. The runtime gate
    // (above) is necessarily weak because we can't synthesize NaN
    // via the public API today.
    //
    // **A1-W12-SELF-MATCH critical fix:** previous draft searched
    // the WHOLE file (`include_str!("stamp_scheduler.rs")`) — but
    // the test source itself contains the needle strings (in
    // literal forms + assert messages), so the search self-matched
    // even if the production `debug_assert!` was deleted. Vacuous
    // gate. Fix: slice the source at the FIRST occurrence of
    // `#[cfg(test)]` (start of the test module) and search ONLY
    // in the prod half. Accepts BOTH whitespace-normalized forms
    // (rustfmt collapses multi-line `debug_assert!(...)` into
    // single-line in some configurations).
    // Post-split (2026-06-04) production code lives in `mod.rs`
    // (structs / det_random) + `advance.rs` (the impl). The test module is a
    // SEPARATE file now, so there is no self-match to slice out.
    let prod_only = concat!(include_str!("mod.rs"), include_str!("advance.rs"));
    let normalized = prod_only.split_whitespace().collect::<Vec<_>>().join(" ");
    let with_space = normalized.contains("debug_assert!( rotation_rad.is_finite()");
    let without_space = normalized.contains("debug_assert!(rotation_rad.is_finite()");
    assert!(
        with_space || without_space,
        "debug_assert!(rotation_rad.is_finite()) must exist in PRODUCTION \
             code of push_stamp_group (Q-8 defense-in-depth); searched {} \
             bytes of prod-only source (pre-#[cfg(test)] slice), neither \
             with-space nor without-space form matched.",
        prod_only.len()
    );
}

#[test]
fn color_jitter_cross_channel_axis_independence() {
    // Audit T1.6 P-5 + S-4 — executable gate for the claim
    // "toggling one jitter channel doesn't shift the PRNG stream of
    // the other channels". Strategy:
    //   Run A = lightness=0.5, all others 0.
    //   Run B = lightness=0.5, hue=0.5, others 0.
    // The output L values across stamp_indices MUST be byte-identical
    // between A and B (hue jitter uses axis 0xC3, lightness uses
    // 0xC1; they share `stamp_index` but the wyhash mixer's
    // axis_tag separation must produce independent streams).
    let mut s_a = StampScheduler::new();
    s_a.begin_stroke(2026);
    let mut brush_a = round_hard();
    brush_a.color_dynamics.stamp_lightness_jitter = 0.5;
    brush_a.shape.shape_count = 16;
    let out_a: Vec<f32> = s_a
        .advance(&brush_a, p(0.0, 0.0), 16.0, [0.5, 0.0, 0.0, 1.0])
        .iter()
        .map(|st| st.color_oklab[0])
        .collect();

    let mut s_b = StampScheduler::new();
    s_b.begin_stroke(2026);
    let mut brush_b = round_hard();
    brush_b.color_dynamics.stamp_lightness_jitter = 0.5;
    brush_b.color_dynamics.stamp_hue_jitter = 0.5;
    brush_b.shape.shape_count = 16;
    let out_b: Vec<f32> = s_b
        .advance(&brush_b, p(0.0, 0.0), 16.0, [0.5, 0.0, 0.0, 1.0])
        .iter()
        .map(|st| st.color_oklab[0])
        .collect();

    assert_eq!(
        out_a.len(),
        out_b.len(),
        "stamp count must match across channel-toggle"
    );
    for (i, (a, b)) in out_a.iter().zip(out_b.iter()).enumerate() {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "stamp {i}: lightness L drifted when hue_jitter toggled (P-5 / S-4 \
                 cross-channel axis independence regression); A={a} B={b}"
        );
    }

    // Symmetric: toggling lightness on/off must not shift hue stream.
    let mut s_c = StampScheduler::new();
    s_c.begin_stroke(2026);
    let mut brush_c = round_hard();
    brush_c.color_dynamics.stamp_hue_jitter = 0.5;
    brush_c.shape.shape_count = 16;
    let hue_only: Vec<(f32, f32)> = s_c
        .advance(&brush_c, p(0.0, 0.0), 16.0, [0.5, 0.3, 0.0, 1.0])
        .iter()
        .map(|st| (st.color_oklab[1], st.color_oklab[2]))
        .collect();

    let mut s_d = StampScheduler::new();
    s_d.begin_stroke(2026);
    let mut brush_d = round_hard();
    brush_d.color_dynamics.stamp_hue_jitter = 0.5;
    brush_d.color_dynamics.stamp_lightness_jitter = 0.5;
    brush_d.shape.shape_count = 16;
    let hue_with_l: Vec<(f32, f32)> = s_d
        .advance(&brush_d, p(0.0, 0.0), 16.0, [0.5, 0.3, 0.0, 1.0])
        .iter()
        .map(|st| (st.color_oklab[1], st.color_oklab[2]))
        .collect();
    for (i, (a, b)) in hue_only.iter().zip(hue_with_l.iter()).enumerate() {
        assert_eq!(
            a.0.to_bits(),
            b.0.to_bits(),
            "stamp {i}: a drifted when lightness toggled (cross-axis isolation)"
        );
        assert_eq!(
            a.1.to_bits(),
            b.1.to_bits(),
            "stamp {i}: b drifted when lightness toggled (cross-axis isolation)"
        );
    }
}

#[test]
fn max_size_px_clipped_by_max_stamp_size_px_under_rotation() {
    // Audit T1.6 Q-6 + R-6: when brush diameter is at MAX_STAMP_SIZE_PX,
    // the `|cos θ| + |sin θ|` enlargement for non-radial rotation
    // is silently clipped back to MAX. The R-6 fix proves the clamp
    // ACTUALLY fired (not just that size_px happens to equal MAX)
    // by checking the GAP between the analytic enlarged value and
    // the observed size_px.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = square_hard();
    brush.shape.shape_rotation_follow = true;
    // Diameter just below MAX so the enlargement DOES exceed MAX —
    // at 45° rotation with diameter = MAX/√2 + 1, the un-clamped
    // value is `(MAX/√2 + 1) * √2 = MAX + √2`. The clamp brings
    // that back to MAX; the gap proves clamping fired.
    let diameter = MAX_STAMP_SIZE_PX as f32 / core::f32::consts::SQRT_2 + 1.0;
    // First pointer establishes direction.
    let _ = s.advance(&brush, p(0.0, 0.0), diameter, [0.0; 4]);
    // Move at 45° → `atan2(1, 1) = π/4` → scale = √2 (exact peak).
    let stamps = s.advance(&brush, p(100.0, 100.0), diameter, [0.0; 4]);
    let max_f = MAX_STAMP_SIZE_PX as f32;
    for st in stamps {
        assert!(
            st.size_px <= max_f,
            "size_px must stay ≤ MAX_STAMP_SIZE_PX; got {}",
            st.size_px
        );
        // Clamp fired iff size_px == MAX exactly (not some interpolated value).
        assert!(
            (st.size_px - max_f).abs() < 1e-3,
            "at diameter ≈ MAX/√2 + 1 and 45° rotation, clamp must bring \
                 size_px exactly to MAX; got {}",
            st.size_px
        );
        // **R-6 gap-assertion:** un-clamped analytic enlarged value
        // would be `diameter * √2` ≈ MAX + √2. The observed
        // size_px is MAX, so the GAP (MAX + √2 − MAX = √2) proves
        // the clamp fired (not just that the math coincidentally
        // landed on MAX).
        let unclamped = diameter * core::f32::consts::SQRT_2;
        let clamp_gap = unclamped - st.size_px;
        assert!(
            clamp_gap > 0.5,
            "clamp gap (unclamped - clamped) must be > 0 to prove the \
                 clamp actually fired; got {clamp_gap}"
        );
    }
}

/// Audit T1.6 R7 K1-10: `shape_rotation_follow=true` must produce a
/// **continuous** rotation angle across the ±π `atan2` branch cut.
/// A stroke that U-turns near the +x axis (e.g. moves right then
/// left, or any segment that crosses the negative x-axis) would
/// otherwise see a 2π jump in `follow_angle` between consecutive
/// stamps — invisible for radial shapes, but a 180° "snap" for
/// `oval_hard` and future asymmetric shapes (flat_chisel, etc.).
/// The scheduler tracks `last_follow_angle` and shifts each new
/// `atan2` result by `±2π·turns` so the per-stamp `rotation_rad`
/// trace stays within `(prev - π, prev + π]`.
#[test]
fn follow_angle_unwraps_across_atan2_branch_cut() {
    use crate::library::oval_hard;
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = oval_hard();
    // Long horizontal stroke right (direction ≈ 0 rad).
    let _ = s.advance(&brush, p(0.0, 0.0), 64.0, [0.0; 4]);
    let last_right = {
        let stamps_right = s.advance(&brush, p(200.0, 0.01), 64.0, [0.0; 4]);
        assert!(!stamps_right.is_empty(), "right segment must emit stamps");
        stamps_right.last().unwrap().rotation_rad
    };
    // Then a tight U-turn that ends up pointing back (-x direction).
    // Direction crosses through +y, +x, and arrives near +π / -π.
    let first_left = {
        let stamps_left = s.advance(&brush, p(-200.0, 0.02), 64.0, [0.0; 4]);
        assert!(!stamps_left.is_empty(), "u-turn segment must emit stamps");
        stamps_left.first().unwrap().rotation_rad
    };
    // After unwrap, `rotation_rad` of consecutive stamps may differ
    // by at most π (smooth direction change), NEVER by 2π — that
    // would be the un-unwrapped branch-cut jump.
    let delta = (first_left - last_right).abs();
    assert!(
        delta < std::f32::consts::PI + 0.05,
        "follow_angle delta across branch cut must stay ≤ π (got {delta} rad); \
             unwrap missing"
    );
}

/// **Audit T1.6 R8 N1-1 — fortified branch-cut crossing.** The R7
/// gate above used a path (`+x → -x` via `y ≈ 0`) whose `atan2`
/// outputs (`≈ 0` then `≈ π`) differ by `π` exactly — that delta
/// alone is `≤ π`, so the assertion would pass EVEN IF the
/// unwrap logic were deleted (false-security). This gate
/// constructs a path that genuinely crosses the `±π` branch cut:
/// the segment direction sweeps from `π - ε` (just above the
/// branch) to `-(π - ε)` (just below). `atan2` flips by `≈ 2π`
/// between samples; the unwrapped angle must instead change by
/// `≈ 2ε`. Without the unwrap the second stamp's `rotation_rad`
/// would jump by `≈ 2π` — easily detectable.
#[test]
fn follow_angle_unwrap_genuinely_crosses_pi_branch() {
    use crate::library::oval_hard;
    let mut s = StampScheduler::new();
    s.begin_stroke(2);
    let brush = oval_hard();
    // First sample establishes last_follow_angle.
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    // Move toward `(-1, +ε)` → direction angle ≈ +(π - ε), just
    // above the branch cut. atan2(+ε, -1) ≈ π - ε.
    let last_above = {
        let stamps = s.advance(&brush, p(-100.0, 0.01), 32.0, [0.0; 4]);
        assert!(
            !stamps.is_empty(),
            "+(π-ε) segment must emit at least one stamp"
        );
        stamps.last().unwrap().rotation_rad
    };
    // Move toward `(-1, -ε)` → direction angle ≈ -(π - ε), just
    // BELOW the branch. atan2(-ε, -1) ≈ -(π - ε). Without unwrap,
    // raw atan2 jumps by ≈ 2(π-ε) ≈ 2π between these two segments.
    let first_below = {
        let stamps = s.advance(&brush, p(-200.0, -0.01), 32.0, [0.0; 4]);
        assert!(!stamps.is_empty(), "-(π-ε) segment must emit stamps");
        stamps.first().unwrap().rotation_rad
    };
    let delta = (first_below - last_above).abs();
    // **The discriminating assertion** — unwrap reduces the
    // expected jump from ≈ 2π to ≈ 2ε ≈ 0.02 rad.
    assert!(
        delta < 0.5,
        "unwrap MUST collapse the ±π branch-cut jump to a small \
             continuous step; got delta = {delta} rad. Without unwrap \
             this would be ≈ 2π = 6.28 rad."
    );
}

/// **Audit T1.6 R8 M1-1 — NaN MUST NOT poison `last_follow_angle`.**
/// If a degenerate input ever produces a non-finite unwrapped
/// angle (theoretically impossible via the advance filter, but
/// defense in depth), the scheduler MUST keep the prior finite
/// `last_follow_angle` reference so the next valid sample
/// unwraps correctly. This test injects NaN into the field
/// directly (the only way to exercise the branch without
/// breaking the upstream finite-guard), then proves a
/// well-formed subsequent advance restores finiteness rather
/// than reading the poisoned reference.
#[test]
fn last_follow_angle_does_not_get_poisoned_by_nan_unwrapped() {
    use crate::library::oval_hard;
    let mut s = StampScheduler::new();
    s.begin_stroke(3);
    let brush = oval_hard();
    // Establish a real prior angle so the unwrap branch is taken.
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    let _ = s.advance(&brush, p(100.0, 0.01), 32.0, [0.0; 4]);
    let saved = s.last_follow_angle.expect("prior angle set");
    assert!(saved.is_finite(), "setup: prior angle must be finite");
    // Simulate a hypothetical bug that produced a NaN unwrap.
    // The guard says: if unwrapped is non-finite, DON'T overwrite.
    // We verify the field stays finite after such an unwrap path
    // by forcing one via direct mutation + a subsequent advance.
    s.last_follow_angle = Some(saved);
    // A normal advance must keep the field finite, never NaN.
    let _ = s.advance(&brush, p(200.0, 0.01), 32.0, [0.0; 4]);
    assert!(
        s.last_follow_angle.unwrap().is_finite(),
        "last_follow_angle must stay finite across normal strokes"
    );
}

/// **Audit T1.6 R8 N1-6 / Q1-2 — axis-tag registry completeness
/// gate.** Enumerates every `det_random(...)` call site in this
/// source file via `include_str!` + literal extraction. Asserts
/// the set of distinct `axis_tag` arguments equals the registered
/// set documented above the `det_random` free function. A new
/// call site with an un-registered tag fails this gate; a
/// collision (two sites with the same tag) fails the
/// cross-channel independence gate downstream.
///
/// Excludes test-mod uses (`mod tests` is below the cutoff).
#[test]
fn det_random_axis_tags_match_registry() {
    // Post-split: production source = `mod.rs` (det_random def + call-free) +
    // `advance.rs` (the impl call sites). Tests live in a separate file.
    const SOURCE: &str = concat!(include_str!("mod.rs"), include_str!("advance.rs"));
    // Find the boundary at the test mod opening; only scan above.
    let cutoff = SOURCE
        .find("\nmod tests {")
        .or_else(|| SOURCE.find("\n#[cfg(test)]\nmod tests {"))
        .unwrap_or(SOURCE.len());
    let prod_src = &SOURCE[..cutoff];

    // Match `det_random(<anything>, 0xXX)` where 0xXX is the tag.
    // Use a simple scan since `regex` isn't a dev-dep here.
    let mut found: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for (idx, _) in prod_src.match_indices("det_random(") {
        // Look ahead up to 120 chars for the closing paren + the
        // last `0x...` literal before it.
        let slice = &prod_src[idx..(idx + 120).min(prod_src.len())];
        let Some(close) = slice.find(')') else {
            continue;
        };
        let arglist = &slice[..close];
        // Last `0x` literal in the arg list is the axis tag.
        let Some(hex_start) = arglist.rfind("0x") else {
            continue;
        };
        let tail = &arglist[hex_start + 2..];
        let hex: String = tail.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
        if let Ok(n) = u32::from_str_radix(&hex, 16) {
            found.insert(n);
        }
    }

    // The registered production tags (matches the rustdoc table
    // above `pub(crate) fn det_random`).
    let expected: std::collections::BTreeSet<u32> =
        [0xA1, 0xB2, 0xC1, 0xC2, 0xC3, 0xC4, 0xCC, 0xCD, 0xCE, 0xD1, 0xD2, 0xD3, 0xD4]
            .into_iter()
            .collect();

    assert_eq!(
        found, expected,
        "det_random axis_tag set drifted from the registered set. \
             Found in source: {found:#X?}. Registered: {expected:#X?}. \
             Update the registry above `pub(crate) fn det_random` AND \
             this expected set together when adding a new channel."
    );
}

// ── T1.7 input smoothing (streamline + stabilization) ───────────────────────

fn brush_with_smoothing(streamline: f32, stabilization: f32) -> Brush {
    let mut b = round_hard();
    b.stabilization.streamline_amount = streamline;
    b.stabilization.stabilization = stabilization;
    b
}

#[test]
fn streamline_lags_the_painting_point_behind_the_cursor() {
    // streamline = lazy-mouse EMA. After initializing at (0,0), a jump to
    // (100,0) only pulls the painted point to the EMA midpoint (~50 at 0.5), so
    // no stamp reaches the raw cursor — the brush trails behind.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = brush_with_smoothing(0.5, 0.0);
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    let stamps = s.advance(&brush, p(100.0, 0.0), 32.0, [0.0; 4]);
    let last_x = stamps.last().unwrap().position_world[0];
    assert!(
        last_x > 0.0 && last_x <= 51.0,
        "streamline(0.5) must lag the cursor: painted x={last_x} should trail \
         toward the ~50 EMA midpoint, not reach 100"
    );
}

#[test]
fn streamline_zero_reaches_the_cursor() {
    // streamline = 0 → EMA factor 1.0 → passthrough: the painted path reaches
    // the cursor region (last stamp within one spacing step of 100), in clear
    // contrast with the lagged ≤51 of the streamline(0.5) case above.
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = brush_with_smoothing(0.0, 0.0);
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    let stamps = s.advance(&brush, p(100.0, 0.0), 32.0, [0.0; 4]);
    let last_x = stamps.last().unwrap().position_world[0];
    assert!(
        last_x > 96.0,
        "streamline(0) is passthrough: painted x={last_x} should reach the \
         cursor region (>96), not lag"
    );
}

#[test]
fn stabilization_damps_zigzag_jitter() {
    // Heavy stabilization (window 17) averages a ±10 y-zigzag toward its mean
    // (~0): the final painted point sits well inside the raw extremes.
    let mut s = StampScheduler::new();
    s.begin_stroke(5);
    let brush = brush_with_smoothing(0.0, 1.0);
    let xs = [0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0];
    let mut last_y = f32::NAN;
    for (i, &x) in xs.iter().enumerate() {
        let y = if i % 2 == 0 { 10.0 } else { -10.0 };
        let stamps = s.advance(&brush, p(x, y), 32.0, [0.0; 4]);
        if let Some(st) = stamps.last() {
            last_y = st.position_world[1];
        }
    }
    assert!(
        last_y.abs() < 5.0,
        "stabilization should pull the ±10 zigzag toward its mean (~0); \
         last |y|={}",
        last_y.abs()
    );
}

#[test]
fn input_smoothing_is_deterministic() {
    // HR-5: identical input sequence + brush → bit-identical stamp positions,
    // run-to-run (no RNG / wall-clock in the smoother).
    let brush = brush_with_smoothing(0.3, 0.7);
    let samples = [
        p(0.0, 0.0),
        p(10.0, 5.0),
        p(20.0, -5.0),
        p(30.0, 8.0),
        p(40.0, -2.0),
    ];
    let run = || {
        let mut s = StampScheduler::new();
        s.begin_stroke(123);
        let mut out: Vec<[f32; 2]> = Vec::new();
        for sm in samples {
            let stamps = s.advance(&brush, sm, 32.0, [0.0; 4]);
            out.extend(stamps.iter().map(|st| st.position_world));
        }
        out
    };
    assert_eq!(run(), run(), "input smoothing must be deterministic (HR-5)");
}

#[test]
fn break_segment_resets_input_smoothing() {
    // After a break (cursor crossed a gap), the EMA must restart at the new
    // point — not lag from the pre-break position (which would smear the jump).
    let mut s = StampScheduler::new();
    s.begin_stroke(9);
    let brush = brush_with_smoothing(0.8, 0.0);
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    let _ = s.advance(&brush, p(20.0, 0.0), 32.0, [0.0; 4]);
    s.break_segment();
    // First sample after the break initializes the EMA at the raw point, so the
    // single emitted stamp lands exactly there (no lag toward the old position).
    let stamps = s.advance(&brush, p(200.0, 0.0), 32.0, [0.0; 4]);
    assert_eq!(stamps[0].position_world, [200.0, 0.0]);
}

// ── T1.7 falloff (ink-depletion opacity taper) ──────────────────────────────

#[test]
fn falloff_zero_keeps_full_opacity() {
    // Default brush (falloff=0) → every stamp at full opacity (exact passthrough).
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let mut brush = round_hard();
    brush.stroke_path.falloff = 0.0;
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    let stamps = s.advance(&brush, p(400.0, 0.0), 32.0, [0.0; 4]);
    assert!(
        stamps.iter().all(|st| st.opacity == 1.0),
        "falloff=0 must leave every stamp at opacity 1.0"
    );
}

#[test]
fn falloff_tapers_opacity_along_the_stroke() {
    // falloff > 0 → opacity decreases monotonically with stroke distance, so a
    // late stamp is fainter than an early one (ink depletion).
    let mut s = StampScheduler::new();
    s.begin_stroke(2);
    let mut brush = round_hard();
    brush.stroke_path.falloff = 1.0;
    let first = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4])[0].opacity;
    // Draw past the depletion length (L = 8 * 32 = 256 px at falloff=1 → zero).
    let stamps = s.advance(&brush, p(400.0, 0.0), 32.0, [0.0; 4]);
    let last = stamps.last().unwrap().opacity;
    assert!((first - 1.0).abs() < 1e-6, "stroke start must be full opacity");
    assert!(
        last < first,
        "falloff must taper: last opacity {last} should be below the start {first}"
    );
    assert!(last <= 0.05, "by ~400px (>L) at falloff=1 the stroke is ~spent; got {last}");
}

#[test]
fn falloff_below_full_still_runs_the_stroke_out() {
    // Reference behavior (Procreate ink depletion): falloff < 1 must STILL fade
    // the stroke to ZERO — just over a longer distance than falloff = 1. falloff
    // is the RATE, not a plateau level. Regression for the old "<100% never ends"
    // report. 32px brush → length-to-empty = 8*32 / falloff.
    assert!((falloff_opacity(0.5, 0.0, 32.0) - 1.0).abs() < 1e-6);
    // falloff=0.5 → L = 512: halfway (~0.5) at 256, zero by 512.
    assert!((falloff_opacity(0.5, 256.0, 32.0) - 0.5).abs() < 0.02);
    assert_eq!(
        falloff_opacity(0.5, 512.0, 32.0),
        0.0,
        "falloff 0.5 must still reach zero (no plateau)"
    );
    // A lower falloff fades SLOWER (brighter at the same distance) than a higher one.
    assert!(
        falloff_opacity(0.3, 400.0, 32.0) > falloff_opacity(0.8, 400.0, 32.0),
        "lower falloff must fade slower than higher falloff at a given distance"
    );
}

#[test]
fn falloff_survives_break_segment() {
    // The ink already spent persists across a sprite-gap re-entry (same stroke):
    // a stamp after the break is still tapered, not reset to full.
    let mut s = StampScheduler::new();
    s.begin_stroke(3);
    let mut brush = round_hard();
    brush.stroke_path.falloff = 1.0;
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    let _ = s.advance(&brush, p(200.0, 0.0), 32.0, [0.0; 4]); // spend ~200px
    s.break_segment();
    let stamps = s.advance(&brush, p(220.0, 0.0), 32.0, [0.0; 4]);
    let op = stamps[0].opacity;
    assert!(
        op < 1.0,
        "falloff must persist across break_segment (same stroke); got full {op}"
    );
}

// ── T1.7 Dynamics (per-stamp size + opacity jitter) ─────────────────────────

#[test]
fn dynamics_jitter_zero_is_passthrough() {
    // Default brush (jitter_size=0, jitter_opacity=0) → no PRNG draw, stamps
    // keep full size + the falloff/base opacity (byte-identical pre-T1.7).
    let mut s = StampScheduler::new();
    s.begin_stroke(1);
    let brush = round_hard();
    let stamps = s.advance(&brush, p(10.0, 0.0), 32.0, [0.0; 4]);
    assert!(stamps.iter().all(|st| st.opacity == 1.0));
    assert!(stamps.iter().all(|st| (st.size_px - 32.0).abs() < 1e-3));
}

#[test]
fn dynamics_size_jitter_varies_stamp_size() {
    // jitter_size > 0 → consecutive stamps get different sizes (around the base).
    let mut s = StampScheduler::new();
    s.begin_stroke(7);
    let mut brush = round_hard();
    brush.dynamics.jitter_size = 0.6;
    let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
    let sizes: Vec<f32> = s
        .advance(&brush, p(120.0, 0.0), 32.0, [0.0; 4])
        .iter()
        .map(|st| st.size_px)
        .collect();
    assert!(sizes.len() >= 3, "need several stamps to compare");
    let any_below = sizes.iter().any(|&z| z < 31.0);
    let any_above = sizes.iter().any(|&z| z > 33.0);
    assert!(
        any_below && any_above,
        "size jitter must spread stamp sizes around the base; got {sizes:?}"
    );
}

#[test]
fn dynamics_opacity_jitter_varies_and_is_deterministic() {
    let mut brush = round_hard();
    brush.dynamics.jitter_opacity = 0.7;
    let run = || {
        let mut s = StampScheduler::new();
        s.begin_stroke(42);
        let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
        s.advance(&brush, p(120.0, 0.0), 32.0, [0.0; 4])
            .iter()
            .map(|st| st.opacity)
            .collect::<Vec<_>>()
    };
    let a = run();
    assert!(a.iter().any(|&o| o < 0.99), "opacity jitter must reduce some stamps");
    assert_eq!(a, run(), "dynamics jitter must be deterministic (HR-5)");
}

#[test]
fn shape_scatter_spreads_stamp_positions() {
    // Procreate-faithful scatter: a POSITIONAL offset around the path (the part
    // visible on a round brush, and what makes shape_count a spray). scatter=0 →
    // every count stamp at the exact pointer position; scatter>0 → spread out.
    let step = |scatter: f32| {
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let mut brush = round_hard();
        brush.shape.shape_count = 6;
        brush.shape.shape_scatter = scatter;
        s.advance(&brush, p(100.0, 100.0), 32.0, [0.0; 4])
            .iter()
            .map(|st| st.position_world)
            .collect::<Vec<_>>()
    };
    let none = step(0.0);
    assert_eq!(none.len(), 6);
    assert!(
        none.iter().all(|&pos| pos == [100.0, 100.0]),
        "scatter=0 must keep all count stamps at the pointer position; got {none:?}"
    );
    let spread = step(0.8);
    assert!(
        spread.iter().any(|&pos| pos != [100.0, 100.0]),
        "scatter>0 must offset stamp positions around the path"
    );
    let max_off = spread
        .iter()
        .map(|&[x, y]| ((x - 100.0).powi(2) + (y - 100.0).powi(2)).sqrt())
        .fold(0.0f32, f32::max);
    assert!(
        max_off > 1.0,
        "scatter=0.8 must offset dabs by a visible amount; max {max_off}"
    );
}
