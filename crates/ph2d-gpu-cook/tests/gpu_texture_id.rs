//! **The device carries the real `texture_id`, and the run partition matches it**
//! (this wave — the GPU render of a `source.object` graph).
//!
//! A `source.object` has no GPU kernel, so it is a CPU BOUNDARY: the object's
//! instance stream (carrying its tile/individual `texture_id`) is cooked on the
//! CPU and handed to the GPU suffix. A per-element deformer copies every column
//! but its own, so `texture_id` reaches the sink in place, and the lowering
//! writes it into word 41 of each `RenderInstance` (metade A). In parallel the
//! cook partitions the instance range into contiguous same-`texture_id` runs
//! from that same boundary column (metade B), and the two MUST describe the same
//! texture sequence — otherwise the renderer would bind a texture the device
//! buffer doesn't wear.
//!
//! This gate cooks that Hybrid path on a real device with a MANUAL boundary (a
//! `source.object` can't run headless — its eval reads the host texture store,
//! which no test has), reads the instances back, and asserts:
//!   1. word 41 == the object ids fed in — **red before this wave**, when the
//!      lowering hardcoded `instances[base + 41u] = 0u` (the `word41=0` the smoke
//!      probe measured, i.e. white atlas quads);
//!   2. `GpuCook::texture_runs()` partitions those SAME ids — the single door
//!      the renderer binds textures from, proven to agree with the device.
//!
//! `#[ignore]`: needs a real adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_texture_id --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan, read_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_render::GpuTexRun;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    // The object boundary (no GPU kernel), a per-element deformer, the sink.
    ph2d_node_source_object::register(&mut reg).unwrap();
    ph2d_node_motion_bend::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    // For the cerca test: a count-changing deformer + its side-port driver.
    ph2d_node_motion_kaleidoscope::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    reg
}

fn connect(g: &mut Graph, from: ph2d_nodegraph::graph::NodeId, fp: u16, to: ph2d_nodegraph::graph::NodeId, tp: u16) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .unwrap();
}

/// **The count-changing cerca reads each node's own declaration, on the PORT-0
/// path** (this wave, and the fix that keeps it from over-recusing). A
/// per-element deformer suffix on an object preserves position →
/// `suffix_changes_count` is false → the object stays on the GPU. The
/// count-changing `kaleidoscope` (`StreamOp::SourceRows`) → true → the object
/// recuses. ⚠️ A `value.lfo` driving the deformer's amount sits on a SIDE port
/// (its own `count_law` is `Some`), so scanning EVERY stage would recuse the
/// common animated case — the walk follows only port 0, so it does not.
///
/// Not `#[ignore]`: pure plan analysis, no device. The object boundary is real
/// (so no generator is a port-0 stage — a generator's count_law sets, it does
/// not change an existing count).
#[test]
fn a_reordering_object_suffix_recuses_but_a_per_element_or_side_driven_one_does_not() {
    let reg = registry();

    // object → bend → output — per-element, position preserved.
    let mut g = Graph::new();
    let obj = g.add_node("source.object");
    let bend = g.add_node("motion.bend");
    let out = g.add_node("motion.output");
    g.set_param(bend, "angle", 30.0);
    connect(&mut g, obj, 0, bend, 0);
    connect(&mut g, bend, 0, out, 0);
    let bp = plan(&g, &reg, &reg, out);
    assert_eq!(bp.boundaries, vec![(obj, 0)], "the object is the boundary");
    assert!(
        !bp.suffix_changes_count(&reg),
        "a bend suffix on an object preserves position → stays on the GPU"
    );

    // object → bend[amount ← value.lfo] → output — the LFO is a SIDE input
    // (port 1); its own count_law must NOT recuse the object.
    let mut d = Graph::new();
    let dobj = d.add_node("source.object");
    let dbend = d.add_node("motion.bend");
    let lfo = d.add_node("value.lfo");
    let dout = d.add_node("motion.output");
    connect(&mut d, dobj, 0, dbend, 0);
    connect(&mut d, lfo, 0, dbend, 1); // bend's `amount` side port
    connect(&mut d, dbend, 0, dout, 0);
    let dp = plan(&d, &reg, &reg, dout);
    assert!(
        !dp.suffix_changes_count(&reg),
        "a side-port value.lfo must not recuse an object (it is not on the port-0 path)"
    );

    // object → kaleidoscope → output — StreamOp SourceRows on port 0.
    let mut k = Graph::new();
    let kobj = k.add_node("source.object");
    let kal = k.add_node("motion.kaleidoscope");
    let spin = k.add_node("value.lfo");
    let kout = k.add_node("motion.output");
    k.set_param(kal, "segments", 8.0);
    connect(&mut k, kobj, 0, kal, 0);
    connect(&mut k, spin, 0, kal, 1);
    connect(&mut k, kal, 0, kout, 0);
    let kp = plan(&k, &reg, &reg, kout);
    assert!(
        kp.suffix_changes_count(&reg),
        "a kaleidoscope suffix (StreamOp SourceRows) on port 0 → the object recuses"
    );
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];

