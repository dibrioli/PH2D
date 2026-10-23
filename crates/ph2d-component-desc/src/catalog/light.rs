//! ⭐ **A família LUZ 2D** — os 3 componentes de `ph2d-light2d-ecs` (`docs/Luz2D/01_plano.md` §2.3).
//!
//! - **`Luz`** é `intrinsic`: chega pelo menu Add (W4) com valores escolhidos; não tem `Default` e
//!   não está na paleta.
//! - **`BloqueiaLuz`** e **`NaoRecebeLuz`** são `authored` para quem tem forma visível
//!   (`DRAWABLE`): a forma do oclusor é a do objeto; um vazio não tem o que bloquear nem o que
//!   iluminar.
//!
//! - Os campos são os que os cartões pintam (doc CodeEditor 08 §3.11): o nome segue o rótulo inglês,
//!   a chave serde é a portuguesa do tipo (`named_as`).
//! ⚠️ **A lista está ORDENADA por `canonical_name`** (gate `the_catalog_is_sorted_and_unique`).

use crate::{
    ComponentCategory as C, ComponentDesc, ComponentDesc as D, FieldDesc, FieldKind as K, Hidden,
    ObjectKinds as O, Propagation,
};

const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
        name: "",
        key: "",
        view: crate::View::Same,
    }
}

/// Os campos da lâmpada, na ordem do cartão. «Limit Range» não é chave: é `range` finito.
const LUZ_FIELDS: &[FieldDesc] = &[
    f(0, "component.field.luz_fields.0", K::Enum).named_as("type", "tipo"),
    f(1, "component.field.luz_fields.1", K::Color).named_as("color", "cor"),
    f(2, "component.field.luz_fields.2", K::Scalar).named_as("intensity", "intensidade"),
    f(3, "component.field.luz_fields.3", K::Scalar).named_as("size", "tamanho_m"),
    f(4, "component.field.luz_fields.4", K::Scalar).named_as("range", "alcance_m"),
];

/// O campo do oclusor.
const OCCLUDER_FIELDS: &[FieldDesc] =
    &[f(0, "component.field.occluder_fields.0", K::Scalar).named_as("opacity", "opacidade")];

/// Os descritores da família.
pub const DESCS: &[ComponentDesc] = &[
    D::authored(
        "ph2d::light2d::BloqueiaLuz",
        "component.bloqueia_luz.name",
        C::Light,
        O::DRAWABLE,
        OCCLUDER_FIELDS,
    )
    .coded("light_occluder"),
    // ⏳ `.coded("light_2d")` espera a ida-e-volta de um real NÃO FINITO: o alcance de fábrica é
    // `f32::INFINITY`, que o `serde_json` escreve `null` e não lê de volta — `ph2d.set` de QUALQUER
    // campo da luz falharia (doc CodeEditor 08 §3.11). Os nomes já estão prontos.
    D::intrinsic(
        "ph2d::light2d::Luz",
        "component.luz.name",
        C::Light,
        LUZ_FIELDS,
    ),
    D::authored(
        "ph2d::light2d::NaoRecebeLuz",
        "component.nao_recebe_luz.name",
        C::Light,
        O::DRAWABLE,
        &[],
    )
    .hidden(Hidden::Presence),
];
