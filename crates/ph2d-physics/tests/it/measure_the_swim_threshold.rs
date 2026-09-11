//! **SONDA — onde é que o personagem deixa de andar e passa a NADAR.**
//!
//! A `W-Swim` precisa de um limiar, e o plano 08 §4.1 diz de onde ele sai:
//! *"com o limiar a sair de medição e não de palpite"*. A grandeza que a lei
//! recebe é a [`FluidAt::buoyed`] — **quantos pesos deste corpo o fluido
//! carrega** —, e a pergunta desta sonda é o que ela vale em cada altura de
//! água, na cápsula e na poça que as fixtures do player já usam.
//!
//! ⚠️ **A leitura sai da porta do PRODUTO** ([`PhysicsWorld::fluid_at`]) — é
//! literalmente a função que a ponte chama por tique de player. Uma segunda
//! conta de área submersa aqui mediria a minha aritmética, não a do solver.
//!
//! ⚠️ **O corpo é FIXO em cada amostra**, e é o que torna isto uma varredura de
//! ALTURA em vez de uma queda: um corpo dinâmico responderia à água enquanto é
//! medido, e cada linha da tabela falaria de um `y` diferente do que a rotula.
//!
//! Rodar: `cargo test -p ph2d-physics --release --test measure_the_swim_threshold -- --ignored --nocapture`

