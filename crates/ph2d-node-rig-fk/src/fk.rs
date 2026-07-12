//! The rig's shared **column contract** and its **forward-kinematics resolve** —
//! the leaf both `rig.skeleton` and `rig.fk` carry (a 60-line copy beats a new
//! foundational crate for two consumers, [[project_brush_along_path_satellite_not_node]]).
//!
//! ## A skeleton is an ordinary instance stream (Motion Nodes M4.N3)
//!
//! The plan floated a `Domain::Rig` for this — which would have meant **unfreezing
//! the node contract**. It is not needed: an element IS a joint, and four ordinary
//! columns describe the chain.
//!
//! | column   | type   | meaning |
//! |----------|--------|---------|
//! | `parent` | Scalar | index of the joint this one hangs from; `< 0` = a **root** |
//! | `len`    | Scalar | length of the bone running from the parent INTO this joint |
//! | `rot`    | Scalar | the joint's **LOCAL** angle (degrees), relative to its parent |
//! | `P`      | Vec2   | the joint's **WORLD** position — *derived*, never authored |
//! | `wrot`   | Scalar | the joint's **WORLD** angle (degrees) — *derived* (skinning reads it) |
//!
//! So a skeleton flows on the SAME wires as everything else: every generic node
//! still works on it (`motion.move` shifts it, `motion.falloff` masks it, and the
//! **`Rotation` channel of `oscillator`/`wiggle`/`noise`/`step` poses it** — they
//! all just read and write columns). Rig is pure fan-out: **zero contract change**.
//!
//! ## Why `rot` is LOCAL and `P` is derived
//!
//! Because that is what makes a chain a chain: rotate one joint and everything
//! downstream of it swings — which only happens if the children's world pose is a
//! *function* of the parent's. Storing world angles per joint (KineFX's choice)
//! would make a generic modifier writing `rot` bend one joint and tear the limb
//! apart. Here a generic modifier writing `rot` poses the joint *locally*, and
//! [`resolve`] rebuilds the world pose — which is exactly what `rig.fk` is for.
//!
//! **The bones can therefore never stretch**: `|P[i] − P[parent]| == len[i]`, by
//! construction, whatever anyone did to `rot`.

use crate::trig;
use ph2d_nodegraph::attr::{Column, Stream};

pub(crate) const PARENT: &str = "parent";
pub(crate) const LEN: &str = "len";
pub(crate) const ROT: &str = "rot";
pub(crate) const WROT: &str = "wrot";

/// Degrees per turn — the `trig` leaf speaks cycles, the columns speak degrees
/// (the app's one authored-angle unit).
const DEGREES_PER_TURN: f32 = 360.0;

/// A Scalar column read to length `n` (absent / short → `default`).
pub(crate) fn scalars(s: &Stream, name: &str, default: f32, n: usize) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, default);
    v
}

/// Every element's position (absent → the origin).
pub(crate) fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) if v.len() == s.count() => v.clone(),
        _ => vec![[0.0, 0.0]; s.count()],
    }
}

