//! The node contract: what an agent implementing a node crate fills in.
//!
//! A node is an FBP black box (ADR-0031): its **only** contract is its ports +
//! effect + clock. It sees nothing of the rest of the graph. That is exactly
//! what makes a node implementable by a parallel agent in isolation, and
//! testable on its own (golden input → golden output).

use crate::cook::EvalCtx;
use crate::effect::Effect;
use crate::port::{Clock, PortType};

/// Stable, deterministic id derived from a node type's canonical name via
/// FNV-1a 64-bit. Matches `ph2d-tool-registry::hash_node_id`; reimplemented
/// dependency-free here so the substrate stays leaf-level. Const so it can be
/// computed in a `const MANIFEST`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeTypeId(pub u64);

impl NodeTypeId {
    pub const fn of(name: &str) -> Self {
        Self(fnv1a_64(name.as_bytes()))
    }
}

const fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        i += 1;
    }
    hash
}

/// One named, typed port on a node.
#[derive(Copy, Clone, Debug)]
pub struct PortSpec {
    pub name: &'static str,
    pub ty: PortType,
}

/// The static descriptor of a node **type** (not an instance). Declared as a
/// `const MANIFEST` in each node crate and registered by codegen (ADR-0031).
#[derive(Clone, Debug)]
pub struct NodeManifest {
    pub id: NodeTypeId,
    /// Canonical name, e.g. `"motion.clone"`. Stable; the id is its hash.
    pub name: &'static str,
    pub inputs: &'static [PortSpec],
    pub outputs: &'static [PortSpec],
    pub effect: Effect,
    /// The clock this node cooks under. Must match its ports' clock unless it
    /// is a membrane-crossing node.
    pub clock: Clock,
}

/// Implemented by every concrete node type. `manifest` is the static contract;
/// `eval` is the pure cook step — it reads typed inputs and emits typed
/// outputs through the [`EvalCtx`], seeing nothing of the rest of the graph
/// (FBP black box, ADR-0031).
pub trait NodeOp: Send + Sync {
    fn manifest(&self) -> &'static NodeManifest;
    fn eval(&self, ctx: &mut EvalCtx<'_>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_stable_and_deterministic() {
        // Same name → same id across runs (FNV-1a is pure).
        assert_eq!(NodeTypeId::of("motion.clone"), NodeTypeId::of("motion.clone"));
        assert_ne!(NodeTypeId::of("motion.clone"), NodeTypeId::of("motion.grid"));
    }

    #[test]
    fn fnv1a_known_vector() {
        // FNV-1a of empty input is the offset basis.
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
    }
}
