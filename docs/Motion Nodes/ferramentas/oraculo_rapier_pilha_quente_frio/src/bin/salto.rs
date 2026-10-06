// O salto efectivo do rapier: uma caixa a 1 u/s contra outra igual parada, ou contra um pino, encostadas.
use rapier2d::prelude::*;
fn corre(e: f32, pino: bool, estab: usize, iter: usize, folga: f32) -> (f32, f32) {
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let mut hs = Vec::new();
    for (x, vx, fixo) in [(-0.22 - folga, 1.0, false), (0.0, 0.0, pino)] {
        let b = if fixo { RigidBodyBuilder::fixed() } else { RigidBodyBuilder::dynamic() };
        let h = bodies.insert(b.translation(Vector::new(x, 0.0)).linvel(Vector::new(vx, 0.0)).can_sleep(false).build());
        colliders.insert_with_parent(ColliderBuilder::cuboid(0.11, 0.11).restitution(e).restitution_combine_rule(CoefficientCombineRule::Max).build(), h, &mut bodies);
        hs.push(h);
    }
    let mut p = IntegrationParameters::default();
    p.num_internal_stabilization_iterations = estab;
    p.num_solver_iterations = iter;
    let (mut pipe, mut isl, mut br, mut na, mut j, mut mj, mut ccd) = (PhysicsPipeline::new(), IslandManager::new(), DefaultBroadPhase::new(), NarrowPhase::new(), ImpulseJointSet::new(), MultibodyJointSet::new(), CCDSolver::new());
    for _ in 0..3 { pipe.step(Vector::ZERO, &p, &mut isl, &mut br, &mut na, &mut bodies, &mut colliders, &mut j, &mut mj, &mut ccd, &(), &()); }
    (bodies[hs[0]].linvel().x, bodies[hs[1]].linvel().x)
}
fn main() {
    for (estab, iter) in [(1, 4), (4, 4), (4, 1), (4, 8)] {
        for folga in [0.0_f32, 0.005] {
            let mut l = format!("estab {estab} iter {iter} folga {folga}:");
            for e in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
                let (a, b) = corre(e, false, estab, iter, folga);
                let (w, _) = corre(e, true, estab, iter, folga);
                l += &format!("  e{e}: par {a:.3}/{b:.3} (conta {:.3}/{:.3}) pino {w:.3}", (1.0 - e) / 2.0, (1.0 + e) / 2.0);
            }
            println!("{l}");
        }
    }
}