use ph2d_physics::{AreaEffect, BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// A cápsula das fixtures do player e a poça 4× mais densa que ela — os mesmos
/// números do `player_in_water.rs` e do `measure_the_bobbing.rs`, para os três
/// falarem da mesma cena.
const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLUID: f32 = 4.0;
const DRAG: f32 = 0.6;

/// A meia-altura TOTAL da cápsula — o que decide onde a água a alcança.
const HALF_TALL: f32 = HALF_H + RADIUS;

fn desc(body_type: RigidBodyType, y: f32, shape: ShapeDesc) -> BodyDesc {
    BodyDesc {
        body_type,
        x: 0.0,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    }
}

/// A poça: a superfície fica em `y = 0`, como na fixture do bobeio.
fn pool(w: &mut PhysicsWorld) {
    w.spawn_body(BodyDesc {
        is_sensor: true,
        effector: Some(AreaEffect {
            force: [0.0, 0.0],
            drag: DRAG,
            density: FLUID,
            form_drag: 0.0,
            torque: 0.0,
            world_axes: false,
            falloff: 0.0,
            mirror: [1.0, 1.0],
        }),
        ..desc(
            RigidBodyType::Fixed,
            -3.0,
            ShapeDesc::Cuboid {
                half_x: 20.0,
                half_y: 3.0,
            },
        )
    });
}

/// Quanto do peso o fluido carrega, com o centro da cápsula em `y`.
fn buoyed_at(y: f32) -> f32 {
    let mut w = PhysicsWorld::new();
    pool(&mut w);
    let h = w.spawn_body(desc(
        RigidBodyType::Fixed,
        y,
        ShapeDesc::Capsule {
            half_height: HALF_H,
            radius: RADIUS,
        },
    ));
    // Um passo só para o grafo de interseção existir — o corpo é fixo, então
    // ele não sai de onde foi posto.
    w.step();
    w.fluid_at(h).buoyed
}

/// **A tabela inteira** — `buoyed` em função da altura, do seco ao fundo.
#[test]
#[ignore = "sonda: roda a pedido"]
fn measure_what_buoyed_reads_at_every_depth() {
    println!("\n== buoyed vs ALTURA (superficie em y = 0) ==");
    println!(
        "capsula: meia-altura {HALF_TALL:.2} m (half_h {HALF_H} + raio {RADIUS}), densidade 1.0"
    );
    println!("poca:    densidade {FLUID:.1}\n");
    println!(
        "{:>8}  {:>8}  {:>9}  {:>8}",
        "y", "topo", "submerso", "buoyed"
    );

    let mut first_wet = None;
    let mut equilibrium = None;
    let mut prev = (f32::NAN, 0.0f32);
    let mut y = 0.6f32;
    while y >= -0.85 {
        let b = buoyed_at(y);
        let top = y + HALF_TALL;
        // A fração da ALTURA que está debaixo da linha d'água — a régua que o
        // artista vê, e que a `buoyed` não é (ela mistura densidade e imersão).
        let sunk = ((-(y - HALF_TALL)) / (2.0 * HALF_TALL)).clamp(0.0, 1.0);
        println!("{y:>8.3}  {top:>8.3}  {:>8.1}%  {b:>8.4}", sunk * 100.0);
        if first_wet.is_none() && b > 0.0 {
            first_wet = Some(y);
        }
        if equilibrium.is_none() && b >= 1.0 {
            // Interpola linearmente entre esta amostra e a anterior.
            let (py, pb) = prev;
            let hit = if py.is_nan() || (b - pb).abs() < 1e-6 {
                y
            } else {
                py + (1.0 - pb) * (y - py) / (b - pb)
            };
            equilibrium = Some(hit);
        }
        prev = (y, b);
        y -= 0.05;
    }

    let deep = buoyed_at(-2.0);
    println!("\ntotalmente submerso: buoyed = {deep:.4}  (= densidade_fluido / densidade_corpo)");
    if let Some(w) = first_wet {
        println!("primeiro contato:    y = {w:.3}");
    }
    if let Some(e) = equilibrium {
        println!("EQUILIBRIO (b = 1):  y = {e:.3}  <- a linha em que ele BOIA parado");
    }

    println!("\nLEITURA — o que cada limiar candidato SIGNIFICA:");
    for t in [0.25f32, 0.5, 0.75, 1.0, 1.5] {
        // Onde a razão cruza `t`, por bisseção sobre a altura. ⚠️ **Afundar
        // AUMENTA a razão**, então o invariante é `cond(lo)` verdadeiro com `lo`
        // mais fundo que `hi` — o oposto da ordem habitual, e escrevê-lo ao
        // contrário devolve a borda errada em silêncio.
        let (mut lo, mut hi) = (-2.0f32, 0.6f32);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if buoyed_at(mid) >= t {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let y = 0.5 * (lo + hi);
        let sunk = ((-(y - HALF_TALL)) / (2.0 * HALF_TALL)).clamp(0.0, 1.0);
        println!(
            "  buoyed >= {t:<4} <=>  y <= {y:>6.3}  ({:>5.1}% submerso)",
            sunk * 100.0
        );
    }
}

/// **O caso que decide se o limiar precisa de um guarda de CHÃO** — vadear.
///
/// Um personagem de pé no fundo de uma poça rasa está molhado e apoiado ao
/// mesmo tempo. Se a razão que ele lê ali já passar do limiar, a lei o poria a
/// nadar em água pela cintura — e o gesto certo é andar.
#[test]
#[ignore = "sonda: roda a pedido"]
fn measure_what_a_wader_reads() {
    println!("\n== VADEAR: buoyed de quem esta' de pe' no fundo ==");
    println!("(a poca vai de y = 0 para baixo; o chao sobe)\n");
    println!(
        "{:>10}  {:>8}  {:>9}  {:>8}",
        "chao y", "centro y", "submerso", "buoyed"
    );

    // O personagem paira `float_height` acima do chão; a fixture do player usa
    // 0,9 — mas o que importa aqui é a altura do CENTRO, então parametrizo por
    // ela e nomeio o chão que a produziria.
    for floor in [-0.2f32, -0.4, -0.6, -0.8, -1.0, -1.4] {
        let y = floor + 0.9;
        let b = buoyed_at(y);
        let sunk = ((-(y - HALF_TALL)) / (2.0 * HALF_TALL)).clamp(0.0, 1.0);
        println!("{floor:>10.2}  {y:>8.3}  {:>8.1}%  {b:>8.4}", sunk * 100.0);
    }
}
