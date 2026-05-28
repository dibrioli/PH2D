//! [`DormantFractureSet`] — stub for the Dormant Fracture Edges
//! innovation (ADR-0063 §2.4 + Vector Module README §8.8).
//!
//! Per [ADR-0056 §2.6](../../../../docs/architecture/decisions/0056-vector-network-data-model.md):
//! the `Ph2dVectorAsset.dormant_fractures: Option<DormantFractureSet>`
//! field is pre-declared at W1 so the runtime physics integration
//! (W16+ per ADR-0063) can land without bumping
//! [`crate::postcard_schema::PH2D_VECTOR_ASSET_SCHEMA_VERSION`] from
//! 1 → 2 + writing a migrator chain — exactly the HR-14 cost that
//! pre-declaration eliminates.
//!
//! W1 ships only the empty stub + serde derives so the schema is
//! forward-compatible (serializes as `None`; deserializes any
//! future shape stored in v1 files as the stub for now — actual
//! payload semantics arrive in W16).

use serde::{Deserialize, Serialize};

/// Stub for the dormant-fracture edge set carried by an asset.
///
/// In W16+ this holds the pre-computed fracture lines that a runtime
/// physics cut can split a region along instantly (no Linesweeper
/// recompute mid-frame). W1 carries only `Default` and serde derives
/// so the field round-trips through postcard schema v1.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DormantFractureSet {
    /// Reserved for the W16 fracture-edge payload. Empty stub today —
    /// any pre-W16 caller setting non-default behavior is racing the
    /// schema design.
    pub _stub: (),
}
