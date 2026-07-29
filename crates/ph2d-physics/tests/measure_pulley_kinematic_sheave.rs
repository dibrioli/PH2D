//! **Uma cadernal MÓVEL montada num corpo KINEMATIC** (W-Pulley W3) — o item que
//! o plano nomeia como *"vira um guincho de graça … não medido, não gateado"*.
//!
//! A afirmação está escrita no kernel, no `end`: *"a velocidade do ponto (`v`) NÃO
//! é zerada junto, e a diferença é uma feature: um corpo kinematic é massa
//! infinita (a corda não o move) mas TEM movimento próprio, então uma bobina
//! animada por curva vira um guincho de graça"*.
//!
//! ⚠️ **Metade disso já é gateada e a outra metade NÃO era.** A ponta da corda
//! amarrada num corpo kinematic tem dois gates em `pulley.rs`
//! (`a_kinematic_drum_hauls_the_load` e o irmão que mede o atraso a duas
//! velocidades, 1:1 a menos de 2 mm). O que ninguém mediu é a **ROLDANA
//! MONTADA** num corpo kinematic — e ela não é o mesmo caso: uma cadernal móvel
//! com enlace de 180° tem **vantagem mecânica 2**, então mover o eixo por `d`
//! muda a rota por `2d` e a ponta livre viaja **o DOBRO**.
//!
//! Se a razão medida for ~2, a nota é verdadeira e a capacidade existe sem
//! ninguém a ter escrito. Se for ~1, a nota confunde o caso da PONTA com o da
//! ROLDANA e tem de ser corrigida.
//!
//! `cargo test -p ph2d-physics --test measure_pulley_kinematic_sheave -- --ignored --nocapture --test-threads=1`

use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};
use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, ShapeDesc};
use rapier2d::dynamics::RigidBodyType;

const R: f32 = 0.2;
/// A altura da lança: a ponta morta e a cadernal FIXA vivem nela.
const BOOM_Y: f32 = 6.0;
/// A meia-distância entre a ponta morta e a cadernal fixa.
const SPAN: f32 = 0.9;
/// Onde o bloco kinematic (que carrega a cadernal móvel) nasce.
const BLOCK_Y: f32 = 3.2;
/// Onde a carga nasce, pendurada sob a cadernal fixa.
const LOAD_Y: f32 = 3.5;

fn body(kind: RigidBodyType, x: f32, y: f32, mass: f32) -> BodyDesc {
    BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density: mass / (std::f32::consts::PI * R * R),
        shape: ShapeDesc::Ball { radius: R },
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

/// **A talha dirigida por curva.**
///
/// ```text
///   morta (-0.9, BOOM)  [estatica]      cadernal FIXA (+0.9, BOOM)  [cenario]
///          \                                   / \
///           \                                 /   carga (+0.9)  [dinamica]
///            \                               /
///             cadernal MOVEL (0, BLOCK_Y)  [montada no bloco KINEMATIC]
/// ```
///
/// `mounted = false` é o CONTROLE: a cadernal móvel sai da rota e o bloco
/// kinematic passa a ser a PONTA da corda — o caso que já é gateado, e cuja razão
/// conhecida é **1:1**. É essa troca, e só ela, que separa as duas perguntas.
fn rig(mounted: bool) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    let dead = w.spawn_body(body(RigidBodyType::Fixed, -SPAN, BOOM_Y, 1.0));
    let mut block_desc = body(RigidBodyType::KinematicPositionBased, 0.0, BLOCK_Y, 1.0);
    block_desc.shape = ShapeDesc::Ball { radius: R };
    let block = w.spawn_body(block_desc);
    let load = w.spawn_body(body(RigidBodyType::Dynamic, SPAN, LOAD_Y, 1.0));

    let mut wheels: Vec<RopeWheel> = Vec::new();
    if mounted {
        // **A CADERNAL MÓVEL**, montada no bloco kinematic. O `local` é `[0,0]`:
        // o eixo no centro do bloco, que é o que uma cadernal pendurada num
        // gancho é.
        wheels.push(RopeWheel {
            centre: [0.0, BLOCK_Y],
            body: Some(block),
            local: [0.0, 0.0],
            radius: 0.0,
            side: 1,
            id: 1,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        });
    }
    // A cadernal FIXA, no cenário.
    wheels.push(RopeWheel {
        centre: [SPAN, BOOM_Y],
        radius: 0.0,
        side: 1,
        id: 2,
        break_force: f32::INFINITY,
        ..RopeWheel::default()
    });

    // ⚠️ **No controle a ponta morta É o bloco**: a corda o segura por UM ramo, e
    // é essa troca que muda a vantagem de 2 para 1 — exatamente como a fixture da
    // cena 61 faz.
    let a = if mounted { dead } else { block };
    let (wa, wb) = (
        if mounted {
            [-SPAN, BOOM_Y]
        } else {
            [0.0, BLOCK_Y]
        },
        [SPAN, LOAD_Y],
    );
    rope_route::resolve_sides(wa, wb, &mut wheels, &mut Vec::new());
    // O comprimento é o da ROTA em repouso: a corda nasce exatamente esticada e o
    // primeiro tique não dá um puxão.
    let total_length = rope_route::route(wa, wb, &wheels, &mut Vec::new())
        .expect("a rota de repouso resolve")
        .length;
    let desc = PulleyDesc {
        body_a: a,
        body_b: load,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: u32::try_from(wheels.len()).expect("conta cabe"),
        id: 1,
        total_length,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels);
    (w, desc, block, load)
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.body_pose(h).expect("corpo vivo").translation.y
}

