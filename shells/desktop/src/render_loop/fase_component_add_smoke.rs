//! ⭐ **A foto do Add Component** (plano `docs/GameEngineFuncional/25_add_component_plano.md` §2.6): o clique
//! sintético não chega à tela virtual, então `PH2D_COMPONENT_ADD_SMOKE=1` monta a moeda da Crítica no
//! objecto que o Add Object acabou de criar (`PH2D_OBJECT_ADD_PICK=object_add.game.sprite`) e abre a
//! janela nele. `PH2D_OBJECT_ADD_HOVER` / `_QUERY` destacam e buscam (os da paleta, partilhados), e
//! `PH2D_COMPONENT_ADD_PICK=<nome canónico>` adiciona uma peça para a foto do recibo.

use ph2d_editor_core::screens::hero::HeroScreen;
use std::sync::atomic::{AtomicBool, Ordering};

/// As peças da moeda da foto de 58 itens da Crítica.
const MOEDA: [&str; 5] = [
    "ph2d::physics::RigidBody",
    "ph2d::ecs::AudioSource2D",
    "ph2d::ecs::SignalActions",
    "ph2d::ecs::SpriteAnimations",
    "ph2d::ecs::SpriteAnimator",
];

/// UMA vez, quando o objecto selecionado já existe: monta a moeda e devolve-o para a janela abrir.
pub(super) fn abre(
    hero: &HeroScreen,
    sim: &mut ph2d_ecs::SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
) -> Option<u64> {
    static FEITO: AtomicBool = AtomicBool::new(false);
    if FEITO.load(Ordering::Relaxed)
        || std::env::var("PH2D_COMPONENT_ADD_SMOKE").as_deref() != Ok("1")
    {
        return None;
    }
    let bits = hero.gizmo.selection?;
    FEITO.store(true, Ordering::Relaxed);
    let seeds = ph2d_app_physics::physics_seed::COMPONENT_SEEDS;
    for name in MOEDA {
        if let Err(e) =
            ph2d_app_components::component_attach::attach_by_name(sim, registry, seeds, bits, name)
        {
            eprintln!("[component-add-smoke] {name}: {e}");
        }
    }
    Some(bits)
}

/// UMA vez, com a janela aberta: o pick de `PH2D_COMPONENT_ADD_PICK`.
pub(super) fn escolhe(hero: &mut HeroScreen) {
    static FEITO: AtomicBool = AtomicBool::new(false);
    if hero.store.command_palette_open()
        && let Ok(name) = std::env::var("PH2D_COMPONENT_ADD_PICK")
        && !FEITO.swap(true, Ordering::Relaxed)
    {
        // Como o clique: escolhe e fecha (`chrome/command_palette.rs`).
        hero.store
            .set_command_pick(ph2d_tool_registry::hash_node_id_runtime(&name));
        hero.store.close_command_palette();
    }
}
