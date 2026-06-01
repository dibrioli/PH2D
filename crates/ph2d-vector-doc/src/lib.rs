#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! `ph2d-vector-doc` — Vector Module document data model.
//!
//! Foundational crate per
//! [ADR-0056](../../../docs/architecture/decisions/0056-vector-network-data-model.md).
//! Defines the canonical [`VectorNetwork`], its constituent
//! [`Vertex`] / [`Segment`] / [`Region`], the event-sourced
//! [`EditLog`] of [`VectorOp`]s, and the bounded postcard schema
//! [`Ph2dVectorAsset`].
//!
//! 31+ downstream crates (`ph2d-node-vector-*`, `ph2d-tool-vector-*`,
//! `ph2d-panel-vector-*`, the Vello render bridge in `ph2d-vector`,
//! etc.) consume these types — the contract surface is **frozen**
//! by [`crate::tests`] arch-gate and any expansion requires an
//! ADR-0056 amendment.
//!
//! ## Module map
//!
//! | Module | Purpose |
//! |---|---|
//! | [`network`]        | [`VectorNetwork`] graph + [`validate`](VectorNetwork::validate). |
//! | [`cubic`]          | [`Vertex`] / [`VertexKind`] / [`TangentsCubic`] / [`TangentSide`]. |
//! | [`region`]         | [`Region`] / [`WindingRule`] / [`SegmentRef`]. |
//! | [`edit_log`]       | [`VectorOp`] enum + [`EditLog`] event sourcing. |
//! | [`style`]          | [`StyleRef`] / [`FillRef`] / [`StyleTable`] / [`StrokeStyle`] / supporting types. |
//! | [`cubic_fit`]      | Levien single-cubic Bézier fitting ([`fit_cubic_levien`](cubic_fit::fit_cubic_levien)). |
//! | [`hobby`]          | Hobby interpolating spline ([`fit_hobby_open`](hobby::fit_hobby_open)) — Pencil tool fitter. |
//! | [`spiro`]          | Spiro / Hyperbezier Assist Modes (stub — W2+ populates). |
//! | [`postcard_schema`]| [`Ph2dVectorAsset`] + [`bounded_decode`](postcard_schema::bounded_decode). |
//! | [`deterministic`]  | Q16.16 fixed-point opt-in (per ADR-0056 §2.7). |
//! | [`crdt`]           | CRDT replay (stub — W1.T1.6 populates). |
//!
//! ## Hard rules referenced
//!
//! - **HR-3**: zero-alloc render hot path. `validate()` is on the editor
//!   write path (not the render tick) so allocation is permitted there;
//!   per-frame draw routines (in `ph2d-vector`) consume the network
//!   read-only.
//! - **HR-5**: no `HashMap` in simulation. [`style::StyleTable`] uses
//!   [`std::collections::BTreeMap`] for deterministic iteration order.
//! - **HR-14**: postcard schema is versioned (`Ph2dVectorAsset.version: u32`).

pub mod crdt;
pub mod cubic;
pub mod cubic_fit;
pub mod deterministic;
pub mod dormant;
pub mod edit_log;
pub mod hit_test;
pub mod hobby;
pub mod network;
pub mod postcard_schema;
pub mod region;
pub mod spiro;
pub mod style;

pub use crdt::CrdtReplay;
pub use cubic::{TangentSide, TangentsCubic, Vertex, VertexId, VertexKind};
pub use dormant::DormantFractureSet;
pub use edit_log::{BooleanOp, EditLog, NetworkSnapshot, VectorOp, VectorOpApplyError};
pub use network::{
    RepresentationMode, VECTOR_NETWORK_SCHEMA_VERSION, VectorNetwork, VectorNetworkInvariant,
};
pub use postcard_schema::{
    AssetBounds, AuthoringMetadata, BoundedDecodeError, EmbeddedAsset, EmbeddedKind,
    LoadAndValidateError, MAX_ASSET_SIZE, PH2D_VECTOR_ASSET_SCHEMA_VERSION, Ph2dVectorAsset,
    bounded_decode, bounded_encode, load_and_validate_vector_asset, load_vector_asset,
    save_vector_asset,
};
pub use region::{Region, RegionId, SegmentRef, WindingRule};
pub use style::{
    FillRef, FillSolid, Segment, SegmentId, StrokeCap, StrokeJoin, StrokeStyle, StyleRef,
    StyleRefMap, StyleTable,
};
