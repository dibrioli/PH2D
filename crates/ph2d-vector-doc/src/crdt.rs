//! CRDT replay state — **stub** for W1.T1.6.
//!
//! Per [ADR-0057](../../../../docs/architecture/decisions/0057-vector-edit-dispatch-crdt.md):
//! hybrid CRDT = LWW-Element-Set (set membership) + RGA (vertex /
//! segment ordering in regions) + custom per-component LWW (tangent
//! handles).
//!
//! T1.6 populates the real `apply` / `merge` / `replay` machinery; this
//! stub exists so [`crate::Ph2dVectorAsset`] can reference the type
//! today without forward-declaring through every file.

use serde::{Deserialize, Serialize};

/// Replayable CRDT state — placeholder for the real LWW + RGA + custom
/// machinery (W1.T1.6).
///
/// W1 single-user mode leaves `crdt_state = None` in
/// [`crate::Ph2dVectorAsset`]; the type exists so the schema is
/// forward-compatible (no need to bump
/// `PH2D_VECTOR_ASSET_SCHEMA_VERSION` when T1.6 fleshes this out, as
/// long as the inner serde shape stays compatible).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CrdtReplay {
    /// Site id — unique per editing replica (assigned at session start).
    pub site_id: u64,

    /// Highest sequence number seen from each peer site.
    /// `BTreeMap` (HR-5) so iteration order is deterministic.
    pub peer_clocks: std::collections::BTreeMap<u64, u64>,
}

impl CrdtReplay {
    /// Construct an empty replay state for the given site.
    #[must_use]
    pub fn new(site_id: u64) -> Self {
        Self {
            site_id,
            peer_clocks: std::collections::BTreeMap::new(),
        }
    }
}
