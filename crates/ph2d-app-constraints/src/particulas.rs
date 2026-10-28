//! O pedido ao Motion: os objectos de que algum ponto de simulação lê partículas, e os `id` de cada
//! um (`docs/BlenderConstraints/07_W6_pontas.md` §5). O Motion arma as tomadas dos sinks deles e
//! devolve só essas partículas em `ph2d_ecs::motion_generator::Particulas`.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_constraints::Kind;
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::SimWorld;
use ph2d_ecs::motion_generator::{IdsPedidos, ParticulasPedidas};

/// Reescreve o pedido (só quando muda: o Motion arma as tomadas pela diferença).
pub(crate) fn pedir(sim: &mut SimWorld) {
    let w = sim.world();
    let mut ids: BTreeMap<_, BTreeSet<u32>> = BTreeMap::new();
    for it in w
        .iter_entities()
        .filter_map(|e| e.get::<Constraints>())
        .flat_map(|c| &c.items)
    {
        if let (Kind::PontoDeSimulacao(p), Some(s)) = (it.kind, it.target) {
            ids.entry(s).or_default().insert(p.id);
        }
    }
    let pedidas: BTreeSet<_> = ids.keys().copied().collect();
    if w.get_resource::<ParticulasPedidas>().map(|p| &p.0) != Some(&pedidas) {
        sim.world_mut().insert_resource(ParticulasPedidas(pedidas));
    }
    if sim.world().get_resource::<IdsPedidos>().map(|p| &p.0) != Some(&ids) {
        sim.world_mut().insert_resource(IdsPedidos(ids));
    }
}
