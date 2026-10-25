#![forbid(unsafe_code)]
//! `rig.ik_2bone` — **two-bone inverse kinematics** by the law of cosines: the arm/leg
//! solver every rig package ships (Maya's `ikRPsolver`, Unity's `TwoBoneIKConstraint`,
//! Unreal's `Two Bone IK`, Blender's IK with a chain length of 2). Motion Nodes M4, doc 41.
//!
//! Three joints — **root · elbow · hand** — and a goal. The hand goes to the goal, and
//! the elbow is placed so both bones keep their length. That is a closed-form problem,
//! not an iteration: the triangle `(root, elbow, hand)` has all three sides known
//! (`l1`, `l2`, and the distance `d` to the goal), so the law of cosines gives the angle
//! at the root outright.
//!
//! ## Transcendental-free, and not by approximating the solver (HR-5)
//!
//! The textbook writes `a = acos((l1² + d² − l2²) / (2·l1·d))` — an `acos`, which HR-5
//! forbids. But the angle itself is never wanted; only the DIRECTION of the first bone
//! is. So stay in vectors:
//!
//! ```text
//! cos a = (l1² + d² − l2²) / (2·l1·d)        clamped to [−1, 1]
//! sin a = √(1 − cos²a)                       ← only a sqrt, which HR-5 allows
//! bone1 = rotate(unit(goal − root), ±a)      ← a 2×2 rotation by (cos a, sin a)
//! ```
//!
//! The rotation of a unit vector by an angle whose cosine and sine are already in hand is
//! pure multiply-add. **The solve is therefore EXACT** — no polynomial approximation of
//! `acos` anywhere. (The only approximation in the node is the round trip back to angles;
//! see the `pose` leaf.)
//!
//! `flip` picks which side of the root→goal line the elbow falls on (the "pole" of every
//! other package, reduced to the one bit that matters in 2D).
//!
//! **Out of reach → the limb points at the goal, fully extended** (`d` is clamped to
//! `[|l1 − l2|, l1 + l2]`), which is what every solver does and what an arm does. It
//! never stretches and never produces `NaN`.
//!
//! **It solves for a POSE, not for positions** — see the `pose` leaf: the solver's
//! positions are converted to local angles and drawn by `fk::resolve`, so the bones stay
//! exactly rigid, the joints below the hand follow it, and solvers compose.
//!
//! An **unconnected goal** is not the origin: with no target the node is a **no-op** and
//! the limb keeps the pose it had. `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

