//! ADR-0154 gates for the `geometry_id` lowering convention — the sibling of
//! `texture_id` (doc 86). A row whose `geometry_id > 0` is a crisp vector shape
//! (lowered to a [`VectorInstance`] the shell draws through `ph2d-vec-render`); a
//! row of 0, or no column at all, is a sprite. The convention is ADDITIVE: a
//! stream without the column lowers exactly as it did before shapes existed.

use crate::lower::{lower_to_instances_onto, lower_to_vector_instances_onto};
use crate::{Column, RenderInstance, Stream, VectorInstance};
use ph2d_render::SinkStyle;

const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SZ: [f32; 2] = [1.0, 1.0];

/// A stream with no `geometry_id` column lowers to ALL sprites and ZERO vectors —
/// byte-identical to the pre-shape world. FALSIFIED by a lowering that invents a
/// vector where the convention column is absent.
#[test]
fn a_stream_without_geometry_id_is_all_sprites_and_no_vectors() {
    let s = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, SinkStyle::PLAIN, &mut sprites);
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
    lower_to_instances_onto(&s, UV, SZ, SinkStyle::PLAIN, &mut sprites);
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
    lower_to_instances_onto(&s, UV, SZ, SinkStyle::PLAIN, &mut sprites);
    assert!(sprites.is_empty(), "a shape row is not a sprite");
    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, &mut vectors);
    assert_eq!(vectors.len(), 1);
    assert_eq!(vectors[0].geometry_id, 9);
}

/// **A COLUNA `blend` DECIDE POR LINHA, E O `0` É *O DO SINK*** (doc 89, folha 07 — o
/// *Echo Operator*).
///
/// ⚠️ **As três metades numa só, porque separá-las esconderia o defeito caro.** A escada é
/// `0 = o do sink`, `m + 1 = o modo m`; um design que guardasse o modo CRU faria a
/// identidade de junção (`0`) baixar toda linha alheia para `Normal` — em silêncio, e só
/// numa cena que compõe em `Add`.
#[test]
fn a_blend_column_overrides_per_row_and_zero_means_the_sinks_mode() {
    // O sink compõe em `Add` (tag 1). A stream traz a coluna: linha 0 sem escolha (`0`),
    // linha 1 escolhe `Normal` (tag 0 ⇒ valor 1), linha 2 escolhe `Screen` (tag 4 ⇒ 5).
    let s = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
        .with("blend", Column::Scalar(vec![0.0, 1.0, 5.0]));
    let mut out = Vec::new();
    lower_to_instances_onto(
        &s,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        SinkStyle {
            blend: 1,
            ..SinkStyle::PLAIN
        },
        &mut out,
    );
    assert_eq!(out.len(), 3);
    let sink = RenderInstance::pack_blend_bits(1);
    assert_eq!(out[0].flip_uv, sink, "0 na coluna = o modo do SINK");
    assert_eq!(out[1].flip_uv, RenderInstance::pack_blend_bits(0), "Normal");
    assert_eq!(out[2].flip_uv, RenderInstance::pack_blend_bits(4), "Screen");
    assert_ne!(
        out[0].flip_uv, out[1].flip_uv,
        "senao a coluna nao decide nada"
    );
}

/// **SEM A COLUNA, NADA MUDA** — o default byte-idêntico, e o controle do gate acima.
#[test]
fn a_stream_without_the_column_lowers_exactly_as_before() {
    let s = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0]; 2]));
    let mut out = Vec::new();
    lower_to_instances_onto(
        &s,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        SinkStyle {
            blend: 3,
            ..SinkStyle::PLAIN
        },
        &mut out,
    );
    let sink = RenderInstance::pack_blend_bits(3);
    assert!(out.iter().all(|i| i.flip_uv == sink));
}

/// **UM NÚMERO ABSURDO NA COLUNA NÃO ESCOLHE UM PIPELINE QUE NÃO EXISTE.**
///
/// ⚠️ A coluna é escrita por um NÓ, mas nada impede um `value.*` de a produzir — e o
/// `flip_uv` indexa um array de pipelines do renderer. O teto é lido DE LÁ.
#[test]
fn a_wild_column_value_is_clamped_to_the_renderers_pipelines() {
    let top = ph2d_render::pipeline::BLEND_PIPELINE_COUNT as f32;
    let s = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
        .with("blend", Column::Scalar(vec![999.0, -4.0, f32::NAN]));
    let mut out = Vec::new();
    lower_to_instances_onto(
        &s,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        SinkStyle {
            blend: 2,
            ..SinkStyle::PLAIN
        },
        &mut out,
    );
    #[expect(clippy::cast_possible_truncation, reason = "o teto cabe num u8")]
    let last = RenderInstance::pack_blend_bits((top as u8) - 1);
    let sink = RenderInstance::pack_blend_bits(2);
    assert_eq!(out[0].flip_uv, last, "999 satura no ultimo modo REAL");
    assert_eq!(out[1].flip_uv, sink, "negativo = sem escolha = o do sink");
    assert_eq!(
        out[2].flip_uv, sink,
        "NaN idem — nunca um pipeline inventado"
    );
}
