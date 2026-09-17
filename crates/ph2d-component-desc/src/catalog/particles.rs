//! ⭐⭐⭐ **O EMISSOR DE PARTÍCULAS** (TOP-20 #18) — a fachada sobre a simulação do Motion.
//!
//! ⚠️ **Categoria `Rendering`, e `O::ANY`:** um emissor não precisa de uma sprite (o fogo de uma
//! tocha vazia é o caso comum), e o que ele faz é sair PIXEL.
//!
//! ⚠️ **Os campos aqui são os do componente, não as linhas do painel** — o painel agrupa-os por
//! módulos (a emissão, a forma, a velocidade, as forças, o aspecto), que é como as quatro
//! referências os mostram; o descritor espelha a ESTRUTURA, porque é ele que dá um `field_id` a
//! cada override da F4.

use crate::{
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, ObjectKinds as O,
    Propagation,
};

const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
    }
}

/// Os campos do `ParticleEmitter` — a ordem é a da struct, e os `field_id` são append-only.
const FIELDS: &[FieldDesc] = &[
    f(1, "component.field.fields.1", K::Toggle),
    f(2, "component.field.fields.2", K::Toggle),
    f(3, "component.field.fields.3", K::Int),
    f(4, "component.field.fields.4", K::Scalar),
    f(5, "component.field.fields.5", K::Scalar),
    f(6, "component.field.fields.6", K::Scalar),
    f(7, "component.field.fields.7", K::Scalar),
    f(8, "component.field.fields.8", K::Scalar),
    f(9, "component.field.fields.9", K::Seed),
    f(10, "component.field.fields.10", K::Enum),
    f(11, "component.field.fields.11", K::Vec2),
    f(12, "component.field.fields.12", K::Scalar),
    f(13, "component.field.fields.13", K::Scalar),
    f(14, "component.field.fields.14", K::Angle),
    f(15, "component.field.fields.15", K::Angle),
    f(16, "component.field.fields.16", K::Vec2),
    f(17, "component.field.fields.17", K::Scalar),
    f(18, "component.field.fields.18", K::Scalar),
    f(19, "component.field.fields.19", K::Scalar),
    f(20, "component.field.fields.20", K::Scalar),
    f(21, "component.field.fields.21", K::Color),
    f(22, "component.field.fields.22", K::Color),
    f(23, "component.field.fields.23", K::Enum),
    f(24, "component.field.fields.24", K::Text),
    f(25, "component.field.fields.25", K::Text),
    f(26, "component.field.fields.26", K::Text),
    f(27, "component.field.fields.27", K::Text),
];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[D::authored(
    "ph2d::ecs::ParticleEmitter",
    "component.particle_emitter.name",
    C::Rendering,
    O::ANY,
    FIELDS,
)];
