//! R4 audit Lens-G remediation tests — exercises the new ergonomic
//! helpers introduced post-R4 (HIGH-G1/G2/G3 + MED-G4/G5/G6/G9/G10).
//!
//! These tests double as living usage examples for downstream tools
//! (T1.5 Pen, T2.1 Pencil, T2.3 Select, T1.7 shell bridge).

use glam::Vec2;
use ph2d_vector_doc::{
    BoundedDecodeError, EditLog, FillSolid, LoadAndValidateError, Ph2dVectorAsset, Region, Segment,
    StrokeStyle, StyleTable, TangentSide, TangentsCubic, VectorNetwork, VectorOp,
    VectorOpApplyError, Vertex, VertexKind, WindingRule, load_and_validate_vector_asset,
    save_vector_asset,
};
use smallvec::smallvec;

// ----------------------------------------------------------------------------
// HIGH-G1 — VectorNetwork::next_*_id helpers
// ----------------------------------------------------------------------------

#[test]
fn next_vertex_id_returns_0_on_empty_network() {
    let net = VectorNetwork::empty();
    assert_eq!(net.next_vertex_id(), 0);
    assert_eq!(net.next_segment_id(), 0);
    assert_eq!(net.next_region_id(), 0);
}

#[test]
fn next_vertex_id_returns_max_plus_one() {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(0, Vec2::ZERO));
    net.vertices.push(Vertex::auto(5, Vec2::ONE));
    net.vertices.push(Vertex::auto(2, Vec2::X));
    assert_eq!(net.next_vertex_id(), 6);
}

#[test]
fn next_segment_id_skips_holes_and_returns_max_plus_one() {
    let mut net = VectorNetwork::empty();
    net.segments.push(Segment::straight(0, 0, 1));
    net.segments.push(Segment::straight(7, 1, 2));
    assert_eq!(net.next_segment_id(), 8);
}

#[test]
fn next_region_id_works() {
    let mut net = VectorNetwork::empty();
    net.regions.push(Region::new(3, WindingRule::NonZero));
    net.regions.push(Region::new(1, WindingRule::EvenOdd));
    assert_eq!(net.next_region_id(), 4);
}

// ----------------------------------------------------------------------------
// HIGH-G2 — StyleTable::insert_fill / insert_stroke
// ----------------------------------------------------------------------------

#[test]
fn style_table_insert_fill_auto_allocates_id_starting_at_0() {
    let mut t = StyleTable::default();
    let id_a = t.insert_fill(FillSolid::default());
    let id_b = t.insert_fill(FillSolid::default());
    assert_eq!(id_a, 0);
    assert_eq!(id_b, 1);
    assert_eq!(t.fills.len(), 2);
}

#[test]
fn style_table_insert_stroke_auto_allocates_id() {
    let mut t = StyleTable::default();
    let id_a = t.insert_stroke(StrokeStyle::default());
    let id_b = t.insert_stroke(StrokeStyle::default());
    assert_eq!(id_a, 0);
    assert_eq!(id_b, 1);
}

#[test]
fn style_table_insert_after_manual_id_skips_to_max_plus_one() {
    let mut t = StyleTable::default();
    t.fills.insert(10, FillSolid::default());
    let next = t.insert_fill(FillSolid::default());
    assert_eq!(next, 11);
}

// ----------------------------------------------------------------------------
// HIGH-G3 — VectorOp::apply_to_network + EditLog::push_and_apply
// ----------------------------------------------------------------------------

#[test]
fn apply_add_vertex_mutates_network() {
    let mut net = VectorNetwork::empty();
    let op = VectorOp::AddVertex {
        id: 7,
        pos: Vec2::new(1.0, 2.0),
        kind: VertexKind::Mirror,
    };
    op.apply_to_network(&mut net).expect("apply");
    assert_eq!(net.vertices.len(), 1);
    assert_eq!(net.vertices[0].id, 7);
    assert_eq!(net.vertices[0].pos, Vec2::new(1.0, 2.0));
    assert_eq!(net.vertices[0].kind, VertexKind::Mirror);
}

