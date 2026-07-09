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
//! | column    | type   | → RenderInstance field   | default                    |
//! |-----------|--------|--------------------------|----------------------------|
//! | `P`       | Vec2   | `world_pos`              | `[0,0]`                    |
//! | `size`    | Vec2   | `size`                   | caller's `default_size`    |
//! | `rot`     | Scalar | `basis` (rotation **deg**) | `0` → identity            |
//! | `tint`    | Vec4   | `tint` (rgba)            | `[1,1,1,1]`                |
//! | `uv_rect` | Vec4   | `atlas_uv` (u0,v0,u1,v1) | caller's `default_uv_rect` |
//!
//! `anchor`/`texture_id`/`premultiplied` use the shared-atlas defaults. A
//! stream with no `uv_rect` column takes the caller-supplied `default_uv_rect`
//! — the shell passes the atlas tile the default document should sample (a
//! single opaque tile, so the raw M0 output reads as clean solid quads rather
//! than a whole-atlas thumbnail), while a headless caller passes the
//! whole-atlas rect. Richer cloners can extend the convention later.
//!
//! Producer coverage so far: the W2 Motion vertical (`motion.grid` →
//! `transform` → `clone`) emits only `P`; `size`/`rot`/`tint`/`uv_rect` are
//! reserved columns of the convention with no producer node yet (a framing node
//! that writes `uv_rect`/`cell` lands in M1). Their lowering is covered by the
//! unit tests below. A later node that sets them needs no change here — the
//! lowering already reads them.

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, CookError, OpResolver};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_render::RenderInstance;

/// Lower a cooked instance stream **into `out`** (one instance per element),
/// reusing `out`'s capacity: `out` is cleared and refilled, so a steady stream
/// count frame-to-frame allocates nothing (M0.T11 — the per-frame bridge path;
/// zero-alloc gated by M0.T12). Pure + headless: no GPU.
///
/// `default_uv_rect` / `default_size` are the `atlas_uv` / `size` for an
/// instance whose stream lacks the matching column (the M0 case — no framing
/// node yet). The shell passes a single opaque atlas tile plus a `size` below
/// the grid spacing so the raw default document renders as clean, distinct
/// quads; a headless caller passes the whole-atlas rect `[0,0,1,1]` and unit
/// size `[1,1]`.
pub fn lower_to_instances_into(
    stream: &Stream,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) {
    out.clear();
    lower_to_instances_onto(stream, default_uv_rect, default_size, out);
}

