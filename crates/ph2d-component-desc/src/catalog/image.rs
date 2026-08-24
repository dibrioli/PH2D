//! **A família da imagem** — a `Sprite` e o que só existe por causa dela.
//!
//! ⚠️ A `Sprite` mora em `ph2d-render` e as outras nove em `ph2d-ecs`; aqui elas ficam
//! juntas porque o catálogo é cortado por **família**, não por crate dona. É a mesma razão
//! por que a chave é o nome canónico: o descritor fala de tipos que ele não pode importar.
//!
//! # ⚠️ A `Sprite` é a peça que este plano vai partir
//!
//! Ela é o **marcador** de [`crate::ObjectKind::Image`] e tem 20 campos congelados
//! (ADR-0074). A F1 corta três deles para componentes próprios (`SpriteCornerTint` ·
//! `SpriteSheet` · `SpriteRegion`) — e o ADR-0166 deu ao corte uma segunda razão que não é
//! tamanho: *enquanto o dado for campo de um componente que todo objeto-imagem tem, não há
//! como não o mostrar*. Quando esses três nascerem, entram **aqui**, e os `field_id` da
//! `Sprite` que eles substituem **não são reusados** (a tabela é append-only, e um id reusado
//! faria um override antigo alvejar o campo novo).
//!
//! ⚠️ A `Sprite` **não deriva `Default`** (precisa de uma `source`), logo não tem
//! `insert_default` no registo — e é por isso que ela não pode ser anexada pela paleta. Ela
//! não é uma escolha do Inspector: é o que o gesto de criar uma imagem põe lá.

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

/// ⚠️ **Parcial de propósito.** A `Sprite` tem 20 campos; a F0 descreve os que a §7/§4 do
/// Inspector já editam, e a tabela cresce **append-only** por procura (F1 corta três; F3
/// descreve o resto quando a seção for derivada). ⛔ Nunca reordene nem reuse um `field_id`.
const SPRITE: &[FieldDesc] = &[
    f(1, "Tint", K::Color),
    f(2, "Flip X", K::Toggle),
    f(3, "Flip Y", K::Toggle),
    f(4, "Pivot", K::Vec2),
    f(5, "Frame", K::Int),
];

const SLICE_NINE: &[FieldDesc] = &[
    f(1, "Left", K::Scalar),
    f(2, "Right", K::Scalar),
    f(3, "Top", K::Scalar),
    f(4, "Bottom", K::Scalar),
    f(5, "Fill Center", K::Toggle),
    f(6, "Tile Mode", K::Enum),
];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    // A âncora é um QUADRO na hierarquia (ADR-0072-amendment-1): o filho monta-se nela.
    // ⚠️ `AnchorMount.anchor` referencia uma âncora do PAI pelo nome — é `RefKind::Object`
    // quando a F1 migrar para `StableId`. Hoje ainda não há campo descrito aqui, e a nota
    // existe para que o remap da F4 não descubra isto tarde.
    D::authored(
        "ph2d::ecs::AnchorMount",
        "Anchor Mount",
        C::Anchors,
        O::ANY,
        &[],
    ),
    D::authored(
        "ph2d::ecs::AnchorVisibility",
        "Anchor Visibility",
        C::Anchors,
        O::ANY,
        &[],
    ),
    D::authored(
        "ph2d::ecs::NamedAnchorList",
        "Anchors",
        C::Anchors,
        O::ANY,
        &[],
    ),
    D::authored(
        "ph2d::ecs::SliceNine",
        "9-Slice",
        C::Image,
        O::IMAGE,
        SLICE_NINE,
    ),
    D::authored(
        "ph2d::ecs::SpriteAnimations",
        "Animations",
        C::Animation,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::SpriteAnimator",
        "Animator",
        C::Animation,
        O::IMAGE,
        &[],
    ),
    // Os pixels editados desta sprite (`project_sprite_pixels.rs`): identidade de CONTEÚDO,
    // posta pelo funil de commit das oito ferramentas de imagem. O artista não a anexa.
    D::machinery("ph2d::ecs::SpritePixels", "Sprite Pixels", C::Image),
    // Proveniência de autoria (que folha esta sprite veio de), não índice de célula — o
    // índice vivo é o `Sprite::frame`. Máquina: quem a põe é o importador.
    D::machinery("ph2d::ecs::SpriteSheetFrame", "Sheet Frame", C::Image),
    D::machinery("ph2d::ecs::SpriteSheetRef", "Sheet Source", C::Image),
    // ⚠️ O MARCADOR de ObjectKind::Image. Sem `Default` ⇒ sem `insert_default` ⇒ a paleta
    // não a oferece; ela chega pelo gesto que cria a imagem. **Tem seção**, e das maiores —
    // é o caso que a variante `Intrinsic` existe para exprimir.
    D::intrinsic("ph2d::render::Sprite", "Sprite", C::Image, SPRITE),
];