/// Baixa o bloco `travel` metros em `ticks` tiques e devolve `(quanto o bloco
/// andou, quanto a carga andou)`.
fn haul(mounted: bool, travel: f32, ticks: u32) -> (f32, f32) {
    let (mut w, _, block, load) = rig(mounted);
    let (b0, l0) = (y(&w, block), y(&w, load));
    for t in 0..ticks {
        let frac = (t + 1) as f32 / ticks as f32;
        w.set_next_kinematic_pose(block, 0.0, BLOCK_Y - travel * frac, 0.0);
        w.step();
    }
    (y(&w, block) - b0, y(&w, load) - l0)
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_whether_a_kinematic_sheave_hauls_and_by_how_much() {
    println!(
        "\n=== A cadernal MOVEL num corpo KINEMATIC baixa `d`; quanto a carga sobe? ===\n\
         (o CONTROLE e a mesma corda com o bloco como PONTA -- razao conhecida 1:1)\n"
    );
    println!(
        "{:>8} | {:>10} | {:>10} | {:>7} || {:>10} | {:>7}",
        "d (m)", "bloco", "carga", "razao", "carga ctl", "razao"
    );
    for travel in [0.2_f32, 0.5, 1.0] {
        let (bm, lm) = haul(true, travel, 90);
        let (bc, lc) = haul(false, travel, 90);
        println!(
            "{travel:>8.2} | {bm:>10.4} | {lm:>10.4} | {:>7.3} || {lc:>10.4} | {:>7.3}",
            lm / -bm,
            lc / -bc
        );
    }
    println!(
        "\n  Se a coluna da esquerda der ~2 e a da direita ~1, a nota do plano e\n  \
         VERDADEIRA e a capacidade existe sem ninguem a ter escrito: uma cadernal\n  \
         movel dirigida por curva e um guincho com vantagem 2."
    );
}

/// **A razão depende da VELOCIDADE?** — a pergunta que a tabela acima não responde.
///
/// ⚠️ **A 1ª versão desta sonda mediu um "atraso" contra o percurso quase-estático
/// e saiu NÃO-MONOTÔNICA** (0,0108 / 0,0328 / 0,0008 m) — porque a razão
/// quase-estática *ela mesma* muda com o percurso: o ângulo da perna abre conforme
/// o bloco desce, então comparar percursos diferentes mistura geometria com
/// dinâmica. *Um oráculo confundido não é um número pequeno, é um número que não
/// significa nada.*
///
/// A pergunta honesta com esta fixture: **o MESMO percurso, em tempos diferentes**.
/// Se a razão for estável, a vantagem é geométrica e não paga pedágio de
/// velocidade.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_whether_the_advantage_survives_speed() {
    println!("\n=== O MESMO percurso (0,5 m), em tempos diferentes ===\n");
    println!(
        "{:>8} | {:>9} | {:>12} | {:>8}",
        "tiques", "m/s", "carga", "razao"
    );
    for ticks in [600_u32, 180, 90, 30, 15] {
        let (b, l) = haul(true, 0.5, ticks);
        println!(
            "{ticks:>8} | {:>9.2} | {l:>12.4} | {:>8.4}",
            0.5 / (ticks as f32 / 60.0),
            l / -b
        );
    }
    println!(
        "\n  Razao estavel = a vantagem e GEOMETRIA. O quase-estatico (600 tiques) e\n           o limite, e o que sobra nas linhas rapidas e o pedagio da dinamica."
    );
}

/// A outra metade: a corda **empurra** de volta? Um corpo kinematic é massa
/// infinita, então a carga não pode movê-lo — e é isso que faz dele um guincho em
/// vez de um par de corpos negociando.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_that_the_load_cannot_move_the_driven_sheave() {
    println!("\n=== A carga pode arrastar a cadernal dirigida? ===\n");
    // Carga PESADA, e o bloco mandado ficar PARADO: se ela o move, a mira
    // kinematic não é honrada e não há guincho nenhum.
    let (mut w, _, block, load) = rig(true);
    let b0 = y(&w, block);
    for _ in 0..120 {
        w.set_next_kinematic_pose(block, 0.0, BLOCK_Y, 0.0);
        w.step();
    }
    println!(
        "  bloco mandado ficar parado: andou {:.6} m · carga em {:.4}",
        y(&w, block) - b0,
        y(&w, load)
    );
    println!(
        "\n  Zero (a menos de um ulp) e o que o `end` promete: `inv_m = 0` para\n  \
         corpo nao-dinamico, logo o impulso da corda nao o acelera."
    );
}
