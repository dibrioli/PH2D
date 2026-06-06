#![forbid(unsafe_code)]
//! `vector.llm-shape` — LLM4SVG semantic tokens → editable [`VectorNetwork`]
//! (Inovação #4, [ADR-0061](../../../docs/architecture/decisions/0061-vector-llm-authoring.md) §2.1).
//!
//! The in-engine half of LLM-as-graph-node: a thin `Effect::Pure` wrapper over
//! [`ph2d_vector_llm::build_network_from_json`] (parse → sanitize → lower), so a
//! model-authored shape becomes a standard, **editable** `VectorNetwork` the
//! user keeps editing downstream — never an opaque SVG dump.
//!
//! ## Where the response comes from (the deferred contract, decided here)
//!
//! The LLM *call* is async host I/O — `ph2d-vector-llm-client` (15 s timeout +
//! `ResultCache` fallback). It is **not** in this Pure node; the node consumes an
//! **already-cached** response. But the node-graph substrate carries only `f32`
//! params and type-erased opaque *edges* (ADR-0030 / ADR-0058): there is no
//! string param and no host-set graph input. The single seam by which
//! per-instance, non-`f32` data can reach a `Pure` op is its
//! [`OpResolver`](ph2d_nodegraph::cook::OpResolver) — so the host supplies an op
//! that carries the response cache. Concretely:
//!
//! - a `seed` `f32` param identifies *which* cached response this instance wants
//!   (the host assigns a distinct integer seed per llm-shape node, and
//!   `ph2d-vector-llm-client` keys its `ResultCache` accordingly);
//! - the host implements [`LlmResponseSource`] over that cache and resolves
//!   `vector.llm-shape` to [`LlmShapeOp::new(source)`](LlmShapeOp::new) via its
//!   own `OpResolver`, layered over the base
//!   [`NodeRegistry`](ph2d_node_registry::NodeRegistry);
//! - the op [`register`]ed for node-sync/discovery uses [`NoCache`] — it emits an
//!   empty network until a host supplies a real cache, so headless / test cooks
//!   and the registry stay valid with zero wiring.
//!
//! This keeps the node `Effect::Pure`: given `seed` + the cache snapshot read at
//! cook time, the output is a deterministic function of `build_network_from_json`
//! (itself pure + total). When a re-author changes the response, the host bumps
//! the seed (a new response = a new key), so cook memoization stays correct.
//!
//! ## Security (ADR-0061 §2.4) is inherited, not re-implemented
//!
//! `build_network_from_json` is the bounds-*before*-allocation sanitizer: a
//! `turns: 1e9` / billion-vertex blob is **rejected, never materialized**. A
//! rejected (or missing) blob lowers to an empty network here — the node is total
//! and never panics.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};
use ph2d_vector_llm::build_network_from_json;

