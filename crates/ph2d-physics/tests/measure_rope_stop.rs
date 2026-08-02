//! **O QUE ACONTECE HOJE quando um guincho recolhe até o fim** (W-RopeStop).
//!
//! Enio: *"vamos criar limitadores de modo que tem a possibilidade dos objetos
//! nunca colidirem com as polias"*. Antes de escrever uma linha da cura, esta
//! sonda mede o defeito pela porta do PRODUTO (`PhysicsWorld::step`) e imprime a
//! coluna que decide tudo: a **folga de tangente** entre a amarração da carga e a
//! roldana.
//!
//! ⚠️ **A folga é `√(d² − r²)`, não `d`** — é o comprimento do trecho de corda
//! que sobra, e ele chega a **zero exatamente quando a amarração toca o aro**.
//! Medir `d` diria que ainda há meio metro quando a carga já está encostada numa
//! roldana de meio metro de raio.
//!
//! `cargo test -p ph2d-physics --test measure_rope_stop -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// O raio da roldana, em metros — grande de propósito: é onde a diferença entre
/// *distância ao centro* e *folga de tangente* é visível.
const WHEEL_R: f32 = 0.5;

/// A altura do eixo.
const WHEEL_Y: f32 = 8.0;

/// **O guincho:** um tambor no teto recolhe a corda; a carga sobe.
///
/// A ponta B é uma amarração MORTA (um corpo estático ao lado), então o único
/// lado que corre é o da carga — que é a montagem em que *"a carga sobe até a
/// roldana"* é uma frase com um só significado.
fn winch(rate: f32) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle) {
    const BODY_R: f32 = 0.2;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (dead, _) = w.add_static_cuboid(1.5, WHEEL_Y, 0.1, 0.1);
    let (load, _) = w.add_dynamic_circle(-0.6, 2.0, BODY_R, 1.0 / area);
    let mut wheels = vec![RopeWheel {
        centre: [0.0, WHEEL_Y],
        radius: WHEEL_R,
        id: 1,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([-0.6, 2.0], [1.5, WHEEL_Y], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        stops: [0.0, 0.0],
        id: 1,
        body_a: load,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        total_length: 0.0,
        motor_rate: rate,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, desc, load)
}

/// A folga de tangente entre a amarração de um corpo e a roldana, em metros.
fn gap(w: &PhysicsWorld, body: RigidBodyHandle) -> f32 {
    let p = w.bodies().get(body).expect("corpo").translation();
    let d = (p.x - 0.0).hypot(p.y - WHEEL_Y);
    (d * d - WHEEL_R * WHEEL_R).max(0.0).sqrt()
}

/// **A carga sobe até ENTRAR na roldana, e nada a impede.**
#[test]
#[ignore = "sonda"]
fn measure_the_load_climbing_into_the_wheel() {
    let (mut w, _, load) = winch(0.5);
    println!("\n=== O GUINCHO SEM LIMITADOR (taxa 0,5 m/s) ===");
    println!("   t (s) |  folga (m) |  y da carga");
    for k in 0..=900i16 {
        if k % 90 == 0 {
            let p = w.bodies().get(load).expect("corpo").translation();
            println!(
                "  {:6.2} | {:10.4} | {:8.3}",
                f32::from(k) / 60.0,
                gap(&w, load),
                p.y
            );
        }
        w.step();
    }
    let g = gap(&w, load);
    println!("\n  folga final: {g:.4} m  (zero = a amarração está no aro)");
}

/// **A MESMA cena com limitador** — a carga sobe e PARA.
///
/// O guincho continua recolhendo o tempo todo: o que segura a carga é a trava, e
/// a corda simplesmente deixa de conseguir puxá-la mais.
///
/// ⚠️ A coluna **`folga MAXIMA`** é o CONTROLE contra injeção de energia: sem a
/// lei do nó travado a carga era arrastada por cima do eixo e a folga crescia sem
/// limite (medido, `y` 13,458 m com a roldana a 8,0). Com ela, a carga vira um
/// pêndulo preso à roda, e o máximo fica na ordem do próprio limitador.
#[test]
#[ignore = "sonda"]
fn measure_the_load_stopping_at_the_mark() {
    println!("\n=== O GUINCHO COM LIMITADOR (mesma cena, taxa 0,5 m/s) ===");
    for stop in [0.0f32, 0.5, 1.5, 3.0] {
        let (mut w, mut d, load) = winch(0.5);
        d.stops = [stop, 0.0];
        let wheels = w.pulley_wheels().to_vec();
        w.set_pulleys(vec![d], wheels);
        let (mut lo, mut hi) = (f32::INFINITY, 0.0f32);
        for _ in 0..900 {
            w.step();
            let g = gap(&w, load);
            lo = lo.min(g);
            hi = hi.max(g);
        }
        println!("  limitador {stop:4.2} m -> folga MINIMA {lo:7.4} m · MAXIMA {hi:7.4} m");
    }
}
