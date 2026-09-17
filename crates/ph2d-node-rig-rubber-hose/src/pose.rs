//! Turning solved POSITIONS back into a **pose** — the leaf both reaching solvers
//! carry (`rig.ik_2bone`, `rig.fabrik`). Sibling of the `fk` leaf, which stays
//! byte-identical across all four `rig.*` crates; the solver-only helpers live here.
//!
//! ## Why a solver never writes `P`
//!
//! An IK solver naturally produces **positions** (where the elbow ended up). Writing
//! them straight into `P` would work exactly once — and then break everything: the
//! skeleton's truth is its **angles** (doc 40), so a chain whose `P` disagrees with its
//! `rot` is torn. The next `rig.fk`, the next solver, the skinning — all of them read
//! the angles and would snap the limb back.
//!
//! So a solver **proposes positions, converts them to LOCAL angles here, and lets
//! `fk::resolve` draw them**. Three things fall out for free:
//!
//! - **the bones stay exactly rigid** (FK builds them from `len`, not from the solver's
//!   arithmetic, so no iteration can stretch a limb);
//! - **joints below the solved span follow** (a hand dragged by IK carries its fingers);
//! - **solvers compose** — IK, then another IK, then a wave, in any order.
//!
//! The price is one round trip through the approximate `atan2` / `cos-sin` (HR-5): the
//! end effector lands within ~0.1 % of the chain's length of the goal, which is well
//! under a pixel at any sane scale. Exactness of the *pose* is worth more than exactness
//! of one point.

use crate::fk;
use ph2d_nodegraph::attr::Stream;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

/// `180/π` — radians to the degrees every angle column speaks (a constant, not a call).
const RAD_TO_DEG: f32 = 57.295_78;

/// `atan2(y, x)` in radians, transcendental-free (Rajan rational approximation of
/// `atan` on `[0,1]`, folded across the eight octants). ~0.0015 rad error, only
/// multiply/add/compare (HR-5). Copied leaf — `motion.look_at` carries the same one.
fn atan2_approx(y: f32, x: f32) -> f32 {
    let (ax, ay) = (x.abs(), y.abs());
    let hi = ax.max(ay);
    if hi == 0.0 {
        return 0.0;
    }
    let a = ax.min(ay) / hi;
    let mut r = FRAC_PI_4 * a - a * (a - 1.0) * (0.2447 + 0.0663 * a);
    if ay > ax {
        r = FRAC_PI_2 - r;
    }
    if x < 0.0 {
        r = PI - r;
    }
    if y < 0.0 {
        r = -r;
    }
    r
}

/// The world heading of a vector, in degrees.
pub(crate) fn heading_deg(dx: f32, dy: f32) -> f32 {
    atan2_approx(dy, dx) * RAD_TO_DEG
}

/// The goal a solver reaches for: the **first element** of the `target` stream. An
/// unconnected port cooks to an empty stream → `None` → the solver is a no-op (a limb
/// with nothing to reach for keeps the pose it had; it does not collapse to the origin).
pub(crate) fn goal(target: &Stream) -> Option<[f32; 2]> {
    (target.count() > 0).then(|| fk::positions(target)[0])
}

/// `v`, normalised — or `fallback` when it has no length (a degenerate configuration
/// must pick a direction, not divide by zero).
pub(crate) fn unit(v: [f32; 2], fallback: [f32; 2]) -> [f32; 2] {
    let d = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if d > f32::EPSILON {
        [v[0] / d, v[1] / d]
    } else {
        fallback
    }
}

