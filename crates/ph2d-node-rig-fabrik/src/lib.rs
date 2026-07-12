#![forbid(unsafe_code)]
//! `rig.fabrik` — **FABRIK**: *Forward And Backward Reaching Inverse Kinematics*
//! (Aristidou & Lasenby, 2011). The reaching solver for a chain of ANY length — a
//! tentacle, a tail, a spine, a whip — where the law of cosines (`rig.ik_2bone`) only
//! ever solves three joints. Motion Nodes M4, doc 41.
//!
//! ## The algorithm, and why it is the gold standard
//!
//! Every other N-bone solver (CCD, Jacobian, damped least squares) works in **angle**
//! space and pays for it — CCD curls the tip joint first and gives you a fishhook,
//! Jacobian methods invert a matrix per iteration and fall apart at singularities.
//! FABRIK works in **position** space and does no linear algebra at all: it just walks
//! the chain twice per iteration, dropping each joint onto the line to its neighbour at
//! the right distance.
//!
//! ```text
//! backward:  put the tip ON the goal, then walk to the root, each joint pulled to
//!            bone-length along the line toward the joint after it
//! forward:   put the root BACK on its anchor, then walk to the tip, each joint pulled
//!            to bone-length along the line toward the joint before it
//! ```
//!
//! Both passes preserve the bone lengths exactly, so the only thing left to converge is
//! the root's anchor and the tip's goal — which it does in a handful of iterations, with
//! visibly natural, smooth poses (no fishhook). Only `sqrt` (the normalisations) — so it
//! is transcendental-free by nature (HR-5), not by approximation.
//!
//! **Out of reach → fully extended toward the goal**, which the algorithm produces on its
//! own (every joint ends up on the straight line to a goal it cannot touch). No stretch,
//! no `NaN`.
//!
//! **It solves for a POSE, not for positions** — the solved positions are converted to
//! local angles and drawn by `fk::resolve` (see the `pose` leaf), so the bones are rigid
//! by construction and the result composes with every other rig node.
//!
//! An **unconnected goal** is not the origin: with no target the node is a no-op.
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod fk;
mod pose;
mod trig;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Hard ceiling on the iteration count (an untrusted `f32` param × a chain walk).
const MAX_ITERATIONS: usize = 32;

/// Close enough to the goal to stop early — a fraction of a pixel at any sane scale.
const TOLERANCE: f32 = 1e-4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("rig.fabrik"),
    name: "rig.fabrik",
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
        name: "iterations",
        default: 10.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// How far off the line to bow a collinear chain, as a fraction of its reach — big enough
/// to escape the degenerate basin, small enough that the first iteration erases any trace
/// of it.
const BOW: f32 = 0.02;

/// **Break the collinear tie.** FABRIK has NO gradient when the root, every joint and the
/// goal lie on ONE LINE: the backward pass drags the chain along that line and the forward
/// pass pushes it straight back, so a straight limb can never FOLD to reach a goal nearer
/// than its full extension — it stalls at full stretch, missing by exactly the slack.
///
/// The paper never meets this, because its chains carry the PREVIOUS FRAME's pose and are
/// therefore never exactly straight. This node is `Pure` (no `pre` loop, by design) and
/// starts from `rig.skeleton`'s rest pose — which is exactly straight. So the degeneracy
/// is not an edge case here, it is the DEFAULT whenever the goal happens to line up with
/// the limb's rest direction.
///
/// The fix is the standard one: bow the interior joints a hair off the line — deterministic
/// (HR-5), and only when the chain really is collinear AND the goal really is nearer than
/// full stretch (a goal at or beyond full reach wants the straight answer, and gets it).
fn break_collinearity(p: &mut [[f32; 2]], len: &[f32], goal: [f32; 2]) {
    let n = p.len();
    let reach: f32 = len[1..n].iter().sum();
    let to_goal = [goal[0] - p[0][0], goal[1] - p[0][1]];
    let d = (to_goal[0] * to_goal[0] + to_goal[1] * to_goal[1]).sqrt();
    if reach - d < BOW * reach {
        return; // the goal is at (or past) full stretch: straight IS the answer
    }
    let u = pose::unit(to_goal, [1.0, 0.0]);
    // How far the chain currently strays from the root→goal line.
    let bend = (1..n)
        .map(|i| {
            let v = [p[i][0] - p[0][0], p[i][1] - p[0][1]];
            (v[0] * u[1] - v[1] * u[0]).abs()
        })
        .fold(0.0, f32::max);
    if bend > BOW * reach {
        return; // already off the line — the iteration has a direction to fall in
    }
    let perp = [-u[1], u[0]];
    let bow = BOW * reach;
    for q in p.iter_mut().take(n - 1).skip(1) {
        q[0] += perp[0] * bow;
        q[1] += perp[1] * bow;
    }
}

