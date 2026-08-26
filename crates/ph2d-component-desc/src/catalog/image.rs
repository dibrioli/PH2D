//! **A família da imagem** — a `Sprite` e o que só existe por causa dela.
//!
//! ⚠️ A `Sprite` mora em `ph2d-render` e as outras nove em `ph2d-ecs`; aqui elas ficam
//! juntas porque o catálogo é cortado por **família**, não por crate dona. É a mesma razão
//! por que a chave é o nome canónico: o descritor fala de tipos que ele não pode importar.
//!
//! # ✅ A `Sprite` FOI partida (ADR-0164 F1 passo 6 / ADR-0166)
//!
//! Ela é o **marcador** de [`crate::ObjectKind::Image`] e tinha **20** campos congelados; hoje
//! tem **13**. Sete saíram para três componentes — [`ph2d_ecs::SpriteCornerTint`] ·
//! [`ph2d_ecs::SpriteGrid`] · [`ph2d_ecs::SpriteRegion`] —, e a razão do corte não é tamanho:
//! *enquanto o dado for campo de um componente que todo objeto-imagem tem, não há como não o
//! mostrar*. Os três estão **aqui**, e os `field_id` da `Sprite` que eles substituem **não são
//! reusados** (a tabela é append-only, e um id reusado faria um override antigo alvejar o campo
//! novo).
//!
//! ⚠️ **O nome é `SpriteGrid` e não `SpriteSheet`**, que o plano dizia: a `ph2d-ecs` já tem
//! `SpriteSheetRef` e `SpriteSheetFrame`, e as duas significam a folha HAND-PACKED — outra coisa.
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

/// ⚠️ **Parcial de propósito.** A `Sprite` tem 13 campos (eram 20 até o corte do ADR-0164 F1
/// passo 6, que levou SETE para três componentes); a F0 descreve os que a §7/§4 do Inspector já editam, e a tabela cresce
/// **append-only** por procura (F3 descreve o resto quando a seção for derivada).
/// ⛔ Nunca reordene nem reuse um `field_id`.
///
/// ⚠️ **O `field_id` 5 está APOSENTADO, não livre.** Ele era o *Frame*, e o índice de célula
/// mudou-se para o [`ph2d_ecs::SpriteGrid`] com a grelha que lhe dá sentido. Reusar o 5 para
/// outro campo faria um override por-campo gravado antes do corte (F4) apontar para o campo
/// errado — a colisão passa muda porque os dois lados são `u16`.
const SPRITE: &[FieldDesc] = &[
    f(1, "Tint", K::Color),
    f(2, "Flip X", K::Toggle),
    f(3, "Flip Y", K::Toggle),
    f(4, "Pivot", K::Vec2),
    // 5 — APOSENTADO (era "Frame"; ver a nota acima).
];

/// Os três grupos que saíram da `Sprite` (ADR-0164 F1 passo 6 / ADR-0166).
const SPRITE_GRID: &[FieldDesc] = &[
    f(1, "Columns", K::Int),
    f(2, "Rows", K::Int),
    f(3, "Frame", K::Int),
];

const SPRITE_REGION: &[FieldDesc] = &[f(1, "Region", K::Vec4), f(2, "Filter Clip", K::Toggle)];

const SPRITE_CORNER_TINT: &[FieldDesc] = &[
    f(1, "Top Left", K::Color),
    f(2, "Top Right", K::Color),
    f(3, "Bottom Left", K::Color),
    f(4, "Bottom Right", K::Color),
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
    // ⛔ **A nota que estava aqui dizia que `AnchorMount.anchor` vira `RefKind::Object` na F1, e
    // a F4.2 REFUTOU-A:** o campo nomeia uma âncora **do PRÓPRIO PAI**, não um objeto do mundo
    // — é uma referência RELATIVA, e uma cópia profunda leva o pai junto. O nome continua a
    // resolver dentro da cópia sem que ninguém reescreva byte nenhum.
    // ⇒ **Declará-lo como referência seria pedir um remap que estragaria o que já funciona.**
    // *A estrutura da cópia apaga o caso especial.* (Ver `shells/desktop/src/instance_refs.rs`,
    // onde o censo confere declaração ↔ remapeador.)
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
    // ⭐ Os TRÊS do corte (ADR-0164 F1 passo 6 / ADR-0166) — `Authored` e `O::IMAGE`: é a
    // paleta do F3 que os anexa, e só a um objeto-imagem. A ausência de cada um é o default
    // benigno que o campo tinha, então nada aparece no Inspector até o artista o pedir.
    D::authored(
        "ph2d::ecs::SpriteCornerTint",
        "Corner Tint",
        C::Image,
        O::IMAGE,
        SPRITE_CORNER_TINT,
    ),
    // ⚠️ Categoria `Image` e não `Animation`: a grelha é um FATO da textura (como ela se
    // divide), e é a `SpriteAnimations` que a percorre. Pô-la em Animation faria um sprite com
    // folha estática — o caso comum — procurar a grelha na secção errada.
    D::authored(
        "ph2d::ecs::SpriteGrid",
        "Sprite Grid",
        C::Image,
        O::IMAGE,
        SPRITE_GRID,
    ),
    // Os pixels editados desta sprite (`project_sprite_pixels.rs`): identidade de CONTEÚDO,
    // posta pelo funil de commit das oito ferramentas de imagem. O artista não a anexa.
    D::machinery("ph2d::ecs::SpritePixels", "Sprite Pixels", C::Image),
    D::authored(
        "ph2d::ecs::SpriteRegion",
        "Region",
        C::Image,
        O::IMAGE,
        SPRITE_REGION,
    ),
    // Proveniência de autoria (que folha esta sprite veio de), não índice de célula — o
    // índice vivo é o `SpriteGrid::frame`. Máquina: quem a põe é o importador.
    D::machinery("ph2d::ecs::SpriteSheetFrame", "Sheet Frame", C::Image),
    D::machinery("ph2d::ecs::SpriteSheetRef", "Sheet Source", C::Image),
    // ⚠️ O MARCADOR de ObjectKind::Image. Sem `Default` ⇒ sem `insert_default` ⇒ a paleta
    // não a oferece; ela chega pelo gesto que cria a imagem. **Tem seção**, e das maiores —
    // é o caso que a variante `Intrinsic` existe para exprimir.
    D::intrinsic("ph2d::render::Sprite", "Sprite", C::Image, SPRITE),
];
