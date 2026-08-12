//! Guards for the **speed limit** ([`limit_speed`], doc 89 folha 13 P1).
//!
//! A CHILD of `tests.rs`, so `moving`/`unlimited` are the same fixtures the step gates use.

use super::*;

fn vel(s: &Stream) -> [f32; 2] {
    match s.get("vel") {
        Some(Column::Vec2(v)) => v[0],
        _ => panic!("no `vel`"),
    }
}

fn speed(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// **DESLIGADO É DESLIGADO, AO BIT** — o mundo pré-wave inteiro, byte a byte.
///
/// Os dois zeros saem da porta pela PRIMEIRA linha, antes de qualquer `sqrt`: um documento que
/// nunca tocou nestes controles não paga sequer a raiz, e o resultado não é *próximo* do de
/// ontem, é o mesmo.
///
/// FALSIFICADO por um limite escrito como `min(v, max)` sem o sentinela: com `max = 0` toda
/// velocidade viraria zero e a cena inteira congelaria.
#[test]
fn off_is_off_to_the_bit() {
    for v in [[0.0f32, 0.0], [3.0, -4.0], [1e-9, 0.0], [-7.5, 0.25]] {
        assert_eq!(limit_speed(v, 0.0, 0.0), v, "{v:?}");
        // E negativo também é desligado: um limite de velocidade negativo não é um pedido.
        assert_eq!(limit_speed(v, -1.0, -1.0), v, "{v:?}");
    }
    let s = moving([0.0, 0.0], [2.0, -3.0], 0.0);
    assert_eq!(
        vel(&step(&s, 0.02, 1.0, 0.0, 0.0)),
        vel(&unlimited(&s, 0.02, 1.0))
    );
}

/// **O TETO CAPA A VELOCIDADE E PRESERVA A DIREÇÃO.**
///
/// Um limite que mexesse na direção não seria um limite, seria uma força — e o artista veria a
/// nuvem TORCER quando ele arrastasse o slider.
#[test]
fn the_ceiling_caps_the_speed_and_keeps_the_direction() {
    let v = limit_speed([30.0, 40.0], 0.0, 10.0); // |v| = 50
    assert!((speed(v) - 10.0).abs() < 1e-5, "capou: {v:?}");
    // A direção é a mesma: os componentes ficam na razão 3:4.
    assert!((v[0] / v[1] - 0.75).abs() < 1e-5, "a direção virou: {v:?}");
    // E o que já está abaixo do teto não é tocado, ao bit.
    assert_eq!(limit_speed([3.0, 4.0], 0.0, 10.0), [3.0, 4.0]);
}

/// **O TETO CAPA A DISTÂNCIA QUE O ELEMENTO ANDA NESTE TIQUE** — e este é o gate que um nó a
/// jusante NÃO CONSEGUIRIA passar.
///
/// A colocação é a feature inteira: o limite roda entre a atualização da velocidade e a da
/// posição, então `P` avança com a velocidade JÁ capada. Um `sim.speed_limit` depois do passo
/// caparia o número que o elemento *reporta* e deixaria a posição andar o passo inteiro — um
/// tique atrasado, por construção, e é exatamente o tique em que um elemento arremessado
/// atravessa a parede.
///
/// FALSIFICADO por mover o clamp para depois de `q += v·dt`: a velocidade sairia certa e a
/// posição, errada — um gate que só olhasse `vel` ficaria verde.
#[test]
fn the_ceiling_caps_the_distance_walked_this_tick() {
    let dt = 0.02f32;
    let fast = moving([0.0, 0.0], [100.0, 0.0], 0.0);
    let capped = step(&fast, dt, 1.0, 0.0, 10.0);
    let free = unlimited(&fast, dt, 1.0);
    let px = |s: &Stream| match s.get("P") {
        Some(Column::Vec2(v)) => v[0][0],
        _ => panic!("no `P`"),
    };
    assert!(
        (px(&capped) - 10.0 * dt).abs() < 1e-5,
        "andou {} onde o teto permite {}",
        px(&capped),
        10.0 * dt
    );
    assert!(
        px(&free) > px(&capped) * 5.0,
        "o controle tem de andar MUITO mais: {} contra {}",
        px(&free),
        px(&capped)
    );
}

/// **O PISO ACELERA O QUE SE MOVE DEVAGAR E NÃO ACORDA O QUE PAROU.**
///
/// `v = 0` não tem direção, e a resposta honesta é deixá-lo onde está em vez de escolher um rumo
/// por ele — o mesmo que o `sim.collide` faz no centro exato de um disco. A consequência que o
/// artista vê está no doc da porta: um piso **não** levanta o que assentou.
///
/// FALSIFICADO por normalizar sem guarda: o elemento parado sairia com `NaN` e envenenaria a zona.
#[test]
fn the_floor_lifts_what_crawls_and_leaves_what_stopped() {
    let slow = limit_speed([0.3, 0.4], 5.0, 0.0); // |v| = 0.5
    assert!((speed(slow) - 5.0).abs() < 1e-5, "acelerou: {slow:?}");
    assert!((slow[0] / slow[1] - 0.75).abs() < 1e-5, "a direção virou");

    let stopped = limit_speed([0.0, 0.0], 5.0, 0.0);
    assert_eq!(stopped, [0.0, 0.0], "sem direção não há o que acelerar");
    assert!(stopped.iter().all(|x| x.is_finite()), "e nada de NaN");
}

/// **O TETO VENCE O PISO** — um piso capaz de empurrar um elemento através do teto tornaria o
/// teto uma sugestão.
///
/// FALSIFICADO por trocar a ordem das duas linhas: com `min > max` o resultado sairia `min`.
#[test]
fn the_ceiling_beats_the_floor() {
    // O artista inverteu os dois: piso 20, teto 5.
    for v in [[1.0f32, 0.0], [100.0, 0.0], [0.0, -3.0]] {
        let out = limit_speed(v, 20.0, 5.0);
        assert!(
            (speed(out) - 5.0).abs() < 1e-5,
            "de {v:?} saiu {out:?}, e o teto era 5"
        );
    }
}

/// **A SONDA** — as velocidades que as cenas de fato produzem, que é de onde a faixa confortável
/// do slider tem de sair (§0: meça antes de escrever um limite).
///
/// `cargo test -p ph2d-node-sim-step tests::speed::probe -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_natural_speeds() {
    // Queda livre sob as gravidades que as cenas usam, até o `MAX_DT` do passo.
    for g in [5.0f32, 6.0, 8.0] {
        let mut s = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("vel", Column::Vec2(vec![[0.0, 0.0]]))
            .with("sim_t", Column::Scalar(vec![0.0]))
            .with("accel", Column::Vec2(vec![[0.0, -g]]));
        let mut peak = 0.0f32;
        for k in 1..=240 {
            let t = k as f32 / 60.0;
            let stepped = step(&s, t, 1.0, 0.0, 0.0);
            peak = peak.max(speed(vel(&stepped)));
            s = stepped
                .with("accel", Column::Vec2(vec![[0.0, -g]]))
                .with("sim_t", Column::Scalar(vec![t]));
        }
        eprintln!("gravidade {g}: pico de {peak:.2} u/s em 4 s de queda livre");
    }
}
