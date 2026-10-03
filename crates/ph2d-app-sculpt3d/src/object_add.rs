//! ⭐ **As peças de escultura no menu Add de objectos** (spec/06 F1) — as entradas e o nascimento.
//!
//! ⚠️ **A peça nasce na CENA, e a entidade pela ponte de sempre** ([`crate::entities`]): a cena é
//! dona da geometria, a entidade da identidade. Quem cria a peça devolve o id dela; a shell corre a
//! sincronia e pergunta pela entidade com [`entity_of_piece`].
//!
//! ⚠️ **Sem cena, o menu CRIA uma com a peça escolhida** — pela mesma porta do pill
//! ([`crate::mode::new_scene`]), e com o barro na tela, como o pill.

use std::sync::Arc;

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};

use crate::{Primitive, Sculpt3dScene};

/// Esfera.
pub const SPHERE: AddEntry = AddEntry::new("object_add.sculpt.sphere", AddGroup::ThreeD);
/// Cubo.
pub const CUBE: AddEntry = AddEntry::new("object_add.sculpt.cube", AddGroup::ThreeD);
/// Cilindro.
pub const CYLINDER: AddEntry = AddEntry::new("object_add.sculpt.cylinder", AddGroup::ThreeD);
/// Toro.
pub const TORUS: AddEntry = AddEntry::new("object_add.sculpt.torus", AddGroup::ThreeD);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[SPHERE, CUBE, CYLINDER, TORUS];

fn primitive_of(entry: AddEntry) -> Option<Primitive> {
    Some(match entry {
        e if e == SPHERE => Primitive::Sphere,
        e if e == CUBE => Primitive::Cube,
        e if e == CYLINDER => Primitive::Cylinder,
        e if e == TORUS => Primitive::Torus,
        _ => return None,
    })
}

/// ⭐ **Cria a peça** desta entrada e devolve o id dela — `None` se a entrada não é desta família.
///
/// `gpu` é o dispositivo e o tamanho da janela: sem eles não há cena a criar (a mesma ausência
/// que o pill trata).
pub fn add(
    entry: AddEntry,
    slot: &mut Option<Sculpt3dScene>,
    gpu: Option<(&Arc<wgpu::Device>, (u32, u32))>,
) -> Option<Result<u32, &'static str>> {
    let kind = primitive_of(entry)?;
    Some(match slot.as_mut() {
        Some(scene) => {
            let i = scene.add_primitive(kind);
            Ok(scene.objects[i].id.0)
        }
        None => match gpu {
            Some((device, size)) => {
                let scene = crate::mode::new_scene(device, size, kind);
                let id = scene.objects[scene.active].id.0;
                *slot = Some(scene);
                Ok(id)
            }
            None => Err(ph2d_i18n::tr("object_add.not_born")),
        },
    })
}

/// A entidade da Hierarquia que espelha a peça `piece`, se a sincronia já a criou.
#[must_use]
pub fn entity_of_piece(sim: &mut SimWorld, piece: u32) -> Option<u64> {
    crate::entities::world_map(sim).get(&piece).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Cada entrada nomeia UMA peça, e uma entrada de outra família não é desta.**
    ///
    /// ⚠️ A metade que cria a cena pede uma placa (`Sculpt3dScene::new(&device, …)`), e nenhum gate
    /// deste repositório a tem sem GPU — o nascimento prova-se no seam da shell e na foto.
    #[test]
    fn every_entry_names_one_primitive() {
        let kinds: Vec<_> = ENTRIES.iter().filter_map(|e| primitive_of(*e)).collect();
        assert_eq!(kinds.len(), ENTRIES.len());
        for (i, a) in kinds.iter().enumerate() {
            assert!(!kinds[i + 1..].contains(a), "duas entradas, a mesma peça");
        }
        let mut slot = None;
        assert!(add(ph2d_editor_core::object_add::EMPTY, &mut slot, None).is_none());
        assert!(slot.is_none());
    }

    /// **Sem placa, o pedido RESPONDE** — não cria uma cena a meio, nem engole o clique.
    #[test]
    fn without_a_device_the_request_answers() {
        let mut slot = None;
        assert!(matches!(add(SPHERE, &mut slot, None), Some(Err(_))));
        assert!(slot.is_none());
    }
}
