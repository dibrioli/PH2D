//! **Regra B (dono 09/11): pontos animados com o Auto Key desligado.** Uma forma com trilha de pontos (FISICA
//! 51/52) e o Auto Key desligado: todo gesto que mudaria os pontos NÃO muda nada e pede o modal «This shape's
//! points are animated». A shell publica o conjunto por quadro ([`publica`]) e lê o pedido ([`pedido`]); os
//! produtores (o ponteiro, o «Merge», o retraço) perguntam [`recusa`]. Plano: `f2_contorno_e_formas_plano.md`.

use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use std::collections::BTreeSet;

/// As formas cujos pontos a timeline conduz sem o Auto Key gravar, e o pedido de modal em aberto.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct PontosAnimados {
    pub formas: BTreeSet<u64>,
    pub pedido: Option<u64>,
}

/// Uma vez por quadro: as formas travadas agora (vazio com o Auto Key ligado). O pedido em aberto fica.
pub fn publica(world: &mut World, formas: BTreeSet<u64>) {
    world
        .get_resource_or_insert_with(PontosAnimados::default)
        .formas = formas;
}

/// `true` ⇒ os pontos de `forma` estão travados (só lê; o retraço, que não tem o mundo mutável).
#[must_use]
pub fn travado(world: &World, forma: u64) -> bool {
    world
        .get_resource::<PontosAnimados>()
        .is_some_and(|p| p.formas.contains(&forma))
}

/// O gesto sobre `forma` é recusado? Se sim, deixa o pedido do modal para a shell.
pub fn recusa(world: &mut World, forma: u64) -> bool {
    if !travado(world, forma) {
        return false;
    }
    if let Some(mut p) = world.get_resource_mut::<PontosAnimados>() {
        p.pedido = Some(forma);
    }
    true
}

/// O pedido do modal (tirado: abre-se uma vez).
pub fn pedido(world: &mut World) -> Option<u64> {
    world.get_resource_mut::<PontosAnimados>()?.pedido.take()
}
