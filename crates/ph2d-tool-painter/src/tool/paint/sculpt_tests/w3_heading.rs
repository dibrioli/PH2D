//! Gates for the CHISEL'S HEADING — the V follows the stroke (sibling of
//! [`super::w3`], split for the workspace file-LOC cap; same wave, same doc).
//!
//! The subject here is one sentence: *the groove runs parallel to the path the
//! artist drew*. The quarter-circle is the only fixture that can tell "follow
//! the heading live" from "lock it at the start"; the straight stroke is the
//! identity that proves the difference is the CURVE and not noise; and the
//! pixel-space groove-angle probe is the oracle that measures the artist's
//! sentence instead of mirroring the tool's intermediates.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

use super::w3::CHISEL;

/// A quarter-circle stroke — the only path that can tell the two Rake modes apart.
///
/// On a STRAIGHT stroke the heading never turns, so "follow it live" and "lock it at the start" are the same
/// instruction and produce the same groove, to the byte. That identity is a gate below, not an inconvenience:
/// it is what proves the difference measured here is the CURVE and not noise.
fn quarter_circle(cx: f32, cy: f32, r: f32) -> Vec<[f32; 2]> {
    (0..=24u8)
        .map(|i| {
            let t = f32::from(i) * (std::f32::consts::FRAC_PI_2 / 24.0);
            [cx + r * t.cos(), cy + r * t.sin()]
        })
        .collect()
}

