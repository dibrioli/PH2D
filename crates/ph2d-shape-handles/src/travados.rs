//! **Regra B (dono 09/11): pontos animados.** Uma forma com trilha de pontos (FISICA 51/52): todo gesto que
//! muda o NÚMERO de pontos NÃO muda nada e pede o modal «This shape's points are animated», com ou sem Auto Key;
//! MOVER recusa-se só com o Auto Key desligado (ligado, grava chave). A shell publica por quadro ([`publica`]) e
//! lê o pedido ([`pedido`]); os produtores perguntam [`recusa`] (mover) ou [`recusa_numero`] (aresta, Alt, Delete,
//! fusão, «Merge», retraço). Plano: `f2_contorno_e_formas_plano.md`.

use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use std::collections::BTreeSet;

/// As formas com trilha de pontos, o Auto Key do quadro, e o pedido de modal em aberto.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct PontosAnimados {
    pub formas: BTreeSet<u64>,
    pub auto_key: bool,
    pub pedido: Option<u64>,
}

/// Uma vez por quadro: as formas com trilha de pontos e o Auto Key. O pedido em aberto fica.
pub fn publica(world: &mut World, formas: BTreeSet<u64>, auto_key: bool) {
    let mut p = world.get_resource_or_insert_with(PontosAnimados::default);
    p.formas = formas;
    p.auto_key = auto_key;
}

fn animada(world: &World, forma: u64) -> Option<bool> {
    let p = world.get_resource::<PontosAnimados>()?;
    p.formas.contains(&forma).then_some(p.auto_key)
}

/// `true` ⇒ MOVER os pontos de `forma` está travado (trilha e Auto Key desligado; ligado, grava chave).
#[must_use]
pub fn travado(world: &World, forma: u64) -> bool {
    animada(world, forma) == Some(false)
}

/// `true` ⇒ mudar o NÚMERO de pontos de `forma` está travado (trilha, com ou sem Auto Key; só lê).
#[must_use]
pub fn numero_travado(world: &World, forma: u64) -> bool {
    animada(world, forma).is_some()
}

fn pede(world: &mut World, forma: u64, travada: bool) -> bool {
    if travada && let Some(mut p) = world.get_resource_mut::<PontosAnimados>() {
        p.pedido = Some(forma);
    }
    travada
}

/// Mover os pontos de `forma` é recusado? Se sim, deixa o pedido do modal para a shell.
pub fn recusa(world: &mut World, forma: u64) -> bool {
    let t = travado(world, forma);
    pede(world, forma, t)
}

/// Mudar o NÚMERO de pontos de `forma` é recusado? Se sim, deixa o pedido do modal para a shell.
pub fn recusa_numero(world: &mut World, forma: u64) -> bool {
    let t = numero_travado(world, forma);
    pede(world, forma, t)
}

/// O pedido do modal (tirado: abre-se uma vez).
pub fn pedido(world: &mut World) -> Option<u64> {
    world.get_resource_mut::<PontosAnimados>()?.pedido.take()
}
