//! **A POLIA** (W-Pulley) — os gates do kernel.
//!
//! O que uma corda por duas roldanas promete, afirmado contra o caminho do
//! produto (`PhysicsWorld::step`). As tabelas que escolheram os números vivem em
//! `measure_pulley.rs`.

use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::RopeWheel;
use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, ShapeDesc};
use rapier2d::dynamics::RigidBodyType;

const R: f32 = 0.2;
const WHEEL_Y: f32 = 4.0;
const START_Y: f32 = 2.0;

fn ball(x: f32, mass: f32) -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x,
        y: START_Y,
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

/// Duas roldanas no alto, um corpo pendurado sob cada uma. `slack` alonga a
/// corda além do vão para pedir uma corda FROUXA.
///
/// ⚠️ **Roldanas de raio ZERO** — o modelo de PONTO que a v1 desta wave shipou, e
/// que a rota reproduz exatamente. É a âncora de regressão: estes gates mediam a
/// polia antes de ela ter raio, e continuam medindo os mesmos números.
///
/// O comprimento inclui o trecho ENTRE as roldanas (`2.0`, a distância entre
/// elas), que o modelo de ponto não contava. É uma constante para roldanas
/// paradas, então a dinâmica é a mesma — o número é que passou a descrever a
/// corda inteira.
fn rig(mass_a: f32, mass_b: f32, slack: f32) -> (PhysicsWorld, PulleyDesc) {
    let mut w = PhysicsWorld::new();
    let a = w.spawn_body(ball(-1.0, mass_a));
    let b = w.spawn_body(ball(1.0, mass_b));
    let span = WHEEL_Y - START_Y;
    let d = PulleyDesc {
        body_a: a,
        body_b: b,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 2,
        total_length: 2.0 * span + WHEEL_SPAN + slack,
    };
    w.set_pulleys(vec![d], point_wheels());
    (w, d)
}

/// A distância entre as duas roldanas — corda que a rota conta e o modelo de
/// ponto ignorava.
const WHEEL_SPAN: f32 = 2.0;

/// As duas roldanas do rig, como a arena as recebe.
fn point_wheels() -> Vec<RopeWheel> {
    vec![
        RopeWheel {
            centre: [-1.0, WHEEL_Y],
            radius: 0.0,
            side: 1,
        },
        RopeWheel {
            centre: [1.0, WHEEL_Y],
            radius: 0.0,
            side: 1,
        },
    ]
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.body_pose(h).unwrap().translation.y
}

fn run(w: &mut PhysicsWorld, ticks: usize) {
    for _ in 0..ticks {
        w.step();
    }
}

/// **A corda é inextensível.** O que um lado desce, o outro sobe — e o controle
/// é o mesmo rig sem polia, onde os dois simplesmente caem.
#[test]
fn what_one_side_gains_the_other_loses() {
    let (mut w, d) = rig(2.0, 1.0, 0.0);
    run(&mut w, 60);
    let fell = START_Y - y(&w, d.body_a);
    let rose = y(&w, d.body_b) - START_Y;
    assert!(fell > 1.0, "o lado pesado tem de descer de verdade: {fell}");
    assert!(
        (fell - rose).abs() < 0.005,
        "a corda esticou {:.4} m em {fell:.4} m de percurso",
        (fell - rose).abs()
    );

    // O controle: sem polia os DOIS caem, e o lado leve nunca sobe.
    let (mut c, cd) = rig(2.0, 1.0, 0.0);
    c.set_pulleys(Vec::new(), Vec::new());
    run(&mut c, 60);
    assert!(
        y(&c, cd.body_b) < START_Y,
        "sem corda o corpo leve tem de CAIR, não subir"
    );
}

/// **Uma corda frouxa não faz nada** — e o oráculo é a igualdade EXATA com um
/// mundo que não tem polia nenhuma.
#[test]
fn a_slack_rope_leaves_the_bodies_in_free_fall() {
    let (mut w, d) = rig(2.0, 1.0, 10.0);
    let (mut c, cd) = rig(2.0, 1.0, 10.0);
    c.set_pulleys(Vec::new(), Vec::new());
    run(&mut w, 60);
    run(&mut c, 60);
    assert_eq!(
        (y(&w, d.body_a), y(&w, d.body_b)),
        (y(&c, cd.body_a), y(&c, cd.body_b)),
        "corda frouxa tem de ser byte-idêntica a não ter corda"
    );
}