/// Run a chisel stroke along `path` with Rake on or off, and hand back the relief **and the angle the knife
/// was holding at the last dab**.
///
/// The lock has to be read from INSIDE the gesture: the session dies at pen-up (`end_sculpt_session`), and
/// with it the lock — as it must, because the angle belongs to the stroke and the stroke is over. So the
/// walk is spelled out here rather than going through `drag`, which finishes the stroke.
fn chisel_run(size: u32, rake: bool, path: &[[f32; 2]]) -> (Vec<f32>, Option<[f32; 2]>) {
    let (mut t, layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, CHISEL, 0.5, 1.0);
    t.set_sculpt_offset(0.5); // −0.5 loads: a real bite
    t.set_sculpt_angle(0.8); // ~48°: a V steep enough to read
    assert!(t.sculpt_rake(), "fixture: Rake is meant to default ON");
    if !rake {
        t.toggle_sculpt_rake();
    }
    assert_eq!(t.sculpt_rake(), rake, "fixture: the toggle did not take");

    t.on_canvas_pointer(cp(path[0], PointerPhase::Down));
    for p in &path[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    let lock = t.paint.sculpt.locked_dir; // …while the gesture is still alive
    t.on_canvas_pointer(cp(*path.last().unwrap(), PointerPhase::Up));
    (heights_of(&t, layer), lock)
}

/// **Rake ON: the knife turns with the stroke, in real time. Rake OFF: it holds the angle it entered at.**
///
/// Enio's own words (2026-07-14): *"o tool identifica a direção do traço e mantém o ângulo … até o final.
/// Isso é bom e deve permanecer assim — mas esse é o modo onde Rake NÃO está checado. Com Rake checado … o
/// ângulo se ajusta em tempo real."*
///
/// Both modes read the stroke; the difference is **when**. So the gate drags a quarter-circle — where the
/// heading sweeps 90° — and asks that the two grooves are not the same groove. On a straight line they would
/// be identical, which is the sibling below.
///
/// **Mutations that must bleed** (checked): make `chisel_dir` ignore the lock (`return dir` always — Rake off
/// becomes Rake on); and set `locked_dir` on every dab batch instead of only the first, so a "locked" knife
/// silently re-aims at each pointer move.
///
/// A third one does **not** bleed, and knowing why is worth more than the mutation: deleting the
/// `if rake { return dir }` early-return — "make Rake on behave like Rake off" — changes nothing, because the
/// lock is only ever *populated* while Rake is OFF (`stamp_dabs_sculpt` guards on `!rake`). So
/// `locked_dir.unwrap_or(dir)` already IS `dir` when raking. Two independent things hold the live path up,
/// and the `lock_on.is_none()` assert above is what pins the second of them. (The mutation that *does* break
/// the live path is `if rake { return [1.0, 0.0] }` — a knife that ignores the stroke — and it takes
/// [`the_chisel_carves_a_crease`] down with it.)
#[test]
fn the_knife_turns_with_the_stroke_only_when_it_rakes() {
    let size = 200u32;
    let curve = quarter_circle(100.0, 60.0, 55.0);

    let (raked, lock_on) = chisel_run(size, true, &curve);
    let (locked, lock_off) = chisel_run(size, false, &curve);

    assert!(
        lock_on.is_none(),
        "a RAKING knife locked an angle ({lock_on:?}). It has no business remembering one — every dab reads \
         its own heading, which is what 'in real time' means."
    );
    let lock = lock_off.expect(
        "the un-raked knife never locked an angle, so it has nothing to hold and every dab falls back to \
         its own heading — which IS raking, wearing the other checkbox",
    );
    // …and it locked onto the direction the stroke SET OFF in (+y at the top of this quarter-circle), not
    // some later one. A lock that drifts is not a lock.
    assert!(
        lock[1] > 0.8 && lock[0].abs() < 0.4,
        "the knife locked onto {lock:?}, which is not the direction the stroke entered at (≈ +y). It is \
         holding an angle from the middle of the gesture — the artist set off one way and the tool aimed \
         another."
    );

    let differing = raked
        .iter()
        .zip(&locked)
        .filter(|(a, b)| (*a - *b).abs() > 1e-3)
        .count();
    assert!(
        differing > 400,
        "the raked and the locked knife cut the SAME groove on a quarter-circle ({differing} texels apart). \
         One is supposed to follow the curve and the other to hold the angle it started at; over 90° of \
         sweep those cannot agree."
    );
}

/// **On a straight stroke, Rake changes nothing — to the byte.**
///
/// The presence sibling of the gate above, and it is the one that gives it meaning. "Follow the heading" and
/// "hold the first heading" are the *same instruction* when the heading never changes, so if these two runs
/// differed the difference above would be measuring the mechanism (a stale lock, a lost heading), not the
/// curve. It also pins the promise the artist actually cares about: unticking Rake must not change how a
/// straight cut looks.
///
/// **Mutation that must bleed:** give the un-raked knife any axis that is not the stroke's — e.g. the brush's
/// `dab_angle_deg`, which is what the first cut of this checkbox did, and which Enio corrected.
#[test]
fn on_a_straight_stroke_the_rake_makes_no_difference_at_all() {
    let size = 200u32;
    let straight = [[60.0, 100.5], [100.0, 100.5], [140.0, 100.5]];

    let (raked, _) = chisel_run(size, true, &straight);
    let (locked, lock) = chisel_run(size, false, &straight);

    assert!(
        lock.is_some(),
        "fixture: even here the un-raked knife must LOCK a heading — otherwise this passes by both runs \
         falling through to the same fallback, and proves nothing"
    );
    // Below the product's OWN "this height did not move" threshold — not byte-identity, and the difference
    // between those two claims is a measurement rather than a preference. The engine's heading is a
    // length-weighted EMA that re-normalises every step, so on a straight run its unit vector wobbles in the
    // last bits; the locked copy is one particular sample of it. Measured, that wobble reaches **4.8e-7
    // loads** — a thousandth of `RELIEF_EPS`, which is the number the deposit itself uses to decide a texel
    // is unchanged. Demanding `to_bits()` equality here would be demanding that an EMA be idempotent, and
    // the gate would be pinning the arithmetic of the smoother instead of the behaviour of the knife.
    let worst = raked
        .iter()
        .zip(&locked)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < super::super::impasto_settle::RELIEF_EPS,
        "Rake changed a STRAIGHT cut by {worst:e} loads (the product's own no-change threshold is {:e}). The \
         heading never turns here, so following it and holding its first value are the same instruction — a \
         difference this size means the un-raked knife is aiming at something other than the stroke.",
        super::super::impasto_settle::RELIEF_EPS
    );
    // …and the fixture cut something, or the equality above is the equality of two no-ops.
    let cut = raked.iter().filter(|v| **v != 0.0).count();
    assert!(cut > 200, "fixture: the chisel removed nothing to compare");
}

