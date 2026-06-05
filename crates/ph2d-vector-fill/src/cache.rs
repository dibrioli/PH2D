//! In-memory compile cache (ADR-0060 §2.7, spec §5.5).
//!
//! Keyed by [`TopologyHash`], so any pure param/enum animation is a **hit** and
//! never re-runs codegen+validation — the mechanism behind the
//! `procedural_fill_no_recompile_on_animate` gate and the >95% hit-rate target.
//!
//! The LRU is hand-rolled (no `lru` crate) over two `BTreeMap`s — entries keyed
//! by [`TopologyHash`], plus a `tick → hash` recency index for O(log n)
//! eviction. `BTreeMap` (not `HashMap`) per ADR-0022 / HR-5. On-disk persistence
//! (the `directories`-crate path layout of spec §5.5.1) is **deferred** — a new
//! dep is a Coordenador decision (handoff §5), and the in-memory cache already
//! satisfies every W6 gate.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::wgsl_codegen::{self, CompiledFill, TopologyHash};
use crate::{FillCodegenError, FillGraph};

/// Default in-memory budget (ADR-0060 §2.7 cap: 256 MB).
pub const DEFAULT_CACHE_BUDGET_BYTES: usize = 256 * 1024 * 1024;

/// Codegen + naga-validate a graph into a [`CompiledFill`] (no caching). This is
/// the "compile" the cache memoizes; "compile" here is *codegen + WGSL
/// validation*, not GPU pipeline creation (that is the renderer's job).
pub fn compile_fill(graph: &FillGraph, backend_id: &str) -> Result<CompiledFill, FillCodegenError> {
    let wgsl = wgsl_codegen::codegen(graph)?;
    let module_src = wgsl_codegen::validation_module(&wgsl);
    let module = naga::front::wgsl::parse_str(&module_src)
        .map_err(|e| FillCodegenError::NagaParse(e.emit_to_string(&module_src)))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|e| FillCodegenError::NagaValidate(e.emit_to_string(&module_src)))?;
    Ok(CompiledFill {
        wgsl,
        topology_hash: wgsl_codegen::topology_hash(graph, backend_id),
    })
}

struct Entry {
    fill: Arc<CompiledFill>,
    tick: u64,
}

/// LRU compile cache for one backend. Clone the returned [`Arc<CompiledFill>`]
/// freely; the cache retains its own.
pub struct CompileCache {
    backend_id: String,
    map: BTreeMap<TopologyHash, Entry>,
    recency: BTreeMap<u64, TopologyHash>,
    tick: u64,
    bytes: usize,
    budget: usize,
    hits: u64,
    compiles: u64,
}

impl CompileCache {
    /// New cache for `backend_id` (e.g. a wgpu adapter signature) with the
    /// default 256 MB budget.
    pub fn new(backend_id: impl Into<String>) -> Self {
        Self::with_budget(backend_id, DEFAULT_CACHE_BUDGET_BYTES)
    }

    pub fn with_budget(backend_id: impl Into<String>, budget: usize) -> Self {
        Self {
            backend_id: backend_id.into(),
            map: BTreeMap::new(),
            recency: BTreeMap::new(),
            tick: 0,
            bytes: 0,
            budget,
            hits: 0,
            compiles: 0,
        }
    }

    /// Fetch the compiled fill for `graph`, compiling (codegen + validate) only
    /// on a topology miss. A param/enum-only change since a prior call is a hit.
    pub fn get_or_compile(
        &mut self,
        graph: &FillGraph,
    ) -> Result<Arc<CompiledFill>, FillCodegenError> {
        let hash = wgsl_codegen::topology_hash(graph, &self.backend_id);
        if self.map.contains_key(&hash) {
            self.touch(&hash);
            self.hits += 1;
            return Ok(self.map[&hash].fill.clone());
        }
        let fill = Arc::new(compile_fill(graph, &self.backend_id)?);
        self.compiles += 1;
        self.insert(hash, fill.clone());
        Ok(fill)
    }

    /// Number of cache hits since construction.
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Number of real compiles (codegen+validate) since construction — the
    /// recompile counter the no-recompile gate asserts is `1` over an animation.
    pub fn compiles(&self) -> u64 {
        self.compiles
    }

    /// Hit-rate over all lookups (`hits / (hits + compiles)`), or `1.0` if no
    /// lookups happened yet.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.compiles;
        if total == 0 {
            1.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Live entry count.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Approximate retained bytes.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    fn touch(&mut self, hash: &TopologyHash) {
        self.tick += 1;
        let new_tick = self.tick;
        if let Some(entry) = self.map.get_mut(hash) {
            self.recency.remove(&entry.tick);
            entry.tick = new_tick;
            self.recency.insert(new_tick, *hash);
        }
    }

    fn insert(&mut self, hash: TopologyHash, fill: Arc<CompiledFill>) {
        self.tick += 1;
        let tick = self.tick;
        self.bytes += fill.byte_size();
        self.recency.insert(tick, hash);
        self.map.insert(hash, Entry { fill, tick });
        self.evict_to_budget();
    }

    fn evict_to_budget(&mut self) {
        // Never evict the entry just inserted to nothing — keep at least one.
        while self.bytes > self.budget && self.map.len() > 1 {
            let Some((&oldest_tick, &oldest_hash)) = self.recency.iter().next() else {
                break;
            };
            self.recency.remove(&oldest_tick);
            if let Some(entry) = self.map.remove(&oldest_hash) {
                self.bytes = self.bytes.saturating_sub(entry.fill.byte_size());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FillGraph, FillNode, NodeId};
    use ph2d_color::OklchColor;
    use smallvec::smallvec;

    fn solid_graph() -> FillGraph {
        FillGraph {
            nodes: smallvec![FillNode::Solid {
                color: OklchColor::opaque(0.6, 0.1, 120.0)
            }],
            connections: smallvec![],
            output_node_id: NodeId(0),
        }
    }

    #[test]
    fn miss_then_hit() {
        let mut cache = CompileCache::new("test-backend");
        let g = solid_graph();
        let _ = cache.get_or_compile(&g).unwrap();
        assert_eq!(cache.compiles(), 1);
        assert_eq!(cache.hits(), 0);
        let _ = cache.get_or_compile(&g).unwrap();
        assert_eq!(cache.compiles(), 1, "same topology must not recompile");
        assert_eq!(cache.hits(), 1);
    }

    #[test]
    fn param_change_is_a_hit() {
        let mut cache = CompileCache::new("test-backend");
        let mut g = solid_graph();
        let _ = cache.get_or_compile(&g).unwrap();
        // Animate the colour: topology unchanged → must be a hit.
        g.nodes[0] = FillNode::Solid {
            color: OklchColor::opaque(0.2, 0.3, 30.0),
        };
        let _ = cache.get_or_compile(&g).unwrap();
        assert_eq!(cache.compiles(), 1);
        assert_eq!(cache.hits(), 1);
    }

    #[test]
    fn topology_change_recompiles() {
        let mut cache = CompileCache::new("test-backend");
        let g1 = solid_graph();
        let _ = cache.get_or_compile(&g1).unwrap();
        // A second (dead) node: different node count → different topology hash.
        let g2 = FillGraph {
            nodes: smallvec![
                FillNode::Solid {
                    color: OklchColor::opaque(0.6, 0.1, 120.0)
                },
                FillNode::Ramp {
                    palette: smallvec![]
                },
            ],
            connections: smallvec![],
            output_node_id: NodeId(0),
        };
        let _ = cache.get_or_compile(&g2).unwrap();
        assert_eq!(cache.compiles(), 2);
    }
}
