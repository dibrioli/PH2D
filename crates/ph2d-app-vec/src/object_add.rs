//! ⭐ **O objecto vetorial no menu Add de objectos** (spec/06 F1/F3) — a entrada e o nascimento.
//!
//! UMA entrada (escolha do dono, 04/10: *«melhor seria se houvesse apenas uma opção no modal:
//! objeto vetorial»*): cria um objecto VAZIO e ele pede o Edit
//! ([`crate::vector_mode::EditTarget::born`]), onde o painel inteiro desenha as formas DENTRO dele.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};

/// O objecto vetorial.
pub const VECTOR_OBJECT: AddEntry = AddEntry::new("object_add.vector.object", AddGroup::TwoD);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[VECTOR_OBJECT];

/// ⭐ **Cria o objecto vazio** em `at` (mundo) e arma o Edit dele — os bits da entidade, ou `None`
/// se a entrada não é desta família.
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    vec: &mut crate::state::VecState,
    at: [f64; 2],
) -> Option<u64> {
    if entry != VECTOR_OBJECT {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)] // posição de mundo: f32 é a do `Transform`
    let at = ph2d_core::Vec2::new(at[0] as f32, at[1] as f32);
    let name = ph2d_i18n::tr("object_add.vector.object_name");
    let bits = ph2d_vec_entities::entities::object::spawn_object(sim, name, at).to_bits();
    vec.edit.born(bits);
    Some(bits)
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
