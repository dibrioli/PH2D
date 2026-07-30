//! **A TALHA DE WESTON** (W-Pulley, W-Weston) — um eixo composto atravessado DUAS
//! vezes, com o que a corda abraça no meio.
//!
//! A lei está em `rope_route::crossing_gear` e as medições que a produziram em
//! `tests/measure_weston.rs`. Aqui ficam as afirmações que TÊM de doer.
//!
//! ⚠️ **O CONTROLE de todo gate de vantagem é o mesmo rig sem o par de eixo**, onde
//! sobra o enlace da cadernal (2) — e não uma versão do rig com números diferentes.
//! Sem ele um gate que só afirma *"a carga sobe"* passa com a vantagem errada.

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// O diâmetro de ENTRADA. Potência de dois, e os retornos derivados por `1 − 1/2^k`,
/// para que `R/(R−r)` seja EXATO em `f32` — o gate e a sonda falam dos mesmos
/// números.
const R_IN: f32 = 0.5;

/// O eixo que os dois contatos compartilham. No produto é o `stable_name_id`.
const AXLE: u64 = 7;

/// A altura do eixo composto. Ver a nota de fixture no [`rig`]: ela existe para o
/// CONTROLE ter espaço, não por gosto.
const SHEAVE_Y: f32 = 30.0;

/// O raio de RETORNO que produz o peso `gear`.
fn return_radius(gear: f32) -> f32 {
    R_IN * (1.0 - 1.0 / gear)
}

/// **O rig da talha:** eixo composto no teto, cadernal MÓVEL na carga, contrapeso
/// numa ponta e a morta na outra.
///
/// `pair` falso é o **CONTROLE**: os mesmos três contatos, os mesmos raios, e
/// nenhum número de eixo — dois contatos que a rota lê como roldanas comuns.
///
/// ⚠️ **O eixo fica ALTO (30 m), e é uma correção de fixture, não estética.** No
/// CONTROLE a vantagem é 2, então uma carga dimensionada para a Weston (8 ou 32 kg)
/// arremessa o contrapeso de 1 kg para cima a metros por segundo; com o eixo a 2 m
/// dele o contrapeso **passava da própria roldana** em menos de um segundo, a rota
/// degenerava, e o controle media +3,05 m (subindo!) onde tinha de descer. Foi o
/// controle atropelado pelo experimento — a quinta vez nesta linha —, e a cura é dar
/// espaço, nunca encurtar a janela de um braço só (dois braços com janelas
/// diferentes é a assimetria que esconde o próximo defeito).
fn rig(
    load: f32,
    counter: f32,
    ret: f32,
    pair: bool,
) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle) {
    const BODY_R: f32 = 0.2;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (dead, _) = w.add_static_cuboid(-0.8, SHEAVE_Y, 0.1, 0.1);
    let (block, _) = w.add_dynamic_circle(0.0, 4.0, BODY_R, load / area);
    let (haul, _) = w.add_dynamic_circle(0.8, 6.0, BODY_R, counter / area);
    let axle = if pair { AXLE } else { 0 };
    let mut wheels = vec![
        RopeWheel {
            centre: [0.0, SHEAVE_Y],
            radius: R_IN,
            axle,
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
        RopeWheel {
            centre: [0.0, SHEAVE_Y],
            radius: ret,
            axle,
            id: 1,
            ..RopeWheel::default()
        },
    ];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([0.8, 6.0], [-0.8, SHEAVE_Y], &mut wheels, &mut scratch);
    let mut desc = PulleyDesc {
        id: 1,
        body_a: haul,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 3,
        total_length: 0.0,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, desc, block)
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).map_or(f32::NAN, |b| b.translation().y)
}

/// Quanto o bloco anda em 1 s, com um contrapeso de 1 kg.
fn travel(load: f32, ret: f32, pair: bool) -> f32 {
    let (mut w, _, block) = rig(load, 1.0, ret, pair);
    let y0 = y(&w, block);
    for _ in 0..60 {
        w.step();
    }
    y(&w, block) - y0
}

/// Abaixo disto a carga está PARADA — a folga do bracket de ±20%, medida (os
/// deslocamentos reais ficam entre 0,029 e 0,112 m em 1 s).
const STILL: f32 = 0.02;

