// Duas peças que nascem sobrepostas: quanto tempo para separar, e que velocidade sobra.
use rapier2d::prelude::*;
fn corre(nome: &str, params: IntegrationParameters, caixa: bool, meio: f32, v: f32) {
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let mut hs = Vec::new();
    for (x, vx) in [(-meio, v), (meio, -v)] {
        let h = bodies.insert(RigidBodyBuilder::dynamic().translation(Vector::new(x, 0.0)).linvel(Vector::new(vx, 0.0)).can_sleep(false).build());
        let c = if caixa { ColliderBuilder::cuboid(0.5, 2.0) } else { ColliderBuilder::ball(0.5) };
        colliders.insert_with_parent(c.build(), h, &mut bodies);
        hs.push(h);
    }
    let (mut pipe, mut isl, mut br, mut na, mut j, mut mj, mut ccd) = (PhysicsPipeline::new(), IslandManager::new(), DefaultBroadPhase::new(), NarrowPhase::new(), ImpulseJointSet::new(), MultibodyJointSet::new(), CCDSolver::new());
    let mut linha = String::new();
    for k in 1..=120 {
        pipe.step(Vector::ZERO, &params, &mut isl, &mut br, &mut na, &mut bodies, &mut colliders, &mut j, &mut mj, &mut ccd, &(), &());
        if [1, 5, 10, 30, 60, 120].contains(&k) {
            let d = bodies[hs[1]].translation().x - bodies[hs[0]].translation().x;
            let vr = bodies[hs[1]].linvel().x - bodies[hs[0]].linvel().x;
            linha += &format!(" | t{k}: d {d:.4} vrel {vr:+.3}");
        }
    }
    println!("{nome:28} {}{linha}", if caixa { "caixas" } else { "discos" });
}
fn main() {
    let a = IntegrationParameters::default();
    let mut b = IntegrationParameters::default();
    b.contact_softness = SpringCoefficients { natural_frequency: 120.0, damping_ratio: 5.0 };
    b.static_contact_softness = SpringCoefficients { natural_frequency: 120.0, damping_ratio: 5.0 };
    b.num_internal_stabilization_iterations = 2;
    b.normalized_allowed_linear_error = 0.001;
    b.normalized_max_corrective_velocity = 10.0;
    b.normalized_prediction_distance = 0.002;
    b.contact_clustering = false;
    b.contact_recycling = false;
    let mut c = IntegrationParameters::default();
    c.num_internal_stabilization_iterations = 4;
    let mut c2 = IntegrationParameters::default();
    c2.num_internal_stabilization_iterations = 2;
    let mut c3 = IntegrationParameters::default();
    c3.num_internal_stabilization_iterations = 3;
    let _ = (a, b);
    for (nome, p) in [("omissao + 2 estabilizacoes", c2), ("omissao + 3 estabilizacoes", c3), ("omissao + 4 estabilizacoes", c)] {
        corre(nome, p, false, 0.1, 0.0);
        corre(nome, p, true, 0.3, 1.0);
        corre(nome, p, true, 0.3, 0.0);
    }
}