/// Like [`lower_to_instances_into`] but **appends** — `out` keeps whatever it
/// already holds. This is how several render sinks compose into one draw: the
/// pump clears once, then each `motion.output` node's stream lowers onto the
/// same buffer (still zero-alloc in steady state — capacity is retained).
pub fn lower_to_instances_onto(
    stream: &Stream,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) {
    let n = stream.count();
    let p = stream.get("P");
    let size = stream.get("size");
    let rot = stream.get("rot");
    let tint = stream.get("tint");
    let uv_rect = stream.get("uv_rect");
    out.reserve(n);
    for i in 0..n {
        // ADR-0070-amendment-4: RenderInstance carries the 2×2 world
        // basis, not a rotation scalar. A Motion stream emits only a
        // rotation (no skew), so the basis is a pure rotation matrix
        // `[cos, sin, -sin, cos]`. RenderInstance is PresentWorld-only
        // (HR-5 exempt), so std `sin_cos` is fine here.
        //
        // The `rot` column is in **degrees** — the app's authored-angle unit
        // (the Painter's `*_angle_deg` fields, the Inspector's `deg` boxes).
        // Radians live nowhere in the Motion authoring surface; only this
        // conversion, at the very edge where the basis is built.
        let (sin_r, cos_r) = scalar_at(rot, i, 0.0).to_radians().sin_cos();
        out.push(RenderInstance {
            world_pos: vec2_at(p, i, [0.0, 0.0]),
            size: vec2_at(size, i, default_size),
            atlas_uv: vec4_at(uv_rect, i, default_uv_rect),
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
/// [`lower_to_instances_into`] to reuse a buffer instead. Uses the whole-atlas
/// UV `[0,0,1,1]` for any instance without a `uv_rect` column (the shell path
/// supplies a real tile via [`lower_to_instances_into`]'s `default_uv_rect`).
pub fn lower_to_instances(stream: &Stream) -> Vec<RenderInstance> {
    let mut out = Vec::new();
    lower_to_instances_into(stream, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0], &mut out);
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
    evaluate_motion_into(
        cook,
        graph,
        ops,
        target,
        playhead,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &mut out,
    )?;
    Ok(out)
}

/// Cook `target` at `playhead` and lower its output port 0 **into `out`**,
/// reusing the buffer's capacity (M0.T11 — the per-frame bridge entry). Same
/// single-port semantics as [`evaluate_motion`]; a target with no output port
/// leaves `out` empty. Reuse the same [`Cook`] AND the same `out` across frames
/// for the zero-alloc steady state (gated by M0.T12).
///
/// `default_uv_rect` / `default_size` are the `atlas_uv` / `size` fallbacks for
/// a stream without the matching column (see [`lower_to_instances_into`]).
#[allow(clippy::too_many_arguments)] // cook + graph + resolver + target + playhead + 2 defaults + out
pub fn evaluate_motion_into(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) -> Result<(), CookError> {
    let outputs = cook.cook(graph, ops, target, playhead)?;
    // A cooked output port is a `CookValue`; a Motion target's port 0 is an
    // instance stream (ADR-0058-amendment-1). A non-stream value lowers to no
    // instances (its `as_stream()` is empty).
    match outputs.first() {
        Some(v) => lower_to_instances_into(v.as_stream(), default_uv_rect, default_size, out),
        None => out.clear(),
    }
    Ok(())
}

/// Per-frame Motion cook driver (plan §1.8). Owns the persistent [`Cook`] (its
/// memo + `pre` feedback must survive across frames) and the reused instance
/// buffer, and re-cooks the sink **only when the frame is dirty**: the transport
/// tick advanced (playing / scrub), the graph was edited ([`Self::mark_dirty`]),
/// or it is the first cook. A paused, unchanged frame skips the cook entirely,
/// so it touches no heap — the [`Cook`] otherwise re-evaluates each node every
/// call (it is NOT playhead-memoized for a `Pure` graph). Gated by
/// `tests/paused_no_alloc.rs` (M0.T12).
pub struct MotionCookPump {
    /// Persistent cook — its memo + `pre`-edge feedback are carried across
    /// frames, so it must NOT be re-created each frame.
    pub cook: Cook,
    /// Reused per-frame lowering buffer — zero-alloc in steady state.
    pub instances: Vec<RenderInstance>,
    /// The transport tick `instances` currently reflects (`None` = never cooked).
    last_cooked_tick: Option<u64>,
    /// Set by a graph edit so the next frame re-cooks even at the same tick.
    dirty: bool,
}

impl Default for MotionCookPump {
    fn default() -> Self {
        Self::new()
    }
}

impl MotionCookPump {
    /// A fresh pump: empty cook + buffer, marked dirty so the first
    /// [`Self::pump`] cooks.
    pub fn new() -> Self {
        Self {
            cook: Cook::new(),
            instances: Vec::new(),
            last_cooked_tick: None,
            dirty: true,
        }
    }

    /// Force a re-cook on the next [`Self::pump`], even at the same tick (call
    /// after editing the graph while paused). The M1 graph-edit path.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Re-cook every sink in `sinks` into `instances` at `playhead` **iff** the
    /// frame is dirty (the `tick` changed since the last cook,
    /// [`Self::mark_dirty`] was called, or this is the first cook). A clean,
    /// unchanged frame (paused, no edit) leaves the buffer untouched — zero
    /// allocation. Returns `true` if it cooked. Reuse the SAME pump across
    /// frames for the steady state.
    ///
    /// **Several sinks compose into one draw**: each `motion.output` node in the
    /// document lowers onto the shared buffer, in the order given (the shell
    /// scans by node id, so it is deterministic). A document may therefore hold
    /// independent scenes — a grid rig and a particle fountain — without a
    /// stream-merging node. An empty `sinks` clears the buffer (nothing renders).
    #[allow(clippy::too_many_arguments)] // graph + resolver + sinks + tick + playhead + 2 defaults
    pub fn pump(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        sinks: &[NodeId],
        tick: u64,
        playhead: f64,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
    ) -> bool {
        if !self.dirty && self.last_cooked_tick == Some(tick) {
            return false; // paused + unchanged → reuse the buffer, no heap traffic
        }
        self.instances.clear();
        for &sink in sinks {
            // A sink that fails to cook (an unknown type mid-edit) contributes
            // nothing; the others still draw.
            if let Ok(outputs) = self.cook.cook(graph, ops, sink, playhead)
                && let Some(v) = outputs.first()
            {
                lower_to_instances_onto(
                    v.as_stream(),
                    default_uv_rect,
                    default_size,
                    &mut self.instances,
                );
            }
        }
        if !sinks.is_empty() {
            // Advance the 1-tick `pre` feedback once per cooked frame — ONCE for
            // the whole graph, not per sink (each sink's `pre` sources are
            // snapshotted by the same call).
            let _ = self.cook.advance_tick(graph, ops, playhead);
        }
        self.last_cooked_tick = Some(tick);
        self.dirty = false;
        true
    }
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
    fn rot_column_is_degrees_not_radians() {
        // The authored unit is degrees (the app's convention); the lowering is
        // the ONLY place it becomes radians, to build the basis. 90 deg maps the
        // x-axis onto the y-axis: basis = [cos, sin, -sin, cos] = [0, 1, -1, 0].
        // Read as radians this would be a meaningless ~1.4 rad rotation.
        let s = Stream::new(3)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]))
            .with("rot", Column::Scalar(vec![0.0, 90.0, 180.0]));
        let out = lower_to_instances(&s);
        assert_eq!(out[0].basis, RenderInstance::IDENTITY_BASIS, "0 deg");
        let b = out[1].basis;
        assert!(
            b[0].abs() < 1e-6 && (b[1] - 1.0).abs() < 1e-6,
            "90 deg: cos 0, sin 1"
        );
        assert!(
            (b[2] + 1.0).abs() < 1e-6 && b[3].abs() < 1e-6,
            "90 deg: -sin -1, cos 0"
        );
        let b = out[2].basis;
        assert!(
            (b[0] + 1.0).abs() < 1e-6 && b[1].abs() < 1e-6,
            "180 deg: cos -1, sin 0"
        );
    }

    #[test]
    fn lower_into_matches_fresh_and_reuses_capacity() {
        let s = Stream::new(3).with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]));
        let fresh = lower_to_instances(&s);
        let mut buf = Vec::new();
        lower_to_instances_into(&s, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0], &mut buf);
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
        lower_to_instances_into(&s2, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0], &mut buf);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0].world_pos, [9.0, 9.0]);
        assert!(
            buf.capacity() >= cap_before,
            "capacity retained, not shrunk"
        );
    }

    #[test]
    fn empty_stream_yields_no_instances() {
        assert!(lower_to_instances(&Stream::new(0)).is_empty());
    }

    #[test]
    fn uv_rect_and_size_columns_override_the_defaults_else_fall_back() {
        // `uv_rect` + `size` columns (M1 framing producer) win per-instance…
        let rect = [0.10, 0.20, 0.30, 0.40];
        let s = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
            .with("uv_rect", Column::Vec4(vec![rect, rect]))
            .with("size", Column::Vec2(vec![[3.0, 3.0], [3.0, 3.0]]));
        let mut out = Vec::new();
        // …even when the caller supplies different defaults.
        lower_to_instances_into(&s, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0], &mut out);
        assert_eq!(out[0].atlas_uv, rect);
        assert_eq!(out[1].atlas_uv, rect);
        assert_eq!(out[0].size, [3.0, 3.0]);

        // No `uv_rect`/`size` columns → the caller's defaults (M0: the shell's
        // atlas tile + a sub-spacing size, so the raw default document reads as
        // clean, distinct quads rather than a merged band).
        let tile = [0.0, 0.0, 0.0078125, 0.0078125];
        let dot = [0.5, 0.5];
        let s2 = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
        lower_to_instances_into(&s2, tile, dot, &mut out);
        assert_eq!(out[0].atlas_uv, tile);
        assert_eq!(out[0].size, dot);
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
    fn several_sinks_compose_into_one_instance_buffer() {
        // Two independent sinks (here the same source cooked twice — the shell
        // wires two `motion.output` nodes) append into one buffer, in order.
        // The capacity is retained across pumps: no per-frame heap traffic.
        let mut g = Graph::new();
        let a = g.add_node("motion.test.src");
        let b = g.add_node("motion.test.src");
        let mut pump = MotionCookPump::new();
        let (uv, size) = ([0.0, 0.0, 1.0, 1.0], [1.0, 1.0]);

        assert!(pump.pump(&g, &Ops, &[a, b], 0, 0.0, uv, size));
        assert_eq!(pump.instances.len(), 4, "both sinks drew (2 + 2)");
        assert_eq!(pump.instances[0].world_pos, [0.0, 0.0]);
        assert_eq!(
            pump.instances[2].world_pos,
            [0.0, 0.0],
            "second sink follows"
        );

        let cap = pump.instances.capacity();
        assert!(pump.pump(&g, &Ops, &[a], 1, 0.0, uv, size));
        assert_eq!(
            pump.instances.len(),
            2,
            "dropping a sink drops its instances"
        );
        assert!(pump.instances.capacity() >= cap, "capacity retained");
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

    #[test]
    fn pump_cooks_on_change_and_skips_a_paused_frame() {
        let mut g = Graph::new();
        let src = g.add_node("motion.test.src");
        let mut pump = MotionCookPump::new();
        let uv = [0.0, 0.0, 1.0, 1.0];
        let size = [1.0, 1.0];

        // First pump (dirty from `new`) cooks the sink.
        assert!(pump.pump(&g, &Ops, &[src], 0, 0.0, uv, size));
        assert_eq!(pump.instances.len(), 2);

        // Same tick, clean → skip (a paused, unchanged frame).
        assert!(!pump.pump(&g, &Ops, &[src], 0, 0.0, uv, size));

        // The tick advanced (playing / scrub) → cook.
        assert!(pump.pump(&g, &Ops, &[src], 1, 0.0, uv, size));
        // …then hold at that tick → skip again.
        assert!(!pump.pump(&g, &Ops, &[src], 1, 0.0, uv, size));

        // A graph edit forces a re-cook at the SAME tick.
        pump.mark_dirty();
        assert!(pump.pump(&g, &Ops, &[src], 1, 0.0, uv, size));

        // No sink clears the buffer (still counts as a cook).
        assert!(pump.pump(&g, &Ops, &[], 2, 0.0, uv, size));
        assert!(pump.instances.is_empty());
    }
}