/// **Uma corda PUXA e não empurra.** Um corpo lançado PARA a própria roldana
/// afrouxa a corda, e a partir daí a polia tem de ser invisível — a comparação
/// é contra o mesmo lançamento sem corda.
#[test]
fn the_rope_pulls_and_never_pushes() {
    let launch = |pulley: bool| {
        let mut w = PhysicsWorld::new();
        let mut up = ball(-1.0, 1.0);
        // Para CIMA, na direção da roldana: encurta o ramo, logo afrouxa a corda.
        up.linvel = [0.0, 6.0];
        let a = w.spawn_body(up);
        let b = w.spawn_body(ball(1.0, 1.0));
        if pulley {
            let span = WHEEL_Y - START_Y;
            w.set_pulleys(
                vec![PulleyDesc {
                    body_a: a,
                    body_b: b,
                    local_a: [0.0, 0.0],
                    local_b: [0.0, 0.0],
                    wheel_start: 0,
                    wheel_count: 2,
                    total_length: 2.0 * span + WHEEL_SPAN,
                }],
                point_wheels(),
            );
        }
        run(&mut w, 20);
        (y(&w, a), y(&w, b))
    };
    assert_eq!(
        launch(true),
        launch(false),
        "com a corda frouxa o voo tem de ser o voo livre"
    );
}

/// **A tensão numa corda única é UNIFORME — não há vantagem mecânica a ganhar de
/// roldanas livres, e é por isso que o `ratio` saiu.**
///
/// A v1 desta wave tinha um `ratio` vendido como talha (`l1 + razão·l2 ≤ L0`) e
/// ele descrevia uma corda que não existe: com uma corda só passando por rodas
/// que giram soltas, os dois corpos sentem a MESMA força, quaisquer que sejam os
/// diâmetros. Este gate afirma a física que ficou no lugar dele — **massas iguais
/// se equilibram, e o que um lado desce o outro sobe na razão 1:1** —, e é o
/// controle que denunciaria qualquer volta do multiplicador escondido.
///
/// ⚠️ A vantagem mecânica volta por onde ela vem no mundo: uma roldana montada
/// num corpo que se MOVE (a cadernal móvel, W3) ou um tambor DIRIGIDO (W2). As
/// duas são peças na cena, não um número no painel.
#[test]
fn a_single_rope_over_free_wheels_has_no_mechanical_advantage() {
    let (mut w, d) = rig(4.0, 1.0, 0.0);
    run(&mut w, 45);
    let dya = START_Y - y(&w, d.body_a);
    let dyb = y(&w, d.body_b) - START_Y;
    assert!(dyb > 0.2, "o lado leve tem de subir: {dyb}");
    let ratio = dya / dyb;
    assert!(
        (ratio - 1.0).abs() < 0.01,
        "um lado tem de andar o que o outro anda: {ratio:.4} (A {dya:.4}, B {dyb:.4})"
    );
    // E o equilíbrio: massas IGUAIS não se movem, o que uma talha de razão ≠ 1
    // não faria.
    let (mut balanced, bd) = rig(1.0, 1.0, 0.0);
    run(&mut balanced, 45);
    let drift = (START_Y - y(&balanced, bd.body_a)).abs();
    assert!(
        drift < 0.02,
        "massas iguais numa corda única se equilibram: derivou {drift:.4} m"
    );
}

/// **O esticamento não depende da CARGA** — a assinatura da massa efetiva exata.
///
/// Um ganho fixo (uma mola PD afinada à mão) daria erro proporcional ao peso; a
/// projeção de velocidade divide pela massa que de fato há, então uma pena e uma
/// tonelada esticam a corda igual.
#[test]
fn the_stretch_is_the_same_for_a_feather_and_for_a_ton() {
    let stretch = |m: f32| {
        let (mut w, d) = rig(m, m, 0.0);
        run(&mut w, 90);
        w.pulley_span(&d).unwrap() - d.total_length
    };
    let light = stretch(0.1);
    let heavy = stretch(100.0);
    assert!(light > 0.0 && heavy > 0.0, "a corda tem de estar esticada");
    assert!(
        (light - heavy).abs() < 1.0e-4,
        "pena {light:.6} m contra tonelada {heavy:.6} m: a massa vazou para o erro"
    );
}

