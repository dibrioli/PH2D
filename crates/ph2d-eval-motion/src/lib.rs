#![forbid(unsafe_code)]
//! `ph2d-eval-motion` — the **Motion domain evaluator** (ADR-0034).
//!
//! Cooks a motion graph (pull, at the playhead) via [`ph2d_nodegraph::cook`]
//! and **lowers** the resulting instance [`Stream`] to `ph2d-render`
//! [`RenderInstance`]s. The data path (`graph → Vec<RenderInstance>`) is
//! **headless** — it constructs plain POD instances and needs no GPU; the
//! upload + draw (`InstanceBuffer::upload`) is the shell's job (the visual
//! smoke). This is the pull-side, HR-5-exempt presentation path (motion is
//! purely visual; only the gameplay domain is bound by determinism).
//!
//! ## Instance stream convention
//!
//! A Motion instance stream carries named columns (SoA); a missing column
//! falls back to a sensible default, so a node only writes what it changes:
//!
//! | column | type   | → RenderInstance field | default        |
//! |--------|--------|------------------------|----------------|
//! | `P`    | Vec2   | `world_pos`            | `[0,0]`        |
//! | `size` | Vec2   | `size`                 | `[1,1]`        |
//! | `rot`  | Scalar | `basis` (rotation rad) | `0` → identity |
//! | `tint` | Vec4   | `tint` (rgba)          | `[1,1,1,1]`    |
//!
//! `atlas_uv`/`anchor`/`texture_id`/`premultiplied` use the shared-atlas
//! defaults; richer cloners can extend the convention later.
//!
//! Producer coverage so far: the W2 Motion vertical (`motion.grid` →
//! `transform` → `clone`) emits only `P`; `size`/`rot`/`tint` are reserved
//! columns of the convention with no producer node yet (their lowering is
//! covered by the `lowering_reads_convention_with_defaults` unit test below). A
//! later node that sets them needs no change here — the lowering already reads
//! them.

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, CookError, OpResolver};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_render::RenderInstance;

/// Lower a cooked instance stream **into `out`** (one instance per element),
/// reusing `out`'s capacity: `out` is cleared and refilled, so a steady stream
/// count frame-to-frame allocates nothing (M0.T11 — the per-frame bridge path;
/// zero-alloc gated by M0.T12). Pure + headless: no GPU.
pub fn lower_to_instances_into(stream: &Stream, out: &mut Vec<RenderInstance>) {
    let n = stream.count();
    let p = stream.get("P");
    let size = stream.get("size");
    let rot = stream.get("rot");
    let tint = stream.get("tint");
    out.clear();
    out.reserve(n);
    for i in 0..n {
        // ADR-0070-amendment-4: RenderInstance carries the 2×2 world
        // basis, not a rotation scalar. A Motion stream emits only a
        // rotation (no skew), so the basis is a pure rotation matrix
        // `[cos, sin, -sin, cos]`. RenderInstance is PresentWorld-only
        // (HR-5 exempt), so std `sin_cos` is fine here.
        let (sin_r, cos_r) = scalar_at(rot, i, 0.0).sin_cos();
        out.push(RenderInstance {
            world_pos: vec2_at(p, i, [0.0, 0.0]),
            size: vec2_at(size, i, [1.0, 1.0]),
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint: vec4_at(tint, i, [1.0, 1.0, 1.0, 1.0]),
            basis: [cos_r, sin_r, -sin_r, cos_r],
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            // Sprite-Inspector-v2 v4 ABI fields: a Motion node stream has
            // no per-corner/opacity/flip authoring surface, so they take
            // their identity values (white gradient, full opacity, no
            // flip) — byte-identical render to the pre-v4 path.
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            texture_id: 0,
            // Node-graph emit doesn't have a hierarchy slot — every
            // motion node's instances share `z_order = 0`. Renderer's
            // tiebreaker (`texture_id`) groups them into one run.
            z_order: 0,
            sampling: 0,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            // Node-graph emit has no hierarchy → no clip silhouette.
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
        });
    }
}

/// Lower a cooked instance stream to render instances (one per element).
/// Pure + headless. Allocates a fresh `Vec`; the per-frame path uses
/// [`lower_to_instances_into`] to reuse a buffer instead.
pub fn lower_to_instances(stream: &Stream) -> Vec<RenderInstance> {
    let mut out = Vec::new();
    lower_to_instances_into(stream, &mut out);
    out
}

