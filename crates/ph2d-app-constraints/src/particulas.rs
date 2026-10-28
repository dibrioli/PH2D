//! O pedido ao Motion: os objectos de que algum ponto de simulação lê partículas
//! (`docs/BlenderConstraints/07_W6_pontas.md` §5). O Motion arma as tomadas dos sinks deles e
//! devolve as partículas em `ph2d_ecs::motion_generator::Particulas`.

use ph2d_constraints::Kind;
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::SimWorld;
use ph2d_ecs::motion_generator::ParticulasPedidas;

/// Reescreve o pedido (só quando muda: o Motion arma as tomadas pela diferença).
pub(crate) fn pedir(sim: &mut SimWorld) {
    let w = sim.world();
    let pedidas: std::collections::BTreeSet<_> = w
        .iter_entities()
        .filter_map(|e| e.get::<Constraints>())
        .flat_map(|c| &c.items)
        .filter(|it| matches!(it.kind, Kind::PontoDeSimulacao(_)))
        .filter_map(|it| it.target)
        .collect();
    if w.get_resource::<ParticulasPedidas>().map(|p| &p.0) != Some(&pedidas) {
        sim.world_mut().insert_resource(ParticulasPedidas(pedidas));
    }
}
