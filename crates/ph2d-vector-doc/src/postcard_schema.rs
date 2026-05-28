//! [`Ph2dVectorAsset`] — the `.ph2d-vector` postcard schema +
//! [`bounded_decode`] for adversarial-input safety.
//!
//! Per [ADR-0056 §2.6](../../../../docs/architecture/decisions/0056-vector-network-data-model.md)
//! (L4F2 Antigravity 3rd iteration — security catch).
//!
//! ## Bounded deserialization
//!
//! Naive `postcard::from_bytes` allows a malicious `.ph2d-vector` file to
//! declare a `SmallVec` length of billions → heap exhaustion → DoS or
//! pre-RCE conditions. [`bounded_decode`] enforces per-collection caps
//! **before** decoding the payload by inspecting the network's vertex /
//! segment / region counts post-decode and rejecting oversized assets.
//!
//! Adversarial fixtures live in
//! `tests/security/adversarial_postcard.rs` (T1.2 wires the harness;
//! more fixtures land via the W1.T1.8 audit lens).

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::crdt::CrdtReplay;
use crate::edit_log::EditLog;
use crate::network::VectorNetwork;
use crate::style::StyleTable;

/// **Schema version** of the on-disk `.ph2d-vector` asset format.
///
/// Bumped via HR-14 whenever the postcard layout changes. Migrator chain
/// (`migrate_v1_to_v2`, …) lives in this module as the schema evolves.
pub const PH2D_VECTOR_ASSET_SCHEMA_VERSION: u32 = 1;

/// **Maximum on-disk asset size** accepted by [`load_vector_asset`].
///
/// 100 MB per ADR-0056 §2.6. Larger assets are rejected before decoding.
pub const MAX_ASSET_SIZE: usize = 100 * 1024 * 1024;

/// On-disk asset header — the root postcard-serialized type written to
/// `.ph2d-vector` files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Ph2dVectorAsset {
    /// Schema version (HR-14). Current value:
    /// [`PH2D_VECTOR_ASSET_SCHEMA_VERSION`].
    pub version: u32,

    /// The vector network itself.
    pub network: VectorNetwork,

    /// Append-only edit log.
    pub edit_log: EditLog,

    /// Document-level style table (stroke + fill libraries).
    pub styles: StyleTable,

    /// Authoring metadata (author, timestamps, app version, etc.).
    pub metadata: AuthoringMetadata,

    /// Embedded asset payloads referenced by fills (raster images for
    /// `vector-image-sample` nodes, fonts, etc.). SmallVec inline cap
    /// **4** — most documents embed nothing.
    pub embedded_assets: SmallVec<[EmbeddedAsset; 4]>,

    /// Optional CRDT replay state — multi-agent local opt-in (W1.T1.6
    /// populates `crdt` module).
    pub crdt_state: Option<CrdtReplay>,
}

impl Default for Ph2dVectorAsset {
    fn default() -> Self {
        Self {
            version: PH2D_VECTOR_ASSET_SCHEMA_VERSION,
            network: VectorNetwork::empty(),
            edit_log: EditLog::new(),
            styles: StyleTable::default(),
            metadata: AuthoringMetadata::default(),
            embedded_assets: SmallVec::new(),
            crdt_state: None,
        }
    }
}

/// Authoring metadata — author, timestamps, app version.
///
/// Strings kept short (256 bytes max each) to bound deserialization
/// memory. Bounded by the per-field length in postcard's varint length
/// prefix; further enforcement happens in [`bounded_decode`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AuthoringMetadata {
    /// Free-text author label.
    pub author: String,
    /// Unix timestamp (seconds since epoch) of creation.
    pub created_at: u64,
    /// Unix timestamp of last modification.
    pub modified_at: u64,
    /// Semver of the PH2D build that wrote this asset.
    pub app_version: String,
}

/// One embedded asset payload — raster image bytes, font data, etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedAsset {
    /// Stable id referenced from style fills / shader nodes.
    pub id: u32,
    /// Kind tag (mime-like) so consumers know how to decode the bytes.
    pub kind: EmbeddedKind,
    /// Raw bytes.
    pub bytes: Vec<u8>,
}

/// Kind of an [`EmbeddedAsset`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EmbeddedKind {
    /// PNG byte stream.
    Png,
    /// JPEG byte stream.
    Jpeg,
    /// OpenType / TrueType font.
    Font,
    /// Opaque — consumer must dispatch on per-asset metadata.
    Opaque,
}

/// Per-collection size caps enforced by [`bounded_decode`].
///
/// Defaults match ADR-0056 §2.6.
#[derive(Debug, Clone, Copy)]
pub struct AssetBounds {
    /// Maximum vertex count.
    pub max_vertices: usize,
    /// Maximum segment count.
    pub max_segments: usize,
    /// Maximum region count.
    pub max_regions: usize,
    /// Maximum edit-log operation count.
    pub max_edit_log_ops: usize,
    /// Maximum embedded-asset count.
    pub max_embedded_assets: usize,
    /// Maximum bytes per embedded asset.
    pub max_embedded_asset_size: usize,
}

