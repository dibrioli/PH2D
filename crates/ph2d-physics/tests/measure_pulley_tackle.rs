//! **A TALHA, medida** (W-Pulley W3) — a vantagem mecânica que o `ratio` prometia
//! e não entregava.
//!
//! Roda pelo caminho do PRODUTO (`PhysicsWorld::step`), nunca por uma segunda
//! cópia do laço: uma sonda que re-implementa o que mede fica cega à porta.
//!
//! Arquivo próprio e não uma seção do `measure_pulley.rs` porque o RIG é outro —
//! lá é a máquina de Atwood (duas roldanas no teto, dois corpos pendurados), aqui
//! é a cadernal MÓVEL, e o irmão já está no teto de LOC.
//!
//! `cargo test -p ph2d-physics --test measure_pulley_tackle -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::RopeWheel;

/// **A talha (*gun tackle*):** a ponta morta amarrada ao teto, a roldana MÓVEL
/// montada no bloco que carrega a carga, a roldana FIXA no teto, e a outra ponta
/// na mão — aqui um contrapeso pendurado.
///
/// ```text
///   morta (-0.4, 8)          roldana FIXA (0.4, 8)  [cenário]
///          \                        / \
///           \                      /   \
///            \                    /     contrapeso (0.4, 6)
///             \                  /
///              roldana MÓVEL (0, 4)  [montada no BLOCO]
/// ```
///
/// ⚠️ **A primeira versão desta fixture não tinha a roldana FIXA, e a medição a
/// derrubou:** com a ponta morta e a mão as DUAS acima do bloco, descer o bloco
/// alonga os dois ramos, então a mão desce `2d` junto — os dois lados liberam
/// energia e **não existe equilíbrio**. O sistema apenas caía, e a tabela dizia
/// *"desce"* em toda linha, para toda massa. A vantagem mecânica precisa da
/// cadernal fixa para INVERTER o sentido da mão: é isso que uma talha é.
///
/// ⚠️ **As duas âncoras da roldana móvel são simétricas em torno dela de
/// propósito** (−0,4 e +0,4): as duas tensões puxam o eixo para cima-esquerda e
/// para cima-direita, e as componentes horizontais se cancelam. Sem isso o bloco
/// derivaria de lado e a medição de equilíbrio mediria a deriva junto.
///
/// `mounted = false` é o **CONTROLE 1:1**: a roldana móvel some da rota e o bloco
/// é amarrado DIRETO na ponta morta da corda, então a corda o segura por UM ramo
/// só. É a mesma corda, o mesmo contrapeso, a mesma roldana fixa — a única
/// diferença é quantos ramos seguram o bloco.
fn tackle(
    block_mass: f32,
    haul_mass: f32,
    mounted: bool,
) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    const R: f32 = 0.2;
    let area = std::f32::consts::PI * R * R;
    // O teto: a ponta MORTA da corda. Estático, então ele é massa infinita para a
    // corda e nunca entra na conta — que é o que uma amarração no teto é.
    let (dead, _) = w.add_static_cuboid(-0.4, 8.0, 0.1, 0.1);
    let (block, _) = w.add_dynamic_circle(0.0, 4.0, R, block_mass / area);
    let (haul, _) = w.add_dynamic_circle(0.4, 6.0, R, haul_mass / area);
    let movable = RopeWheel {
        centre: [0.0, 4.0],
        // A montagem: o eixo no CENTRO do bloco, então a corda não lhe faz torque.
        body: Some(block),
        local: [0.0, 0.0],
        id: 1,
        ..RopeWheel::default()
    };
    let fixed = RopeWheel {
        centre: [0.4, 8.0],
        id: 2,
        ..RopeWheel::default()
    };
    let wheels = if mounted {
        vec![movable, fixed]
    } else {
        vec![fixed]
    };
    let desc = PulleyDesc {
        id: 1,
        // No controle 1:1 a ponta morta É o bloco: a corda o segura por um ramo.
        body_a: if mounted { dead } else { block },
        body_b: haul,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: wheels.len() as u32,
        total_length: 0.0,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    // O comprimento é o da rota que a montagem tem AGORA: a corda nasce esticada
    // e o primeiro passo não dá um puxão. A mesma porta que a ponte usa.
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, desc, block, haul)
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).map_or(f32::NAN, |b| b.translation().y)
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_mechanical_advantage() {
    println!("\n=== O CONTRAPESO QUE SEGURA UM BLOCO DE 2 kg (1 s) ===");
    println!("TALHA: o bloco pende da roda MONTADA, entao dois ramos o seguram.");
    println!("1:1  : o bloco e amarrado na ponta da corda -- um ramo so.");
    println!(
        "{:>10} | {:>12} {:>10} | {:>12} {:>10}",
        "haul (kg)", "talha dy", "", "1:1 dy", ""
    );
    for haul in [0.25_f32, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5] {
        let mut row = [0.0_f32; 2];
        for (slot, mounted) in [(0, true), (1, false)] {
            let (mut w, _, block, _) = tackle(2.0, haul, mounted);
            let y0 = y(&w, block);
            for _ in 0..60 {
                w.step();
            }
            row[slot] = y(&w, block) - y0;
        }
        let verdict = |d: f32| {
            if d.abs() < 0.05 {
                "EQUILIBRIO"
            } else if d > 0.0 {
                "sobe"
            } else {
                "desce"
            }
        };
        println!(
            "{haul:>10.2} | {:>12.4} {:>10} | {:>12.4} {:>10}",
            row[0],
            verdict(row[0]),
            row[1],
            verdict(row[1])
        );
    }
    println!("\nPrevisto: a talha equilibra em ~1 kg (METADE), o 1:1 em ~2 kg.");
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_axle_load() {
    println!("\n=== A CARGA NO EIXO contra a TENSAO da corda ===");
    println!("Num enlace de 180 graus a resultante e 2T -- e e a MESMA conta do");
    println!("Jacobiano, que e o que da a vantagem mecanica. Uma conta, tres usos.");
    println!(
        "{:>10} | {:>12} | {:>12} | {:>8} | {:>12} | {:>8}",
        "haul (kg)", "tensao (N)", "movel (N)", "razao", "fixa (N)", "razao"
    );
    for haul in [0.5_f32, 1.0, 1.5] {
        let (mut w, d, _, _) = tackle(2.0, haul, true);
        for _ in 0..30 {
            w.step();
        }
        let t = w.pulley_tension(d.id);
        // Os ids são os do rig: 1 = a roldana MÓVEL, 2 = a FIXA.
        let (movable, fixed) = (w.pulley_axle_load(1), w.pulley_axle_load(2));
        let r = |x: f32| if t > 0.0 { x / t } else { f32::NAN };
        println!(
            "{haul:>10.2} | {t:>12.4} | {movable:>12.4} | {:>8.4} | {fixed:>12.4} | {:>8.4}",
            r(movable),
            r(fixed)
        );
    }
    println!("\nA MOVEL carrega ~2T (enlace de 180 graus) -- e e essa resultante");
    println!("que sobe o bloco. A FIXA carrega o que o desvio dela pedir.");
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_mount_geometry() {
    println!("\n=== A RODA SEGUE O BLOCO? (a metade geometrica) ===");
    println!("A arena e a lista que o DESENHO le: se o centro nao a acompanha, o");
    println!("overlay desenha a corda passando por onde a roldana NAO esta.");
    println!(
        "{:>6} | {:>12} | {:>12} | {:>10}",
        "tique", "bloco y", "roda y", "delta"
    );
    // Contrapeso leve: o bloco DESCE, e a roda tem de descer com ele.
    let (mut w, _, block, _) = tackle(2.0, 0.25, true);
    for tick in 0..=60 {
        if tick % 15 == 0 {
            let by = y(&w, block);
            let wy = w.pulley_wheels()[0].centre[1];
            println!("{tick:>6} | {by:>12.4} | {wy:>12.4} | {:>10.6}", wy - by);
        }
        w.step();
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_cost() {
    println!("\n=== O CUSTO de uma roldana MONTADA, por sub-passo ===");
    println!("O eixo montado acrescenta uma ponta a restricao: um `end`, um `k`,");
    println!("uma projecao e um impulso. A pergunta e se isso se ve no relogio.");
    println!(
        "{:>10} | {:>14} | {:>14}",
        "roldanas", "cenario (us)", "montada (us)"
    );
    for n in [1_usize, 2, 4, 8] {
        let mut row = [0.0_f64; 2];
        for (slot, mounted) in [(0, false), (1, true)] {
            let mut w = PhysicsWorld::new();
            const R: f32 = 0.2;
            let area = std::f32::consts::PI * R * R;
            let (dead, _) = w.add_static_cuboid(-0.5, 20.0, 0.1, 0.1);
            let (block, _) = w.add_dynamic_circle(0.0, 4.0, R, 2.0 / area);
            let (haul, _) = w.add_dynamic_circle(0.5, 20.0, R, 1.0 / area);
            let wheels: Vec<RopeWheel> = (0..n)
                .map(|i| RopeWheel {
                    centre: [0.0, 4.0 + i as f32 * 0.001],
                    body: if mounted { Some(block) } else { None },
                    local: [0.0, i as f32 * 0.001],
                    id: i as u64,
                    ..RopeWheel::default()
                })
                .collect();
            let mut desc = PulleyDesc {
                id: 1,
                body_a: dead,
                body_b: haul,
                local_a: [0.0, 0.0],
                local_b: [0.0, 0.0],
                wheel_start: 0,
                wheel_count: n as u32,
                total_length: 0.0,
                motor_rate: 0.0,
                break_force: f32::INFINITY,
            };
            w.set_pulleys(vec![desc], wheels.clone());
            desc.total_length = w.pulley_span(&desc).unwrap_or(30.0);
            w.set_pulleys(vec![desc], wheels);
            for _ in 0..20 {
                w.step();
            }
            let t = std::time::Instant::now();
            for _ in 0..200 {
                w.step();
            }
            row[slot] = t.elapsed().as_secs_f64() * 1.0e6 / 200.0;
        }
        println!("{n:>10} | {:>14.2} | {:>14.2}", row[0], row[1]);
    }
}