/// The node's static type manifest: no inputs, one geometry output, one `seed`
/// param, `Pure` + `Static` (re-cooked only on a param edit). Eight fields — at
/// the frozen `NodeManifest` cap (ADR-0039), like every other node.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.llm-shape"),
    name: "vector.llm-shape",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VECTOR_PORT,
    }],
    effect: Effect::Pure,
    clock: Clock::Static,
    params: &[ParamSpec {
        name: "seed",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// The host's bridge to the cached LLM4SVG responses (populated off-thread by
/// `ph2d-vector-llm-client`). Keyed by the node instance's `seed` param.
///
/// Read-only at cook time — that is what keeps [`LlmShapeOp`] `Effect::Pure`.
pub trait LlmResponseSource: Send + Sync {
    /// The cached LLM4SVG JSON blob for `seed`, or `None` if not yet fetched.
    fn response_for(&self, seed: u64) -> Option<&str>;
}

/// The empty source: every lookup misses. Backs the default [`register`]ed op so
/// a graph cooked through the plain registry (headless / tests / discovery)
/// yields an empty network instead of requiring host wiring.
pub struct NoCache;

impl LlmResponseSource for NoCache {
    fn response_for(&self, _seed: u64) -> Option<&str> {
        None
    }
}

/// The node op: reads `seed`, looks up the cached blob via `S`, lowers it with
/// [`build_network_from_json`], and emits the resulting [`VectorNetwork`]. A
/// missing or rejected blob lowers to an empty network (total, never panics).
pub struct LlmShapeOp<S: LlmResponseSource> {
    source: S,
}

impl<S: LlmResponseSource> LlmShapeOp<S> {
    /// Wrap a response source. The host calls this with its cache-backed source
    /// and resolves `vector.llm-shape` to the result via its own `OpResolver`.
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S: LlmResponseSource + 'static> NodeOp for LlmShapeOp<S> {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // `seed` is an instance index, not a magnitude: clamp the f32 to a
        // non-negative integer key. (f32 represents integer seeds exactly well
        // past any practical node count.)
        let seed = ctx.param("seed").max(0.0) as u64;
        let net = self
            .source
            .response_for(seed)
            .and_then(|json| build_network_from_json(json).ok())
            .unwrap_or_else(VectorNetwork::empty);
        ctx.emit_network(net);
    }
}

/// Register the node type with [`NoCache`] (see module docs — the host swaps in a
/// cache-backed op via its own `OpResolver`). Wired into `register_all_nodes` by
/// `cargo run -p ph2d-node-sync`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(LlmShapeOp::new(NoCache)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    const HEXAGON: &str = r#"{ "shape_type": "polygon", "params": { "sides": 6, "radius": 80 } }"#;

    /// A test cache mapping seed 7 → a hexagon; every other seed misses.
    struct OneShape;
    impl LlmResponseSource for OneShape {
        fn response_for(&self, seed: u64) -> Option<&str> {
            (seed == 7).then_some(HEXAGON)
        }
    }

    /// Resolve `vector.llm-shape` to a given op — mirrors what the host's layered
    /// `OpResolver` does in production.
    struct HostOps<S: LlmResponseSource + 'static>(LlmShapeOp<S>);
    impl<S: LlmResponseSource + 'static> OpResolver for HostOps<S> {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&self.0 as &dyn NodeOp)
        }
    }

    /// Cook a single llm-shape node at `seed` and return its emitted network.
    fn cook_seed<S: LlmResponseSource + 'static>(ops: &HostOps<S>, seed: f32) -> VectorNetwork {
        let mut g = Graph::new();
        let n = g.add_node("vector.llm-shape");
        g.set_param(n, "seed", seed);
        let mut cook = Cook::new();
        let out = cook.cook(&g, ops, n, 0.0).unwrap();
        out[0]
            .as_any()
            .and_then(|a| a.downcast_ref::<VectorNetwork>())
            .cloned()
            .expect("port 0 carries a VectorNetwork")
    }

    #[test]
    fn registers() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }

    #[test]
    fn cached_blob_lowers_to_editable_geometry() {
        let ops = HostOps(LlmShapeOp::new(OneShape));
        let net = cook_seed(&ops, 7.0);
        assert!(net.validate().is_ok());
        assert_eq!(
            net.vertices.len(),
            6,
            "the cached hexagon, editable downstream"
        );
    }

    #[test]
    fn missing_cache_entry_is_empty_not_a_panic() {
        let ops = HostOps(LlmShapeOp::new(OneShape));
        let net = cook_seed(&ops, 999.0); // no entry for seed 999
        assert!(net.vertices.is_empty());
    }

    #[test]
    fn default_nocache_op_emits_empty() {
        // The registered default op (NoCache) never has a response → empty.
        let ops = HostOps(LlmShapeOp::new(NoCache));
        let net = cook_seed(&ops, 0.0);
        assert!(net.vertices.is_empty());
    }

    #[test]
    fn adversarial_blob_lowers_to_empty_not_materialized() {
        // turns: 1e9 → the sanitizer rejects before any geometry exists.
        struct Evil;
        impl LlmResponseSource for Evil {
            fn response_for(&self, _seed: u64) -> Option<&str> {
                Some(r#"{ "shape_type": "spiral", "params": { "turns": 1000000000 } }"#)
            }
        }
        let ops = HostOps(LlmShapeOp::new(Evil));
        let net = cook_seed(&ops, 0.0);
        assert!(
            net.vertices.is_empty(),
            "billion-vertex spiral rejected, never materialized"
        );
    }
}