#[test]
fn apply_move_vertex_updates_position() {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(0, Vec2::ZERO));
    let op = VectorOp::MoveVertex {
        id: 0,
        new_pos: Vec2::splat(10.0),
    };
    op.apply_to_network(&mut net).expect("apply");
    assert_eq!(net.vertices[0].pos, Vec2::splat(10.0));
}

#[test]
fn apply_move_vertex_errors_on_unknown_id() {
    let mut net = VectorNetwork::empty();
    let op = VectorOp::MoveVertex {
        id: 99,
        new_pos: Vec2::ZERO,
    };
    assert_eq!(
        op.apply_to_network(&mut net),
        Err(VectorOpApplyError::UnknownVertex(99))
    );
}

#[test]
fn apply_remove_vertex_drops_vertex() {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(0, Vec2::ZERO));
    net.vertices.push(Vertex::auto(1, Vec2::ONE));
    let op = VectorOp::RemoveVertex { id: 0 };
    op.apply_to_network(&mut net).expect("apply");
    assert_eq!(net.vertices.len(), 1);
    assert_eq!(net.vertices[0].id, 1);
}

#[test]
fn apply_remove_vertex_errors_on_unknown_id() {
    let mut net = VectorNetwork::empty();
    let op = VectorOp::RemoveVertex { id: 99 };
    assert_eq!(
        op.apply_to_network(&mut net),
        Err(VectorOpApplyError::UnknownVertex(99))
    );
}

#[test]
fn apply_add_segment_with_tangents() {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(0, Vec2::ZERO));
    net.vertices.push(Vertex::auto(1, Vec2::X));
    let op = VectorOp::AddSegment {
        id: 0,
        start: 0,
        end: 1,
        tangents: TangentsCubic {
            out_at_start: Vec2::new(0.3, 0.5),
            in_at_end: Vec2::new(-0.3, 0.5),
        },
    };
    op.apply_to_network(&mut net).expect("apply");
    assert_eq!(net.segments.len(), 1);
    assert_eq!(net.segments[0].out_at_start, Vec2::new(0.3, 0.5));
    assert_eq!(net.segments[0].in_at_end, Vec2::new(-0.3, 0.5));
}

#[test]
fn apply_move_tangent_updates_segment() {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(0, Vec2::ZERO));
    net.vertices.push(Vertex::auto(1, Vec2::X));
    net.segments.push(Segment::straight(0, 0, 1));
    let op = VectorOp::MoveTangent {
        seg: 0,
        which: TangentSide::OutAtStart,
        new_pos: Vec2::new(5.0, 5.0),
    };
    op.apply_to_network(&mut net).expect("apply");
    assert_eq!(net.segments[0].out_at_start, Vec2::new(5.0, 5.0));
}

#[test]
fn apply_set_region_fill() {
    let mut net = VectorNetwork::empty();
    net.regions.push(Region::new(0, WindingRule::NonZero));
    let op = VectorOp::SetRegionFill {
        id: 0,
        fill: Some(42),
    };
    op.apply_to_network(&mut net).expect("apply");
    assert_eq!(net.regions[0].fill, Some(42));
}

#[test]
fn apply_returns_needs_asset_context_for_apply_boolean() {
    let mut net = VectorNetwork::empty();
    let op = VectorOp::ApplyBoolean {
        op: ph2d_vector_doc::BooleanOp::Union,
        regions: smallvec![0, 1],
        result_id: 2,
    };
    assert_eq!(
        op.apply_to_network(&mut net),
        Err(VectorOpApplyError::NeedsAssetContext)
    );
}

#[test]
fn apply_returns_needs_asset_context_for_set_stroke_style() {
    let mut net = VectorNetwork::empty();
    let op = VectorOp::SetStrokeStyle {
        seg: 0,
        style: StrokeStyle::default(),
    };
    assert_eq!(
        op.apply_to_network(&mut net),
        Err(VectorOpApplyError::NeedsAssetContext)
    );
}