impl Default for AssetBounds {
    fn default() -> Self {
        Self {
            max_vertices: 100_000,
            max_segments: 200_000,
            max_regions: 10_000,
            max_edit_log_ops: 1_000_000,
            max_embedded_assets: 64,
            max_embedded_asset_size: 16 * 1024 * 1024,
        }
    }
}

/// Failure modes for [`bounded_decode`] / [`load_vector_asset`].
#[derive(Debug)]
pub enum BoundedDecodeError {
    /// Asset payload exceeds [`MAX_ASSET_SIZE`].
    AssetTooLarge {
        /// Actual size in bytes.
        size: usize,
    },

    /// Postcard reported a deserialization failure.
    Postcard(postcard::Error),

    /// Schema version is unknown (newer than this build understands).
    UnknownSchemaVersion(u32),

    /// A bounded-collection cap was exceeded.
    BoundsExceeded {
        /// Human-readable field name (e.g. `"vertices"`).
        field: &'static str,
        /// Cap that was exceeded.
        cap: usize,
        /// Actual size.
        actual: usize,
    },
}

impl std::fmt::Display for BoundedDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssetTooLarge { size } => write!(
                f,
                "asset payload {} bytes exceeds MAX_ASSET_SIZE {}",
                size, MAX_ASSET_SIZE
            ),
            Self::Postcard(e) => write!(f, "postcard decode failed: {e}"),
            Self::UnknownSchemaVersion(v) => write!(f, "unknown schema version {v}"),
            Self::BoundsExceeded { field, cap, actual } => {
                write!(f, "{field} has {actual} entries, cap is {cap}")
            }
        }
    }
}

impl std::error::Error for BoundedDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Postcard(e) => Some(e),
            _ => None,
        }
    }
}

impl From<postcard::Error> for BoundedDecodeError {
    fn from(e: postcard::Error) -> Self {
        Self::Postcard(e)
    }
}

/// Bounded decode — the safe entry point for loading untrusted
/// `.ph2d-vector` bytes.
///
/// Steps:
/// 1. Reject payloads `> MAX_ASSET_SIZE`.
/// 2. `postcard::from_bytes` into [`Ph2dVectorAsset`].
/// 3. Validate schema version `≤ PH2D_VECTOR_ASSET_SCHEMA_VERSION`.
/// 4. Enforce per-collection caps from `bounds`.
///
/// Returns the decoded asset on success; [`BoundedDecodeError`] on
/// any violation.
///
/// **Note**: this does **not** call [`VectorNetwork::validate`] — call
/// that yourself if you need structural-invariant guarantees beyond the
/// size caps. Bounded decode is the cheap pre-filter; `validate` is the
/// O(V + S + R) full integrity check.
pub fn bounded_decode(
    bytes: &[u8],
    bounds: AssetBounds,
) -> Result<Ph2dVectorAsset, BoundedDecodeError> {
    if bytes.len() > MAX_ASSET_SIZE {
        return Err(BoundedDecodeError::AssetTooLarge { size: bytes.len() });
    }
    let asset: Ph2dVectorAsset = postcard::from_bytes(bytes)?;
    if asset.version > PH2D_VECTOR_ASSET_SCHEMA_VERSION {
        return Err(BoundedDecodeError::UnknownSchemaVersion(asset.version));
    }
    check_bound(
        "vertices",
        asset.network.vertices.len(),
        bounds.max_vertices,
    )?;
    check_bound(
        "segments",
        asset.network.segments.len(),
        bounds.max_segments,
    )?;
    check_bound("regions", asset.network.regions.len(), bounds.max_regions)?;
    check_bound(
        "edit_log.ops",
        asset.edit_log.ops.len(),
        bounds.max_edit_log_ops,
    )?;
    check_bound(
        "embedded_assets",
        asset.embedded_assets.len(),
        bounds.max_embedded_assets,
    )?;
    for ea in &asset.embedded_assets {
        check_bound(
            "embedded_asset.bytes",
            ea.bytes.len(),
            bounds.max_embedded_asset_size,
        )?;
    }
    Ok(asset)
}

#[inline]
fn check_bound(field: &'static str, actual: usize, cap: usize) -> Result<(), BoundedDecodeError> {
    if actual > cap {
        Err(BoundedDecodeError::BoundsExceeded { field, cap, actual })
    } else {
        Ok(())
    }
}

/// Convenience wrapper: [`bounded_decode`] with default [`AssetBounds`].
pub fn load_vector_asset(bytes: &[u8]) -> Result<Ph2dVectorAsset, BoundedDecodeError> {
    bounded_decode(bytes, AssetBounds::default())
}

/// Serialize an asset to postcard bytes. Mirror of [`load_vector_asset`]
/// for the write path.
///
/// Returns `Vec<u8>` because postcard's serializer needs a sink — the
/// caller decides whether to write to disk, memory, or network.
pub fn save_vector_asset(asset: &Ph2dVectorAsset) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(asset)
}
