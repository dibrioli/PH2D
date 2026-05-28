// Audit harness — compile-only Send+Sync verification of W1 ph2d-vector-doc public types.
use ph2d_vector_doc::*;
fn _assert_send_sync<T: Send + Sync>() {}
#[test]
fn vector_doc_data_types_are_send_sync() {
    _assert_send_sync::<VectorNetwork>();
    _assert_send_sync::<Vertex>();
    _assert_send_sync::<VertexKind>();
    _assert_send_sync::<TangentsCubic>();
    _assert_send_sync::<TangentSide>();
    _assert_send_sync::<Segment>();
    _assert_send_sync::<Region>();
    _assert_send_sync::<WindingRule>();
    _assert_send_sync::<RepresentationMode>();
    _assert_send_sync::<VectorNetworkInvariant>();
    _assert_send_sync::<EditLog>();
    _assert_send_sync::<VectorOp>();
    _assert_send_sync::<BooleanOp>();
    _assert_send_sync::<NetworkSnapshot>();
    _assert_send_sync::<Ph2dVectorAsset>();
    _assert_send_sync::<AssetBounds>();
    _assert_send_sync::<AuthoringMetadata>();
    _assert_send_sync::<EmbeddedAsset>();
    _assert_send_sync::<BoundedDecodeError>();
    _assert_send_sync::<StyleTable>();
    _assert_send_sync::<StrokeStyle>();
    _assert_send_sync::<FillSolid>();
    _assert_send_sync::<StyleRefMap>();
    _assert_send_sync::<StrokeCap>();
    _assert_send_sync::<StrokeJoin>();
    _assert_send_sync::<DormantFractureSet>();
}
