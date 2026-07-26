#![forbid(unsafe_code)]
//! ph2d-vector — Vello GPU vector renderer wrapper (M11).
//!
//! Vello is Linebender's experimental compute-based vector renderer.
//! It is currently 0.x and the API churns roughly per-quarter; we
//! pin to a specific minor version and treat regressions as
//! ADR-worthy events (plan note: "Vello em alpha — risco aceito,
//! ADR-0004 a escrever quando aparecer regressão").
//!
//! M11 ships the wrapper surface only:
//! - [`VectorScene`] — a `vello::Scene` with our naming convention
//!   and a couple of convenience helpers (filled rect, clipped
//!   path push/pop).
//! - The `Renderer` (which owns wgpu pipeline state) is NOT wrapped
//!   yet — callers that need to actually render to a surface go
//!   through `vello::Renderer` directly. M12 (editor) wires it
//!   into the desktop shell with a real surface.
//!
//! Re-exports the kurbo + peniko types that show up in the public
//! API so downstream crates don't need direct deps on every
//! Linebender package.

pub mod scene;
pub mod vector_network;

pub use scene::{StableImage, VectorScene};
pub use vector_network::{
    ProceduralFillImage, build_region_path, draw_variable_width_stroke, draw_vector_network,
    draw_vector_network_with_fills, oklch_to_color, variable_width_band,
};

// Re-export the Vector Module document types so downstream callers
// (e.g. the W1.T1.7 shell bridge) can construct/inspect networks
// without an extra `ph2d-vector-doc` import line. The planned arch-gate
// `vello_kurbo_only_in_ph2d_vector` (ADR-0059 §2.8 + L6F1 long-tail) —
// **planned for W2+, NOT active today** — would force outside crates
// to reach Vello/kurbo via this re-export wall. ~20 pre-existing crates
// already import vello/kurbo direct (ph2d-editor-core, ph2d-render,
// ph2d-imageio-svg, tool/panel crates, etc.); the W2+ gate landing
// will need a whitelist or migration sweep.
pub use ph2d_vector_doc::{
    AssetBounds, AuthoringMetadata, BooleanOp, BoundedDecodeError, CrdtReplay, DormantFractureSet,
    EditLog, EmbeddedAsset, EmbeddedKind, FillRef, FillSolid, LoadAndValidateError, MAX_ASSET_SIZE,
    NetworkSnapshot, PH2D_VECTOR_ASSET_SCHEMA_VERSION, Ph2dVectorAsset, Region, RegionId,
    RepresentationMode, Segment, SegmentId, SegmentRef, StrokeCap, StrokeJoin, StrokeStyle,
    StyleRef, StyleRefMap, StyleTable, TangentSide, TangentsCubic, VECTOR_NETWORK_SCHEMA_VERSION,
    VectorNetwork, VectorNetworkInvariant, VectorOp, VectorOpApplyError, Vertex, VertexId,
    VertexKind, WindingRule, bounded_decode, load_and_validate_vector_asset, load_vector_asset,
    save_vector_asset,
};

// Re-export the few kurbo/peniko primitives that callers need to
// build paths and brushes. We reach for them via `vello::*` rather
// than declaring kurbo/peniko as direct deps — guarantees the
// Brush/Color/Fill types our wrapper hands to vello are the same
// monomorphic types vello expects (no version-skew accidents).
pub use vello::kurbo::{
    Affine, BezPath, Cap, Circle, Join, PathEl, Point, Rect, RoundedRect, Shape, Stroke, Vec2,
};
pub use vello::peniko::{
    Brush, Color, ColorStop, ColorStops, Fill, Gradient, GradientKind, ImageQuality,
    LinearGradientPosition,
};
// Glyph + Scene used by callers that drive `Scene::draw_glyphs`
// directly (text rendering in ph2d-editor::paint). Same version-skew
// rationale as kurbo/peniko above.
pub use vello::{Glyph, Scene};
