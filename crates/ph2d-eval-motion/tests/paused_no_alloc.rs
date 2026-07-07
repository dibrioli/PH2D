//! Motion Nodes M0.T12 — paused-frame zero-alloc gate.
//!
//! Plan §1.8: "paused = 0 allocs". The per-frame [`MotionCookPump`] re-cooks the
//! sink only when the frame is dirty; a paused, unchanged frame (same transport
//! tick, no edit) skips the cook and reuses the instance buffer. This proves
//! that skip path allocates nothing across many frames.
//!
//! Counted with `dhat`, bracketed like editor-core's `interaction_no_alloc`
//! bench: the warm-up cook runs BEFORE the profiler window, so only the paused
//! hot loop is observed. (A single cook DOES allocate — the `Cook` re-evaluates
//! each node; the whole point of the pump is to skip that while paused.)

use dhat::{HeapStats, Profiler};
use ph2d_eval_motion::MotionCookPump;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const FRAMES: usize = 30;
/// Paused → the transport tick is identical every frame.
const TICK: u64 = 0;
const PLAYHEAD: f64 = 0.0;
const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SIZE: [f32; 2] = [0.5, 0.5];

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_transform::register(&mut reg).unwrap();
    ph2d_node_motion_clone::register(&mut reg).unwrap();
    reg
}

/// The default document's vertical: grid (3×3) → transform (identity) → clone
/// (×3). Returns the clone sink.
fn default_vertical(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    let xf = g.add_node("motion.transform");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (grid, 0),
        to: (xf, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (xf, 0),
        to: (clone, 0),
        delayed: false,
    })
    .unwrap();
    clone
}

#[test]
fn paused_frames_allocate_nothing() {
    let reg = registry();
    let mut g = Graph::new();
    let sink = default_vertical(&mut g);
    g.validate(&reg).expect("default vertical is well-typed");

    let mut pump = MotionCookPump::new();

    // Warm: the first pump cooks (dirty from `new`) + fills the buffer to its
    // steady capacity. A second pump at the same tick must already skip.
    assert!(
        pump.pump(&g, &reg, Some(sink), TICK, PLAYHEAD, UV, SIZE),
        "first pump cooks"
    );
    assert_eq!(pump.instances.len(), 27, "grid 3×3 × clone ×3");
    assert!(
        !pump.pump(&g, &reg, Some(sink), TICK, PLAYHEAD, UV, SIZE),
        "a second paused frame at the same tick skips the cook"
    );
    let cap_before = pump.instances.capacity();

    // Measure: only the paused hot loop.
    let profiler = Profiler::builder().testing().build();
    let before = HeapStats::get();
    for _ in 0..FRAMES {
        let cooked = pump.pump(&g, &reg, Some(sink), TICK, PLAYHEAD, UV, SIZE);
        debug_assert!(!cooked);
    }
    let after = HeapStats::get();
    drop(profiler);

    let blocks = after.total_blocks - before.total_blocks;
    let bytes = after.total_bytes - before.total_bytes;

    assert_eq!(
        pump.instances.capacity(),
        cap_before,
        "the reused buffer must not reallocate on a paused frame"
    );
    assert_eq!(
        blocks, 0,
        "paused frames allocated {blocks} block(s) / {bytes} bytes across {FRAMES} frames — \
         the pump is re-cooking a static frame instead of skipping it"
    );
}
