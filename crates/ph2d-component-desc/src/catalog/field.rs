//! **A modelagem 3D por campo implícito** — os 5 de `ph2d-field-ecs` ([ADR-0161](../../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)).
//!
//! ⚠️ **Quase tudo aqui é máquina, e a razão é a arquitetura do módulo:** *a hierarquia da
//! cena É o documento* — o `FieldDoc` é **cozido** dela a cada quadro. Quem põe um `FieldNode`
//! numa entidade é o gesto que cria a primitiva ou o modificador, e o painel `MODEL` é que os
//! edita. Anexar um `FieldNode` a uma sprite pelo `+` do Inspector produziria um nó que a
//! derivação do documento leria sem ninguém o ter desenhado.
//!
//! ⚠️ **`FieldObject` é o MARCADOR de [`crate::ObjectKind::Model3D`]** — e por isso, como as
//! outras pontes, ser máquina não o impede de responder *"que objeto é este?"*.
//!
//! ⚠️ **`register_field_components` É chamado no boot** (`init.rs:517`) desde a ADR-0161, com
//! o comentário a dizer porquê: *"sem esta linha o WorldSnapshot descarta o componente EM
//! SILENCIO, e o sintoma é o objeto sumir ao desfazer"*. É a mesma armadilha que ainda apanha
//! o `LuauScript` ([`super::script`]).

use crate::{ComponentCategory as C, ComponentDesc as D};

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    D::machinery("ph2d::field::FieldMods", "Field Modifiers", C::Model3D),
    D::machinery("ph2d::field::FieldNode", "Field Node", C::Model3D),
    // O MARCADOR de ObjectKind::Model3D.
    D::machinery("ph2d::field::FieldObject", "3D Model", C::Model3D),
    D::machinery("ph2d::field::FieldPose", "Field Pose", C::Model3D),
    D::machinery(
        "ph2d::field::FieldProfileSource",
        "Profile Source",
        C::Model3D,
    ),
];
