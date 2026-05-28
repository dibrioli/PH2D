//! [`DormantFractureSet`] — stub for the Dormant Fracture Edges
//! innovation (ADR-0063 §2.4 + Vector Module README §8.8).
//!
//! Per [ADR-0056 §2.6](../../../../docs/architecture/decisions/0056-vector-network-data-model.md):
//! the `Ph2dVectorAsset.dormant_fractures: Option<DormantFractureSet>`
//! field is pre-declared at W1 so the runtime physics integration
//! (W16+ per ADR-0063) can land **without bumping**
//! [`crate::postcard_schema::PH2D_VECTOR_ASSET_SCHEMA_VERSION`] when
//! the body of `DormantFractureSet` grows internally — postcard is
//! field-positional, so adding sub-fields *inside* a struct that's
//! already on the wire is backward-compatible as long as deserializers
//! consume the same number of top-level fields.
//!
//! ## What this pre-declaration buys (and what it does NOT)
//!
//! - ✓ **Avoids a v1 → v2 bump** at the [`crate::Ph2dVectorAsset`]
//!   *top level* when W16+ adds the real payload — because the field
//!   slot is already on the wire (as `None` in W1 files).
//! - ✗ **Does NOT preserve backward read of pre-R1 v1 files** —
//!   pre-R1 v1 files were serialized without the field; post-R1
//!   readers will hit EOF mid-decode. There are no pre-R1 v1 files
//!   in the wild (greenfield W1), so this is a theoretical concern
//!   that disappears with the commit-line drawing the v1 schema in
//!   the sand.
//!
//! W1 ships only the empty stub + serde derives.

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
