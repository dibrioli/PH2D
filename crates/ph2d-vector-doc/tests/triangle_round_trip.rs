//! Smoke test for T1.2 acceptance criterion #3:
//! "criar triangle network, serialize, deserialize, hash match".
//!
//! Builds a 3-vertex / 3-segment / 1-region triangle network, runs
//! `validate()` (structural invariant), round-trips it through postcard
//! via [`save_vector_asset`] / [`load_vector_asset`], and asserts the
//! decoded network equals the original.

use glam::Vec2;
use ph2d_vector_doc::postcard_schema::AssetBounds;
use ph2d_vector_doc::{
    BooleanOp, EditLog, FillRef, Ph2dVectorAsset, Region, RepresentationMode, Segment, StrokeStyle,
    TangentSide, TangentsCubic, VectorNetwork, VectorOp, Vertex, VertexKind, WindingRule,
    bounded_decode, load_vector_asset, save_vector_asset,
};
use smallvec::smallvec;

fn build_triangle() -> VectorNetwork {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
    net.vertices.push(Vertex::auto(1, Vec2::new(100.0, 0.0)));
    net.vertices.push(Vertex::auto(2, Vec2::new(50.0, 86.6)));

    net.segments.push(Segment::straight(0, 0, 1));
    net.segments.push(Segment::straight(1, 1, 2));
    net.segments.push(Segment::straight(2, 2, 0));

    let mut region = Region::new(0, WindingRule::NonZero);
    region
        .segments
        .extend_from_slice(&[(0, true), (1, true), (2, true)]);
    net.regions.push(region);
    net
}

#[test]
fn triangle_network_validates() {
    let net = build_triangle();
    assert!(
        net.validate().is_ok(),
        "valid triangle should pass validate()"
    );
}

#[test]
fn triangle_asset_round_trips_through_postcard() {
    let mut asset = Ph2dVectorAsset::default();
    asset.network = build_triangle();

    let bytes = save_vector_asset(&asset).expect("serialize");
    let decoded = load_vector_asset(&bytes).expect("deserialize");

    assert_eq!(decoded.version, asset.version);
    assert_eq!(decoded.network, asset.network);
}

#[test]
fn triangle_serialization_is_byte_stable_across_two_calls() {
    let mut asset = Ph2dVectorAsset::default();
    asset.network = build_triangle();
    let a = save_vector_asset(&asset).expect("serialize a");
    let b = save_vector_asset(&asset).expect("serialize b");
    assert_eq!(
        a, b,
        "postcard output should be deterministic for the same input"
    );
}

#[test]
fn validate_catches_dangling_segment_start() {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(0, Vec2::ZERO));
    // segment references vertex 99 which doesn't exist
    net.segments.push(Segment::straight(0, 99, 0));
    let err = net.validate().unwrap_err();
    assert!(
        matches!(
            err,
            ph2d_vector_doc::VectorNetworkInvariant::DanglingSegmentStart {
                segment: 0,
                vertex: 99
            }
        ),
        "expected dangling-start, got {err:?}"
    );
}

#[test]
fn validate_catches_duplicate_vertex_id() {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(7, Vec2::ZERO));
    net.vertices.push(Vertex::auto(7, Vec2::ONE));
    let err = net.validate().unwrap_err();
    assert!(
        matches!(
            err,
            ph2d_vector_doc::VectorNetworkInvariant::DuplicateVertexId(7)
        ),
        "expected duplicate-vertex, got {err:?}"
    );
}

#[test]
fn bounded_decode_rejects_oversized_vertex_count() {
    // Build a network with 11 vertices and a bounds cap of 10.
    let mut net = VectorNetwork::empty();
    for i in 0..11_u32 {
        net.vertices.push(Vertex::auto(i, Vec2::new(i as f32, 0.0)));
    }
    let mut asset = Ph2dVectorAsset::default();
    asset.network = net;
    let bytes = save_vector_asset(&asset).expect("serialize");

    let bounds = AssetBounds {
        max_vertices: 10,
        ..AssetBounds::default()
    };
    let err = bounded_decode(&bytes, bounds).expect_err("should reject");
    match err {
        ph2d_vector_doc::BoundedDecodeError::BoundsExceeded { field, cap, actual } => {
            assert_eq!(field, "vertices");
            assert_eq!(cap, 10);
            assert_eq!(actual, 11);
        }
        other => panic!("expected BoundsExceeded, got {other:?}"),
    }
}