/// **Forward kinematics**: rebuild `P` and `wrot` from (`parent`, `len`, `rot`).
///
/// A **root** (no parent) stays exactly where it is — it keeps the `P` it arrived
/// with, so a `motion.move` upstream still places the rig, and re-resolving never
/// drags the limb back to the origin. Every other joint hangs off its parent:
///
/// ```text
/// wrot[i] = wrot[parent] + rot[i]
/// P[i]    = P[parent] + len[i] · (cos wrot[i], sin wrot[i])
/// ```
///
/// **A stream with no `parent` column is all roots → every `P` survives untouched.**
/// That is the identity rule (doc 39): dropping `rig.fk` on a plain point cloud must
/// not move a single element.
///
/// Joints are assumed **topologically ordered** (a parent before its children) —
/// which every rig source emits. A forward reference (`parent >= i`) is treated as a
/// root rather than read as garbage: it cannot deadlock or read uninitialised state.
pub(crate) fn resolve(input: &Stream) -> Stream {
    let n = input.count();
    let parent = scalars(input, PARENT, -1.0, n);
    let len = scalars(input, LEN, 0.0, n);
    let rot = scalars(input, ROT, 0.0, n);
    let base = positions(input);

    let mut p = vec![[0.0f32; 2]; n];
    let mut w = vec![0.0f32; n];
    for i in 0..n {
        let pi = parent[i];
        // A finite, backward-pointing index is a parent; anything else is a root.
        let par = (pi >= 0.0 && pi.is_finite() && (pi as usize) < i).then_some(pi as usize);
        match par {
            None => {
                p[i] = base[i];
                w[i] = rot[i];
            }
            Some(j) => {
                w[i] = w[j] + rot[i];
                let (cos, sin) = trig::cos_sin_cycles(w[i] / DEGREES_PER_TURN);
                p[i] = [p[j][0] + len[i] * cos, p[j][1] + len[i] * sin];
            }
        }
    }

    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != "P" && name != WROT {
            out.set(name.clone(), col.clone());
        }
    }
    out.set("P", Column::Vec2(p));
    out.set(WROT, Column::Scalar(w));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A chain of `n` joints, each `len` long, each turned `rot` degrees from its
    /// parent. Joint 0 is the root: anchored at `root`, and pointing along `+x` (its
    /// `rot` is a WORLD angle, exactly as `rig.skeleton` publishes it).
    pub(crate) fn chain(n: usize, len: f32, rot: f32, root: [f32; 2]) -> Stream {
        let mut p = vec![[0.0, 0.0]; n];
        p[0] = root;
        Stream::new(n)
            .with(
                PARENT,
                Column::Scalar((0..n).map(|i| i as f32 - 1.0).collect()),
            )
            .with(
                LEN,
                Column::Scalar((0..n).map(|i| if i == 0 { 0.0 } else { len }).collect()),
            )
            .with(
                ROT,
                Column::Scalar((0..n).map(|i| if i == 0 { 0.0 } else { rot }).collect()),
            )
            .with("P", Column::Vec2(p))
    }

    fn ps(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    /// A straight chain lies along `+x`, spaced by the bone length, anchored at the
    /// root's own position — the root is NOT dragged to the origin.
    #[test]
    fn a_straight_chain_hangs_off_its_root_where_the_root_already_is() {
        let out = resolve(&chain(4, 1.0, 0.0, [5.0, 2.0]));
        assert_eq!(
            ps(&out),
            vec![[5.0, 2.0], [6.0, 2.0], [7.0, 2.0], [8.0, 2.0]]
        );
    }

    /// The local angles COMPOUND down the chain (that is what makes it a chain): a
    /// quarter turn per joint walks a square. FALSIFIED by treating `rot` as a world
    /// angle, which would fan the bones out from the root instead of curling them.
    #[test]
    fn local_angles_compound_so_the_chain_curls() {
        let out = resolve(&chain(4, 1.0, 90.0, [0.0, 0.0]));
        let p = ps(&out);
        // wrot: 90, 180, 270 → right-angle turns: up, left, down.
        let near =
            |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
        assert!(near(p[1], [0.0, 1.0]), "{:?}", p[1]);
        assert!(near(p[2], [-1.0, 1.0]), "{:?}", p[2]);
        assert!(near(p[3], [-1.0, 0.0]), "{:?}", p[3]);
    }

    /// **Bones never stretch**, no matter the pose — the invariant the whole
    /// representation rests on.
    #[test]
    fn every_bone_keeps_its_length_whatever_the_pose() {
        for rot in [0.0, 17.0, 90.0, -140.0, 400.0] {
            let p = ps(&resolve(&chain(6, 0.7, rot, [1.0, 1.0])));
            for i in 1..6 {
                let (dx, dy) = (p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]);
                let d = (dx * dx + dy * dy).sqrt();
                assert!((d - 0.7).abs() < 1e-3, "bone {i} at rot {rot} measured {d}");
            }
        }
    }

    /// Resolving twice changes nothing (the pose is a pure function of the columns),
    /// and a stream with NO `parent` column is all roots → every position survives.
    /// The second half is the identity rule: `rig.fk` on a point cloud is a no-op.
    #[test]
    fn resolve_is_idempotent_and_a_point_cloud_is_untouched() {
        let once = resolve(&chain(5, 1.0, 30.0, [0.0, 0.0]));
        assert_eq!(ps(&resolve(&once)), ps(&once));

        let cloud =
            Stream::new(3).with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]));
        assert_eq!(
            ps(&resolve(&cloud)),
            vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]],
            "a stream with no bones passes through untouched"
        );
    }

    /// A forward-referencing parent (a hand-authored / MCP-edited document) degrades
    /// to a root — no panic, no uninitialised read.
    #[test]
    fn a_forward_parent_reference_degrades_to_a_root() {
        let s = Stream::new(2)
            .with(PARENT, Column::Scalar(vec![1.0, -1.0])) // joint 0 points AHEAD
            .with(LEN, Column::Scalar(vec![9.0, 0.0]))
            .with(ROT, Column::Scalar(vec![0.0, 0.0]))
            .with("P", Column::Vec2(vec![[4.0, 4.0], [0.0, 0.0]]));
        assert_eq!(
            ps(&resolve(&s))[0],
            [4.0, 4.0],
            "treated as a root, kept put"
        );
    }
}
