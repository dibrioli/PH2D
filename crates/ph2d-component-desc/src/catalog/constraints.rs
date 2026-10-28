//! ⭐ **As CONSTRAINTS** — a pilha de `ph2d-constraints-ecs` (`docs/BlenderConstraints/01_plano.md`).
//!
//! `authored` para todo objecto: chega pela paleta vazia e enche-se pelo cartão. Sem campos planos —
//! a pilha é uma lista, e o canal de cada item é a família `CONSTRAINT` (plano §2.2).

use crate::{ComponentCategory as C, ComponentDesc, ComponentDesc as D, Hidden, ObjectKinds as O};

pub const DESCS: &[ComponentDesc] = &[
    // A cola (W5, ADR-0193): máquina — nasce só pelo «Add Constraint ▸ Glue» (pela paleta seria uma lista
    // vazia, um controlo morto) e mostra-se no cartão das constraints dos DOIS objectos.
    D::machinery(
        "ph2d::constraints::Colas",
        "component.colas.name",
        C::Transform,
    ),
    D::authored(
        "ph2d::constraints::Constraints",
        "component.constraints.name",
        C::Transform,
        O::ANY,
        &[],
    )
    .hidden(Hidden::List),
];