#[test]
#[ignore = "needs a GPU adapter"]
fn the_device_carries_the_real_texture_id_and_the_run_partition_matches() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_texture_id");
        return;
    };
    let reg = registry();

    // object → bend → output.
    let mut g = Graph::new();
    let obj = g.add_node("source.object");
    let bend = g.add_node("motion.bend");
    let out = g.add_node("motion.output");
    g.set_param(bend, "angle", 30.0);
    g.connect(Edge {
        from: (obj, 0),
        to: (bend, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (bend, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();

    let p = plan(&g, &reg, &reg, out);
    // The object is the CPU boundary (it has no kernel) → Hybrid, not fully-GPU.
    assert_eq!(p.boundaries, vec![(obj, 0)], "the object is the boundary");
    assert!(!p.is_fully_gpu());
    // A per-element deformer suffix preserves position, so the cerca does NOT
    // recuse it — the boundary `texture_id` stays aligned with the sink.
    assert!(
        !p.suffix_changes_count(&reg),
        "a bend suffix must preserve position (no recusal)"
    );

    // Feed the boundary by hand: two objects, three instances each (K=2 runs),
    // laid on a small line so bend has a non-degenerate X extent.
    let ids = [7.0f32, 7.0, 7.0, 9.0, 9.0, 9.0];
    let n = ids.len();
    let ps: Vec<[f32; 2]> = (0..n).map(|i| [i as f32 * 0.5 - 1.25, 0.0]).collect();
    let mut boundary = Stream::new(n);
    boundary.set("P", Column::Vec2(ps));
    boundary.set("texture_id", Column::Scalar(ids.to_vec()));

    let mut gc = GpuCook::new();
    gc.cook(
        &gpu,
        &g,
        &reg,
        &reg,
        &p,
        &[(obj, &boundary)],
        CookClock {
            playhead: 0.0,
            tick: None,
        },
        DEFAULT_UV,
        DEFAULT_SIZE,
    )
    .expect("gpu cook");

    // Metade A — the device instance carries the REAL texture_id (through bend's
    // column passthrough), not the atlas hardcode. RED before this wave.
    let inst = read_instances(&gpu, gc.instances().expect("cooked"));
    assert_eq!(inst.len(), n, "one instance per boundary element");
    let got: Vec<u32> = inst.iter().map(|i| i.texture_id).collect();
    assert_eq!(
        got,
        vec![7, 7, 7, 9, 9, 9],
        "the device must write each object's texture_id into word 41, not 0 \
         (word41=0 is the atlas hardcode → white quads)"
    );

    // Metade B — the run partition (CPU-side, no readback) describes the SAME
    // texture sequence the device carries. The single door proven to agree.
    assert_eq!(
        gc.texture_runs(),
        [
            GpuTexRun {
                texture_id: 7,
                start: 0,
                end: 3
            },
            GpuTexRun {
                texture_id: 9,
                start: 3,
                end: 6
            },
        ],
        "the texture runs must partition the same ids the device wrote to word 41"
    );

    eprintln!(
        "gpu_texture_id: {n} instances, word41 = {got:?}, runs = {:?}",
        gc.texture_runs()
    );
}