/// **A vantagem é `2R/(R−r)`**, e ela vem das DUAS circunferências.
///
/// ⚠️ **Bracket de previsão, nunca bisseção.** O sistema não é monotônico na carga
/// (muito acima do equilíbrio o contrapeso leve é arremessado até o eixo e *desce*
/// volta a virar *sobe*), e uma busca binária sobre isso **já mentiu nesta linha**.
///
/// ⚠️ Mutação: `crossing_gear` devolvendo `wheels[i].gear()` no primeiro contato
/// (isto é, ignorando o par) faz o peso virar `1` e a vantagem colapsar em 2 — a
/// coluna `−20%` DESCE em toda linha, que é exatamente o que o CONTROLE mede.
#[test]
fn the_weston_advantage_is_twice_the_radius_over_the_difference() {
    for gear in [4.0_f32, 16.0] {
        let ret = return_radius(gear);
        let predicted = 2.0 * gear;
        let lo = travel(predicted * 0.8, ret, true);
        let hi = travel(predicted * 1.2, ret, true);
        assert!(
            lo > STILL,
            "peso {gear}: a {:.2} kg (20% abaixo do equilíbrio) a carga tinha de SUBIR, andou {lo:.4}",
            predicted * 0.8
        );
        assert!(
            hi < -STILL,
            "peso {gear}: a {:.2} kg (20% acima) a carga tinha de DESCER, andou {hi:.4}",
            predicted * 1.2
        );
        // O CONTROLE: sem par de eixo sobra a cadernal, e a MESMA carga desce.
        let plain = travel(predicted * 0.8, ret, false);
        assert!(
            plain < -STILL,
            "peso {gear}: sem par de eixo a vantagem é 2, então {:.2} kg tinha de DESCER, andou {plain:.4}",
            predicted * 0.8
        );
    }
}

/// **O ramo depois do retorno é a corda MORTA** — peso zero, e é isso que faz o
/// contato de retorno se comportar como um enrolamento terminal.
///
/// ⚠️ Mutação: devolver `1.0` em vez de `0.0` no segundo contato faz o trecho morto
/// entrar no orçamento e a ponta B passar a puxar — `weight_b` sai de 0.
#[test]
fn the_dead_strand_after_the_return_carries_nothing() {
    let (w, d, _) = rig(8.0, 1.0, return_radius(4.0), true);
    let mut legs = Vec::new();
    let route = rope_route::route(
        [0.8, 6.0],
        [-0.8, SHEAVE_Y],
        d.wheels(w.pulley_wheels()),
        &mut legs,
    )
    .expect("rota sã");
    assert_eq!(legs.len(), 4, "três contatos dão quatro trechos");
    assert_eq!(legs[0].weight, 1.0, "o trecho do esforço é a referência");
    let g = rope_route::weston_gear(R_IN, return_radius(4.0));
    assert_eq!(legs[1].weight, g, "o trecho abraçado é pesado pela Weston");
    assert_eq!(legs[2].weight, g, "e o outro ramo da cadernal também");
    assert_eq!(legs[3].weight, 0.0, "o ramo morto não carrega");
    assert_eq!(route.weight_b, 0.0, "logo a ponta B não é puxada");
    assert_eq!(
        route.weight_max, g,
        "o pico da corda é o trecho abraçado — é ele que a ruptura tem de olhar"
    );
}

/// **Um retorno que não é pelo diâmetro MENOR é recusado** — travado (`r = R`),
/// invertido (`r > R`) e sem superfície (`r = 0`).
///
/// A recusa devolve duas roldanas comuns, então sobra a vantagem da cadernal.
/// ⚠️ Mutação: tirar a condição `entry.radius > ret.radius` do `axle_pair` faz o
/// caso `r > R` produzir peso NEGATIVO, e o orçamento deixa de ser uma soma.
#[test]
fn a_return_by_a_radius_that_is_not_smaller_is_refused() {
    for ret in [R_IN, R_IN * 1.5, 0.0] {
        let (w, d, _) = rig(2.0, 1.0, ret, true);
        assert!(
            rope_route::axle_pair(d.wheels(w.pulley_wheels()), 0).is_none(),
            "retorno {ret} não descreve uma Weston que este orçamento segure"
        );
        // E o rig equilibra na vantagem da cadernal: 2.
        assert!(
            travel(2.0 * 0.8, ret, true) > STILL,
            "retorno {ret}: a 1,6 kg a carga tinha de subir (vantagem 2)"
        );
        assert!(
            travel(2.0 * 1.2, ret, true) < -STILL,
            "retorno {ret}: a 2,4 kg a carga tinha de descer"
        );
    }
}

