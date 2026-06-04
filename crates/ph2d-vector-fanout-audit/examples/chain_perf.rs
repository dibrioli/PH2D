//! W4 T4.13 perf: time the 6-node geometry chain cooking through the real
//! registry. Run with `--release` (dev is ~7× slower and misleading).
//!
//! `cargo run --release -p ph2d-vector-fanout-audit --example chain_perf`

use std::time::Instant;

use ph2d_node_registry::NodeRegistry;
use ph2d_node_registry_init::register_all_nodes;
use ph2d_nodegraph::cook::Cook;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_fanout_audit::build_chain;

fn main() {
    let mut reg = NodeRegistry::new();
    register_all_nodes(&mut reg).expect("register all nodes");
    let (g, target) = build_chain();
    let mut cook = Cook::new();

    // Cold cook: every node evaluates once.
    let t = Instant::now();
    let out = cook.cook(&g, &reg, target, 0.0).expect("cook");
    let cold = t.elapsed().as_secs_f64() * 1000.0;
    let net = out[0]
        .as_any()
        .and_then(|x| x.downcast_ref::<VectorNetwork>())
        .expect("VectorNetwork");
    let (v, s, r) = (net.vertices.len(), net.segments.len(), net.regions.len());

    // Memoized re-cook (no edit): the Cook memo should make this ~free.
    let t = Instant::now();
    cook.cook(&g, &reg, target, 0.0).expect("re-cook");
    let warm = t.elapsed().as_secs_f64() * 1000.0;

    println!("6-node chain (source→corner-round→mirror→twist→bend→warp):");
    println!("  cold cook:        {cold:.3} ms   (output {v} verts / {s} segs / {r} regions)");
    println!("  memoized re-cook: {warm:.3} ms   (Cook memo hit — no param/input change)");
    println!("frame-budget refs: 120 Hz = 8.3 ms, 60 Hz = 16.6 ms");
}