/// Rewrite the LOCAL angles of `joints` (ascending, each after its parent) so the chain
/// passes through `solved`. Every other joint keeps the angle it was authored with.
///
/// A solved joint's world heading comes from the solved positions; an untouched one's
/// comes from the `wrot` the last resolve published — so the span the solver moved is
/// stitched onto the pose around it without disturbing it.
pub(crate) fn relocal(input: &Stream, solved: &[[f32; 2]], joints: &[usize]) -> Vec<f32> {
    let n = input.count();
    let parent = fk::scalars(input, fk::PARENT, -1.0, n);
    let mut rot = fk::scalars(input, fk::ROT, 0.0, n);
    let mut world = fk::scalars(input, fk::WROT, 0.0, n);

    for &i in joints {
        let pi = parent[i];
        let Some(j) = (pi >= 0.0 && pi.is_finite() && (pi as usize) < i).then_some(pi as usize)
        else {
            continue; // a root has no bone, so it has no angle to solve for
        };
        world[i] = heading_deg(solved[i][0] - solved[j][0], solved[i][1] - solved[j][1]);
        rot[i] = world[i] - world[j];
    }
    rot
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::Column;

    #[test]
    fn the_heading_is_degrees_and_covers_every_quadrant() {
        for (dx, dy, want) in [
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 90.0),
            (-1.0, 0.0, 180.0),
            (0.0, -1.0, -90.0),
            (1.0, 1.0, 45.0),
            (-1.0, -1.0, -135.0),
        ] {
            let got = heading_deg(dx, dy);
            assert!(
                (got - want).abs() < 0.2,
                "({dx},{dy}) -> {got}, want {want}"
            );
        }
        assert_eq!(heading_deg(0.0, 0.0), 0.0, "no direction, no heading");
    }

    /// An unconnected target cooks to an empty stream — the solver must see "no goal",
    /// not "the goal is the origin" (which would yank every limb to the middle).
    #[test]
    fn an_unconnected_target_is_no_goal_at_all() {
        assert_eq!(goal(&Stream::new(0)), None);
        let t = Stream::new(2).with("P", Column::Vec2(vec![[3.0, 4.0], [9.0, 9.0]]));
        assert_eq!(goal(&t), Some([3.0, 4.0]), "the FIRST element is the goal");
    }

    #[test]
    fn unit_falls_back_instead_of_dividing_by_zero() {
        assert_eq!(unit([2.0, 0.0], [0.0, 1.0]), [1.0, 0.0]);
        assert_eq!(unit([0.0, 0.0], [0.0, 1.0]), [0.0, 1.0]);
    }
}

/// The column a constraint reads as its **strength** — the catalogue's existing mask channel.
///
/// ⭐⭐ It is deliberately **not** a new name: `falloff` is what `motion.falloff` writes and what
/// the whole `field.*` family produces, **with a canvas gizmo**. So *"an IK whose influence fades
/// with the distance from a box the artist drags on screen"* is a WIRE here, where Rive and Spine
/// have a single keyed number per constraint. That is the `SUPERAR:` item of conference sheet 16,
/// paid for by spending nothing.
pub(crate) const FALLOFF: &str = "falloff";

/// Blend a solved pose back towards the one that came IN, joint by joint, by [`FALLOFF`] — the
/// `Strength` of every Rive constraint and the `Mix` of Spine's.
///
/// ## Angles, never positions
///
/// The blend is on the **local angles**, which is the only place it is legitimate: mixing the
/// solved `P` with the incoming `P` would produce a chain whose positions disagree with its
/// angles, which is precisely the torn limb this whole leaf exists to prevent (see the module
/// docs). Conference sheet 16 reaches the same conclusion from the other side — it is why
/// `motion.mixer` cannot serve as a strength.
///
/// ## Absent is `1.0`, and the FORM is what makes it exact
///
/// A missing column means full effect (the convention `motion.scale` documents), so a graph that
/// never heard of falloff gets today's solve **bit for bit**.
///
/// ⚠️ That exactness is a property of the expression, not a hope: `a·(1−t) + b·t` at `t = 1` is
/// `a·0 + b·1 = b` **exactly**, while the textbook `a + (b−a)·t` is `a + (b−a)`, which rounds
/// whenever `a` and `b` are far apart. *One of the two forms silently moves every rig in the repo.*
///
/// ⚠️ `t` is clamped to `0..1`: a strength is a mix, and a field handing out `2.0` must not make a
/// constraint overshoot its own solve.
pub(crate) fn mix_by_falloff(input: &Stream, solved: Vec<f32>) -> Vec<f32> {
    let n = solved.len();
    let t = fk::scalars(input, FALLOFF, 1.0, n);
    // ⚠️ This is a SHORTCUT, not the guarantee. The first draft of this comment said the
    // short-circuit was what kept full strength byte-exact, and a mutation refuted it: delete
    // these three lines and all three ladder gates still pass, because `a·(1−t) + b·t` at
    // `t = 1` is already exact. *A line a mutation cannot kill is a comment with the syntax of
    // code* — so this one keeps the name it earns: with no column (the common case) it skips an
    // allocation and a whole pass, and nothing else.
    if t.iter().all(|&w| w >= 1.0) {
        return solved;
    }
    let was = fk::scalars(input, fk::ROT, 0.0, n);
    solved
        .iter()
        .zip(&was)
        .zip(&t)
        .map(|((&b, &a), &w)| {
            let w = if w.is_finite() {
                w.clamp(0.0, 1.0)
            } else {
                1.0
            };
            a * (1.0 - w) + b * w
        })
        .collect()
}
