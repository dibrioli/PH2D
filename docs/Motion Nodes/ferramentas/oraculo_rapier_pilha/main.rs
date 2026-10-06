// Oráculo (doc 121 §9.19): a pilha da =114 no rapier2d — k×k caixas de meio-lado 0,11 numa taça, gravidade 4.
use rapier2d::prelude::*;
use std::time::Instant;

fn main() {
    let arg = |i: usize, d: usize| std::env::args().nth(i).and_then(|v| v.parse().ok()).unwrap_or(d);
    let k = arg(1, 32);
    let passos = arg(2, 1);
    let iter = arg(3, 4);
    let (gap, lado, altura, taca_y) = (0.32_f32, 0.11_f32, -0.35_f32, -1.2_f32);
    let raio = k as f32 * gap * 0.75 + (altura - taca_y) + 0.2;
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    // A taça: o círculo inteiro em segmentos (as peças nascem dentro dele).
    let n = 512;
    let pts: Vec<Vector> = (0..=n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            Vector::new(raio * a.cos(), taca_y + raio * a.sin())
        })
        .collect();
    colliders.insert(ColliderBuilder::polyline(pts, None).friction(0.6).build());
    let meio = (k as f32 - 1.0) * 0.5;
    for i in 0..k * k {
        let (c, r) = ((i % k) as f32, (i / k) as f32);
        let rb = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(Vector::new((c - meio) * gap, altura + (r - meio) * gap))
                .build(),
        );
        colliders.insert_with_parent(ColliderBuilder::cuboid(lado, lado).friction(0.5).build(), rb, &mut bodies);
    }
    let mut params = IntegrationParameters::default();
    params.dt = 1.0 / 60.0 / passos as f32;
    params.num_solver_iterations = iter;
    let mut pipeline = PhysicsPipeline::new();
    let mut islands = IslandManager::new();
    let mut broad = DefaultBroadPhase::new();
    let mut narrow = NarrowPhase::new();
    let mut joints = ImpulseJointSet::new();
    let mut multi = MultibodyJointSet::new();
    let mut ccd = CCDSolver::new();
    let g = Vector::new(0.0, -4.0);
    let mut t = Vec::new();
    // `6` s: a pilha assenta e fica (sem o recomeço da cena do Motion).
    for _ in 0..360 {
        let t0 = Instant::now();
        for _ in 0..passos {
            pipeline.step(
                g, &params, &mut islands, &mut broad, &mut narrow, &mut bodies, &mut colliders, &mut joints,
                &mut multi, &mut ccd, &(), &(),
            );
        }
        t.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let mut todos = t.clone();
    todos.sort_by(f64::total_cmp);
    let ys: Vec<f32> = bodies.iter().map(|(_, b)| b.translation().y).collect();
    let ymin = ys.iter().cloned().fold(f32::INFINITY, f32::min);
    eprintln!(
        "RAPIER {} caixas · {passos} sub-passo(s) · {iter} iteracoes · tique: med {:.2} ms · p95 {:.2} · max {:.2} · peca mais funda y {ymin:.2} (fundo {:.2})",
        k * k,
        todos[todos.len() / 2],
        todos[todos.len() * 95 / 100],
        todos[todos.len() - 1],
        taca_y - raio
    );
}
