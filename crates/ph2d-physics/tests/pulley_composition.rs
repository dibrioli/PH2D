//! **O TAMBOR E A CADERNAL, JUNTOS** (W-Pulley W5) — a costura entre o W3 (a
//! roldana montada num corpo) e o W4 (a roldana de dois raios).
//!
//! As duas waves shiparam com gates fortes e **nenhuma fixture as pôs na mesma
//! corda**. O `wheel_jacobian` de um eixo montado passou a pesar os dois ramos
//! pelo peso da engrenagem, mas todo eixo montado do repo vivia numa rota de peso
//! `1`: aquela multiplicação nunca rodou com outro número. Uma multiplicação por
//! um que ninguém viu falhar é uma multiplicação não medida — e é este arquivo
//! que a mede.
//!
//! ⚠️ **E ela desmente uma nota do plano.** O §9 do
//! [`docs/Physics/03_plano_polia.md`] afirmava que *"a talha de WESTON
//! (`2R/(R−r)`) sai por COMPOSIÇÃO"*. Sai uma composição, e ela é **`2R/r`** — as
//! duas fórmulas **coincidem apenas em `R = 2r`**, que por acaso é o exemplo
//! natural (`0,5 → 0,25`) com que a nota foi escrita. A Weston de verdade precisa
//! que o MESMO eixo seja atravessado DUAS vezes *com a cadernal no meio*, e no
//! nosso modelo os dois contatos de um tambor são adjacentes por construção
//! (§W5 do plano). Aqui a divergência é **executável**: o gate mede a `2R/r`
//! numa fixture onde `R = 4r`, exatamente onde a `2R/(R−r)` diria outra coisa.
//!
//! Números medidos em `tests/measure_pulley_composition.rs`.

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// O raio por onde a corda ENTRA no tambor. Potência de dois, e os de saída
/// também, para a engrenagem `R/r` ser exata em `f32`.
const R_IN: f32 = 0.5;

/// **O sarilho com talha:** o tambor de dois raios no teto, e a cadernal MÓVEL
/// pendurada nele carregando o bloco.
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
/// ao teto. Os dois ramos que seguram o bloco estão **depois** do tambor, logo os
/// dois valem `R/r` no orçamento da corda — e é essa multiplicação que decide
/// tudo o que este arquivo afirma.
///
/// ⚠️ **As duas âncoras da cadernal são simétricas em torno dela** (−0,4 e +0,4):
/// as componentes horizontais das duas tensões se cancelam e o bloco não deriva
/// de lado, senão a medição de equilíbrio mediria a deriva junto (a lição da
/// fixture do W3).
///
/// `r_out = None` é o **CONTROLE**: o mesmo rig com um tambor comum, onde sobra
/// a vantagem da talha sozinha (dois ramos ⇒ 2). Ele é o que impede um erro de
/// escala global de passar — uma mutação que multiplicasse tudo por uma constante
/// moveria as duas linhas juntas.
fn windlass_tackle(
    load: f32,
    counter: f32,
    r_out: Option<f32>,
) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle) {
    const BODY_R: f32 = 0.2;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
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
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, desc, block)
}

/// Quanto o bloco anda em 1 s, com um contrapeso de 1 kg.
fn travel(load: f32, r_out: Option<f32>) -> f32 {
    let (mut w, _, block) = windlass_tackle(load, 1.0, r_out);
    let y0 = w.bodies().get(block).expect("bloco").translation().y;
    for _ in 0..60 {
        w.step();
    }
    w.bodies().get(block).expect("bloco").translation().y - y0
}