#[test]
fn vector_op_variants_round_trip_through_postcard() {
    // Exercises every VectorOp variant through postcard to catch any
    // variant that breaks serde derivation (e.g. by referencing a
    // non-Serialize type).
    let ops = vec![
        VectorOp::AddVertex {
            id: 1,
            pos: Vec2::ONE,
            kind: VertexKind::Auto,
        },
        VectorOp::MoveVertex {
            id: 1,
            new_pos: Vec2::splat(2.0),
        },
        VectorOp::RemoveVertex { id: 1 },
        VectorOp::AddSegment {
            id: 0,
            start: 0,
            end: 1,
            tangents: TangentsCubic::ZERO,
        },
        VectorOp::MoveTangent {
            seg: 0,
            which: TangentSide::OutAtStart,
            new_pos: Vec2::Y,
        },
        VectorOp::RemoveSegment { id: 0 },
        VectorOp::AddRegion {
            id: 0,
            segments: smallvec![(0, true), (1, true)],
            winding: WindingRule::NonZero,
        },
        VectorOp::SetRegionFill {
            id: 0,
            fill: Some(7 as FillRef),
        },
        VectorOp::ApplyBoolean {
            op: BooleanOp::Union,
            regions: smallvec![0, 1],
            result_id: 2,
        },
        VectorOp::CrdtMerge {
            peer_id: 42,
            peer_seq: 100,
        },
        VectorOp::SetAuthoringHint {
            mode: RepresentationMode::SpiroAssist,
        },
        VectorOp::SetStrokeStyle {
            seg: 0,
            style: StrokeStyle::default(),
        },
        VectorOp::Checkpoint { hash: [0xAB; 32] },
    ];

    for op in ops {
        let bytes = postcard::to_allocvec(&op).expect("serialize op");
        let back: VectorOp = postcard::from_bytes(&bytes).expect("deserialize op");
        assert_eq!(op, back, "round-trip mismatch on {op:?}");
    }
}

#[test]
fn edit_log_appends_and_round_trips() {
    let mut log = EditLog::new();
    log.push(VectorOp::AddVertex {
        id: 0,
        pos: Vec2::ZERO,
        kind: VertexKind::Auto,
    });
    log.push(VectorOp::AddVertex {
        id: 1,
        pos: Vec2::X,
        kind: VertexKind::Auto,
    });
    assert_eq!(log.ops.len(), 2);

    let bytes = postcard::to_allocvec(&log).expect("serialize log");
    let back: EditLog = postcard::from_bytes(&bytes).expect("deserialize log");
    assert_eq!(log, back);
}

#[test]
fn empty_default_asset_round_trips() {
    let asset = Ph2dVectorAsset::default();
    let bytes = save_vector_asset(&asset).expect("serialize");
    let decoded = load_vector_asset(&bytes).expect("deserialize");
    assert_eq!(decoded, asset);
}

#[test]
fn bounded_decode_rejects_asset_too_large() {
    // Build a >100 MB payload (just zeros — postcard will fail to parse,
    // but the size check fires first).
    let oversized = vec![0u8; ph2d_vector_doc::postcard_schema::MAX_ASSET_SIZE + 1];
    let err = bounded_decode(&oversized, AssetBounds::default()).expect_err("should reject");
    match err {
        ph2d_vector_doc::BoundedDecodeError::AssetTooLarge { size } => {
            assert_eq!(size, oversized.len());
        }
        other => panic!("expected AssetTooLarge, got {other:?}"),
    }
}

#[test]
fn bounded_decode_rejects_unknown_outer_schema_version() {
    // Forge an asset with version = 999 (future schema we don't understand).
    let mut asset = Ph2dVectorAsset::default();
    asset.version = 999;
    let bytes = save_vector_asset(&asset).expect("serialize");
    let err = bounded_decode(&bytes, AssetBounds::default()).expect_err("should reject");
    assert!(
        matches!(
            err,
            ph2d_vector_doc::BoundedDecodeError::UnknownSchemaVersion(999)
        ),
        "expected UnknownSchemaVersion(999), got {err:?}"
    );
}

