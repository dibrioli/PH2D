//! **O TAMBOR E A CADERNAL JUNTOS, medidos** (W-Pulley W5) — a costura entre o
//! W3 (a roldana montada num corpo) e o W4 (a roldana de dois raios), que
//! **nenhuma fixture exercitava**.
//!
//! O `wheel_jacobian` de um eixo montado passou a pesar os dois ramos pelo peso
//! da engrenagem (W4), e até aqui todo eixo montado vivia numa rota de peso `1`:
//! a linha nova nunca foi executada com um número diferente de `1.0` nela. Uma
//! multiplicação por um que ninguém viu falhar é uma multiplicação não medida.
//!
//! Roda pelo caminho do PRODUTO (`PhysicsWorld::step`), nunca por uma segunda
//! cópia do laço: uma sonda que re-implementa o que mede fica cega à porta.
//!
//! `cargo test -p ph2d-physics --test measure_pulley_composition -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// O raio por onde a corda ENTRA no tambor.
///
/// ⚠️ **Potência de dois, e os raios de saída também**, para que a engrenagem
/// `R/r` seja EXATA em `f32`: `0,4 / 0,1` dá 4,0000005 e uma tabela de previsão
/// com folga de 20% não notaria, mas um gate que compara razões notaria — e a
/// sonda e o gate têm de falar dos mesmos números.
const R_IN: f32 = 0.5;

/// A engrenagem que um raio de saída produz. `None` é a roldana comum.
fn gear_of(r_out: Option<f32>) -> f32 {
    r_out.map_or(1.0, |r| R_IN / r)
}

/// **O sarilho com talha:** um tambor de dois raios no teto e a cadernal MÓVEL
/// pendurada nele, carregando a carga.
///
/// ```text
///   morta (-0.4, 8)              tambor (0.4, 8)   R = 0.5 · r = r_out
///          \                        / \
///           \                      /   \
///            \                    /     contrapeso (0.4, 6)
///             \                  /
///              cadernal MÓVEL (0, 4)  [montada no BLOCO]
/// ```
///
/// A corda anda **do contrapeso** (ponta A) para a **ponta morta** (B): entra no
/// tambor pelo raio `R`, sai pelo raio `r`, desce até a cadernal, abraça e volta
/// ao teto. Os dois ramos que seguram o bloco estão **depois** do tambor, então
/// os dois carregam o peso `R/r` — e é essa multiplicação que esta sonda mede.
///
/// `r_out = None` é o **CONTROLE**: o mesmo rig com um tambor comum, onde a
/// vantagem é a da talha sozinha (dois ramos ⇒ 2).
fn windlass_tackle(
    load: f32,
    counter: f32,
    r_out: Option<f32>,
) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle, RigidBodyHandle) {
    const BODY_R: f32 = 0.2;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    // O teto: a ponta MORTA. Estático, logo massa infinita para a corda — que é
    // o que uma amarração no teto é.
    let (dead, _) = w.add_static_cuboid(-0.4, 8.0, 0.1, 0.1);
    let (block, _) = w.add_dynamic_circle(0.0, 4.0, BODY_R, load / area);
    let (haul, _) = w.add_dynamic_circle(0.4, 6.0, BODY_R, counter / area);
    let mut wheels = vec![
        RopeWheel {
            centre: [0.4, 8.0],
            radius: R_IN,
            radius_out: r_out,
            id: 1,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [0.0, 4.0],
            // A montagem: o eixo no CENTRO do bloco, então a corda não lhe faz
            // torque.
            body: Some(block),
            local: [0.0, 0.0],
            radius: 0.15,
            id: 2,
            ..RopeWheel::default()
        },
    ];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([0.4, 6.0], [-0.4, 8.0], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        id: 1,
        body_a: haul,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 2,
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

/// Quanto o bloco anda em 1 s, com um contrapeso de 1 kg.
fn travel(load: f32, r_out: Option<f32>) -> f32 {
    let (mut w, _, block, _) = windlass_tackle(load, 1.0, r_out);
    let y0 = y(&w, block);
    for _ in 0..60 {
        w.step();
    }
    y(&w, block) - y0
}

/// **A composição MULTIPLICA?** — a previsão é `2 · (R/r)`, e a tabela a testa
/// nos dois lados de cada linha.
///
/// ⚠️ **A tabela é de PREVISÃO, não de bisseção.** O sistema **não é monotônico
/// na carga**: muito acima do equilíbrio o contrapeso leve é arremessado até o
/// tambor, a rota degenera e *"desce"* volta a virar *"sobe"* — foi o que
/// derrubou a primeira sonda do W4, que reportava o teto do intervalo em toda
/// linha. Aqui cada engrenagem é medida a **−20% e +20%** do equilíbrio previsto:
/// abaixo o contrapeso vence e a carga SOBE, acima ela DESCE.
#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_composed_advantage() {
    println!("\n=== O TAMBOR E A CADERNAL MULTIPLICAM? (contrapeso de 1 kg) ===");
    println!("A cadernal movel da 2. O tambor da R/r. Previsto: o produto.");
    println!(
        "{:>10} | {:>8} | {:>10} | {:>22} | {:>22}",
        "r_sai", "engren.", "previsto", "carga -20% (sobe?)", "carga +20% (desce?)"
    );
    for r_out in [None, Some(0.25_f32), Some(0.125), Some(0.0625)] {
        let gear = gear_of(r_out);
        let predicted = 2.0 * gear;
        let (lo, hi) = (predicted * 0.8, predicted * 1.2);
        let (dlo, dhi) = (travel(lo, r_out), travel(hi, r_out));
        let verdict = |d: f32| {
            if d.abs() < 0.02 {
                "parado"
            } else if d > 0.0 {
                "SOBE"
            } else {
                "desce"
            }
        };
        println!(
            "{:>10} | {gear:>8.2} | {predicted:>10.2} | {lo:>10.2} kg {:>9} | {hi:>10.2} kg {:>8}",
            r_out.map_or("--".to_string(), |r| format!("{r:.3}")),
            format!("{:.3} {}", dlo, verdict(dlo)),
            format!("{:.3} {}", dhi, verdict(dhi))
        );
    }
    println!("\nSe a engrenagem NAO alcancasse o eixo montado, toda linha");
    println!("equilibraria em 2 kg -- e a coluna '-20%' desceria a partir da 2a.");
}

/// **A carga no EIXO montado** — a resultante que sustenta o bloco.
///
/// ⚠️ **A primeira versão desta sonda previu a coluna errada, e o produto estava
/// certo:** eu esperava `eixo / tensão = 2·(R/r)` e a medição deu **2,00 em toda
/// engrenagem**. O motivo está escrito no `apply` — `pulley_tension` publica o
/// **PICO** (`base · weight_max`, W4), enquanto o eixo recebe a tensão **BASE**
/// porque o Jacobiano dele já carrega os pesos dos dois ramos. A razão entre os
/// dois é o fator de enlace **sozinho**, e ele é 2 em qualquer engrenagem.
///
/// O que multiplica está na coluna ABSOLUTA: em equilíbrio o eixo carrega o
/// **peso do bloco que ele segura**, e a tensão do lado do contrapeso é esse peso
/// dividido por `2·(R/r)`. É a vantagem mecânica lida por uma terceira porta.
#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_geared_axle_load() {
    println!("\n=== A CARGA NO EIXO MONTADO, depois da engrenagem ===");
    println!("Em equilibrio o eixo carrega o PESO do bloco; a tensao e esse peso");
    println!("dividido pela vantagem. A razao eixo/tensao e o ENLACE (2), sempre.");
    println!(
        "{:>10} | {:>8} | {:>10} | {:>12} | {:>12} | {:>10} | {:>10}",
        "r_sai", "engren.", "carga kg", "pico (N)", "eixo (N)", "eixo/pico", "peso (N)"
    );
    for r_out in [None, Some(0.25_f32), Some(0.125)] {
        let gear = gear_of(r_out);
        let load = 2.0 * gear;
        let (mut w, d, _, _) = windlass_tackle(load, 1.0, r_out);
        for _ in 0..30 {
            w.step();
        }
        let t = w.pulley_tension(d.id);
        let axle = w.pulley_axle_load(2);
        println!(
            "{:>10} | {gear:>8.2} | {load:>10.2} | {t:>12.4} | {axle:>12.4} | {:>10.4} | {:>10.4}",
            r_out.map_or("--".to_string(), |r| format!("{r:.3}")),
            if t > 0.0 { axle / t } else { f32::NAN },
            load * 9.81
        );
    }
}

