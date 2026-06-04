#![forbid(unsafe_code)]
//! W4 audit harness (T4.13): cook a **6-node** vector geometry chain through the
//! real generated registry, validating that the fan-out composes end-to-end.
//!
//! Chain: `vector.source(polygon)` → `corner-round` → `mirror` → `twist` →
//! `bend-path` → `warp`. Each node is `Effect::Pure` and consumes/emits a
//! `VectorNetwork` on the opaque channel, so they pull-cook in one pass with the
//! `Cook` memoizing unchanged upstream (the "cache by (input, params)").
//!
//! The [`cook_chain`] helper is exercised by the unit test (correctness, runs in
//! CI) and timed by `examples/chain_perf.rs` (perf, run with `--release`).

use ph2d_node_registry::NodeRegistry;
use ph2d_node_registry_init::register_all_nodes;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_vector_doc::VectorNetwork;

/// Build the 6-node chain graph and return it plus the terminal node to cook.
#[must_use]
pub fn build_chain() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("vector.source");
    g.set_param(src, "kind", 2.0); // polygon
    g.set_param(src, "sides", 7.0);
    g.set_param(src, "width", 200.0);

    let chain = [
        "vector.corner-round",
        "vector.mirror",
        "vector.twist",
        "vector.bend-path",
        "vector.warp",
    ];
    let mut prev = src;
    for ty in chain {
        let n = g.add_node(ty);
        g.connect(Edge {
            from: (prev, 0),
            to: (n, 0),
            delayed: false,
        })
        .expect("connect chain edge");
        prev = n;
    }
    (g, prev)
}

/// Cook the chain through the real registry and downcast the result. Panics on
/// any cook/registry error (this is an audit harness, not production).
#[must_use]
pub fn cook_chain() -> VectorNetwork {
    let mut reg = NodeRegistry::new();
    register_all_nodes(&mut reg).expect("register all nodes");
    let (g, target) = build_chain();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, target, 0.0).expect("cook the chain");
    out[0]
        .as_any()
        .and_then(|x| x.downcast_ref::<VectorNetwork>())
        .expect("chain output carries a VectorNetwork")
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_node_chain_cooks_to_a_valid_network() {
        let net = cook_chain();
        assert!(
            net.validate().is_ok(),
            "the 6-node chain output must be valid"
        );
        assert!(!net.regions.is_empty(), "the chain must produce geometry");
        // mirror(Both) 4-ups the corner-rounded heptagon → many regions.
        assert!(net.regions.len() >= 4, "mirror should 4-up the shape");
        assert!(
            net.deterministic,
            "deterministic source → deterministic chain"
        );
    }

    #[test]
    fn chain_is_reproducible() {
        assert_eq!(cook_chain(), cook_chain(), "byte-stable end to end");
    }
}