/// **Um eixo congelado é massa infinita para a corda.**
///
/// `effective_inv_mass` é um VETOR por-eixo, e é ele que carrega o Freeze Y do
/// W-LockPos. O oráculo é que um corpo com Y travado se comporta, para a corda,
/// exatamente como um corpo ESTÁTICO — quem trocar a forma quadrática por um
/// escalar `1/m` dá massa finita a um eixo que não tem nenhuma, e o outro lado
/// da corda passa a andar diferente.
#[test]
fn a_frozen_axis_is_infinite_mass_to_the_rope() {
    let lift = |anchor: RigidBodyType, lock_y: bool| {
        let mut w = PhysicsWorld::new();
        let mut fixed_side = ball(-1.0, 1.0);
        fixed_side.body_type = anchor;
        fixed_side.lock_y = lock_y;
        let a = w.spawn_body(fixed_side);
        let b = w.spawn_body(ball(1.0, 3.0));
        let span = WHEEL_Y - START_Y;
        w.set_pulleys(
            vec![PulleyDesc {
                body_a: a,
                body_b: b,
                local_a: [0.0, 0.0],
                local_b: [0.0, 0.0],
                wheel_start: 0,
                wheel_count: 2,
                total_length: 2.0 * span + WHEEL_SPAN,
            }],
            point_wheels(),
        );
        run(&mut w, 45);
        (y(&w, a), y(&w, b))
    };
    let (frozen_a, frozen_b) = lift(RigidBodyType::Dynamic, true);
    let (_, wall_b) = lift(RigidBodyType::Fixed, false);
    assert!(
        (frozen_a - START_Y).abs() < 1.0e-6,
        "um eixo Y travado não anda em Y: {frozen_a}"
    );
    assert!(
        (frozen_b - wall_b).abs() < 1.0e-3,
        "para a corda, Y travado tem de valer o mesmo que uma parede: {frozen_b:.5} vs {wall_b:.5}"
    );
}

/// Uma polia que perdeu um corpo sai da mesa — não é higiene: a tabela ficaria
/// nomeando um handle morto que a arena pode reciclar.
#[test]
fn a_pulley_whose_body_is_gone_is_dropped() {
    let (mut w, d) = rig(2.0, 1.0, 0.0);
    assert_eq!(w.pulleys().len(), 1);
    w.remove_body(d.body_b);
    assert!(w.pulleys().is_empty(), "a polia tem de cair com o corpo");
    run(&mut w, 30);
    assert!(
        y(&w, d.body_a) < START_Y,
        "sem a corda o corpo que sobrou apenas cai"
    );
}

/// **Uma parede é massa infinita para a corda** — o defeito que o gate acima
/// pegou, afirmado direto.
///
/// Um corpo `Fixed` guarda o `effective_inv_mass` dos próprios colliders (medido:
/// uma parede de 1 kg reporta `1.0`), porque quem honra o TIPO no rapier é o
/// solver e não a tabela de massas. Uma corda que dividisse a correção com a
/// parede ficaria frouxa: o oráculo é que amarrar na parede segura MELHOR do que
/// amarrar num parceiro livre de mesma massa, que é o que qualquer um espera de
/// uma parede.
#[test]
fn a_wall_is_infinite_mass_to_the_rope() {
    let sag = |anchor: RigidBodyType| {
        let mut w = PhysicsWorld::new();
        let mut side = ball(-1.0, 1.0);
        side.body_type = anchor;
        let a = w.spawn_body(side);
        let b = w.spawn_body(ball(1.0, 3.0));
        let span = WHEEL_Y - START_Y;
        let d = PulleyDesc {
            body_a: a,
            body_b: b,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 2,
            total_length: 2.0 * span + WHEEL_SPAN,
        };
        w.set_pulleys(vec![d], point_wheels());
        run(&mut w, 90);
        w.pulley_span(&d).unwrap() - d.total_length
    };
    let wall = sag(RigidBodyType::Fixed);
    let free_partner = sag(RigidBodyType::Dynamic);
    assert!(
        wall < free_partner * 0.5,
        "a parede tem de segurar melhor que um parceiro livre: {wall:.6} vs {free_partner:.6}"
    );
    // E o número: abaixo da tolerância de repouso do próprio rapier (1,3 mm), que
    // é o teto que o `PULLEY_BIAS` foi escolhido para casar.
    assert!(wall < 0.0013, "a corda na parede esticou {wall:.6} m");
}

