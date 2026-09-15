//! **SONDA: o que o NOSSO controlador faz ao deslizar** — o par do
//! `docs/Components/ferramentas/godot_topdown_probe.gd`, com as MESMAS perguntas
//! (o plano 10, TOP-20 #13).
//!
//! Correr: `cargo test -p ph2d-physics sonda_do_deslize -- --nocapture --ignored`
//!
//! ⚠️ **Ela é uma SONDA, não um gate** — `#[ignore]` de propósito: imprime uma
//! tabela para quem desenha, e quem afirma uma lei é um gate com barra.
//!
//! As três perguntas, na ordem em que o oráculo as respondeu:
//!
//! - **B/I** a varredura do ângulo de incidência: quanto anda num tique, de
//!   cabeça contra a parede (0°) a rasante (90°)? O Godot em `FLOATING` conserva
//!   o **orçamento inteiro** (`|d| = |v|·dt`) acima de `wall_min_slide_angle` e
//!   **pára** abaixo dele;
//! - **F** a parede inclinada, abordada de longe;
//! - o **custo** por corpo.
//!
//! ⚠️ **Os transcendentais são do `libm`, e não do `std`** — mesmo aqui, que é uma sonda
//! `#[cfg(test)]` e nunca alcança o `physics_ecs_c9`. O censo
//! (`no_std_transcendental_on_the_hash_path`) é **textual** e varre a crate inteira, e isso é uma
//! escolha: uma varredura que soubesse distinguir «isto é teste» seria uma que alguém podia
//! enganar. *Custa nada obedecer, e a régua fica sem excepções.*
//!
//! ⚠️ A unidade aqui é o **METRO** (a casa não tem escala: o `Transform` já é
//! metros), e a do oráculo é o pixel. As leis de deslize são invariantes à
//! escala **menos as margens**, que é exactamente o que esta tabela deixa ver.

use super::*;
use crate::CharacterParams;
use crate::world::desc::BodyDesc;
use crate::world::shape::ShapeDesc;
use rapier2d::dynamics::RigidBodyType;

const DT: f32 = 1.0 / 60.0;
const V: f32 = 4.0; // m/s ⇒ 0,0667 m por tique
const RAIO: f32 = 0.2;

fn params_de_vista_de_cima() -> CharacterParams {
    CharacterParams {
        // ⚠️ Numa vista de cima não há «cima». O `up` fica a apontar para +Y
        // porque o controlador exige um, e o `max_slope_deg = 0` é o que faz
        // TODA superfície ser parede — que é o `motion_mode = FLOATING`.
        up: [0.0, 1.0],
        snap_distance: 0.0,
        max_slope_deg: 0.0,
        step_height: 0.0,
    }
}

fn corpo(x: f32, y: f32) -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::KinematicPositionBased,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: RAIO },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    }
}

/// Um mundo com uma parede VERTICAL cuja face esquerda fica em `x = -1`.
fn cena_parede(em: [f32; 2]) -> (PhysicsWorld, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 1.0, 20.0); // x ∈ [−1, 1]
    let eu = w.spawn_body(corpo(em[0], em[1]));
    w.step();
    (w, eu)
}

fn anda(w: &mut PhysicsWorld, eu: RigidBodyHandle, pedido: [f32; 2]) -> [f32; 2] {
    let mut hits = Vec::new();
    let m = w.move_character(eu, pedido, params_de_vista_de_cima(), None, 0, &mut hits);
    m.translation
}

#[test]
#[ignore = "sonda: imprime a tabela do plano 10, nao afirma lei nenhuma"]
fn sonda_do_deslize_varredura_do_angulo() {
    println!("\n## NOSSO move_character — varredura do angulo (corpo JA encostado)");
    println!(
        "# raio={RAIO} m · V={V} m/s · dt={DT:.6} ⇒ orcamento {:.6} m/tique",
        V * DT
    );
    println!("# ang  dx  dy  |d|  tangencial_esperada  razao");
    for passo in 0..=18 {
        let ang_g = passo as f32 * 5.0;
        let a = ang_g.to_radians();
        // encostado: a face esta' em x = −1, o raio e' 0,2 ⇒ o centro em −1,2
        let (mut w, eu) = cena_parede([-1.2, 0.0]);
        let pedido = [libm::cosf(a) * V * DT, libm::sinf(a) * V * DT];
        let d = anda(&mut w, eu, pedido);
        let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let tang = libm::sinf(a) * V * DT;
        let razao = if tang.abs() > 1e-9 { n / tang } else { 0.0 };
        println!(
            "N {ang_g:.0} {:.6} {:.6} {n:.6} {tang:.6} {razao:.6}",
            d[0], d[1]
        );
    }
}

#[test]
#[ignore = "sonda: imprime a tabela do plano 10, nao afirma lei nenhuma"]
fn sonda_do_deslize_aproximacao_livre() {
    println!("\n## NOSSO move_character — aproximacao LIVRE a 45 graus (o bloco A do oraculo)");
    println!("# tique  x  y  |d_do_tique|");
    let (mut w, eu) = cena_parede([-3.0, 0.0]);
    let dir = [
        core::f32::consts::FRAC_1_SQRT_2,
        core::f32::consts::FRAC_1_SQRT_2,
    ];
    let mut p = [-3.0f32, 0.0];
    for t in 0..60 {
        let d = anda(&mut w, eu, [dir[0] * V * DT, dir[1] * V * DT]);
        p = [p[0] + d[0], p[1] + d[1]];
        w.set_body_pose(eu, p[0], p[1], 0.0, true);
        w.step();
        if t < 4 || t % 10 == 0 || t == 59 {
            let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
            println!("N {t} {:.6} {:.6} {n:.6}", p[0], p[1]);
        }
    }
}

#[test]
#[ignore = "sonda: imprime a tabela do plano 10, nao afirma lei nenhuma"]
fn sonda_do_deslize_custo() {
    println!("\n## NOSSO move_character — custo por corpo (encostado, a deslizar)");
    println!("# n  ms_total  us_por_corpo");
    for n in [100usize, 1000, 5000] {
        let mut w = PhysicsWorld::new();
        w.add_static_cuboid(0.0, 0.0, 1.0, 20_000.0);
        let mut corpos = Vec::with_capacity(n);
        for k in 0..n {
            corpos.push(w.spawn_body(corpo(-1.2, -10_000.0 + (k as f32) * 0.5)));
        }
        w.step();
        let mut hits = Vec::new();
        let a = 45.0f32.to_radians();
        let pedido = [libm::cosf(a) * V * DT, libm::sinf(a) * V * DT];
        let t0 = std::time::Instant::now();
        for &c in &corpos {
            let _ = w.move_character(c, pedido, params_de_vista_de_cima(), None, 0, &mut hits);
        }
        let us = t0.elapsed().as_secs_f64() * 1e6;
        println!("N {n} {:.3} {:.3}", us / 1000.0, us / (n as f64));
    }
}
