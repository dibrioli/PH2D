//! **O EMPURRÃO LATERAL do limitador** — o que o smoke do Enio reportou.
//!
//! Enio: *"quando o corpo encosta no limitador gera uma força bizarra que empurra
//! o objeto na direção x das polias e o objeto fica pendulando para dentro"*.
//!
//! Esta sonda mede pela porta do PRODUTO (`PhysicsWorld::step`).
//!
//! ⚠️ **Uma corda puxa AO LONGO DE SI MESMA.** Qualquer componente perpendicular é
//! força sem matéria que a transmita — não existe.
//!
//! ⚠️ **E o oráculo NÃO é um ângulo: é ONDE A CARGA DESCANSA.** Uma corda vertical
//! deixa a roda pela TANGENTE, então uma carga pendurada por uma corda repousa em
//! `x = ±r` — embaixo do ponto de tangência. Uma força RADIAL a põe em `x = 0`,
//! embaixo do CENTRO. Os dois lugares distam exatamente o raio, e essa é a
//! diferença que se vê na tela.
//!
//! ⚠️ **O teste decisivo é o SOLTAR:** posta em repouso no lugar onde a corda fica
//! vertical, uma carga segurada ao longo da corda **fica**. Segurada pelo radial,
//! ela é empurrada de lado — `tan(atan(r/s))` do próprio peso, 44% em `r = 0,7`
//! com limitador `1,6`.
//!
//! `cargo test -p ph2d-physics --test measure_stop_sideways -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// O raio da roldana da cena 75.
const R: f32 = 0.7;
/// A altura do eixo.
const WHEEL_Y: f32 = 6.0;
/// O limitador da cena 75, em metros de corda.
const STOP: f32 = 1.6;

