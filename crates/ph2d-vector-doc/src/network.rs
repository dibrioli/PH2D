//! [`VectorNetwork`] — the primary topology of the Vector Module.
//!
//! Per [ADR-0056 §2.2 + §2.3](../../../../docs/architecture/decisions/0056-vector-network-data-model.md).
//! SmallVec inline caps: vertices 32, segments 64, regions 8.
//!
//! `VectorNetwork` is intentionally **not** capped at a number of fields —
//!   the ADR's caps target the constituent `Vertex` / `Segment` / `Region`.
//!   The arch-gate audits those.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::cubic::Vertex;
use crate::region::Region;
use crate::style::{Segment, StyleRefMap};

/// **Current schema version** of the [`VectorNetwork`] struct.
///
/// Bumped via [HR-14](../../../../docs/SKILL_Stack_PH2D_Definitiva.md)
/// whenever a postcard-incompatible layout change ships. Migrators in
/// [`crate::postcard_schema`] handle older versions.
pub const VECTOR_NETWORK_SCHEMA_VERSION: u32 = 1;

/// Topologia primária do Vector Module — non-directed graph of vertices,
/// segments, and regions, with per-region fills and auto-resolved
/// crossings (Figma vector-network model).
///
/// **SmallVec inline budgets** (ADR-0056 §2.2):
/// - `vertices`: 32 (covers simple logos / icons without heap allocation)
/// - `segments`: 64
/// - `regions`: 8
///
/// Beyond those caps storage spills to the heap automatically.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VectorNetwork {
    /// Schema version (HR-14). Bump in any postcard-incompatible layout
    /// change. Current value: [`VECTOR_NETWORK_SCHEMA_VERSION`].
    pub version: u32,

    /// Vertices indexed by [`crate::VertexId`].
    pub vertices: SmallVec<[Vertex; 32]>,

    /// Segments indexed by [`crate::SegmentId`].
    pub segments: SmallVec<[Segment; 64]>,

    /// Regions with winding rule + segment refs.
    pub regions: SmallVec<[Region; 8]>,

    /// Per-network style-ref index (deduplicates style data shared by
    /// many segments / regions).
    pub style_refs: StyleRefMap,

    /// Authoring representation hint (cubic = default visible; spiro /
    /// hyperbezier = Assist Modes opt-in per ADR-0056 §2.4).
    pub authoring_hint: RepresentationMode,

    /// Determinism opt-in flag (per ADR-0056 §2.7 + ADR-0021 SimWorld).
    /// When `true`, callers must use Q16.16 fixed-point coords + ordered
    /// reductions + FMA-off shader pragmas.
    pub deterministic: bool,
}

impl Default for VectorNetwork {
    fn default() -> Self {
        Self {
            version: VECTOR_NETWORK_SCHEMA_VERSION,
            vertices: SmallVec::new(),
            segments: SmallVec::new(),
            regions: SmallVec::new(),
            style_refs: StyleRefMap::default(),
            authoring_hint: RepresentationMode::Cubic,
            deterministic: false,
        }
    }
}

impl VectorNetwork {
    /// Construct an empty network at the current schema version.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Return the next unused [`crate::VertexId`] (max + 1, or 0 if empty).
    ///
    /// **W1 ergonomic helper (R4 audit Lens-G HIGH-G1)** for tools
    /// that need to allocate a fresh vertex id during interactive
    /// authoring. **Not CRDT-safe** — concurrent edits from peers can
    /// race to the same id. T1.6 reintroduces a site-id-spaced
    /// allocator via `CrdtReplay::alloc_vertex_id(&self, site)` for
    /// multi-agent sessions; until then, single-user W1 callers may
    /// rely on this.
    ///
    /// O(N_vertices) linear scan; cache the result if calling per-click.
    #[must_use]
    pub fn next_vertex_id(&self) -> crate::cubic::VertexId {
        self.vertices
            .iter()
            .map(|v| v.id)
            .max()
            .map_or(0, |m| m + 1)
    }

    /// Return the next unused [`crate::SegmentId`]. Same CRDT caveat as
    /// [`Self::next_vertex_id`].
    #[must_use]
    pub fn next_segment_id(&self) -> crate::style::SegmentId {
        self.segments
            .iter()
            .map(|s| s.id)
            .max()
            .map_or(0, |m| m + 1)
    }

    /// Return the next unused [`crate::RegionId`]. Same CRDT caveat as
    /// [`Self::next_vertex_id`].
    #[must_use]
    pub fn next_region_id(&self) -> crate::region::RegionId {
        self.regions.iter().map(|r| r.id).max().map_or(0, |m| m + 1)
    }

