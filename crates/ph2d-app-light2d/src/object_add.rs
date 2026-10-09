//! ⭐ **«Luz» e «Luz ambiente» no menu Add** (plano §4) — um objeto vazio com a `Luz` já posta.
//!
//! ⚠️ A `Luz` não pede nenhum outro componente (o catálogo a declara sem `requires`), logo nascer
//! com ela é a mesma coisa que o `attach_by_name` do Inspector faria — e a família não chama a
//! `ph2d-app-components` (ADR-0075: família não chama família); o esqueleto nasce igual.

use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_light2d_ecs::Luz;

/// A lâmpada (a de fábrica: branco a 1 m).
pub const LUZ: AddEntry = AddEntry::new("object_add.light.point", AddGroup::Game);
/// A luz ambiente (a de fábrica: o forno branco — a cena como sem luz; o artista baixa-a).
pub const LUZ_AMBIENTE: AddEntry = AddEntry::new("object_add.light.ambient", AddGroup::Game);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[LUZ, LUZ_AMBIENTE];

/// A `Luz` com que cada entrada nasce — `None` se a entrada não é desta família.
#[must_use]
pub fn luz_de(entry: AddEntry) -> Option<Luz> {
    Some(match entry {
        e if e == LUZ => Luz::default(),
        e if e == LUZ_AMBIENTE => Luz::ambiente(),
        _ => return None,
    })
}

/// ⭐ **Cria a luz** desta entrada na raiz, com nome único — `None` se a entrada não é desta
/// família. Devolve os bits do objeto.
pub fn add(entry: AddEntry, sim: &mut SimWorld) -> Option<u64> {
    let luz = luz_de(entry)?;
    let name = ph2d_unique_name::unique_name(sim, entry.key.tr());
    let bits = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name), luz))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    Some(bits)
}