#[test]
fn bounded_decode_rejects_unknown_inner_network_version() {
    // R1 audit Lens-B HIGH-3: an asset legitimate at version=1 can still
    // carry a network forged at version=999.
    let mut asset = Ph2dVectorAsset::default();
    asset.network.version = 999;
    let bytes = save_vector_asset(&asset).expect("serialize");
    let err = bounded_decode(&bytes, AssetBounds::default()).expect_err("should reject");
    assert!(
        matches!(
            err,
            ph2d_vector_doc::BoundedDecodeError::UnknownSchemaVersion(999)
        ),
        "expected UnknownSchemaVersion(999) on inner network, got {err:?}"
    );
}

#[test]
fn bounded_decode_rejects_oversized_author_string() {
    let mut asset = Ph2dVectorAsset::default();
    asset.metadata.author = "x".repeat(300); // > default 256
    let bytes = save_vector_asset(&asset).expect("serialize");
    let err = bounded_decode(&bytes, AssetBounds::default()).expect_err("should reject");
    match err {
        ph2d_vector_doc::BoundedDecodeError::BoundsExceeded { field, cap, actual } => {
            assert_eq!(field, "metadata.author");
            assert_eq!(cap, 256);
            assert_eq!(actual, 300);
        }
        other => panic!("expected BoundsExceeded, got {other:?}"),
    }
}

#[test]
fn bounded_decode_rejects_oversized_peer_clocks_map() {
    use ph2d_vector_doc::crdt::CrdtReplay;
    let mut asset = Ph2dVectorAsset::default();
    let mut crdt = CrdtReplay::new(7);
    for i in 0..2000_u64 {
        crdt.peer_clocks.insert(i, 0);
    }
    asset.crdt_state = Some(crdt);
    let bytes = save_vector_asset(&asset).expect("serialize");
    let err = bounded_decode(&bytes, AssetBounds::default()).expect_err("should reject");
    match err {
        ph2d_vector_doc::BoundedDecodeError::BoundsExceeded { field, cap, actual } => {
            assert_eq!(field, "crdt_state.peer_clocks");
            assert_eq!(cap, 1024);
            assert_eq!(actual, 2000);
        }
        other => panic!("expected BoundsExceeded, got {other:?}"),
    }
}

#[test]
fn asset_bounds_defaults_match_adr_0056_section_2_6() {
    // R1 audit Lens-A MED-2: pin the ADR-0056 §2.6 numerical caps
    // executably so future drift (e.g. someone bumping
    // max_vertices: 100_000 → 1_000_000_000 silently) fails CI.
    let b = AssetBounds::default();
    assert_eq!(b.max_vertices, 100_000);
    assert_eq!(b.max_segments, 200_000);
    assert_eq!(b.max_regions, 10_000);
    assert_eq!(b.max_edit_log_ops, 1_000_000);
    assert_eq!(b.max_embedded_assets, 64);
    assert_eq!(b.max_embedded_asset_size, 16 * 1024 * 1024);
    // R1 audit Lens-D HIGH-D4/D5 + MED extensions.
    assert_eq!(b.max_author_len, 256);
    assert_eq!(b.max_app_version_len, 64);
    assert_eq!(b.max_peer_clocks, 1024);
    assert_eq!(b.max_style_strokes, 4096);
    assert_eq!(b.max_style_fills, 4096);
}

#[test]
fn dormant_fractures_field_round_trips_through_postcard() {
    // R1 audit Lens-A HIGH-1: dormant_fractures field pre-declared in
    // v1 schema to avoid HR-14 version bump in W17+.
    let mut asset = Ph2dVectorAsset::default();
    asset.dormant_fractures = Some(ph2d_vector_doc::DormantFractureSet::default());
    let bytes = save_vector_asset(&asset).expect("serialize");
    let decoded = load_vector_asset(&bytes).expect("deserialize");
    assert_eq!(decoded.dormant_fractures, asset.dormant_fractures);
}