/// **Um eixo, UM sentido de abraço** — e é isso que faz o diferencial SUBTRAIR.
///
/// ⚠️ Mutação: apagar o corpo do `tie_axle_pairs` deixa o lado do retorno ao sabor
/// da geometria; com os lados OPOSTOS o peso seria `R/(R+r)` — menor que 1 — e a
/// máquina viraria uma **desvantagem**. O gate afirma as duas metades: os lados
/// batem, E o peso é o da subtração.
#[test]
fn a_compound_axle_has_one_wrap_sense() {
    let ret = return_radius(4.0);
    // ⚠️ **A ponta MORTA fica do MESMO lado do esforço**, e essa é a fixture inteira:
    // com ela do lado oposto (a montagem natural) a poligonal dos centros já resolve
    // os dois contatos para o mesmo lado, o amarre é redundante, e **a mutação
    // sobrevive** — foi o que aconteceu na primeira rodada. Aqui a geometria QUER
    // lados opostos, então só o amarre pode dar o peso da subtração.
    let mut wheels = vec![
        RopeWheel {
            centre: [0.0, SHEAVE_Y],
            radius: R_IN,
            axle: AXLE,
            id: 1,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [0.0, 4.0],
            radius: 0.15,
            id: 2,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [0.0, SHEAVE_Y],
            radius: ret,
            axle: AXLE,
            id: 1,
            ..RopeWheel::default()
        },
    ];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([0.8, 6.0], [0.8, SHEAVE_Y], &mut wheels, &mut scratch);
    let wheels = &wheels[..];
    let (first, second) = rope_route::axle_pair(wheels, 0).expect("é um par");
    assert_eq!(
        wheels[first].side, wheels[second].side,
        "os dois contatos do mesmo eixo abraçam para o mesmo lado"
    );
    let g = rope_route::crossing_gear(wheels, first);
    assert_eq!(
        g,
        R_IN / (R_IN - ret),
        "o peso é o da SUBTRAÇÃO; com lados opostos ele seria R/(R+r) = {}",
        R_IN / (R_IN + ret)
    );
    assert!(g > 1.0, "uma Weston é uma vantagem, não uma desvantagem");
}

/// **Uma corda sem eixo composto é BYTE-IDÊNTICA** — a âncora de regressão.
///
/// A rota inteira (comprimentos, pesos, direções) da MESMA montagem lida como
/// roldanas comuns tem de bater com a de antes da wave, e o oráculo é `1.0` exato:
/// `x * 1.0 == x` no IEEE-754.
#[test]
fn a_rope_without_a_compound_axle_is_untouched() {
    let (w, d, _) = rig(2.0, 1.0, 0.3, false);
    let wheels = d.wheels(w.pulley_wheels());
    for i in 0..wheels.len() {
        assert!(rope_route::axle_pair(wheels, i).is_none());
        assert_eq!(
            rope_route::crossing_gear(wheels, i),
            wheels[i].gear(),
            "sem par, o cruzamento é a engrenagem de sempre"
        );
    }
    let mut legs = Vec::new();
    let route =
        rope_route::route([0.8, 6.0], [-0.8, SHEAVE_Y], wheels, &mut legs).expect("rota sã");
    assert!(legs.iter().all(|l| l.weight == 1.0));
    assert_eq!(route.weight_b, 1.0);
    assert_eq!(route.weight_max, 1.0);
}

