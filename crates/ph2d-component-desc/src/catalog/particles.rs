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

const fn f(field_id: u16, name: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        name,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
    }
}

/// Os campos do `ParticleEmitter` — a ordem é a da struct, e os `field_id` são append-only.
const FIELDS: &[FieldDesc] = &[
    f(1, "Emitting", K::Toggle),
    f(2, "One Shot", K::Toggle),
    f(3, "Amount", K::Int),
    f(4, "Lifetime", K::Scalar),
    f(5, "Lifetime Randomness", K::Scalar),
    f(6, "Explosiveness", K::Scalar),
    f(7, "Preprocess", K::Scalar),
    f(8, "Speed Scale", K::Scalar),
    f(9, "Seed", K::Seed),
    f(10, "Emission Shape", K::Enum),
    f(11, "Shape Size", K::Vec2),
    f(12, "Speed", K::Scalar),
    f(13, "Speed Randomness", K::Scalar),
    f(14, "Direction", K::Angle),
    f(15, "Spread", K::Angle),
    f(16, "Gravity", K::Vec2),
    f(17, "Damping", K::Scalar),
    f(18, "Size", K::Scalar),
    f(19, "Size Randomness", K::Scalar),
    f(20, "Size at Death", K::Scalar),
    f(21, "Color", K::Color),
    f(22, "Color at Death", K::Color),
    f(23, "Simulation Space", K::Enum),
    f(24, "Start On", K::Text),
    f(25, "Stop On", K::Text),
    f(26, "Restart On", K::Text),
    f(27, "Finished Signal", K::Text),
];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[D::authored(
    "ph2d::ecs::ParticleEmitter",
    "Particles",
    C::Rendering,
    O::ANY,
    FIELDS,
)];
