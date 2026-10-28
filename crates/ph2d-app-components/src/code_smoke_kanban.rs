//! ⭐ **A cena 10 do editor de scripts** (`PH2D_CODE_SMOKE=10`, TrelloClone J3): um script que mexe no
//! KANBAN do projecto (`ph2d.kanban`), aberto numa aba. O Kanban «Short film — production» põe-no o
//! `ph2d-app-kanban` (lê a mesma variável); aqui só o script. Nenhuma forma: o script não é da cena.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_script::{ProjectScripts, ScriptEditRequests};

pub use ph2d_kanban_scenes::script::{KANBAN_LUAU, KANBAN_NAME};

/// Põe o script no projecto e pede a aba dele. Nenhuma entidade.
pub fn monta(sim: &mut SimWorld) -> Vec<Entity> {
    let world = sim.world_mut();
    let id = world
        .get_resource_or_init::<ProjectScripts>()
        .insert(KANBAN_NAME, KANBAN_LUAU);
    world
        .get_resource_or_init::<ScriptEditRequests>()
        .0
        .push(id);
    Vec::new()
}

/// A porta da shell: monta e anuncia. Nada fica escolhido.
pub fn montar(cx: crate::scene_ctx::SceneCtx<'_>) -> Option<u64> {
    monta(cx.sim);
    println!(
        "[code-smoke] =10 o kanban.luau, já aberto numa aba, mexe no Kanban «Short film — production» \
         do projeto: lê-o (ph2d.kanban.get), procura os atrasados (search \"due:overdue\"), comenta e \
         move «Animate shot 12». Escreva  card:  na linha 1): a lista dos métodos do cartão abre."
    );
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_cena_10_abre_o_script_do_kanban_e_nenhuma_forma() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut cena = ph2d_vec_scene::VecScene::new();
        let mut mapa = ph2d_vec_entities::entities::VecEntityMap::new();
        let formas = crate::code_smoke::monta_em(&mut sim, &mut cena, &mut mapa, 10);
        assert!(formas.is_empty());
        let w = sim.world();
        let script = w
            .resource::<super::ProjectScripts>()
            .by_name(super::KANBAN_NAME)
            .expect("o script no projeto");
        assert_eq!(w.resource::<super::ScriptEditRequests>().0, [script]);
    }
}