/// **Um corpo KINEMATIC é um guincho.**
///
/// Massa infinita (a corda não o move) **mas com movimento próprio**: é a metade
/// que a correção acima deixou viva de propósito ao zerar só o `k` e nunca o
/// `rate`. Uma bobina dirigida por curva ergue a carga.
#[test]
fn a_kinematic_body_is_a_winch() {
    let mut w = PhysicsWorld::new();
    let mut drum = ball(-1.0, 1.0);
    drum.body_type = RigidBodyType::KinematicPositionBased;
    let a = w.spawn_body(drum);
    let b = w.spawn_body(ball(1.0, 1.0));
    let span = WHEEL_Y - START_Y;
    w.set_pulleys(
        vec![PulleyDesc {
            body_a: a,
            body_b: b,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 2,
            total_length: 2.0 * span + WHEEL_SPAN,
        }],
        point_wheels(),
    );
    // A bobina DESCE: o ramo dela alonga, então a corda tem de encurtar o outro
    // — a carga sobe.
    for tick in 0..60 {
        let target = START_Y - 0.02 * (tick + 1) as f32;
        w.set_next_kinematic_pose(a, -1.0, target, 0.0);
        w.step();
    }
    let load = y(&w, b);
    assert!(
        load > START_Y + 1.0,
        "o guincho tem de erguer a carga; ela está em {load:.4}"
    );
}

/// **O clamp de só-puxa tem gate próprio** — a guarda de folga sozinha não o
/// cobre, e é a lição [[feedback_layered_defenses_need_per_layer_gates]].
///
/// A camada que só ELE vê: a corda está esticada (`C > 0`, logo a guarda de
/// folga deixa passar) e os dois corpos correm PARA as roldanas, então `Ċ` é
/// muito negativo e `λ` sai negativo. Sem o clamp isso vira a corda **empurrando**
/// os corpos para longe das roldanas para "manter-se esticada" — que é
/// exatamente o que uma corda não faz.
#[test]
fn a_taut_rope_that_is_slackening_fast_still_never_pushes() {
    let launch = |pulley: bool| {
        let mut w = PhysicsWorld::new();
        let mut up_a = ball(-1.0, 1.0);
        let mut up_b = ball(1.0, 1.0);
        up_a.linvel = [0.0, 6.0];
        up_b.linvel = [0.0, 6.0];
        let a = w.spawn_body(up_a);
        let b = w.spawn_body(up_b);
        if pulley {
            let span = WHEEL_Y - START_Y;
            w.set_pulleys(
                vec![PulleyDesc {
                    body_a: a,
                    body_b: b,
                    local_a: [0.0, 0.0],
                    local_b: [0.0, 0.0],
                    wheel_start: 0,
                    wheel_count: 2,
                    // Mal esticada: `C = +0,001` no primeiro sub-passo, e os dois
                    // corpos já subindo — a janela em que só o clamp responde.
                    total_length: 2.0 * span + WHEEL_SPAN - 0.001,
                }],
                point_wheels(),
            );
        }
        run(&mut w, 20);
        (y(&w, a), y(&w, b))
    };
    assert_eq!(
        launch(true),
        launch(false),
        "uma corda que afrouxa depressa não pode empurrar ninguém"
    );
}

/// **O guincho é ANTECIPADO, não só corrigido.**
///
/// Um corpo não-dinâmico não entra na massa efetiva (é infinito) mas entra em
/// `Ċ`, e é isso que faz a carga seguir a bobina em vez de ficar atrás dela. A
/// assinatura: com o termo, o atraso é **constante** na velocidade da bobina;
/// sem ele, o atraso é puramente posicional e cresce com ela (medido: 0,00053 m
/// nas duas velocidades, contra 0,01095 e 0,02553).
#[test]
fn the_winch_does_not_lag_further_the_faster_it_reels() {
    let lag = |speed: f32| {
        let mut w = PhysicsWorld::new();
        let mut drum = ball(-1.0, 1.0);
        drum.body_type = RigidBodyType::KinematicPositionBased;
        let a = w.spawn_body(drum);
        let b = w.spawn_body(ball(1.0, 1.0));
        let span = WHEEL_Y - START_Y;
        w.set_pulleys(
            vec![PulleyDesc {
                body_a: a,
                body_b: b,
                local_a: [0.0, 0.0],
                local_b: [0.0, 0.0],
                wheel_start: 0,
                wheel_count: 2,
                total_length: 2.0 * span + WHEEL_SPAN,
            }],
            point_wheels(),
        );
        let dt = 1.0 / 60.0;
        for tick in 0..60 {
            w.set_next_kinematic_pose(a, -1.0, START_Y - speed * dt * (tick + 1) as f32, 0.0);
            w.step();
        }
        (speed * dt * 60.0) - (y(&w, b) - START_Y)
    };
    for speed in [0.5_f32, 1.2] {
        let got = lag(speed);
        assert!(
            got.abs() < 0.002,
            "a {speed} m/s a carga ficou {got:.5} m atrás da bobina"
        );
    }
}
