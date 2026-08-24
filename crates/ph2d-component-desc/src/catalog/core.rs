//! **O núcleo do `ph2d-ecs`** — identidade, pose, ordenação, e o que o renderer de sprite lê.
//!
//! # A aplicabilidade desta família foi MEDIDA, não arbitrada
//!
//! Método (2026-08-24, refazível):
//!
//! ```text
//! grep -rln '\bBlendMode\b' crates/ph2d-vec-render/src crates/ph2d-render/src \
//!                           crates/ph2d-flip-render/src
//! ```
//!
//! Os de `Rendering` (`BlendMode` · `TextureFilter` · `TextureRepeat` · `UvTransform` ·
//! `Mask2D` · `MaskInteraction` · `VisibilityLayer` · `OnScreenEnabler` · `SpriteEmissive`)
//! resolvem **só em `ph2d-render`** — nenhum é lido pelo renderer vetorial nem pelo do Flip.
//! Logo: [`ObjectKinds::IMAGE`]. *É uma medição do consumidor, não uma opinião sobre o nome.*
//!
//! ⚠️ **A ordenação NÃO é Image-only, e a medição é que o diz:** o `ZIndexOverride` é lido
//! pelo `ph2d-panel-vector`, e um `SortingGroup`/`ClipChildren`/`TopLevel` sobre um pai
//! **vazio** é o caso canónico do Unity (agrupar filhos que desenham). Por isso a família de
//! ordenação é [`ObjectKinds::ANY`] — incluindo `Empty`, que é o pai do agrupamento.
//!
//! ⚠️ **O que esta medição NÃO fecha:** ela responde *"quem lê o componente"*, e não *"em que
//! objeto ele tem efeito VISÍVEL"*. As duas coincidem acima porque o leitor é um renderer só;
//! onde houver dois leitores em famílias diferentes, a resposta pede o smoke.

use crate::{
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, ObjectKinds as O,
    Propagation,
};

/// Um campo simples que segue o mestre e não é referência — o caso esmagadoramente comum.
const fn f(field_id: u16, name: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        name,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
    }
}

/// **`Transform`** — ⚠️ a pose da RAIZ de uma instância é *Local da instância*, e a de uma
/// PEÇA propaga. A política aqui é a da peça (o caso geral); o sítio (raiz × peça) é o que
/// decide, e quem decide é o passe de sync da F4, não esta tabela. *Um tipo, duas respostas,
/// escolhidas pelo lugar.*
///
/// ⚠️ A rotação e os skews vivem em **radianos** no componente; `FieldKind::Angle` é o
/// CONTROLO (que fala graus, a unidade autorada do app). A conversão é do consumidor —
/// declarar graus aqui faria a tabela mentir sobre os bytes.
const TRANSFORM: &[FieldDesc] = &[
    f(1, "Position", K::Vec2),
    f(2, "Rotation", K::Angle),
    f(3, "Scale", K::Vec2),
    f(4, "Skew X", K::Angle),
    f(5, "Skew Y", K::Angle),
];

/// **`Name`** — ⚠️ `InstanceLocal`: o nome da raiz de uma instância é dela. Senão três
/// instâncias do mesmo mestre partilhariam o nome, e a unicidade que o editor impõe
/// (`name_unique.rs`) entraria em guerra com o sync todo o quadro.
const NAME: &[FieldDesc] = &[FieldDesc {
    field_id: 1,
    name: "Name",
    kind: K::Text,
    policy: Propagation::InstanceLocal,
    is_ref: None,
}];

const VISIBILITY: &[FieldDesc] = &[f(1, "Hidden", K::Toggle)];

// ── Ordenação: a família PILOTO da F0 (a §7 do Inspector) ──────────────────────────

const SORTING_LAYER: &[FieldDesc] = &[f(1, "Sorting Layer", K::Enum)];
const ORDER_IN_LAYER: &[FieldDesc] = &[f(1, "Order in Layer", K::Int)];
const Z_INDEX: &[FieldDesc] = &[f(1, "Z Index", K::Int)];
const Z_AS_RELATIVE: &[FieldDesc] = &[f(1, "Z as Relative", K::Toggle)];
// ⚠️ **Os rótulos abaixo são os que o Inspector JÁ PINTA, ao byte** — não os que eu
// escolheria. A seção §7 é o piloto da F0, e ao ligá-la ao descritor apareceu o defeito que
// justifica a ligação: as duas fontes já discordavam (*"Sort At Root"* × *"Sort at Root"*,
// *"Y-Sort"* × *"Enabled"*). Quem manda é o produto, então o descritor foi corrigido para
// ele — e agora há **uma** fonte, que é o ponto.
const SORTING_GROUP: &[FieldDesc] = &[f(1, "Sort At Root", K::Toggle)];
const Y_SORT: &[FieldDesc] = &[
    f(1, "Y-Sort", K::Toggle),
    f(2, "Axis", K::Vec2),
    f(3, "Sort Point", K::Enum),
];
const CLIP_CHILDREN: &[FieldDesc] = &[f(1, "Mode", K::Enum), f(2, "Alpha Cutoff", K::Scalar)];