/// The direction of the groove AT a point, read off the RENDERED relief.
///
/// A chisel cuts a V whose crease runs along the stroke, so from inside the trench the relief varies least
/// **along** the groove and most **across** it. Sweeping a probe line through the point and taking the
/// direction of least variation therefore recovers the groove's axis from the pixels themselves — no
/// knowledge of `Dab::dir`, of `chisel_dir`, or of any intermediate the tool computed.
///
/// That independence is the whole point: the gate above measures that two grooves DIFFER, which a knife
/// rotated to any wrong angle also satisfies. This measures the thing the artist actually sees.
fn groove_angle_deg(h: &[f32], size: u32, cx: usize, cy: usize, arm: usize) -> f32 {
    let at = |x: f32, y: f32| -> f32 {
        let (xi, yi) = (x.round() as i64, y.round() as i64);
        if xi < 0 || yi < 0 || xi >= i64::from(size) || yi >= i64::from(size) {
            return 0.0;
        }
        h[(yi as usize) * (size as usize) + xi as usize]
    };
    let (mut best, mut best_var) = (0.0f32, f32::MAX);
    // 1° steps over a half-turn: a groove has no head or tail, so θ and θ+180° are the same axis.
    for step in 0..180 {
        let th = (step as f32).to_radians();
        let (dx, dy) = (th.cos(), th.sin());
        let (mut sum, mut sum2, mut n) = (0.0f32, 0.0f32, 0.0f32);
        for k in -(arm as i32)..=(arm as i32) {
            let v = at(cx as f32 + dx * k as f32, cy as f32 + dy * k as f32);
            sum += v;
            sum2 += v * v;
            n += 1.0;
        }
        let var = sum2 / n - (sum / n) * (sum / n);
        if var < best_var {
            best_var = var;
            best = step as f32;
        }
    }
    best
}

/// **The groove ends up PARALLEL to the stroke — measured on the pixels, not on the tool's intermediates.**
///
/// This is the gate that was missing while the defect lived, and the reason it lived: its neighbour
/// `the_knife_turns_with_the_stroke_only_when_it_rakes` asserts only that the raked and locked grooves
/// **differ**, which is equally true of a knife rotated to the wrong angle. Enio reported exactly that
/// (*"Rake não consegue rotacionar o brush para que a vala seja sempre paralela à direção do traço"*) with
/// the whole Chisel suite green: the axis was the SMOOTHED tangent (`Dab::dir`, which the Rake texture
/// rides and wants), and a smoothed tangent trails — measured up to **52° behind at a 60-px brush**,
/// because the smoothing window scales with the radius.
///
/// So this reads the groove's direction out of the RENDERED height and compares it to the path's own
/// tangent. It knows nothing about `chisel_dir`, and that is what makes it an oracle rather than a mirror.
///
/// **Mutation that must bleed:** take the axis from `d.dir` again in `stamp_dabs_sculpt` (i.e. undo
/// `path_axis`) — the lag returns and the assert names the angle.
#[test]
fn the_raked_groove_runs_parallel_to_the_stroke() {
    let size = 200u32;
    let curve = quarter_circle(100.0, 60.0, 55.0);
    let (raked, _) = chisel_run(size, true, &curve);

    // Sample WELL INSIDE the run: the first dabs are the heading warm-up and the last is the pen-up.
    let mut worst = 0.0f32;
    for k in [8usize, 12, 16] {
        let p = curve[k];
        let (prev, next) = (curve[k - 1], curve[k + 1]);
        let want = (next[1] - prev[1]).atan2(next[0] - prev[0]).to_degrees();
        let got = groove_angle_deg(&raked, size, p[0] as usize, p[1] as usize, 7);
        // Both are AXES, not directions: fold the difference into [0, 90].
        let mut d = (want - got).rem_euclid(180.0);
        if d > 90.0 {
            d = 180.0 - d;
        }
        assert!(
            d < 12.0,
            "at station {k} the stroke runs at {want:.1}° and the groove at {got:.1}° — {d:.1}° apart. \
             The V is folded about the stroke's axis, so a groove that is not parallel to the path is a \
             knife pointing somewhere the artist never pointed it. (`Dab::dir` is the SMOOTHED tangent \
             and trails by roughly the brush radius; the axis has to come from the dab CENTRES.)"
        );
        worst = worst.max(d);
    }
    assert!(
        worst > 0.0,
        "fixture: every station measured exactly 0° off, which a groove read from real pixels does not \
         do — the probe is reading a constant, not the relief"
    );
}
