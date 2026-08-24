//! **As pontes de identidade** — os quatro tipos que o artista NUNCA anexa à mão.
//!
//! Cada um é um `u32`/id opaco que liga a entidade ao documento do módulo dela
//! (`VecPathRef` → o caminho no `VecScene`; `PaintedDoc` → o documento do Painter;
//! `BakedForm` → os canais assados do 3D; `FlipObjectRef` → o objeto do Flip). Eles são
//! **máquina**: quem os cria é o gesto que cria o objeto, e um deles anexado a uma entidade
//! qualquer é um id que não aponta para nada.
//!
//! ⚠️ **Estão registados** — logo aparecem no `ComponentRegistry`, logo apareceriam numa
//! paleta que listasse *"todo tipo registado"*. É esta família que existe para os tirar de
//! lá, e por [`crate::Attach::Machinery`] ser uma **declaração**, o censo consegue exigir
//! que toda ausência da paleta tenha um autor (ADR-0166 §3).
//!
//! ⚠️ Três deles são também **marcadores de tipo de objeto**
//! ([`crate::ObjectKind::marker`]): ser máquina não os impede de RESPONDER *"que objeto é
//! este?"* — pelo contrário, é exatamente por serem postos pelo gesto de criação que a
//! resposta deles é confiável.

use crate::{ComponentCategory as C, ComponentDesc as D};

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    D::machinery("ph2d::ecs::BakedForm", "Baked Form", C::Model3D),
    D::machinery("ph2d::ecs::FlipObjectRef", "Flip Object", C::Identity),
    D::machinery("ph2d::ecs::PaintedDoc", "Painted Document", C::Identity),
    D::machinery("ph2d::ecs::VecPathRef", "Vector Path", C::Vector),
];