/// One FABRIK solve: the chain's world positions, reaching for `goal` from its anchored
/// root. `len[i]` is the bone from joint `i-1` into joint `i`.
fn fabrik(p: &mut [[f32; 2]], len: &[f32], goal: [f32; 2], iterations: usize) {
    let n = p.len();
    if n < 2 {
        return;
    }
    break_collinearity(p, len, goal);
    let anchor = p[0];
    for _ in 0..iterations {
        // BACKWARD: the tip takes the goal, and the chain is dragged after it.
        p[n - 1] = goal;
        for i in (0..n - 1).rev() {
            let d = [p[i][0] - p[i + 1][0], p[i][1] - p[i + 1][1]];
            let u = pose::unit(d, [1.0, 0.0]);
            p[i] = [
                p[i + 1][0] + len[i + 1] * u[0],
                p[i + 1][1] + len[i + 1] * u[1],
            ];
        }
        // FORWARD: the root goes back where it is nailed, and the chain follows it home.
        p[0] = anchor;
        for i in 1..n {
            let d = [p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]];
            let u = pose::unit(d, [1.0, 0.0]);
            p[i] = [p[i - 1][0] + len[i] * u[0], p[i - 1][1] + len[i] * u[1]];
        }
        let miss = [p[n - 1][0] - goal[0], p[n - 1][1] - goal[1]];
        if (miss[0] * miss[0] + miss[1] * miss[1]).sqrt() < TOLERANCE {
            break; // the goal is in reach and we are on it
        }
    }
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

/// Clamp the iteration count: junk → 1 (a solver that runs zero passes is a silent
/// no-op, which reads as a broken node).
fn iteration_count(iterations: f32) -> usize {
    if iterations.is_finite() && iterations >= 1.0 {
        (iterations.round() as usize).min(MAX_ITERATIONS)
    } else {
        1
    }
}

/// One evaluation: solve the chain, write it back as a pose, resolve.
fn reach(input: &Stream, target: &Stream, iterations: f32) -> Stream {
    let n = input.count();
    let Some(goal) = pose::goal(target) else {
        return input.clone(); // nothing to reach for → keep the pose we had
    };
    let parent = fk::scalars(input, fk::PARENT, -1.0, n);
    if n < 2 || !is_chain(&parent) {
        return input.clone(); // not a chain → an honest no-op
    }

    let len = fk::scalars(input, fk::LEN, 0.0, n);
    let mut p = fk::positions(input);
    fabrik(&mut p, &len, goal, iteration_count(iterations));

    let joints: Vec<usize> = (1..n).collect();
    let rot = pose::relocal(input, &p, &joints);
    fk::resolve(&input.clone().with(fk::ROT, Column::Scalar(rot)))
}

struct RigFabrik;

impl NodeOp for RigFabrik {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let iterations = ctx.param("iterations");
        let out = reach(ctx.input(0), ctx.input(1), iterations);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(RigFabrik))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "FABRIK",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "iterations",
    label: "Iterations",
    min: 1.0,
    max: MAX_ITERATIONS as f32,
    step: 1.0,
    widget: ParamWidget::IntSlider,
}];

#[cfg(test)]
mod tests {
    use super::*;

    const BONE: f32 = 1.0;

    /// A straight chain of `n` joints along `+x`, rooted at the origin.
    fn tentacle(n: usize) -> Stream {
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
        let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
        (dx * dx + dy * dy).sqrt()
    }

    /// **The tip lands on the goal, from anywhere, with rigid bones and a nailed root.**
    /// The tolerance is the pose round trip (approximate `atan2` + `cos/sin`), not slack
    /// in the solver.
    #[test]
    fn the_tip_reaches_the_goal_from_any_direction() {
        for goal in [[2.0, 2.0], [-3.0, 1.0], [0.0, -3.5], [1.0, 0.0]] {
            let out = reach(&tentacle(6), &point(goal[0], goal[1]), 10.0);
            let p = ps(&out);
            assert!(
                dist(p[5], goal) < 0.02,
                "goal {goal:?}: the tip landed at {:?}",
                p[5]
            );
            assert_eq!(p[0], [0.0, 0.0], "the root is nailed");
            for i in 1..6 {
                assert!(
                    (dist(p[i], p[i - 1]) - BONE).abs() < 1e-3,
                    "bone {i} stretched to {}",
                    dist(p[i], p[i - 1])
                );
            }
        }
    }

