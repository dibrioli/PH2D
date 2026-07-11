//! Unit tests for `ph2d-eval-motion` (lowering + `MotionCookPump`), split from
//! `lib.rs` for the HR-18 700-LOC cap. Declared there as a `#[path]` sibling, so
//! `super` is the crate root.

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