/// O guincho da cena 75: eixo em `(0, 6)`, ponta morta ao lado, carga em `(x, y)`.
fn rig_at(stop: f32, x: f32, y: f32) -> (PhysicsWorld, RigidBodyHandle) {
    const BODY_R: f32 = 0.3;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (dead, _) = w.add_static_cuboid(2.0, WHEEL_Y, 0.15, 0.15);
    let (load, _) = w.add_dynamic_circle(x, y, BODY_R, 1.0 / area);
    let mut wheels = vec![RopeWheel {
        centre: [0.0, WHEEL_Y],
        radius: R,
        id: 1,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([x, y], [2.0, WHEEL_Y], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        stops: [stop, 0.0],
        id: 1,
        body_a: load,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        total_length: 0.0,
        motor_rate: 0.5,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, load)
}

/// O ângulo, em graus, entre a direção RADIAL e a da CORDA — ou seja, **quanto uma
/// lei radial erra**. É `atan(r / folga)`, e é zero só quando a roldana é um ponto.
///
/// ⚠️ Isto é GEOMETRIA, não uma medição de força: ele não muda quando o kernel
/// muda. Quem julga o kernel é [`measure_the_release_at_the_rope_vertical`].
fn tilt_deg(w: &PhysicsWorld, load: RigidBodyHandle) -> f32 {
    let p = w.bodies().get(load).expect("corpo").translation();
    let d = p.x.hypot(p.y - WHEEL_Y);
    let len = (d * d - R * R).max(0.0).sqrt().max(1e-6);
    (R / len).atan().to_degrees()
}

/// **O que o smoke viu.**
#[test]
#[ignore = "sonda"]
fn measure_the_sideways_push() {
    println!("\n=== A CARGA ENCOSTA NO LIMITADOR (raio {R}, limitador {STOP} m) ===");
    println!("   t (s) |   x (m) |   y (m) |  folga (m)");
    let (mut w, load) = rig_at(STOP, 0.0, 0.4);
    let (mut worst_x, mut settled) = (0.0f32, 0.0f32);
    let (mut mean_x, mut n) = (0.0f32, 0.0f32);
    for k in 0..=900i16 {
        if k % 60 == 0 {
            let p = w.bodies().get(load).expect("corpo").translation();
            let d = p.x.hypot(p.y - WHEEL_Y);
            println!(
                "  {:6.2} | {:7.3} | {:7.3} | {:10.4}",
                f32::from(k) / 60.0,
                p.x,
                p.y,
                (d * d - R * R).max(0.0).sqrt()
            );
        }
        w.step();
        let p = w.bodies().get(load).expect("corpo").translation();
        worst_x = worst_x.max(p.x.abs());
        settled = p.x;
        if k >= 600 {
            mean_x += p.x;
            n += 1.0;
        }
    }
    println!("\n  |x| MAXIMO: {worst_x:.4} m   ·   x final: {settled:.4} m");
    println!("  CENTRO do pendulo apos travar: {:.4} m", mean_x / n);
    println!(
        "  (uma corda o poe em -r = {:.2}; o radial o poe em 0,00)",
        -R
    );
    println!(
        "  uma lei radial erraria {:.2} graus aqui",
        tilt_deg(&w, load)
    );
}

/// **O TESTE DECISIVO — a carga solta onde a corda fica VERTICAL.**
///
/// Em `(-r, eixo − s)` a corda desce a prumo do ponto de tangência e o peso é
/// vertical: quem segura ao longo da corda equilibra exatamente, e a carga **não
/// anda de lado**. Quem segura pelo radial empurra com `tan(atan(r/s))` do peso.
#[test]
#[ignore = "sonda"]
fn measure_the_release_at_the_rope_vertical() {
    let (mut w, load) = rig_at(STOP, -R, WHEEL_Y - STOP);
    println!(
        "\n=== SOLTA NO PRUMO DA TANGENCIA (x = {:.2}, y = {:.2}) ===",
        -R,
        WHEEL_Y - STOP
    );
    println!("   t (s) |  desvio lateral (m) |  vx (m/s)");
    let mut worst = 0.0f32;
    for k in 0..=180i16 {
        if k % 30 == 0 {
            let b = w.bodies().get(load).expect("corpo");
            println!(
                "  {:6.2} | {:19.4} | {:9.4}",
                f32::from(k) / 60.0,
                b.translation().x + R,
                b.linvel().x
            );
        }
        w.step();
        let b = w.bodies().get(load).expect("corpo");
        worst = worst.max((b.translation().x + R).abs());
    }
    println!("\n  DESVIO LATERAL MAXIMO em 3 s: {worst:.4} m");
}

/// **A tabela que nomeia o defeito, sem relógio nenhum.**
///
/// O desvio é pura geometria: `atan(raio / limitador)`. Ele não depende da cena,
/// da massa nem do guincho — só de quão grande é a roldana comparada à corda que
/// o artista mandou sobrar.
#[test]
#[ignore = "sonda"]
fn measure_the_tilt_is_pure_geometry() {
    println!("\n=== QUANTO A FORCA SAI DA CORDA (graus) ===");
    println!("  raio \\ limitador |   0.5 |   1.0 |   1.6 |   3.0");
    for r in [0.2f32, 0.5, 0.7, 1.0, 2.0] {
        print!("  {r:16.1} |");
        for s in [0.5f32, 1.0, 1.6, 3.0] {
            print!(" {:5.1} |", (r / s).atan().to_degrees());
        }
        println!();
    }
    println!("\n  (zero = a corda puxa ao longo de si mesma, que e' o unico jeito de puxar)");
}

/// **A RODA GRANDE** — o regime em que o gate existente reprova: `r = 2,0` com
/// limitador `0,5`, onde o balanço perpendicular consome folga `r/len = 4×` mais
/// rápido do que a trava a repõe pela corda.
#[test]
#[ignore = "sonda"]
fn measure_the_big_wheel() {
    const BIG_R: f32 = 2.0;
    const BODY_R: f32 = 0.2;
    const WY: f32 = 8.0;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (dead, _) = w.add_static_cuboid(3.0, WY, 0.1, 0.1);
    let (load, _) = w.add_dynamic_circle(0.0, 2.0, BODY_R, 1.0 / area);
    let mut wheels = vec![RopeWheel {
        centre: [0.0, WY],
        radius: BIG_R,
        id: 1,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([0.0, 2.0], [3.0, WY], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        id: 1,
        body_a: load,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        total_length: 0.0,
        motor_rate: 0.5,
        break_force: f32::INFINITY,
        stops: [0.0, 0.0],
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut d2 = desc;
    d2.total_length = w.pulley_span(&d2).expect("rota sã");
    d2.stops = [0.5, 0.0];
    w.set_pulleys(vec![d2], wheels);
    println!("\n=== RODA GRANDE (r 2,0 · limitador 0,5) ===");
    println!("   t (s) |   x (m) |   y (m) |  folga (m)");
    let mut lo = f32::INFINITY;
    for k in 0..=900i16 {
        if k % 60 == 0 {
            let p = w.bodies().get(load).expect("corpo").translation();
            let d = p.x.hypot(p.y - WY);
            println!(
                "  {:6.2} | {:7.3} | {:7.3} | {:10.4}",
                f32::from(k) / 60.0,
                p.x,
                p.y,
                (d * d - BIG_R * BIG_R).max(0.0).sqrt()
            );
        }
        w.step();
        let p = w.bodies().get(load).expect("corpo").translation();
        let d = p.x.hypot(p.y - WY);
        lo = lo.min((d * d - BIG_R * BIG_R).max(0.0).sqrt());
    }
    println!("\n  folga MINIMA: {lo:.4} m");
}