/// Cook `target` at `playhead` and lower its **output port 0** to render
/// instances. Reuse the same [`Cook`] across frames for incremental cheapness.
///
/// Lowering a single port is intentional: a Motion render target is one
/// instance stream. A target with several output ports has only port 0 lowered
/// here (a multi-port target would select the port at the call site — not
/// needed by any Motion node today, all of which have exactly one output). A
/// target that legitimately declares **zero** outputs yields an empty `Vec`;
/// note the cook itself already rejects a node that *declares* an output but
/// emits none ([`CookError::OutputCountMismatch`]), so an empty result here
/// means "no output port", never a dropped stream.
pub fn evaluate_motion(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
) -> Result<Vec<RenderInstance>, CookError> {
    let mut out = Vec::new();
    evaluate_motion_into(cook, graph, ops, target, playhead, &mut out)?;
    Ok(out)
}

/// Cook `target` at `playhead` and lower its output port 0 **into `out`**,
/// reusing the buffer's capacity (M0.T11 — the per-frame bridge entry). Same
/// single-port semantics as [`evaluate_motion`]; a target with no output port
/// leaves `out` empty. Reuse the same [`Cook`] AND the same `out` across frames
/// for the zero-alloc steady state (gated by M0.T12).
pub fn evaluate_motion_into(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
    out: &mut Vec<RenderInstance>,
) -> Result<(), CookError> {
    let outputs = cook.cook(graph, ops, target, playhead)?;
    // A cooked output port is a `CookValue`; a Motion target's port 0 is an
    // instance stream (ADR-0058-amendment-1). A non-stream value lowers to no
    // instances (its `as_stream()` is empty).
    match outputs.first() {
        Some(v) => lower_to_instances_into(v.as_stream(), out),
        None => out.clear(),
    }
    Ok(())
}

fn scalar_at(c: Option<&Column>, i: usize, default: f32) -> f32 {
    match c {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
fn vec2_at(c: Option<&Column>, i: usize, default: [f32; 2]) -> [f32; 2] {
    match c {
        Some(Column::Vec2(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
fn vec4_at(c: Option<&Column>, i: usize, default: [f32; 4]) -> [f32; 4] {
    match c {
        Some(Column::Vec4(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

    #[test]
    fn lowering_reads_convention_with_defaults() {
        let s = Stream::new(2)
            .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
            .with("size", Column::Vec2(vec![[5.0, 5.0], [6.0, 6.0]]));
        // rot/tint absent → defaults.
        let out = lower_to_instances(&s);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].world_pos, [1.0, 2.0]);
        assert_eq!(out[1].world_pos, [3.0, 4.0]);
        assert_eq!(out[0].size, [5.0, 5.0]);
        // rot default 0 → identity basis (ADR-0070-amendment-4).
        assert_eq!(out[0].basis, RenderInstance::IDENTITY_BASIS);
        assert_eq!(out[1].tint, [1.0, 1.0, 1.0, 1.0]); // default
        assert_eq!(out[0].atlas_uv, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn lower_into_matches_fresh_and_reuses_capacity() {
        let s = Stream::new(3).with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]));
        let fresh = lower_to_instances(&s);
        let mut buf = Vec::new();
        lower_to_instances_into(&s, &mut buf);
        // Same lowered geometry (RenderInstance is not PartialEq → compare fields).
        assert_eq!(buf.len(), fresh.len());
        for (a, b) in buf.iter().zip(&fresh) {
            assert_eq!(a.world_pos, b.world_pos);
            assert_eq!(a.size, b.size);
            assert_eq!(a.tint, b.tint);
            assert_eq!(a.basis, b.basis);
        }
        // Reuse: a second lower with a smaller stream refills the SAME buffer,
        // retaining capacity (the zero-alloc steady-state property, M0.T11/T12).
        let cap_before = buf.capacity();
        let s2 = Stream::new(1).with("P", Column::Vec2(vec![[9.0, 9.0]]));
        lower_to_instances_into(&s2, &mut buf);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0].world_pos, [9.0, 9.0]);
        assert!(buf.capacity() >= cap_before, "capacity retained, not shrunk");
    }

    #[test]
    fn empty_stream_yields_no_instances() {
        assert!(lower_to_instances(&Stream::new(0)).is_empty());
    }

    // A minimal source emitting two instances at P=(0,0),(10,0) — stands in for
    // a real generator (W2.T2) so the cook→lower path is exercised end to end.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.test.src"),
        name: "motion.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [10.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == SRC_MAN.id).then_some(&Src as &dyn NodeOp)
        }
    }

    #[test]
    fn evaluate_motion_cooks_and_lowers() {
        let mut g = Graph::new();
        let src = g.add_node("motion.test.src");
        let mut cook = Cook::new();
        let instances = evaluate_motion(&mut cook, &g, &Ops, src, 0.0).unwrap();
        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].world_pos, [0.0, 0.0]);
        assert_eq!(instances[1].world_pos, [10.0, 0.0]);
    }
}
