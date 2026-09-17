//! **As pontes de identidade** — os CINCO tipos que o artista NUNCA anexa à mão.
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
//!
//! ⭐ **E é por isso que a CÓPIA PROFUNDA não os leva** (F4.2): o id é opaco, então copiá-lo daria
//! **duas entidades a escrever no mesmo documento** — duplicar uma sprite pintada devolveria um
//! sósia que apaga a tinta do original, e duplicar uma forma vetorial poria duas entidades sobre um
//! path que o `vec_entities::sync` mantém 1:1. Os quatro declaram-no com
//! [`crate::ComponentDesc::owned_document`], e o gate `the_bridges_are_the_owned_documents` prende
//! esta família àquela flag. ⏳ Ensinar cada documento a copiar-se é outra obra (vetor = F4.6).

use crate::{ComponentCategory as C, ComponentDesc as D};

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    D::owned_bridge(
        "ph2d::ecs::BakedForm",
        "component.baked_form.name",
        C::Model3D,
    ),
    D::owned_bridge(
        "ph2d::ecs::FlipObjectRef",
        "component.flip_object_ref.name",
        C::Identity,
    ),
    D::owned_bridge(
        "ph2d::ecs::PaintedDoc",
        "component.painted_doc.name",
        C::Identity,
    ),
    // ADR-0150: a peça da escultura. Entrou em 2026-09-04, quando o mesh passou a APARECER na
    // Hierarquia — e ela é `owned_document` pelo mesmo motivo dos outros: copiar o id daria duas
    // entidades sobre a mesma peça, e o `sculpt3d::entities::sync` mantém-nas 1:1.
    D::owned_bridge(
        "ph2d::ecs::Sculpt3dPieceRef",
        "component.sculpt_3d_piece_ref.name",
        C::Model3D,
    ),
    D::owned_bridge(
        "ph2d::ecs::VecPathRef",
        "component.vec_path_ref.name",
        C::Vector,
    ),
];
