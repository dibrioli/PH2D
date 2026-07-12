#![forbid(unsafe_code)]
//! `rig.rubber_hose` — the **rubber hose limb**: a chain that reaches its goal by bending
//! into one smooth curve, with **no elbow anywhere**. Motion Nodes M4, doc 42.
//!
//! The look is the oldest one in animation — the noodly arms and legs of the 1930s
//! (Fleischer, early Disney), revived by *Cuphead* and, in motion graphics, by the
//! reference tool for it: **Battle Axe's `RubberHose`** for After Effects. The whole point
//! of the style is that a limb has **no joint**: it is a hose. An IK solver gives you an
//! elbow; this one refuses to.
//!
//! ## The algorithm: constant curvature, found by bisection
//!
//! A chain where **every joint turns by the SAME angle `α`** traces a **circular arc** —
//! that is exactly what "no elbow" means, because curvature that is constant along the
//! limb has nowhere to be a corner. So the entire solve is one unknown:
//!
//! ```text
//! find α such that |endpoint(α) − root| = |goal − root|
//! then aim the whole arc at the goal
//! ```
//!
//! `|endpoint(α)|` shrinks monotonically from the full stretch (`α = 0`, a straight limb)
//! to zero (`α = 360° / bones`, the chain closed into a circle), so a **bisection** finds
//! the α that reaches the goal — 24 halvings, deterministic, and correct for **unequal
//! bone lengths** too (the endpoint is walked, not assumed to be the chord of a regular
//! polygon). Then the arc is rotated bodily so its endpoint lands on the goal:
//!
//! ```text
//! first bone's heading = heading(goal − root) − heading(endpoint(α))
//! ```
//!
//! Transcendental-free (HR-5): the parabolic `cos/sin` leaf and a bisection — no `asin`,
//! no closed form that would need one. **Out of reach → straight**, aimed at the goal.
//!
//! `flip` bends the hose the other way (its one aesthetic choice).
//!
//! **It writes a POSE** (the local angles), like every rig solver — see the `pose` leaf —
//! so the bones stay rigid, everything downstream composes, and `rig.skin_deformer` can
//! wrap flesh around it. An unconnected goal is a no-op, not "reach for the origin".
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod fk;
// The `pose` leaf is carried BYTE-IDENTICAL by every reaching solver. This one solves in
// angle space from the start, so it needs only the goal and the heading — not the
// position-to-pose conversion the other two lean on.
#[allow(dead_code)]
mod pose;
mod trig;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// A full turn in the degrees the columns speak.
const DEGREES_PER_TURN: f32 = 360.0;

/// Halvings of the bisection. 24 is far past `f32`'s ability to tell two angles apart —
/// the search is over a bounded interval, so this is a fixed, deterministic cost.
const BISECT_STEPS: usize = 24;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("rig.rubber_hose"),
    name: "rig.rubber_hose",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "target",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "flip",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// Where the chain's tip lands, relative to its root, when every joint turns `alpha`
/// degrees and the first bone points along `+x`.
fn endpoint(bones: &[f32], alpha: f32) -> [f32; 2] {
    let mut p = [0.0f32, 0.0];
    for (i, b) in bones.iter().enumerate() {
        let (cos, sin) = trig::cos_sin_cycles(i as f32 * alpha / DEGREES_PER_TURN);
        // Normalise: the parabolic pair is ~0.1 % off the unit circle, and an
        // un-normalised step would make this walk disagree with the FK that draws it.
        let inv = 1.0 / (cos * cos + sin * sin).sqrt();
        p = [p[0] + b * cos * inv, p[1] + b * sin * inv];
    }
    p
}