    /// Validate the network's structural invariants.
    ///
    /// Checks performed:
    /// 1. Every `Segment.start` / `Segment.end` references an existing vertex.
    /// 2. Every `Region.segments[i].0` references an existing segment.
    /// 3. Vertex / segment / region ids are unique within their respective
    ///    collections.
    /// 4. Schema version is `≤ VECTOR_NETWORK_SCHEMA_VERSION` (older OK
    ///    via migrator; newer = unknown future = reject).
    ///
    /// Returns `Ok(())` if all invariants hold, or the first violation
    /// encountered (deterministic order: vertices → segments → regions).
    ///
    /// **Performance note**: O(V + S × log V + R × seg × log S) — uses a
    /// `BTreeSet` of ids for membership testing. Not on the render hot
    /// path; called only on document load + after `EditLog` apply.
    pub fn validate(&self) -> Result<(), VectorNetworkInvariant> {
        use std::collections::BTreeSet;

        if self.version > VECTOR_NETWORK_SCHEMA_VERSION {
            return Err(VectorNetworkInvariant::UnknownSchemaVersion(self.version));
        }

        // 1) Vertex id uniqueness.
        let mut seen_v: BTreeSet<u32> = BTreeSet::new();
        for v in &self.vertices {
            if !seen_v.insert(v.id) {
                return Err(VectorNetworkInvariant::DuplicateVertexId(v.id));
            }
        }

        // 2) Segment id uniqueness + endpoint resolution.
        let mut seen_s: BTreeSet<u32> = BTreeSet::new();
        for s in &self.segments {
            if !seen_s.insert(s.id) {
                return Err(VectorNetworkInvariant::DuplicateSegmentId(s.id));
            }
            if !seen_v.contains(&s.start) {
                return Err(VectorNetworkInvariant::DanglingSegmentStart {
                    segment: s.id,
                    vertex: s.start,
                });
            }
            if !seen_v.contains(&s.end) {
                return Err(VectorNetworkInvariant::DanglingSegmentEnd {
                    segment: s.id,
                    vertex: s.end,
                });
            }
        }

        // 3) Region id uniqueness + segment-ref resolution.
        let mut seen_r: BTreeSet<u32> = BTreeSet::new();
        for r in &self.regions {
            if !seen_r.insert(r.id) {
                return Err(VectorNetworkInvariant::DuplicateRegionId(r.id));
            }
            for &(seg_id, _direction) in &r.segments {
                if !seen_s.contains(&seg_id) {
                    return Err(VectorNetworkInvariant::DanglingRegionSegment {
                        region: r.id,
                        segment: seg_id,
                    });
                }
            }
        }

        Ok(())
    }
}

/// Structural-invariant violation detected by [`VectorNetwork::validate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorNetworkInvariant {
    /// Schema version is newer than what this build understands.
    UnknownSchemaVersion(u32),

    /// Two vertices share the same id.
    DuplicateVertexId(u32),

    /// Two segments share the same id.
    DuplicateSegmentId(u32),

    /// Two regions share the same id.
    DuplicateRegionId(u32),

    /// A segment references a non-existent start vertex.
    DanglingSegmentStart {
        /// Offending segment id.
        segment: u32,
        /// Missing vertex id.
        vertex: u32,
    },

    /// A segment references a non-existent end vertex.
    DanglingSegmentEnd {
        /// Offending segment id.
        segment: u32,
        /// Missing vertex id.
        vertex: u32,
    },

    /// A region references a non-existent segment.
    DanglingRegionSegment {
        /// Offending region id.
        region: u32,
        /// Missing segment id.
        segment: u32,
    },
}

impl std::fmt::Display for VectorNetworkInvariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSchemaVersion(v) => write!(f, "unknown schema version {v}"),
            Self::DuplicateVertexId(id) => write!(f, "duplicate vertex id {id}"),
            Self::DuplicateSegmentId(id) => write!(f, "duplicate segment id {id}"),
            Self::DuplicateRegionId(id) => write!(f, "duplicate region id {id}"),
            Self::DanglingSegmentStart { segment, vertex } => {
                write!(
                    f,
                    "segment {segment} references missing start vertex {vertex}"
                )
            }
            Self::DanglingSegmentEnd { segment, vertex } => {
                write!(
                    f,
                    "segment {segment} references missing end vertex {vertex}"
                )
            }
            Self::DanglingRegionSegment { region, segment } => {
                write!(f, "region {region} references missing segment {segment}")
            }
        }
    }
}

impl std::error::Error for VectorNetworkInvariant {}

/// Authoring representation hint per ADR-0056 §2.4 (decisão D Antigravity).
///
/// - [`Cubic`]: Bézier cúbico = default visível (paridade
///   Illustrator / Affinity / Figma).
/// - [`SpiroAssist`]: Clothoid splines, opt-in via HUD `S` toggle (W2+).
/// - [`HyperbezierAssist`]: Elastica-under-tension, opt-in via HUD `H`
///   toggle (W2+).
///
/// **FROZEN at 3 variants** (ADR-0056 §2.3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepresentationMode {
    /// Bézier cúbico — default visible.
    #[default]
    Cubic,
    /// Spiro / clothoid Assist Mode.
    SpiroAssist,
    /// Hyperbezier / elastica-under-tension Assist Mode.
    HyperbezierAssist,
}
