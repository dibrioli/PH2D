//! **O TAMBOR DIFERENCIAL** (W-Pulley W4) — a vantagem mecânica CONTÍNUA, e a
//! prova de que ela cai de duas circunferências em vez de ser digitada.
//!
//! O `ratio` que o W1 aposentou descrevia *"uma talha diferencial com o eixo
//! invisível"* (§3 do plano). Aqui o eixo é UMA roldana com DOIS raios — o que a
//! corda entra e o que ela sai —, os dois são desenhados, e o número cai deles.
//!
//! ⚠️ **Duas roldanas concêntricas seriam a leitura ingênua e são impossíveis:** a
//! tangente comum exige `|C₂−C₁| > |s₂r₂ − s₁r₁|`, que dois círculos de mesmo
//! centro nunca satisfazem ⇒ a rota inteira seria recusada e a corda sumiria. É
//! por isso que os dois raios moram na MESMA roldana.
//!
//! Os números vêm de `tests/measure_pulley_differential.rs`, pelo caminho do
//! produto (`PhysicsWorld::step`).

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// O sarilho: o contrapeso no lado por onde a corda ENTRA, a carga no lado por
/// onde ela SAI, e um tambor no teto.
///
/// `r_out = None` é o **CONTROLE**: a mesma montagem com roldana comum, onde a
/// vantagem é 1 porque a tensão de uma corda que desliza é uniforme.
fn windlass(
    load: f32,
    counter: f32,
    r_in: f32,
    r_out: Option<f32>,
) -> (PhysicsWorld, RigidBodyHandle, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    const BODY_R: f32 = 0.2;
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (a, _) = w.add_dynamic_circle(-1.0, 5.0, BODY_R, counter / area);
    let (b, _) = w.add_dynamic_circle(1.0, 5.0, BODY_R, load / area);
    let mut wheels = vec![RopeWheel {
        centre: [0.0, 8.0],
        radius: r_in,
        radius_out: r_out,
        id: 1,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([-1.0, 5.0], [1.0, 5.0], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        id: 1,
        body_a: a,
        body_b: b,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        total_length: 0.0,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, a, b)
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).map_or(f32::NAN, |b| b.translation().y)
}

/// Quanto a CARGA anda em 1 s. Positivo é subir.
fn load_travel(load: f32, counter: f32, r_in: f32, r_out: Option<f32>) -> f32 {
    let (mut w, _, b) = windlass(load, counter, r_in, r_out);
    let y0 = y(&w, b);
    for _ in 0..60 {
        w.step();
    }
    y(&w, b) - y0
}

/// **A vantagem mecânica é o quociente dos dois raios.**
///
/// O oráculo é uma tabela 2×2 onde a **MESMA carga com o MESMO contrapeso** dá
/// vereditos OPOSTOS, e a única coisa que muda entre as duas colunas é o raio de
/// saída do tambor. Um limiar sobre um número só não distinguiria *a engrenagem
/// funciona* de *a corda está frouxa*.
///
/// Medido: com contrapeso de 1 kg, a carga de equilíbrio é **1,00 kg** na roldana
/// comum e **2,00 kg** no tambor 0,50 → 0,25. Então 1,6 kg **desce** numa e
/// **sobe** na outra.
#[test]
fn the_advantage_is_the_quotient_of_the_two_radii() {
    const COUNTER: f32 = 1.0;
    const LOAD: f32 = 1.6;
    let plain = load_travel(LOAD, COUNTER, 0.5, None);
    let geared = load_travel(LOAD, COUNTER, 0.5, Some(0.25));
    assert!(
        plain < -0.1,
        "numa roldana COMUM a tensão é uniforme, então {LOAD} kg tinha de vencer \
         {COUNTER} kg e descer; ela andou {plain:.4} m"
    );
    assert!(
        geared > 0.1,
        "com o tambor 0,50 → 0,25 a vantagem é 2, então {LOAD} kg está ABAIXO do \
         equilíbrio (2,0 kg) e tinha de subir; ela andou {geared:.4} m"
    );
}

/// **E o equilíbrio pousa no número previsto**, não perto dele: a carga prevista
/// fica parada e as duas vizinhas a ±20 % andam em sentidos OPOSTOS.
///
/// Exaustivo sobre quatro engrenagens — uma única razão poderia estar certa por
/// acidente de escala.
#[test]
fn the_balance_lands_on_the_predicted_load_for_every_gear() {
    for r_out in [0.5_f32, 0.25, 0.125, 0.1] {
        let gear = 0.5 / r_out;
        let light = load_travel(gear * 0.8, 1.0, 0.5, Some(r_out));
        let heavy = load_travel(gear * 1.2, 1.0, 0.5, Some(r_out));
        assert!(
            light > 0.0,
            "engrenagem {gear:.2}: 20 % abaixo do previsto a carga tinha de SUBIR; \
             andou {light:.4} m"
        );
        assert!(
            heavy < 0.0,
            "engrenagem {gear:.2}: 20 % acima do previsto a carga tinha de DESCER; \
             andou {heavy:.4} m"
        );
    }
}

/// **Uma roldana comum é BYTE-IDÊNTICA à de antes do W4** — a âncora de regressão
/// da wave, e a mesma do W1.
///
/// `gear()` devolve **exatamente** `1.0`, e `x * 1.0 == x` é exato no IEEE-754;
/// então dizer `radius_out: None` e dizer `Some(r)` com o mesmo `r` produzem a
/// mesma rota **bit a bit**, e nenhum peso se move.
#[test]
fn an_ordinary_wheel_is_byte_identical() {
    let mk = |r_out: Option<f32>| RopeWheel {
        centre: [0.0, 8.0],
        radius: 0.5,
        radius_out: r_out,
        side: -1,
        id: 1,
        ..RopeWheel::default()
    };
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    let a = rope_route::route([-1.0, 5.0], [1.0, 5.0], &[mk(None)], &mut s1).expect("rota");
    let b = rope_route::route([-1.0, 5.0], [1.0, 5.0], &[mk(Some(0.5))], &mut s2).expect("rota");
    assert_eq!(
        a.length.to_bits(),
        b.length.to_bits(),
        "o mesmo raio dos dois lados tem de ser a corda de sempre, ao bit"
    );
    assert_eq!(a.weight_b.to_bits(), 1.0_f32.to_bits(), "sem engrenagem");
    assert_eq!(a.weight_max.to_bits(), 1.0_f32.to_bits(), "sem engrenagem");
    for (x, y) in s1.iter().zip(s2.iter()) {
        assert_eq!(x.weight.to_bits(), 1.0_f32.to_bits());
        assert_eq!(x.len.to_bits(), y.len.to_bits());
    }
    assert_eq!(mk(None).gear().to_bits(), 1.0_f32.to_bits());
    assert_eq!(mk(Some(0.5)).gear().to_bits(), 1.0_f32.to_bits());
}

/// **A corda SAI tangente ao círculo de saída** — a metade GEOMÉTRICA, que a
/// metade dinâmica não prova.
///
/// Sem ela, ignorar o segundo raio na geometria (e honrá-lo só na engrenagem)
/// passaria em todos os gates de força: a corda seria desenhada saindo do
/// diâmetro errado e a conta continuaria fechando.
#[test]
fn the_rope_leaves_on_the_out_circle() {
    let wheel = RopeWheel {
        centre: [0.0, 8.0],
        radius: 0.5,
        radius_out: Some(0.125),
        side: -1,
        id: 1,
        ..RopeWheel::default()
    };
    let mut legs = Vec::new();
    rope_route::route([-1.0, 5.0], [1.0, 5.0], &[wheel], &mut legs).expect("rota");
    let dist = |p: [f32; 2]| ((p[0] - 0.0_f32).powi(2) + (p[1] - 8.0_f32).powi(2)).sqrt();
    assert!(
        (dist(legs[0].to) - 0.5).abs() < 1.0e-4,
        "a corda tinha de CHEGAR no círculo de 0,50; encostou a {:.4} do centro",
        dist(legs[0].to)
    );
    assert!(
        (dist(legs[1].from) - 0.125).abs() < 1.0e-4,
        "a corda tinha de SAIR do círculo de 0,125; largou a {:.4} do centro",
        dist(legs[1].from)
    );
}

/// **O EIXO do tambor carrega as DUAS tensões, cada uma com o seu peso.**
///
/// Num diferencial os dois lados não puxam igual, então a resultante no centro
/// não é a de uma roldana comum. O oráculo deriva as direções dos PONTOS de
/// tangência — não do campo `dir` —, para não ser um espelho da função.
#[test]
fn the_axle_of_a_differential_carries_two_different_tensions() {
    let wheel = RopeWheel {
        centre: [0.0, 8.0],
        radius: 0.5,
        radius_out: Some(0.25),
        side: -1,
        id: 1,
        ..RopeWheel::default()
    };
    let mut legs = Vec::new();
    rope_route::route([-1.0, 5.0], [1.0, 5.0], &[wheel], &mut legs).expect("rota");
    let unit = |t: &ph2d_physics::world::rope_route::Tangent| {
        let d = [t.to[0] - t.from[0], t.to[1] - t.from[1]];
        let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
        [d[0] / n, d[1] / n]
    };
    let (u_in, u_out) = (unit(&legs[0]), unit(&legs[1]));
    let gear = wheel.gear();
    let want = [u_in[0] - u_out[0] * gear, u_in[1] - u_out[1] * gear];
    let got = rope_route::wheel_jacobian(&legs, 0).expect("o eixo tem Jacobiano");
    assert!(
        (got[0] - want[0]).abs() < 1.0e-4 && (got[1] - want[1]).abs() < 1.0e-4,
        "o eixo do diferencial devolveu {got:?}; os dois lados pesados dizem {want:?}"
    );
    // E ele difere do de uma roldana comum na MESMA geometria — senão o gate
    // acima poderia estar verde sobre pesos que não fazem nada.
    let plain = RopeWheel {
        radius_out: None,
        ..wheel
    };
    let mut plain_legs = Vec::new();
    rope_route::route([-1.0, 5.0], [1.0, 5.0], &[plain], &mut plain_legs).expect("rota");
    let plain_j = rope_route::wheel_jacobian(&plain_legs, 0).expect("Jacobiano");
    let mag = |v: [f32; 2]| (v[0] * v[0] + v[1] * v[1]).sqrt();
    assert!(
        (mag(got) - mag(plain_j)).abs() > 0.2,
        "a carga do eixo de um diferencial ({:.4}) não pode ser a de uma roldana \
         comum ({:.4})",
        mag(got),
        mag(plain_j)
    );
}

/// **Um raio de saída não-positivo é *não é um diferencial*, não um `NaN`.**
///
/// Uma regra, dois consumidores: a geometria cai para o raio de entrada e a
/// engrenagem para `1.0`, então elas não podem discordar. Sem isso, `r/0` seria
/// `inf` no orçamento da corda e a rota chegaria envenenada ao hash C9.
#[test]
fn a_non_positive_out_radius_is_not_a_differential() {
    for bad in [Some(0.0_f32), Some(-0.25)] {
        let w = RopeWheel {
            centre: [0.0, 8.0],
            radius: 0.5,
            radius_out: bad,
            side: -1,
            id: 1,
            ..RopeWheel::default()
        };
        assert_eq!(w.gear().to_bits(), 1.0_f32.to_bits(), "{bad:?}");
        assert_eq!(w.radius_out().to_bits(), 0.5_f32.to_bits(), "{bad:?}");
        let mut legs = Vec::new();
        let r = rope_route::route([-1.0, 5.0], [1.0, 5.0], &[w], &mut legs).expect("rota sã");
        assert!(r.length.is_finite(), "{bad:?} envenenou o comprimento");
        assert_eq!(r.weight_max.to_bits(), 1.0_f32.to_bits(), "{bad:?}");
    }
}

/// **A ruptura olha o lado MAIS carregado da corda.**
///
/// ⚠️ O `break_force` do `PulleyDesc` justifica ser um número só afirmando que *a
/// tensão é uniforme* — e o diferencial **falsifica** essa premissa: é
/// exatamente onde a corda não desliza, e os dois lados carregam `T` e `T·gear`.
/// Uma corda com engrenagem 4 que só comparasse o lado leve aguentaria quatro
/// vezes o que o artista dimensionou, em silêncio.
///
/// Medido (carga 4 kg, contrapeso 1 kg, 0,5 s): pico **17,4 N** na roldana comum
/// · **32,3 N** com engrenagem 2 · **42,6 N** com engrenagem 4. O limiar de 25 N
/// senta entre o primeiro e o último de propósito.
///
/// ⚠️ **O discriminante é o MESMO rig com e sem limiar** — comparar contra a
/// roldana comum sozinho seria fraco, porque ali a carga já cai por ser pesada, e
/// *"caiu"* não distingue *a corda partiu* de *a corda a está segurando devagar*.
#[test]
fn the_break_threshold_sees_the_heavier_side() {
    let run = |limit: f32, r_out: Option<f32>| {
        let (mut w, _, b) = windlass(4.0, 1.0, 0.5, r_out);
        let mut d = w.pulleys()[0];
        d.break_force = limit;
        let wheels = w.pulley_wheels().to_vec();
        w.set_pulleys(vec![d], wheels);
        let y0 = y(&w, b);
        for _ in 0..30 {
            w.step();
        }
        y(&w, b) - y0
    };
    const LIMIT: f32 = 25.0;
    let held = run(f32::INFINITY, Some(0.125));
    assert!(
        held > -0.3,
        "com engrenagem 4 uma carga de 4 kg está quase em equilíbrio e a corda a \
         SEGURA; sem limiar ela andou {held:.4} m — a fixture não contém o \
         fenômeno que o gate mede"
    );
    let parted = run(LIMIT, Some(0.125));
    assert!(
        parted < -1.0,
        "o lado pesado passa de {LIMIT} N (medido: 42,6), então a MESMA corda tinha \
         de partir e largar a carga; ela andou {parted:.4} m contra {held:.4} sem \
         limiar"
    );
    // O CONTROLE: na roldana comum o pico é 17,4 N, abaixo do limiar — a corda
    // segura, e a carga desce só o que uma carga pesada desce.
    let plain = run(LIMIT, None);
    assert!(
        plain > -1.0,
        "o mesmo limiar não pode partir a corda COMUM, cujo pico é 17,4 N; ela \
         andou {plain:.4} m"
    );
}