fn norm(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// The per-joint turn that puts the tip exactly `d` away from the root.
///
/// `|endpoint(α)|` falls monotonically from the full stretch to zero across
/// `α ∈ [0, 360/bones]`, so bisection is exact to `f32` in a fixed number of steps and can
/// neither diverge nor spin. A goal at (or past) full stretch gives `α = 0` — straight.
fn solve_alpha(bones: &[f32], d: f32) -> f32 {
    let full: f32 = bones.iter().sum();
    if d >= full {
        return 0.0; // out of reach: a hose cannot stretch, so it points
    }
    let (mut lo, mut hi) = (0.0f32, DEGREES_PER_TURN / bones.len() as f32);
    for _ in 0..BISECT_STEPS {
        let mid = 0.5 * (lo + hi);
        if norm(endpoint(bones, mid)) > d {
            lo = mid; // still too long → curl more
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Is this stream a single chain, joint `i` hanging off joint `i-1`?
fn is_chain(parent: &[f32]) -> bool {
    !parent.is_empty()
        && parent[0] < 0.0
        && parent
            .iter()
            .enumerate()
            .skip(1)
            .all(|(i, &pi)| pi as i32 == i as i32 - 1)
}

/// One evaluation: curl the hose onto the goal and hand back the pose.
fn reach(input: &Stream, target: &Stream, flip: bool) -> Stream {
    let n = input.count();
    let Some(goal) = pose::goal(target) else {
        return input.clone(); // nothing to reach for → keep the pose we had
    };
    let parent = fk::scalars(input, fk::PARENT, -1.0, n);
    if n < 3 || !is_chain(&parent) {
        return input.clone(); // fewer than two bones cannot be a hose → honest no-op
    }

    let len = fk::scalars(input, fk::LEN, 0.0, n);
    let bones = &len[1..n]; // the root has no bone into it
    let root = fk::positions(input)[0];
    let to_goal = [goal[0] - root[0], goal[1] - root[1]];

    let mut alpha = solve_alpha(bones, norm(to_goal));
    if flip {
        alpha = -alpha;
    }
    // Rotate the whole arc so its tip lands on the goal.
    let tip = endpoint(bones, alpha);
    let aim = pose::heading_deg(to_goal[0], to_goal[1]) - pose::heading_deg(tip[0], tip[1]);

    // The pose: the root aims the limb, the first bone adds nothing, every other joint
    // turns by the same α — one constant curvature, and so no elbow anywhere.
    let mut rot = vec![alpha; n];
    rot[0] = aim;
    rot[1] = 0.0;
    fk::resolve(&input.clone().with(fk::ROT, Column::Scalar(rot)))
}

struct RigRubberHose;

impl NodeOp for RigRubberHose {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let flip = ctx.param("flip");
        let out = reach(ctx.input(0), ctx.input(1), flip >= 0.5);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(RigRubberHose))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Rubber Hose",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "flip",
    label: "Flip Bend",
    min: 0.0,
    max: 1.0,
    step: 1.0,
    widget: ParamWidget::Toggle,
}];

#[cfg(test)]
mod tests {
    use super::*;

    const BONE: f32 = 1.0;
    const JOINTS: usize = 7; // 6 bones → a reach of 6

    /// A straight chain along `+x`, rooted at the origin — what `rig.skeleton` emits.
    fn hose(n: usize) -> Stream {
        fk::resolve(
            &Stream::new(n)
                .with(
                    fk::PARENT,
                    Column::Scalar((0..n).map(|i| i as f32 - 1.0).collect()),
                )
                .with(
                    fk::LEN,
                    Column::Scalar((0..n).map(|i| if i == 0 { 0.0 } else { BONE }).collect()),
                )
                .with(fk::ROT, Column::Scalar(vec![0.0; n]))
                .with("P", Column::Vec2(vec![[0.0, 0.0]; n])),
        )
    }

    fn point(x: f32, y: f32) -> Stream {
        Stream::new(1).with("P", Column::Vec2(vec![[x, y]]))
    }

    fn ps(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
        norm([a[0] - b[0], a[1] - b[1]])
    }

    /// The turn angle between two consecutive bones, **wrapped into (−180, 180]**.
    ///
    /// The wrap is the whole point: `heading` lives on a circle, so the difference of two
    /// headings straddling ±180° comes back as `−318°` where the turn is plainly `+42°`.
    /// Without the wrap this oracle reports an elbow that is not there — it did, on the
    /// first run, and the code it accused was correct.
    fn turns(p: &[[f32; 2]]) -> Vec<f32> {
        (2..p.len())
            .map(|i| {
                let a = pose::heading_deg(p[i - 1][0] - p[i - 2][0], p[i - 1][1] - p[i - 2][1]);
                let b = pose::heading_deg(p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]);
                let mut t = b - a;
                while t <= -180.0 {
                    t += 360.0;
                }
                while t > 180.0 {
                    t -= 360.0;
                }
                t
            })
            .collect()
    }

    /// **The tip lands on the goal, and there is NO ELBOW**: every joint turns by the same
    /// angle, so the limb is one arc of constant curvature.
    ///
    /// FALSIFIED by any IK solver: `ik_2bone` and `fabrik` both land the tip too, but they
    /// concentrate the bend where the geometry wants it — their turn angles are not all
    /// equal, and the limb has a corner. That is the whole difference between a jointed
    /// limb and a hose.
    #[test]
    fn the_tip_reaches_the_goal_and_the_curvature_is_constant() {
        for goal in [[3.0, 3.0], [-2.0, 1.0], [0.0, -4.0], [1.0, 0.5]] {
            let p = ps(&reach(&hose(JOINTS), &point(goal[0], goal[1]), false));
            let miss = dist(p[JOINTS - 1], goal);
            assert!(miss < 0.03, "goal {goal:?}: the tip missed by {miss}");

            let t = turns(&p);
            let (lo, hi) = (
                t.iter().copied().fold(f32::MAX, f32::min),
                t.iter().copied().fold(f32::MIN, f32::max),
            );
            assert!(
                hi - lo < 0.5,
                "goal {goal:?}: the turns are not all equal ({lo} .. {hi}) — that is an elbow"
            );
            assert!(hi.abs() > 1.0, "goal {goal:?}: the hose actually bends");

            for i in 1..JOINTS {
                assert!(
                    (dist(p[i], p[i - 1]) - BONE).abs() < 1e-3,
                    "bone {i} stretched"
                );
            }
            assert_eq!(p[0], [0.0, 0.0], "the root is nailed");
        }
    }

    /// Out of reach → straight, aimed at the goal. A hose cannot stretch.
    #[test]
    fn an_unreachable_goal_straightens_the_hose_toward_it() {
        let p = ps(&reach(&hose(JOINTS), &point(0.0, 40.0), false));
        for (i, q) in p.iter().enumerate() {
            assert!(q[0].abs() < 0.02, "joint {i} is off the line: {q:?}");
            assert!((q[1] - i as f32 * BONE).abs() < 0.02, "joint {i}: {q:?}");
        }
        assert_eq!(solve_alpha(&[1.0; 6], 6.0), 0.0, "at full stretch: no curl");
    }

    /// `flip` curls the hose the other way; the tip lands in the same place.
    #[test]
    fn flip_curls_the_other_way() {
        let goal = [2.0, 2.0];
        let a = ps(&reach(&hose(JOINTS), &point(goal[0], goal[1]), false));
        let b = ps(&reach(&hose(JOINTS), &point(goal[0], goal[1]), true));
        assert!(dist(a[JOINTS - 1], b[JOINTS - 1]) < 0.05, "same tip");
        assert!(
            dist(a[3], b[3]) > 0.5,
            "the belly of the hose swapped sides"
        );
        // Mirror images about the reach line: the two bellies bow to opposite sides.
        let cross = |q: [f32; 2]| goal[0] * q[1] - goal[1] * q[0];
        assert!(cross(a[3]) * cross(b[3]) < 0.0);
    }

    /// The bisection is monotone and total: it never spins, never diverges, and lands
    /// inside its bracket for every distance from a folded nothing to full stretch.
    #[test]
    fn the_bisection_covers_the_whole_range() {
        let bones = [1.0; 6];
        let mut last = f32::MAX;
        for k in 0..=12 {
            let d = k as f32 * 0.5; // 0 .. 6
            let a = solve_alpha(&bones, d);
            assert!(
                a.is_finite() && (0.0..=60.0).contains(&a),
                "alpha {a} at d {d}"
            );
            // The nearer the goal, the tighter the curl — strictly.
            assert!(a <= last + 1e-3, "curl must not loosen as the goal nears");
            last = a;
            if d > 0.0 && d < 6.0 {
                assert!(
                    (norm(endpoint(&bones, a)) - d).abs() < 0.01,
                    "the solved arc is {d} long"
                );
            }
        }
    }

    /// No goal, or too few bones to be a hose, is an honest no-op.
    #[test]
    fn no_goal_or_no_chain_is_a_no_op() {
        let before = ps(&hose(JOINTS));
        assert_eq!(ps(&reach(&hose(JOINTS), &Stream::new(0), false)), before);
        let cloud = Stream::new(3).with("P", Column::Vec2(vec![[1.0, 1.0]; 3]));
        assert_eq!(
            ps(&reach(&cloud, &point(5.0, 5.0), false)),
            vec![[1.0, 1.0]; 3]
        );
    }
}
