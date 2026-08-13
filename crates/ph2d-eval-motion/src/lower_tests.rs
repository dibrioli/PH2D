//! ADR-0154 gates for the `geometry_id` lowering convention — the sibling of
//! `texture_id` (doc 86). A row whose `geometry_id > 0` is a crisp vector shape
//! (lowered to a [`VectorInstance`] the shell draws through `ph2d-vec-render`); a
//! row of 0, or no column at all, is a sprite. The convention is ADDITIVE: a
//! stream without the column lowers exactly as it did before shapes existed.

use crate::lower::{lower_to_instances_onto, lower_to_vector_instances_onto};
use crate::{Column, RenderInstance, Stream, VectorInstance};

const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SZ: [f32; 2] = [1.0, 1.0];

/// A stream with no `geometry_id` column lowers to ALL sprites and ZERO vectors —
/// byte-identical to the pre-shape world. FALSIFIED by a lowering that invents a
/// vector where the convention column is absent.
#[test]
fn a_stream_without_geometry_id_is_all_sprites_and_no_vectors() {
    let s = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, 0, &mut sprites);
    assert_eq!(sprites.len(), 3, "every row is a sprite");
    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, &mut vectors);
    assert!(vectors.is_empty(), "no geometry_id column ⇒ no vectors");
}

/// A mixed stream SPLITS by `geometry_id`: rows of 0 are sprites, rows > 0 are
/// vectors — each side keeping its own rows, in order. FALSIFIED by an inverted
/// filter (the split is the whole convention).
#[test]
fn geometry_id_splits_sprites_from_vectors() {
    // Rows 0 & 2 are sprites (id 0); rows 1 & 3 are shapes (id 5, 3).
    let s = Stream::new(4)
        .with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        )
        .with("geometry_id", Column::Scalar(vec![0.0, 5.0, 0.0, 3.0]));

    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, 0, &mut sprites);
    assert_eq!(sprites.len(), 2, "the two id-0 rows are sprites");
    assert_eq!(sprites[0].world_pos, [0.0, 0.0]);
    assert_eq!(sprites[1].world_pos, [2.0, 0.0]);

    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, &mut vectors);
    assert_eq!(vectors.len(), 2, "the two id>0 rows are vectors");
    assert_eq!(vectors[0].geometry_id, 5);
    assert_eq!(vectors[0].world_pos, [1.0, 0.0]);
    assert_eq!(vectors[1].geometry_id, 3);
    assert_eq!(vectors[1].world_pos, [3.0, 0.0]);
}

/// The `geometry_id` and `texture_id` conventions COMPOSE: a shape row carries a
/// live `geometry_id` AND is skipped by the sprite lowering, so a shape is never
/// ALSO stamped as a shared-atlas quad (the doc-86 pattern, one axis over).
#[test]
fn a_shape_row_is_not_also_a_sprite() {
    let s = Stream::new(1)
        .with("P", Column::Vec2(vec![[7.0, 8.0]]))
        .with("geometry_id", Column::Scalar(vec![9.0]));
    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, 0, &mut sprites);
    assert!(sprites.is_empty(), "a shape row is not a sprite");
    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, &mut vectors);
    assert_eq!(vectors.len(), 1);
    assert_eq!(vectors[0].geometry_id, 9);
}