#[test]
fn push_and_apply_atomic_does_not_log_on_apply_failure() {
    // Log/network drift prevention: failed apply must NOT push to log.
    let mut net = VectorNetwork::empty();
    let mut log = EditLog::new();
    let result = log.push_and_apply(
        VectorOp::MoveVertex {
            id: 99,
            new_pos: Vec2::ZERO,
        },
        &mut net,
    );
    assert!(result.is_err());
    assert_eq!(log.ops.len(), 0, "failed apply must NOT push to log");
}

#[test]
fn push_and_apply_succeeds_logs_and_mutates() {
    let mut net = VectorNetwork::empty();
    let mut log = EditLog::new();
    log.push_and_apply(
        VectorOp::AddVertex {
            id: 0,
            pos: Vec2::ZERO,
            kind: VertexKind::Auto,
        },
        &mut net,
    )
    .expect("apply");
    assert_eq!(log.ops.len(), 1);
    assert_eq!(net.vertices.len(), 1);
}

// ----------------------------------------------------------------------------
// MED-G4 — EditLog::pop
// ----------------------------------------------------------------------------

#[test]
fn edit_log_pop_returns_last_op() {
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
    let popped = log.pop().expect("pop");
    assert!(matches!(popped, VectorOp::AddVertex { id: 1, .. }));
    assert_eq!(log.ops.len(), 1);
}

#[test]
fn edit_log_pop_returns_none_when_empty() {
    let mut log = EditLog::new();
    assert!(log.pop().is_none());
}

// ----------------------------------------------------------------------------
// MED-G6 — load_and_validate_vector_asset + LoadAndValidateError
// ----------------------------------------------------------------------------

#[test]
fn load_and_validate_passes_for_clean_asset() {
    let asset = Ph2dVectorAsset::default();
    let bytes = save_vector_asset(&asset).expect("serialize");
    let loaded = load_and_validate_vector_asset(&bytes).expect("load+validate");
    assert_eq!(loaded, asset);
}

#[test]
fn load_and_validate_fails_with_invariant_on_dangling_segment() {
    let mut asset = Ph2dVectorAsset::default();
    asset.network.vertices.push(Vertex::auto(0, Vec2::ZERO));
    // Segment 0 starts at non-existent vertex 99.
    asset.network.segments.push(Segment::straight(0, 99, 0));
    let bytes = save_vector_asset(&asset).expect("serialize");
    let err = load_and_validate_vector_asset(&bytes).expect_err("should fail validate");
    assert!(
        matches!(err, LoadAndValidateError::Invariant(_)),
        "expected Invariant variant, got {err:?}"
    );
}

#[test]
fn load_and_validate_fails_with_decode_on_oversized_asset() {
    let oversized = vec![0u8; ph2d_vector_doc::MAX_ASSET_SIZE + 1];
    let err = load_and_validate_vector_asset(&oversized).expect_err("should fail decode");
    assert!(
        matches!(
            err,
            LoadAndValidateError::Decode(BoundedDecodeError::AssetTooLarge { .. })
        ),
        "expected Decode AssetTooLarge, got {err:?}"
    );
}

// ----------------------------------------------------------------------------
// MED-G9 — BoundedDecodeError derives (Clone + PartialEq + Eq)
// ----------------------------------------------------------------------------

#[test]
fn bounded_decode_error_is_cloneable_and_eq() {
    let a = BoundedDecodeError::AssetTooLarge { size: 42 };
    let b = a.clone();
    assert_eq!(a, b);
    let c = BoundedDecodeError::UnknownSchemaVersion(7);
    assert_ne!(a, c);
}

// ----------------------------------------------------------------------------
// MED-G10 — Ph2dVectorAsset::from_network helper
// ----------------------------------------------------------------------------

#[test]
fn from_network_wraps_with_default_metadata_and_empty_log() {
    let mut net = VectorNetwork::empty();
    net.vertices.push(Vertex::auto(0, Vec2::ZERO));
    let mut styles = StyleTable::default();
    let _fill = styles.insert_fill(FillSolid::default());
    let asset = Ph2dVectorAsset::from_network(net.clone(), styles.clone());
    assert_eq!(asset.network, net);
    assert_eq!(asset.styles, styles);
    assert_eq!(asset.edit_log.ops.len(), 0);
    assert!(asset.crdt_state.is_none());
    assert!(asset.dormant_fractures.is_none());
}