/// **A talha de WESTON sai daqui?** — a pergunta que o plano do W4 deixou
/// escrita, respondida com números em vez de com uma fórmula lembrada.
///
/// A Weston vale `2R/(R−r)`; esta composição vale `2R/r`. As duas **coincidem
/// em `R = 2r`** e divergem em todo o resto — e é por isso que a nota do plano
/// parecia certa: o exemplo natural (`0,5 → 0,25`) é exatamente o ponto onde as
/// duas fórmulas se cruzam.
#[test]
#[ignore = "measurement, not a gate"]
fn the_weston_formula_is_not_what_this_composition_gives() {
    println!("\n=== A COMPOSICAO x A TALHA DE WESTON ===");
    println!(
        "{:>8} | {:>8} | {:>14} | {:>14} | {:>10}",
        "R", "r", "composicao 2R/r", "Weston 2R/(R-r)", "iguais?"
    );
    for (r_in, r_out) in [(0.5_f32, 0.25_f32), (0.4, 0.2), (0.5, 0.125), (0.5, 0.0625)] {
        let ours = 2.0 * r_in / r_out;
        let weston = 2.0 * r_in / (r_in - r_out);
        println!(
            "{r_in:>8.3} | {r_out:>8.3} | {ours:>14.3} | {weston:>14.3} | {:>10}",
            if (ours - weston).abs() < 1e-4 {
                "SIM"
            } else {
                "nao"
            }
        );
    }
    println!("\nAs duas coincidem SO em R = 2r. A Weston precisa que o MESMO eixo");
    println!("seja atravessado DUAS vezes com a cadernal no meio -- outra topologia.");
}