/// **A COMPOSIÇÃO MULTIPLICA:** o tambor dá `R/r`, a cadernal dá 2, e a vantagem
/// é o PRODUTO — `1 kg` segura `8 kg` com `R = 4r`.
///
/// O oráculo é um **bracket**, não um ponto: abaixo do equilíbrio previsto o
/// contrapeso vence e a carga SOBE, acima ela DESCE. Bracket porque o sistema
/// **não é monotônico na carga** muito acima do equilíbrio (o contrapeso leve é
/// arremessado até o tambor e a rota degenera) — foi isso que derrubou a primeira
/// sonda do W4.
///
/// ⚠️ **O controle (`None`) é metade do gate.** Sozinha, a linha engrenada passa
/// sob qualquer erro que escale o sistema inteiro; é o par *2 no comum, 8 no
/// engrenado* que só a engrenagem chegando ao eixo montado explica.
///
/// ⚠️ **E a carga de 4 kg é a que desmente a nota do plano:** ela está ACIMA do
/// que a fórmula de Weston (`2R/(R−r)` = 2,67) sustentaria e ABAIXO do que esta
/// composição sustenta, então ela SOBE — e sobe por um mecanismo que a Weston não
/// descreve.
#[test]
fn the_drum_and_the_tackle_multiply() {
    // Sem tambor: sobra a talha, e a talha vale 2.
    assert!(
        travel(1.6, None) > 0.02,
        "o controle e uma talha comum: 1,6 kg esta abaixo dos 2 kg que um \
         contrapeso de 1 kg segura, logo o bloco tinha de SUBIR; andou {:.4} m",
        travel(1.6, None)
    );
    assert!(
        travel(2.4, None) < -0.02,
        "2,4 kg passa dos 2 kg da talha comum e tinha de DESCER; andou {:.4} m",
        travel(2.4, None)
    );
    // Com o tambor 0,5 → 0,125 a engrenagem é 4, e a vantagem tem de ser 8.
    const R_OUT: Option<f32> = Some(0.125);
    let gear = R_IN / 0.125;
    assert_eq!(gear, 4.0, "a fixture quer engrenagem EXATA");
    let up = travel(6.4, R_OUT);
    assert!(
        up > 0.02,
        "com a engrenagem 4 a vantagem e 2x4 = 8, entao 6,4 kg (80% dos 8) tinha \
         de SUBIR — e uma talha SOZINHA deixaria cair qualquer coisa acima de 2 \
         kg. Andou {up:.4} m: a engrenagem nao esta alcancando o eixo montado"
    );
    let down = travel(9.6, R_OUT);
    assert!(
        down < -0.02,
        "9,6 kg passa dos 8 e tinha de DESCER; andou {down:.4} m — uma vantagem \
         MAIOR que a prevista tambem e um defeito"
    );
    // A nota do plano, executável: a Weston pararia em 2,67 kg.
    let weston = 2.0 * R_IN / (R_IN - 0.125);
    let above_weston = travel(4.0, R_OUT);
    assert!(
        above_weston > 0.02,
        "4 kg esta acima do que a formula de Weston ({weston:.2}) sustentaria e \
         abaixo dos 8 desta composicao, logo o bloco tinha de SUBIR; andou \
         {above_weston:.4} m"
    );
}

/// **O eixo montado carrega o PESO que ele segura — e a engrenagem é contada UMA
/// vez.**
///
/// O `apply` entrega ao ledger dos eixos a tensão **BASE**, não o pico, *porque o
/// Jacobiano de cada roldana já carrega os pesos dos dois lados dela*. É uma
/// decisão de uma linha cuja violação é invisível em toda cena sem tambor (lá o
/// peso é `1`) e cuja consequência aqui é a carga do eixo lida com a engrenagem
/// ao QUADRADO.
///
/// Os dois oráculos são independentes e nenhum repete a fórmula:
///
/// - **em equilíbrio o eixo carrega o peso do bloco** (`m·g`), fato de estática
///   que não conhece rota nem Jacobiano;
/// - **a razão `eixo / pico` é o fator de ENLACE sozinho** (≈2 num enlace de
///   180°) em QUALQUER engrenagem, porque o readout de tensão já publica o lado
///   pesado.
#[test]
fn the_geared_axle_carries_the_weight_it_holds_and_counts_the_gear_once() {
    const LOAD: f32 = 8.0;
    let (mut w, desc, _) = windlass_tackle(LOAD, 1.0, Some(0.125));
    for _ in 0..30 {
        w.step();
    }
    let peak = w.pulley_tension(desc.id);
    let axle = w.pulley_axle_load(2);
    let weight = LOAD * 9.81;
    assert!(
        (axle - weight).abs() / weight < 0.05,
        "em equilibrio o eixo montado sustenta o bloco: esperado ~{weight:.1} N, \
         medido {axle:.1} N (a engrenagem contada duas vezes daria ~{:.0})",
        weight * 4.0
    );
    let wrap = axle / peak;
    assert!(
        (wrap - 2.0).abs() < 0.05,
        "a razao eixo/tensao e o ENLACE (2), nao 2x a engrenagem: o readout de \
         tensao ja publica o lado pesado. Medido {wrap:.4}"
    );
}
