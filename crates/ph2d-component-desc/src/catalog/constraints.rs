//! ⭐ **As CONSTRAINTS** — a pilha de `ph2d-constraints-ecs` (`docs/BlenderConstraints/01_plano.md`).
//!
//! `authored` para todo objecto: chega pela paleta vazia e enche-se pelo cartão. Sem campos planos —
//! a pilha é uma lista, e o canal de cada item é a família `CONSTRAINT` (plano §2.2).

use crate::{ComponentCategory as C, ComponentDesc, ComponentDesc as D, ObjectKinds as O};

pub const DESCS: &[ComponentDesc] = &[D::authored(
    "ph2d::constraints::Constraints",
    "component.constraints.name",
    C::Transform,
    O::ANY,
    &[],
)];
