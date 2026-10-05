//! ⭐ **O desenho vetorial no menu Add de objectos** (spec/06 F1/F3 ▸ Vector, 2.ª volta).
//!
//! UMA entrada, e ela NÃO cria entidade (escolha do dono, 05/10: *«o painel vector abre e nada
//! aparece no canvas ou na hierarquia até que o usuário crie alguma forma ou linha»*): arma a
//! ferramenta ([`crate::vector_mode::EditTarget::arm`]); cada forma desenhada é um objecto e pede o
//! Edit sobre ela.

use ph2d_editor_core::object_add::{AddEntry, AddGroup};

/// O desenho vetorial.
pub const VECTOR: AddEntry = AddEntry::new("object_add.vector.drawing", AddGroup::TwoD);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[VECTOR];

/// ⭐ **Arma a ferramenta do vetor** — `true` se a entrada é desta família. Nada nasce aqui.
pub fn add(entry: AddEntry, vec: &mut crate::state::VecState) -> bool {
    if entry != VECTOR {
        return false;
    }
    vec.edit.arm();
    true
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