use ph2d_rig_kinematics::fk;
use ph2d_rig_kinematics::pose;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("rig.ik_2bone"),
    name: "rig.ik_2bone",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The goal: the FIRST element of whatever is wired here. Any moving point will
        // do — an orbiting dot, a `motion.make_point` driven by expressions, a gizmo.
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
    params: &[
        // The root of the 3-joint span to solve (root, root+1, root+2).
        ParamSpec {
            name: "root",
            default: 0.0,
        },
        // Which side of the root→goal line the elbow bends to.
        ParamSpec {
            name: "flip",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The elbow and hand that put the hand on `goal` with rigid bones.
///
/// Returns `(elbow, hand)` in world space. The law is `ph2d_skeleton::reach` — one owner for the
/// two-bone law (docs/BlenderConstraints/02_W2_ossos.md): `flip` picks the side (`Cw`), the
/// default is `Ccw`.
#[expect(clippy::cast_possible_truncation, reason = "the stream is f32; the law is f64")]
fn solve(root: [f32; 2], l1: f32, l2: f32, goal: [f32; 2], flip: bool) -> ([f32; 2], [f32; 2]) {
    let r = [f64::from(root[0]), f64::from(root[1])];
    let (a, b) = (f64::from(l1), f64::from(l2));
    let mut j = [r, [r[0] + a, r[1]], [r[0] + a + b, r[1]]];
    ph2d_skeleton::reach(
        &mut j,
        &[a, b],
        [f64::from(goal[0]), f64::from(goal[1])],
        ph2d_skeleton::Reach {
            bend: if flip {
                ph2d_skeleton::BendSide::Cw
            } else {
                ph2d_skeleton::BendSide::Ccw
            },
            ..ph2d_skeleton::Reach::default()
        },
    );
    let f = |p: [f64; 2]| [p[0] as f32, p[1] as f32];
    (f(j[1]), f(j[2]))
}

/// Is `root, root+1, root+2` a real 3-joint chain in this stream?
fn is_chain(parent: &[f32], n: usize, root: usize) -> bool {
    root + 2 < n
        && parent[root + 1] as i32 == root as i32
        && parent[root + 2] as i32 == root as i32 + 1
}

/// One evaluation: solve the span, write it back as a pose, resolve.
fn reach(input: &Stream, target: &Stream, root_ix: f32, flip: bool) -> Stream {
    let n = input.count();
    let Some(goal) = pose::goal(target) else {
        return input.clone(); // nothing to reach for → keep the pose we had
    };
    let parent = fk::scalars(input, fk::PARENT, -1.0, n);
    let root = if root_ix.is_finite() && root_ix >= 0.0 {
        root_ix.round() as usize
    } else {
        0
    };
    if !is_chain(&parent, n, root) {
        return input.clone(); // not a 3-joint chain here → an honest no-op
    }

    let len = fk::scalars(input, fk::LEN, 0.0, n);
    let mut p = fk::positions(input);
    let (elbow, hand) = solve(p[root], len[root + 1], len[root + 2], goal, flip);
    p[root + 1] = elbow;
    p[root + 2] = hand;

    // The solved positions become the POSE; FK draws it (so the bones are rigid by
    // construction and any joints hanging off the hand follow it).
    // ⭐ O `Strength` da restrição: a pose resolvida mistura-se com a que entrou, pela coluna
    // `falloff` — ver [`pose::mix_by_falloff`]. Ausente ⇒ força cheia ⇒ o solve de sempre, ao bit.
    let rot = pose::mix_by_falloff(input, pose::relocal(input, &p, &[root + 1, root + 2]));
    // ⚠️ **O solve é uma pose LOCAL** (ângulos relativos ao pai), logo entra pela coluna do
    // LOCAL — escrevê-lo em `fk::ROT` fazia o `resolve` deitá-lo fora e devolver a pose da
    // resolução anterior. *A coluna `rot` que sai daqui é o MUNDO, e é o que o desenho lê.*
    fk::resolve(&input.clone().with(fk::LROT, Column::Scalar(rot)))
}

struct RigIk2Bone;

impl NodeOp for RigIk2Bone {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (root, flip) = (ctx.param("root"), ctx.param("flip"));
        let out = reach(ctx.input(0), ctx.input(1), root, flip >= 0.5);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(RigIk2Bone))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.rig.ik_2bone.name",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "root",
        label: "node.rig.ik_2bone.param.root",
        min: 0.0,
        max: 62.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "flip",
        label: "node.rig.ik_2bone.param.flip",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ A lei dos 2 ossos tem UM dono (`ph2d_skeleton::reach`): o nó dá os bits dela, cast a `f32`.
    #[test]
    fn the_node_solves_with_the_skeleton_law() {
        let (l1, l2) = (1.3f32, 0.7f32);
        let mut alvos = vec![[0.0f32, 0.0], [5.0, 1.0], [0.6, 0.0], [-0.2, 0.3]];
        for i in 0..64 {
            let a = i as f32 * 0.37;
            let r = 0.5 + 0.03 * i as f32;
            alvos.push([r * a.cos(), r * a.sin()]);
        }
        // A quase-extensão, onde a lei fechada encosta na recta.
        alvos.push([l1 + l2 - 1e-5, 0.0]);
        alvos.push([(l1 + l2) * 0.999_95, 0.01]);
        for goal in alvos {
            for flip in [false, true] {
                let root = [0.25f32, -0.5];
                let goal = [root[0] + goal[0], root[1] + goal[1]];
                let mut j = [
                    [f64::from(root[0]), f64::from(root[1])],
                    [f64::from(root[0] + l1), f64::from(root[1])],
                    [f64::from(root[0] + l1 + l2), f64::from(root[1])],
                ];
                ph2d_skeleton::reach(
                    &mut j,
                    &[f64::from(l1), f64::from(l2)],
                    [f64::from(goal[0]), f64::from(goal[1])],
                    ph2d_skeleton::Reach {
                        bend: if flip {
                            ph2d_skeleton::BendSide::Cw
                        } else {
                            ph2d_skeleton::BendSide::Ccw
                        },
                        ..ph2d_skeleton::Reach::default()
                    },
                );
                let f = |p: [f64; 2]| [p[0] as f32, p[1] as f32];
                assert_eq!(solve(root, l1, l2, goal, flip), (f(j[1]), f(j[2])), "{goal:?} {flip}");
            }
        }
    }

    const L1: f32 = 2.0;
    const L2: f32 = 1.0;

    /// A 3-joint arm at the origin, pointing along `+x`, with bones of 2 and 1.
    fn arm() -> Stream {
        fk::resolve(
            &Stream::new(3)
                .with(fk::PARENT, Column::Scalar(vec![-1.0, 0.0, 1.0]))
                .with(fk::LEN, Column::Scalar(vec![0.0, L1, L2]))
                .with(fk::LROT, Column::Scalar(vec![0.0, 0.0, 0.0]))
                .with("P", Column::Vec2(vec![[0.0, 0.0]; 3])),
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

    /// O mesmo braço com a coluna de força (o `falloff`) escrita.
    fn arm_com_forca(w: f32) -> Stream {
        arm().with(pose::FALLOFF, Column::Scalar(vec![w; 3]))
    }

    /// ⭐⭐⭐ **A ESCADA DA FORÇA** — o `Strength` que o Rive põe em 7 de 7 restrições e o Spine
    /// em 4 de 4, e que esta família não tinha em nenhuma.
    ///
    /// Três degraus, e cada um afirma uma coisa diferente:
    ///
    /// - **`0`** = a restrição não faz nada: a pose que ENTROU sobrevive **ao bit**;
    /// - **`1`** = o solve de sempre, **ao bit** — é isto que torna a wave aditiva;
    /// - **`½`** = entre os dois, e *estritamente* entre (senão «mistura» seria um interruptor).
    ///
    /// ⚠️ A ausência da coluna tem de dar o mesmo que `1`, senão todo rig já autorado muda.
    #[test]
    fn a_escada_da_forca_vai_da_pose_que_entrou_ate_ao_solve() {
        let alvo = point(1.0, 1.5);
        let parado = ps(&arm());
        let cheio = ps(&reach(&arm(), &alvo, 0.0, false));
        assert!(
            dist(parado[2], cheio[2]) > 0.5,
            "o alvo nao mexe o braco — a fixtura nao mede forca nenhuma"
        );

        // `1` e a AUSÊNCIA são o mesmo solve, ao bit.
        assert_eq!(
            ps(&reach(&arm_com_forca(1.0), &alvo, 0.0, false)),
            cheio,
            "forca 1 nao e' o solve de sempre ao bit"
        );
        // `0` deixa a pose exactamente como entrou.
        assert_eq!(
            ps(&reach(&arm_com_forca(0.0), &alvo, 0.0, false)),
            parado,
            "forca 0 mexeu no braco"
        );
        // `½` fica ESTRITAMENTE entre os dois.
        let meio = ps(&reach(&arm_com_forca(0.5), &alvo, 0.0, false))[2];
        let (a, b) = (dist(meio, parado[2]), dist(meio, cheio[2]));
        assert!(
            a > 1e-3 && b > 1e-3,
            "forca ½ colapsou num dos extremos: {meio:?} (parado {:?}, cheio {:?})",
            parado[2],
            cheio[2]
        );
    }

    /// **The hand lands on the goal**, and the bones keep their lengths getting there.
    /// The tolerance is the price of the pose round trip (approximate `atan2` + `cos/sin`,
    /// ~0.1 % of the chain) — not slack in the solver, which is exact.
    #[test]
    fn the_hand_reaches_the_goal_with_rigid_bones() {
        for goal in [[1.5, 1.5], [-2.0, 0.5], [0.0, -2.5], [2.9, 0.0]] {
            let out = reach(&arm(), &point(goal[0], goal[1]), 0.0, false);
            let p = ps(&out);
            assert!(
                dist(p[2], goal) < 0.01,
                "goal {goal:?}: the hand landed at {:?}",
                p[2]
            );
            assert!((dist(p[0], p[1]) - L1).abs() < 1e-3, "upper bone stretched");
            assert!((dist(p[1], p[2]) - L2).abs() < 1e-3, "lower bone stretched");
        }
    }

    /// `flip` puts the elbow on the OTHER side of the root→goal line — the one bit of
    /// "pole vector" a 2D rig needs. The hand does not move.
    #[test]
    fn flip_mirrors_the_elbow_across_the_reach_line() {
        let goal = [1.5, 1.5];
        let a = ps(&reach(&arm(), &point(goal[0], goal[1]), 0.0, false));
        let b = ps(&reach(&arm(), &point(goal[0], goal[1]), 0.0, true));
        assert!(dist(a[2], b[2]) < 0.01, "the hand lands in the same place");
        assert!(dist(a[1], b[1]) > 0.5, "but the elbow swapped sides");
        // Mirror image about the reach line: the two elbows sit at equal distance from
        // the root, on opposite sides.
        let cross = |e: [f32; 2]| goal[0] * e[1] - goal[1] * e[0];
        assert!(
            cross(a[1]) * cross(b[1]) < 0.0,
            "the elbows are on opposite sides"
        );
    }

    /// Out of reach: the limb points AT the goal, fully extended — it never stretches to
    /// touch it and never returns a NaN. FALSIFIED by the unclamped law of cosines, whose
    /// `sqrt` of a negative would hand back a NaN limb.
    #[test]
    fn an_unreachable_goal_extends_the_limb_instead_of_stretching_it() {
        let p = ps(&reach(&arm(), &point(100.0, 0.0), 0.0, false));
        assert!(p.iter().all(|q| q[0].is_finite() && q[1].is_finite()));
        assert!(
            (dist(p[0], p[2]) - (L1 + L2)).abs() < 0.01,
            "the arm is at full stretch: {:?}",
            dist(p[0], p[2])
        );
        assert!((p[2][1]).abs() < 0.01, "…and pointing straight at the goal");
        // The other degenerate end: a goal inside the fold. Still finite, still rigid.
        let q = ps(&reach(&arm(), &point(0.0, 0.0), 0.0, false));
        assert!(q.iter().all(|r| r[0].is_finite() && r[1].is_finite()));
        assert!((dist(q[0], q[1]) - L1).abs() < 1e-3);
    }

    /// No goal wired = **no-op**, not "reach for the origin". An unconnected port cooks
    /// to an empty stream, and a limb yanked to the middle of the world by an unplugged
    /// wire is the kind of surprise this module refuses.
    #[test]
    fn an_unconnected_goal_leaves_the_pose_alone() {
        let before = ps(&arm());
        assert_eq!(ps(&reach(&arm(), &Stream::new(0), 0.0, false)), before);
        // And a stream that is not a 3-joint chain here is likewise untouched.
        let cloud = Stream::new(3).with("P", Column::Vec2(vec![[1.0, 1.0]; 3]));
        assert_eq!(
            ps(&reach(&cloud, &point(5.0, 5.0), 0.0, false)),
            vec![[1.0, 1.0]; 3]
        );
    }

    /// **The solver leaves a VALID SKELETON, not just the right dots.** A `rig.fk` run
    /// after it must find nothing to fix — which is only true because the solver wrote
    /// its answer as ANGLES and let FK draw it.
    ///
    /// FALSIFIED by the tempting shortcut: write the solved positions straight into `P`.
    /// Every assertion above would still pass (the dots land where they should), and the
    /// limb would snap back to its old pose the moment anything downstream — another
    /// solver, a wave, the skinning — resolved the chain from its angles.
    #[test]
    fn a_downstream_fk_finds_nothing_to_fix() {
        let solved = reach(&arm(), &point(1.5, 1.5), 0.0, false);
        let resolved_again = fk::resolve(&solved);
        for (a, b) in ps(&solved).iter().zip(ps(&resolved_again).iter()) {
            assert!(
                dist(*a, *b) < 1e-4,
                "FK moved a solved joint: {a:?} -> {b:?}"
            );
        }
    }

    /// The law of cosines itself, checked against the geometry it claims: with the goal
    /// exactly `l1 + l2` away the triangle is degenerate (elbow ON the line), and at
    /// `|l1 − l2|` it is folded back.
    #[test]
    fn the_triangle_closes_at_both_extremes() {
        let (elbow, hand) = solve([0.0, 0.0], L1, L2, [L1 + L2, 0.0], false);
        assert!(
            (elbow[1]).abs() < 1e-3,
            "full stretch: the elbow is on the line"
        );
        assert!((hand[0] - (L1 + L2)).abs() < 1e-3);

        let (elbow, hand) = solve([0.0, 0.0], L1, L2, [L1 - L2, 0.0], false);
        assert!((elbow[0] - L1).abs() < 1e-3, "folded: the elbow overshoots");
        assert!((hand[0] - (L1 - L2)).abs() < 1e-3);
    }
}