// ----------------------------------------------------------------------------
// End-to-end: Pen tool 3-click triangle authoring flow using all helpers
// ----------------------------------------------------------------------------

#[test]
fn pen_tool_three_click_triangle_authoring_via_new_helpers() {
    // R4 audit Lens-G smoke: verify the new helpers actually compose
    // cleanly for the T1.5 Pen tool 3-click triangle flow.
    let mut net = VectorNetwork::empty();
    let mut log = EditLog::new();
    let mut styles = StyleTable::default();

    // Click 1: add first vertex.
    let v0 = net.next_vertex_id();
    log.push_and_apply(
        VectorOp::AddVertex {
            id: v0,
            pos: Vec2::ZERO,
            kind: VertexKind::Auto,
        },
        &mut net,
    )
    .unwrap();

    // Click 2: add second vertex + segment connecting.
    let v1 = net.next_vertex_id();
    log.push_and_apply(
        VectorOp::AddVertex {
            id: v1,
            pos: Vec2::new(100.0, 0.0),
            kind: VertexKind::Auto,
        },
        &mut net,
    )
    .unwrap();
    let s0 = net.next_segment_id();
    log.push_and_apply(
        VectorOp::AddSegment {
            id: s0,
            start: v0,
            end: v1,
            tangents: TangentsCubic::ZERO,
        },
        &mut net,
    )
    .unwrap();

    // Click 3: add third vertex + 2 segments closing the triangle.
    let v2 = net.next_vertex_id();
    log.push_and_apply(
        VectorOp::AddVertex {
            id: v2,
            pos: Vec2::new(50.0, 86.6),
            kind: VertexKind::Auto,
        },
        &mut net,
    )
    .unwrap();
    let s1 = net.next_segment_id();
    log.push_and_apply(
        VectorOp::AddSegment {
            id: s1,
            start: v1,
            end: v2,
            tangents: TangentsCubic::ZERO,
        },
        &mut net,
    )
    .unwrap();
    let s2 = net.next_segment_id();
    log.push_and_apply(
        VectorOp::AddSegment {
            id: s2,
            start: v2,
            end: v0,
            tangents: TangentsCubic::ZERO,
        },
        &mut net,
    )
    .unwrap();

    // Close path: add region + auto-allocated fill.
    let fill = styles.insert_fill(FillSolid::default());
    let r0 = net.next_region_id();
    log.push_and_apply(
        VectorOp::AddRegion {
            id: r0,
            segments: smallvec![(s0, true), (s1, true), (s2, true)],
            winding: WindingRule::NonZero,
        },
        &mut net,
    )
    .unwrap();
    log.push_and_apply(
        VectorOp::SetRegionFill {
            id: r0,
            fill: Some(fill),
        },
        &mut net,
    )
    .unwrap();

    // Verify final state: validated triangle network + 8-op log
    // (3 AddVertex + 3 AddSegment + 1 AddRegion + 1 SetRegionFill).
    assert!(net.validate().is_ok());
    assert_eq!(net.vertices.len(), 3);
    assert_eq!(net.segments.len(), 3);
    assert_eq!(net.regions.len(), 1);
    assert_eq!(log.ops.len(), 8);
    assert_eq!(net.regions[0].fill, Some(0));

    // Ctrl+Z: pop last op (SetRegionFill). Caller responsible for
    // reverting effect on `net` — pre-CRDT contract.
    let undone = log.pop().expect("pop");
    assert!(matches!(undone, VectorOp::SetRegionFill { .. }));

    // Save asset using from_network helper.
    let asset = Ph2dVectorAsset::from_network(net, styles);
    let bytes = save_vector_asset(&asset).expect("save");
    let loaded = load_and_validate_vector_asset(&bytes).expect("load+validate");
    assert_eq!(loaded.network.vertices.len(), 3);
    assert_eq!(loaded.styles.fills.len(), 1);
}
