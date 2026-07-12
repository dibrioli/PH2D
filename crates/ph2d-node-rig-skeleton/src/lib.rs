#![forbid(unsafe_code)]
//! `rig.skeleton` — the rig **source**: a chain of joints (Motion Nodes M4, doc 40).
//!
//! Emits `joints` elements — joint `0` is the root, each of the rest hangs off the
//! previous one by a bone of `length`, turned `angle` degrees from it. So `angle = 0`
//! is a straight limb, a small `angle` an arc, a big one a spiral. The whole chain is
//! resolved to world positions on the way out (see the `fk` leaf), so a bare skeleton
//! already renders: its joints are elements like any other.
//!
//! **A skeleton is an ordinary instance stream** — the M4.N3 decision, in full in
//! [`fk`]. `parent` / `len` / `rot` are ordinary columns, so every generic node still
//! works on a rig (move it, mask it, and **pose it with the `Rotation` channel** of
//! `oscillator` / `wiggle` / `noise`), and `rig.*` needs **no contract change** and no
//! `Domain::Rig`.
//!
//! Transcendental-free (HR-5): the parabolic `cos/sin` leaf. `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod fk;
mod trig;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Hard ceiling on the joint count — `joints` is an untrusted `f32` (a loaded
/// document, an MCP edit), and a chain is walked once per joint.
const MAX_JOINTS: usize = 64;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("rig.skeleton"),
    name: "rig.skeleton",
    // A source: it generates the chain (no input).
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "joints",
            default: 8.0,
        },
        ParamSpec {
            name: "length",
            default: 0.7,
        },
        // The LOCAL turn from each joint to the next: 0 = a straight limb.
        ParamSpec {
            name: "angle",
            default: 0.0,
        },
        // The root's own world angle — where the limb points out of its socket.
        ParamSpec {
            name: "root_angle",
            default: 90.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// How many joints to emit. Junk (`NaN`, negative) → a lone root, never zero
/// elements (an empty source reads as "the node is broken", a single joint reads as
/// "the slider is at its floor").
fn joint_count(joints: f32) -> usize {
    if joints.is_finite() && joints >= 1.0 {
        (joints.round() as usize).min(MAX_JOINTS)
    } else {
        1
    }
}

/// Author the chain, then resolve it to world space.
fn build(joints: f32, length: f32, angle: f32, root_angle: f32) -> Stream {
    let n = joint_count(joints);
    let rest = Stream::new(n)
        // Joint 0 is the root (`-1`); every other joint hangs off the previous one.
        .with(
            fk::PARENT,
            Column::Scalar((0..n).map(|i| i as f32 - 1.0).collect()),
        )
        // The root has no bone leading into it.
        .with(
            fk::LEN,
            Column::Scalar((0..n).map(|i| if i == 0 { 0.0 } else { length }).collect()),
        )
        .with(
            fk::ROT,
            Column::Scalar(
                (0..n)
                    .map(|i| if i == 0 { root_angle } else { angle })
                    .collect(),
            ),
        )
        // The same Index / Count a generator like `motion.grid` publishes, so falloffs,
        // staggers and expressions address a joint the way they address any element.
        .with("Index", Column::Scalar((0..n).map(|i| i as f32).collect()))
        .with("Count", Column::Scalar(vec![n as f32; n]))
        .with("P", Column::Vec2(vec![[0.0, 0.0]; n]));
    fk::resolve(&rest)
}

struct RigSkeleton;

impl NodeOp for RigSkeleton {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (joints, length) = (ctx.param("joints"), ctx.param("length"));
        let (angle, root_angle) = (ctx.param("angle"), ctx.param("root_angle"));
        let out = build(joints, length, angle, root_angle);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(RigSkeleton))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Skeleton",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "joints",
        label: "Joints",
        min: 1.0,
        max: MAX_JOINTS as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "length",
        label: "Bone Length",
        min: 0.05,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Bend",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "root_angle",
        label: "Root Angle",
        min: -180.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn ps(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    /// The default skeleton is a straight limb standing on its root at the origin,
    /// pointing where `root_angle` says (90° = up). FALSIFIED by a chain that fans out
    /// of the root instead of stacking end-to-end.
    #[test]
    fn a_straight_limb_stacks_bone_on_bone_out_of_the_root() {
        let s = build(4.0, 1.0, 0.0, 90.0);
        assert_eq!(s.count(), 4);
        let p = ps(&s);
        for (i, q) in p.iter().enumerate() {
            assert!(q[0].abs() < 1e-3, "joint {i} stays on the axis: {q:?}");
            assert!((q[1] - i as f32).abs() < 1e-3, "joint {i} at y = {i}");
        }
    }

    /// `angle` bends each joint relative to the LAST one, so the chain curls — the
    /// property that makes it a skeleton and not a fan.
    #[test]
    fn the_bend_compounds_down_the_chain() {
        let p = ps(&build(4.0, 1.0, 90.0, 0.0));
        // World angles 0, 90, 180, 270 → right, up, left: a square.
        let near =
            |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
        assert!(near(p[1], [0.0, 1.0]), "{:?}", p[1]);
        assert!(near(p[2], [-1.0, 1.0]), "{:?}", p[2]);
        assert!(near(p[3], [-1.0, 0.0]), "{:?}", p[3]);
    }

    /// The chain it publishes is the one the `fk` leaf can re-resolve: parents point
    /// backwards, the root has no bone, and re-resolving changes nothing.
    #[test]
    fn it_publishes_a_well_formed_re_resolvable_chain() {
        let s = build(5.0, 0.5, 20.0, 0.0);
        match s.get(fk::PARENT).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![-1.0, 0.0, 1.0, 2.0, 3.0]),
            _ => panic!("parent"),
        }
        match s.get(fk::LEN).unwrap() {
            Column::Scalar(v) => assert_eq!(v[0], 0.0, "the root has no bone into it"),
            _ => panic!("len"),
        }
        assert_eq!(ps(&fk::resolve(&s)), ps(&s), "already resolved");
    }

    /// An untrusted `joints` never yields zero elements and never allocates without
    /// bound.
    #[test]
    fn the_joint_count_is_clamped_and_never_empty() {
        assert_eq!(joint_count(8.0), 8);
        assert_eq!(joint_count(999.0), MAX_JOINTS);
        assert_eq!(joint_count(f32::NAN), 1);
        assert_eq!(joint_count(-4.0), 1);
        assert_eq!(joint_count(0.0), 1);
    }
}
