//! ⭐ **O vazio e os objectos de JOGO no menu Add de objectos** (spec/06 F1, escolha 6 do dono).
//!
//! Um objecto de jogo é **um vazio com o componente já posto** — e o componente entra pela MESMA
//! porta que o `+` do Inspector usa ([`crate::component_attach::attach_by_name`]): a cascata do
//! que ele não funciona sem e a semente da família dona vêm juntas. Uma segunda porta aqui seria
//! a segunda resposta a *«o que nasce com um corpo de física?»*.

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor_core::object_add::{AddEntry, AddGroup};

/// Câmara de jogo.
pub const CAMERA: AddEntry = AddEntry::new("object_add.game.camera", AddGroup::Game);
/// Corpo de física.
pub const BODY: AddEntry = AddEntry::new("object_add.game.body", AddGroup::Game);
/// Fonte de som.
pub const SOUND: AddEntry = AddEntry::new("object_add.game.sound", AddGroup::Game);
/// Tela de HUD.
pub const HUD: AddEntry = AddEntry::new("object_add.game.hud", AddGroup::Game);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[CAMERA, BODY, SOUND, HUD];

/// O componente que faz cada entrada de jogo ser o que é — pelo nome canónico do registo.
#[must_use]
pub fn component_of(entry: AddEntry) -> Option<&'static str> {
    Some(match entry {
        e if e == CAMERA => "ph2d::ecs::GameCamera",
        e if e == BODY => "ph2d::physics::RigidBody",
        e if e == SOUND => "ph2d::ecs::AudioSource2D",
        e if e == HUD => "ph2d::ecs::UiCanvas",
        _ => return None,
    })
}

/// Cria um objecto vazio na raiz, com o nome dado (tornado único), e devolve os bits dele.
///
/// ⚠️ **`Transform` + `Name`, e mais NADA.** O Inspector mostra o que o objecto TEM, então um
/// vazio acabado de nascer mostra duas secções, não doze (ADR-0166 / F3).
///
/// ⚠️ **Os dois assigners correm aqui, não «depois»:** uma raiz sem `RootOrder` desempata pelo
/// `Entity::to_bits()`, que muda a cada respawn do undo (BUGS #15); o `StableId` é a identidade
/// que sobrevive a esse respawn.
pub fn spawn_empty_root(sim: &mut SimWorld, name: &str) -> u64 {
    let name = ph2d_unique_name::unique_name(sim, name);
    let bits = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    bits
}

/// ⭐ **Cria o objecto de jogo** desta entrada — `None` se a entrada não é desta família.
///
/// ⚠️ Se o componente não puder entrar, o vazio SAI: um objecto chamado *«Camera»* sem câmara
/// seria um nome que mente.
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    seeds: crate::component_seed::SeedTable<'_>,
) -> Option<Result<u64, String>> {
    let component = component_of(entry)?;
    let bits = spawn_empty_root(sim, entry.key.tr());
    Some(
        match crate::component_attach::attach_by_name(sim, registry, seeds, bits, component) {
            Ok(()) => Ok(bits),
            Err(msg) => {
                sim.world_mut().despawn(ph2d_ecs::Entity::from_bits(bits));
                Err(msg)
            }
        },
    )
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
