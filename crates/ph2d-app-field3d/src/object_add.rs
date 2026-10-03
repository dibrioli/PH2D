//! ⭐ **O objecto Model no menu Add de objectos** (spec/06 F1) — a entrada e o nascimento.
//!
//! ⚠️ **Um Model por cena, por agora, e o menu DIZ-O.** A ponte com a cena coze a PRIMEIRA raiz
//! com [`FieldObject`] (`scene::sync_scene_and_birth`): uma segunda raiz nasceria e nunca seria
//! cozida — um objecto morto na Hierarquia. Até o modo Edit abrir o módulo sobre a entidade
//! (spec/06 F3), a entrada aparece com a razão no rótulo, e o clique responde com ela.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Xform};
use ph2d_field_ecs::FieldObject;

/// A entrada do menu.
pub const MODEL: AddEntry = AddEntry::new("object_add.model", AddGroup::ThreeD);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[MODEL];

/// A forma com que um Model novo nasce: a esfera do catálogo do *Add shape…*.
const FIRST_SHAPE: &str = "panel.model3d.add.sphere";

fn root_of(sim: &mut SimWorld) -> Option<u64> {
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
    q.iter(world).next().map(|(e, _)| e.to_bits())
}

/// Porque é que um Model não pode nascer agora — `None` quando pode.
#[must_use]
pub fn why_not(sim: &mut SimWorld) -> Option<&'static str> {
    root_of(sim)
        .is_some()
        .then(|| ph2d_i18n::tr("object_add.model.one_per_scene"))
}

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

/// ⭐ **Cria o Model** e devolve os bits da raiz — ou a razão pela qual não pode.
///
/// ⚠️ **Pelo MESMO nascimento que o painel usa** ([`crate::scene::sync_scene_and_birth`] com uma semente):
/// a luz de abertura e o material entram ali, e uma segunda porta divergiria da primeira.
/// E pede ao painel que abra: o Model só se desenha com o módulo aberto.
pub fn add(sim: &mut SimWorld) -> Result<u64, &'static str> {
    if let Some(reason) = why_not(sim) {
        return Err(reason);
    }
    crate::scene::sync_scene_and_birth(
        sim,
        Some(&first_doc()),
        &[],
        0.0,
        &ph2d_vec_scene::VecScene::default(),
    );
    crate::smoke::ask_open_panel();
    root_of(sim).ok_or_else(|| ph2d_i18n::tr("object_add.not_born"))
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
