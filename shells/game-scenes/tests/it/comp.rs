//! ⭐ **As portas como a composição as liga** — o espelho da `shells/desktop` (`janela_do_jogo::game_smoke` e
//! `render_loop::fase_object_add`): o vazio na raiz e o `+` dos componentes, com as sementes da física, e a
//! *Camera* do menu dos componentes. Uma ligação diferente aqui seria testar outro programa.

use ph2d_app_components::{component_attach, object_add};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, SimWorld};

/// O catálogo de componentes que o produto monta (com gate na família dos componentes).
pub fn registo() -> ComponentRegistry {
    ph2d_app_components::test_support::registo()
}

/// Corre `f` com as portas do nível ligadas sobre `reg`.
pub fn com_portas<R>(
    reg: &ComponentRegistry,
    f: impl FnOnce(ph2d_app_game::smoke::Portas<'_>) -> R,
) -> R {
    com_portas_e_tags(reg, f).0
}

/// [`com_portas`], e a árvore de tags do projecto que a cena escreveu (a que a shell dá à ponte).
pub fn com_portas_e_tags<R>(
    reg: &ComponentRegistry,
    f: impl FnOnce(ph2d_app_game::smoke::Portas<'_>) -> R,
) -> (R, ph2d_tags::TagTree) {
    let seeds = ph2d_app_physics::physics_seed::COMPONENT_SEEDS;
    let anexa =
        |s: &mut SimWorld, b, n: &str| component_attach::attach_by_name(s, reg, seeds, b, n);
    let componentes = |s: &mut SimWorld, c: &str| object_add::add_by_component(c, s, reg, seeds);
    let menu = ph2d_app_game::object_add::Portas {
        raiz: object_add::spawn_empty_root,
        anexa: &anexa,
    };
    let arvore = std::cell::RefCell::new(ph2d_tags::TagTree::new());
    let marca = |s: &mut SimWorld, e, c: &str| {
        ph2d_app_components::tags_doc::marca(&mut arvore.borrow_mut(), s.world_mut(), e, c)
    };
    let r = f(ph2d_app_game::smoke::Portas {
        menu,
        componentes: &componentes,
        importa_som: ph2d_app_components::sound_project::import_for_scene,
        marca: &marca,
    });
    (r, arvore.into_inner())
}

/// Monta o nível `nivel` como o smoke o monta. Devolve o jogador.
pub fn montar(sim: &mut SimWorld, nivel: u32) -> Entity {
    montar_com_tags(sim, nivel).0
}

/// [`montar`], e a árvore de tags que a cena escreveu.
pub fn montar_com_tags(sim: &mut SimWorld, nivel: u32) -> (Entity, ph2d_tags::TagTree) {
    com_portas_e_tags(&registo(), |p| ph2d_app_game::smoke::montar(sim, p, nivel))
}

/// Um objecto do menu Add do JOGO pelas portas do produto — `None` se a entrada não é dele.
pub fn menu(
    entry: ph2d_editor_core::object_add::AddEntry,
    sim: &mut SimWorld,
) -> Option<Result<u64, String>> {
    com_portas(&registo(), |p| {
        ph2d_app_game::object_add::add(entry, sim, p.menu, None)
    })
}
