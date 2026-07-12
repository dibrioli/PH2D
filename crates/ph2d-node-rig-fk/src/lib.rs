#![forbid(unsafe_code)]
//! `rig.fk` — **forward kinematics**: rebuild a skeleton's world pose from the local
//! joint angles somebody just posed (Motion Nodes M4, doc 40).
//!
//! ## Why this is a node at all
//!
//! Because a skeleton is an **ordinary instance stream** (the M4.N3 decision — see the
//! [`fk`] leaf), every generic Motion node already works on it. In particular the
//! **`Rotation` channel** of `motion.oscillator` / `wiggle` / `noise` / `step` writes
//! the `rot` column — which is exactly how you pose a joint.
//!
//! But those nodes know nothing about bones: they set the ANGLE and leave `P` where it
//! was. A posed skeleton that nobody resolved is a limb whose joints are all bent and
//! yet has not moved a pixel. `rig.fk` is the resolve:
//!
//! ```text
//! skeleton ─> oscillator (Rotation) ─> rig.fk ─> …
//!               poses the angles       makes them a POSE
//! ```
//!
//! Drop it after anything that touched `rot`, and the limb swings. It is the whole
//! reason FK is its own node rather than something the source does once.
//!
//! **It is the identity on anything that is not a skeleton.** A stream with no
//! `parent` column is all roots, so every position survives untouched (doc 39: a node
//! at its default must not move a single element).
//!
//! Idempotent — the pose is a pure function of `(parent, len, rot)`, so resolving twice
//! changes nothing. Transcendental-free (HR-5). `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod fk;
mod trig;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("rig.fk"),
    name: "rig.fk",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // No params: the pose is entirely in the columns. A dial here would be a second
    // source of truth for the same angle.
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct RigFk;

impl NodeOp for RigFk {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let out = fk::resolve(ctx.input(0));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(RigFk))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "FK",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};

    fn ps(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    /// A straight chain of `n` joints, root at the origin — what `rig.skeleton` emits.
    fn chain(n: usize) -> Stream {
        Stream::new(n)
            .with(
                fk::PARENT,
                Column::Scalar((0..n).map(|i| i as f32 - 1.0).collect()),
            )
            .with(
                fk::LEN,
                Column::Scalar((0..n).map(|i| if i == 0 { 0.0 } else { 1.0 }).collect()),
            )
            .with(fk::ROT, Column::Scalar(vec![0.0; n]))
            .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
    }

    /// **The node's whole reason to exist**: something posed `rot` and left `P` behind.
    /// FK swings the limb — and only from the posed joint DOWNSTREAM; the joints above
    /// it must not budge. FALSIFIED by a resolve that rebuilds the chain from the origin
    /// (everything moves) or that forwards `P` untouched (nothing moves).
    #[test]
    fn posing_one_joint_swings_everything_below_it_and_nothing_above() {
        let n = 5;
        let straight = chain(n);
        let before = ps(&fk::resolve(&straight));

        // Somebody (an oscillator on the Rotation channel) bent joint 3 by 90°.
        let mut rot = fk::scalars(&straight, fk::ROT, 0.0, n);
        rot[3] = 90.0;
        let posed = straight.clone().with(fk::ROT, Column::Scalar(rot));
        let after = ps(&fk::resolve(&posed));

        for i in 0..3 {
            assert_eq!(after[i], before[i], "joint {i} is upstream of the bend");
        }
        assert_ne!(after[3], before[3], "the bent joint moved");
        assert_ne!(after[4], before[4], "and it carried its child with it");
        // The bone into joint 3 turned a quarter turn: it now points +y, not +x.
        assert!(
            (after[3][1] - after[2][1] - 1.0).abs() < 1e-3,
            "{:?}",
            after[3]
        );
    }

    /// A stream that is not a skeleton passes through untouched — `rig.fk` dropped on a
    /// point cloud is a no-op (the identity rule, doc 39).
    #[test]
    fn a_stream_with_no_bones_is_untouched() {
        let cloud = Stream::new(2).with("P", Column::Vec2(vec![[3.0, 1.0], [7.0, 9.0]]));
        assert_eq!(ps(&fk::resolve(&cloud)), vec![[3.0, 1.0], [7.0, 9.0]]);
    }
}
