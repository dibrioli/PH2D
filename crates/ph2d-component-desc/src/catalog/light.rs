//! ⭐ **A família LUZ 2D** — os 3 componentes de `ph2d-light2d-ecs` (`docs/Luz2D/01_plano.md` §2.3).
//!
//! - **`Luz`** é `intrinsic`: chega pelo menu Add (W4) com valores escolhidos; não tem `Default` e
//!   não está na paleta.
//! - **`BloqueiaLuz`** e **`NaoRecebeLuz`** são `authored` para quem tem forma visível
//!   (`DRAWABLE`): a forma do oclusor é a do objeto; um vazio não tem o que bloquear nem o que
//!   iluminar.
//!
//! ⚠️ Sem campos nesta onda: a secção do Inspector é da W4.
//! ⚠️ **A lista está ORDENADA por `canonical_name`** (gate `the_catalog_is_sorted_and_unique`).

use crate::{ComponentCategory as C, ComponentDesc, ComponentDesc as D, ObjectKinds as O};

/// Os descritores da família.
pub const DESCS: &[ComponentDesc] = &[
    D::authored(
        "ph2d::light2d::BloqueiaLuz",
        "component.bloqueia_luz.name",
        C::Light,
        O::DRAWABLE,
        &[],
    ),
    D::intrinsic("ph2d::light2d::Luz", "component.luz.name", C::Light, &[]),
    D::authored(
        "ph2d::light2d::NaoRecebeLuz",
        "component.nao_recebe_luz.name",
        C::Light,
        O::DRAWABLE,
        &[],
    ),
];