/// Um marcador de tamanho zero: a **presença** é o valor.
const MARKER: &[FieldDesc] = &[f(1, "Present", K::Marker)];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    D::authored(
        "ph2d::ecs::BlendMode",
        "Blend Mode",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::ClipChildren",
        "Clip Children",
        C::Ordering,
        O::ANY,
        CLIP_CHILDREN,
    ),
    D::authored(
        "ph2d::ecs::GroupedChildren",
        "Grouped Children",
        C::Identity,
        O::ANY,
        MARKER,
    ),
    D::authored("ph2d::ecs::Locked", "Locked", C::Identity, O::ANY, MARKER),
    D::authored("ph2d::ecs::Mask2D", "Mask", C::Rendering, O::IMAGE, &[]),
    D::authored(
        "ph2d::ecs::MaskInteraction",
        "Mask Interaction",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⚠️ `Name` NÃO tem `Default` (o compilador disse-o ao converter o registador), e a
    // ausência está certa: um objeto sem nome não é um objeto com nome vazio. Ele chega com
    // o gesto que cria o objeto e é **base** — a paleta oferecê-lo seria oferecer algo que
    // toda entidade já tem.
    D::intrinsic("ph2d::ecs::Name", "Name", C::Identity, NAME),
    D::authored(
        "ph2d::ecs::OnScreenEnabler",
        "On-Screen Enabler",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::OrderInLayer",
        "Order in Layer",
        C::Ordering,
        O::ANY,
        ORDER_IN_LAYER,
    ),
    // ⚠️ Máquina: o editor mantém-no para desempatar raízes (*"não se escolhe um desempate
    // melhor, não se tem empate"*). Um artista que o pusesse à mão estaria a escrever num
    // campo que o próprio editor reescreve no quadro seguinte.
    D::machinery("ph2d::ecs::RootOrder", "Root Order", C::Ordering),
    D::authored(
        "ph2d::ecs::ShowBehindParent",
        "Show Behind Parent",
        C::Ordering,
        O::ANY,
        MARKER,
    ),
    // ⚠️ Máquina, pela MESMA razão do `RootOrder` (o gémeo dele para raízes): a ordem entre
    // irmãos é escrita pelo GESTO de arrastar na Hierarquia, e o editor mantém-na. Um artista
    // que a pusesse à mão estaria a escrever num campo que a varredura reescreve.
    D::machinery("ph2d::ecs::SiblingOrder", "Sibling Order", C::Ordering),
    D::authored(
        "ph2d::ecs::SortingGroup",
        "Sorting Group",
        C::Ordering,
        O::ANY,
        SORTING_GROUP,
    ),
    D::authored(
        "ph2d::ecs::SortingLayer",
        "Sorting Layer",
        C::Ordering,
        O::ANY,
        SORTING_LAYER,
    ),
    D::authored(
        "ph2d::ecs::SpriteEmissive",
        "Emissive",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⚠️ **Máquina, e das mais duras.** A identidade durável de um objeto (F1) é posta pela
    // varredura e nunca pelo artista — um `StableId` escolhido à mão é uma referência que
    // aponta para outra coisa. Não deriva `Default`, logo não teria `insert_default` nem se
    // alguém a marcasse `Authored`: o censo da shell recusaria antes.
    D::machinery("ph2d::ecs::StableId", "Stable Id", C::Identity),
    D::authored(
        "ph2d::ecs::TextureFilter",
        "Texture Filter",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::TextureRepeat",
        "Texture Repeat",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::TopLevel",
        "Top Level",
        C::Ordering,
        O::ANY,
        MARKER,
    ),
    D::authored(
        "ph2d::ecs::Transform",
        "Transform",
        C::Transform,
        O::ANY,
        TRANSFORM,
    ),
    D::authored(
        "ph2d::ecs::UvTransform",
        "UV Transform",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::Visibility",
        "Visibility",
        C::Identity,
        O::ANY,
        VISIBILITY,
    ),
    D::authored(
        "ph2d::ecs::VisibilityLayer",
        "Visibility Layer",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored("ph2d::ecs::YSort", "Y Sort", C::Ordering, O::ANY, Y_SORT),
    D::authored(
        "ph2d::ecs::ZAsRelative",
        "Z as Relative",
        C::Ordering,
        O::ANY,
        Z_AS_RELATIVE,
    ),
    D::authored(
        "ph2d::ecs::ZIndexOverride",
        "Z Index",
        C::Ordering,
        O::ANY,
        Z_INDEX,
    ),
];
