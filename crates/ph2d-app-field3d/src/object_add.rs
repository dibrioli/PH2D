//! ⭐ **O objecto Model no menu Add de objectos** (spec/06 F1) — a entrada e o nascimento.
//!
//! ⭐ **N Models por cena** (spec/06 F3): cada um é uma raiz própria, e o módulo edita a do modo Edit
//! ([`crate::model_mode`]). O Model novo nasce em Edit — fora dele não se desenha.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Xform};

/// A entrada do menu.
pub const MODEL: AddEntry = AddEntry::new("object_add.model", AddGroup::ThreeD);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[MODEL];

/// A forma com que um Model novo nasce: a esfera do catálogo do *Add shape…*.
const FIRST_SHAPE: &str = "panel.model3d.add.sphere";

/// O documento de um Model novo: uma união com a esfera do catálogo dentro, no tamanho que o
/// *Add shape…* usaria na vista de abertura — para o próximo *Add shape…* ter onde pôr a forma.
fn first_doc() -> FieldDoc {
    let slot = crate::shapes::SHAPES
        .iter()
        .position(|s| s.key == FIRST_SHAPE)
        .expect("a esfera está no catálogo");
    let r = crate::scene::new_shape_size(ph2d_field_render::Orbit::default().half_extent);
    let prim = crate::shapes::shape_at(slot, r).expect("a esfera sai de um raio");
    FieldDoc::new(
        vec![
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(prim),
                mods: Vec::new(),
                verb: None,
            },
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(1),
    )
    .expect("uma união com uma folha é um documento válido")
}

/// ⭐ **Cria o Model** ao lado dos que já existem e devolve os bits da raiz; ele pede o Edit.
///
/// ⚠️ **Pelo MESMO plantio que a semente do módulo usa** ([`crate::scene::plant`]): o material e a
/// luz de abertura entram ali, e uma segunda porta divergiria da primeira.
pub fn add(sim: &mut SimWorld) -> u64 {
    let (root, _) = crate::scene::plant(sim.world_mut(), &first_doc());
    let bits = root.to_bits();
    crate::model_mode::born(bits);
    bits
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