    /// Out of reach → the chain lies STRAIGHT along the line to the goal (every joint on
    /// it), fully extended. FABRIK produces this on its own; nothing special-cases it.
    #[test]
    fn an_unreachable_goal_straightens_the_chain_toward_it() {
        let p = ps(&reach(&tentacle(6), &point(0.0, 40.0), 10.0));
        // 5 bones of 1.0 → the tip is 5 up, and every joint sits on the way.
        for (i, q) in p.iter().enumerate() {
            assert!(q[0].abs() < 0.02, "joint {i} is off the line: {q:?}");
            assert!(
                (q[1] - i as f32).abs() < 0.02,
                "joint {i} is not extended: {q:?}"
            );
        }
    }

    /// The solver converges: more iterations never make the reach worse, and one pass is
    /// already close. FALSIFIED by a backward pass that forgets to re-anchor the root
    /// (the chain would drift away instead of converging).
    #[test]
    fn more_iterations_only_close_the_gap() {
        let goal = [-2.0, 2.5];
        let miss = |it: f32| {
            let p = ps(&reach(&tentacle(6), &point(goal[0], goal[1]), it));
            assert_eq!(p[0], [0.0, 0.0], "the root never drifts, at any count");
            dist(p[5], goal)
        };
        let (one, many) = (miss(1.0), miss(16.0));
        assert!(many <= one + 1e-4, "1 pass {one}, 16 passes {many}");
        assert!(many < 0.02, "it converges onto the goal ({many})");
    }

    /// **The collinear degeneracy** — the one that only a whole-chain smoke test finds.
    ///
    /// A STRAIGHT chain asked to reach a goal ON ITS OWN AXIS, nearer than full stretch,
    /// has no gradient: every FABRIK pass drags it along that same line and pushes it back,
    /// so it stalls fully extended and misses by exactly the slack (here `5 − 3.5 = 1.5`).
    /// FALSIFIED by removing `break_collinearity` — the tip lands at 5, not 3.5.
    ///
    /// This is not exotic: this node is `Pure`, so it starts from the skeleton's REST pose
    /// every frame, and that pose is exactly straight. Any goal that lines up with the limb
    /// hits it. (The paper's chains never do, because they carry the previous frame's pose.)
    #[test]
    fn a_straight_chain_still_folds_to_a_goal_on_its_own_axis() {
        // The rest chain lies along +x with a reach of 5; the goal sits at 3.5 along it.
        let p = ps(&reach(&tentacle(6), &point(3.5, 0.0), 12.0));
        let miss = dist(p[5], [3.5, 0.0]);
        assert!(
            miss < 0.02,
            "the tip stalled at full stretch, missing by {miss}"
        );
        // It got there by BENDING, not by shrinking a bone.
        for i in 1..6 {
            assert!((dist(p[i], p[i - 1]) - BONE).abs() < 1e-3);
        }
        let bowed = (1..5).map(|i| p[i][1].abs()).fold(0.0f32, f32::max);
        assert!(bowed > 0.1, "the chain bowed off the axis ({bowed})");

        // …and a goal AT full stretch still gets the straight answer (no gratuitous bow).
        let straight = ps(&reach(&tentacle(6), &point(5.0, 0.0), 12.0));
        for (i, q) in straight.iter().enumerate() {
            assert!(q[1].abs() < 0.02, "joint {i} bowed for nothing: {q:?}");
        }
    }

    /// No goal wired = no-op (not "reach for the origin"), and a stream that is not a
    /// chain is untouched. An untrusted `iterations` never yields zero passes.
    #[test]
    fn no_goal_or_no_chain_is_an_honest_no_op() {
        let before = ps(&tentacle(4));
        assert_eq!(ps(&reach(&tentacle(4), &Stream::new(0), 10.0)), before);

        let cloud = Stream::new(3).with("P", Column::Vec2(vec![[1.0, 1.0]; 3]));
        assert_eq!(
            ps(&reach(&cloud, &point(5.0, 5.0), 10.0)),
            vec![[1.0, 1.0]; 3]
        );

        assert_eq!(iteration_count(f32::NAN), 1);
        assert_eq!(iteration_count(0.0), 1);
        assert_eq!(iteration_count(999.0), MAX_ITERATIONS);
    }
}