/// **Um MOTOR num eixo de Weston é um sarilho DIFERENCIAL** — a carga sobe a
/// `ω(R−r)/2`, não a `ωR/2`.
///
/// ⚠️ **O termo de recolhimento é UM por eixo**, e é o do contato de ENTRADA
/// (`ω·R`, que é o que o sarilho paga do lado do esforço). O orçamento pesa esse
/// lado por `1` e o trecho abraçado por `R/(R−r)`, então a carga anda o pago
/// dividido por `2·R/(R−r)` — a razão contra um tambor comum é a própria
/// engrenagem.
///
/// ⚠️ **A ponta de esforço é ESTÁTICA, e isso é a fixture inteira.** Com um
/// contrapeso ali a medição fica confundida: o recolhimento vai para onde o
/// equilíbrio de forças mandar, e a primeira versão deste gate leu **2,05×** onde
/// tinha de ler 4 — não porque a lei estivesse errada, mas porque metade do
/// movimento era o contrapeso de 1 kg ganhando de uma carga de 1 kg pendurada em dois
/// ramos. Presa a ponta, o único lugar em que a corda recolhida pode caber é o trecho
/// abraçado, e o que se mede é CINEMÁTICA — que é o que a pergunta é (o guincho é
/// onipotente: medido, a mesma carga sobe o mesmo pesando 0,1 kg ou 1000 kg).
///
/// ⚠️ Mutação: somar também `ω·r` no retorno (contar a mesma volta duas vezes) muda
/// a taxa e o gate morde.
#[test]
fn a_motor_on_a_compound_axle_hoists_at_the_differential_rate() {
    const OMEGA: f32 = 1.0;
    const BODY_R: f32 = 0.2;
    let gear = 4.0_f32;
    let ret = return_radius(gear);
    let mut lifted = [0.0_f32; 2];
    for (k, pair) in [true, false].into_iter().enumerate() {
        let mut w = PhysicsWorld::new();
        let area = std::f32::consts::PI * BODY_R * BODY_R;
        // As DUAS pontas presas: a morta e a de esforço.
        let (dead, _) = w.add_static_cuboid(-0.8, SHEAVE_Y, 0.1, 0.1);
        let (effort, _) = w.add_static_cuboid(0.8, SHEAVE_Y - 2.0, 0.1, 0.1);
        let (block, _) = w.add_dynamic_circle(0.0, 4.0, BODY_R, 1.0 / area);
        let axle = if pair { AXLE } else { 0 };
        let mut wheels = vec![
            RopeWheel {
                centre: [0.0, SHEAVE_Y],
                radius: R_IN,
                axle,
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
            RopeWheel {
                centre: [0.0, SHEAVE_Y],
                radius: ret,
                axle,
                id: 1,
                ..RopeWheel::default()
            },
        ];
        let mut scratch = Vec::new();
        let ea = [0.8, SHEAVE_Y - 2.0];
        rope_route::resolve_sides(ea, [-0.8, SHEAVE_Y], &mut wheels, &mut scratch);
        let mut d = PulleyDesc {
            id: 1,
            body_a: effort,
            body_b: dead,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 3,
            total_length: 0.0,
            motor_rate: OMEGA * R_IN,
            break_force: f32::INFINITY,
        };
        w.set_pulleys(vec![d], wheels.clone());
        d.total_length = w.pulley_span(&d).expect("rota sã");
        w.set_pulleys(vec![d], wheels);
        // ⚠️ **Entre duas amostras TARDIAS**, nunca do repouso: a corda estica um
        // tanto até a correção suprir a tensão que segura a carga, e esse transitório
        // é um deslocamento ABSOLUTO — ele come 2% do braço rápido e 22% do lento, o
        // que fez a primeira versão deste gate ler 5,06 no lugar de 4. O que a
        // pergunta quer é a TAXA em regime.
        for _ in 0..60 {
            w.step();
        }
        let y0 = y(&w, block);
        for _ in 0..60 {
            w.step();
        }
        lifted[k] = y(&w, block) - y0;
    }
    let (weston, drum) = (lifted[0], lifted[1]);
    assert!(
        weston > 0.0 && drum > 0.0,
        "recolher ergue nos dois casos (weston {weston:.4}, tambor {drum:.4})"
    );
    let ratio = drum / weston;
    assert!(
        (ratio - gear).abs() < 0.15 * gear,
        "o tambor comum tinha de erguer {gear}x mais depressa; ergueu {ratio:.3}x \
         (weston {weston:.4} m, tambor {drum:.4} m em 1 s)"
    );
}
