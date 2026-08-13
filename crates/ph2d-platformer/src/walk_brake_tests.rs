//! Os gates do **FREIO** (`W-Brake`) — e a sonda que os numerou.
//!
//! Irmão de `walk.rs` pelo teto de LOC, e o corte é por ASSUNTO: o pai fica com
//! *o que a caminhada é*, este com *quanto custa parar*.
//!
//! ⚠️ **Módulo FILHO** (via `#[path]`), não um `tests/` de integração — é isso
//! que mantém `use super::*` a alcançar o [`brake_scale`] privado.
//!
//! ⚠️ **A distância de paragem é medida pela porta do PRODUTO**
//! ([`crate::kinematic_advance`]), e não por um integrador escrito aqui: o
//! caminho de um número que vira decisão de produto tem de sair da porta que o
//! artista de facto atravessa. Sobre chão plano o modo cinemático absorve a
//! gravidade, então o que sobra no eixo é exactamente `v += accel·dt + boost` —
//! a mesma aritmética que a ponte dinâmica faz do outro lado da cerca.

use super::*;
use crate::{KinematicState, PlayerConfig, PlayerInput, PlayerState, Support, kinematic_advance};

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
const DT: f32 = 1.0 / 60.0;

fn flat_at(distance: f32) -> GroundSample {
    GroundSample {
        distance,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 0.0],
        one_way: false,
    }
}

/// **Quanto ele percorre até parar**, largando o direcional à velocidade de
/// cruzeiro — e quantos tiques leva.
///
/// ⚠️ **O critério de parada é `|v| < 1 mm/s`, e o teto de tiques é o que impede
/// um `brake_scale = 0` de rodar para sempre** — em `0` a resposta HONESTA é
/// *"não pára"*, e é isso que o `None` diz.
fn stopping(brake: f32) -> Option<(f32, u32)> {
    let mut cfg = PlayerConfig::STARTING_POINT;
    cfg.walk.brake_scale = brake;

    let ground = flat_at(cfg.ride.float_height);
    let mut state = KinematicState {
        velocity: [cfg.walk.speed, 0.0],
        grounded: true,
    };
    let mut travelled = 0.0_f32;

    for tick in 1..=600 {
        let step = crate::player_motor(
            &cfg,
            Some(&ground),
            None,
            None,
            None,
            None,
            PlayerInput::default(),
            PlayerState::default(),
            state.velocity,
            G,
            UP,
            DT,
            crate::Buoyed::DRY,
            Support::Snap,
        );
        let (next, delta) = kinematic_advance(
            state,
            step.motor,
            Some(&ground),
            G,
            UP,
            DT,
            crate::Fluid::DRY,
        );
        state = KinematicState {
            grounded: true,
            ..next
        };
        travelled += delta[0];
        if state.velocity[0].abs() < 1.0e-3 {
            return Some((travelled, tick));
        }
    }
    None
}

/// **A SONDA que numerou os gates** — a varredura do freio.
///
/// `cargo test -p ph2d-platformer measure_the_brake -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_brake() {
    eprintln!("  brake   distancia   tiques");
    for b in [0.0, 0.25, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 12.0] {
        match stopping(b) {
            Some((d, t)) => eprintln!("  {b:5.2}   {d:9.4}   {t:6}"),
            None => eprintln!("  {b:5.2}      (nunca)   >600"),
        }
    }
    let cfg = PlayerConfig::STARTING_POINT;
    // O ponto de saturação previsto pela lei: a sobra cabe num tique quando
    // `speed <= turn·accel·dt·b`, com `turn = 1 + speed/(2·speed) = 1.5`.
    let sat = cfg.walk.speed / (1.5 * cfg.walk.acceleration * DT);
    eprintln!("  saturacao prevista: brake = {sat:.4}");
}

/// **O freio ENCURTA a parada, e os números são os da sonda.**
///
/// ⚠️ **O plano 10 §2 previa *"`2.0` para em METADE da distância"* e a medição o
/// REFUTOU: para em `0,343×`.** A previsão vem do modelo contínuo (`v²/2a`, onde
/// dobrar `a` corta a distância ao meio) e a lei não é contínua — o fator de
/// viragem faz `a` crescer com a sobra, e a paragem inteira cabe em **3 a 5
/// tiques**, onde quem manda é o ramo do `boost`. ⇒ o gate afirma o que a
/// medição deu: **monotónica, e o dobro do freio corta MAIS que metade**.
///
/// A tabela (`PlayerConfig::STARTING_POINT`, cruzeiro 6 m/s, `dt = 1/60`):
///
/// | brake | distância | tiques |
/// |------:|----------:|-------:|
/// |  0,25 |    0,8486 |     20 |
/// |  0,50 |    0,3957 |     10 |
/// |  1,00 |    0,1700 |      5 |
/// |  2,00 |    0,0583 |      3 |
/// |  4,00 |    0,0000 |      1 |
///
/// **Mutação que deve sangrar:** ignorar o `cfg.brake_scale` no [`brake_scale`]
/// (toda linha colapsa em 0,1700).
#[test]
fn a_bigger_brake_stops_him_shorter_and_doubling_it_cuts_more_than_half() {
    let half = stopping(0.5).expect("meio freio pára").0;
    let one = stopping(1.0).expect("o mundo de antes pára").0;
    let two = stopping(2.0).expect("o dobro pára").0;

    assert!(
        (one - 0.1700).abs() < 5.0e-3,
        "o mundo de antes desta wave mede 0,1700 m: {one:.4}"
    );
    assert!(
        half > one && one > two,
        "a distancia tem de cair com o freio: {half:.4} > {one:.4} > {two:.4}"
    );
    assert!(
        two < one * 0.5,
        "dobrar o freio corta MAIS que metade (medido 0,343x): {:.4}x",
        two / one
    );
}

/// ⚠️ **A ausência de teto é MEDIDA, não escolhida** — a lei é auto-limitada.
///
/// O ponto de saturação `speed / (turn·accel·dt)` é **função da config**, então
/// não cabe num `MAX_*`: com o perfil de partida ele vale **4,0000**, e a
/// medição pousa exactamente lá (1 tique, distância 0,0000). Acima dele nada mais
/// acontece — e, o que decide a ausência do teto, **ele nunca ultrapassa o
/// alvo**: a sobra que cabe num tique é escrita EXATA.
///
/// **Mutação que deve sangrar:** escrever `push` em vez do `boost` quando a
/// sobra cabe no tique (o personagem passa do zero e recua).
#[test]
fn a_huge_brake_stops_dead_without_ever_overshooting() {
    let cfg = PlayerConfig::STARTING_POINT;
    let saturation = cfg.walk.speed / (1.5 * cfg.walk.acceleration * DT);
    assert!(
        (saturation - 4.0).abs() < 1.0e-4,
        "a saturacao prevista pela lei: {saturation:.4}"
    );

    for b in [4.0_f32, 6.0, 12.0, 1.0e6] {
        let (distance, ticks) = stopping(b).expect("um freio grande pára");
        assert_eq!(ticks, 1, "brake {b} pára no primeiro tique");
        assert!(
            distance.abs() < 1.0e-6,
            "e sem andar mais nada: {distance:.6}"
        );
    }

    // E o overshoot: a velocidade pousa em ZERO, nunca do outro lado.
    let mut cfg = WalkConfig::STARTING_POINT;
    for b in [4.0_f32, 100.0] {
        cfg.brake_scale = b;
        let m = walk(
            &cfg,
            Some(&flat_at(0.5)),
            [cfg.speed, 0.0],
            UP,
            0.0,
            [0.0, 0.0],
            DT,
        );
        let landed = cfg.speed + m.boost[0] + m.accel[0] * DT;
        assert!(
            landed.abs() < 1.0e-5,
            "brake {b} pousa no alvo e nao o atravessa: {landed}"
        );
    }
}

/// **`1` é o mundo de antes desta wave, AO BIT.**
///
/// ⚠️ **É o gate que torna o degrau de schema o ÚNICO preço da wave:** nenhum
/// projeto já autorado reabre a andar diferente, e o `physics_ecs_c9` não se
/// move. A redução é literal (`x * 1.0` é `x` em IEEE-754), então o oráculo pode
/// ser igualdade EXATA em vez de uma tolerância.
///
/// **Mutação que deve sangrar:** trocar o `1.0` do `brake_scale` do
/// [`WalkConfig::STARTING_POINT`] por qualquer outro número.
#[test]
fn a_brake_of_one_is_the_world_before_this_wave_to_the_bit() {
    let mut cfg = WalkConfig::STARTING_POINT;
    assert_eq!(cfg.brake_scale, 1.0, "o neutro e' o default");

    for v in [0.0_f32, 0.5, 2.0, 6.0, -6.0, 11.0] {
        let neutral = walk(&cfg, Some(&flat_at(0.5)), [v, 0.0], UP, 0.0, [0.0, 0.0], DT);
        // A rota SEM o fator: o produto de antes desta wave.
        cfg.brake_scale = 1.0;
        let before = {
            let axis = perp_cw([0.0, 1.0]);
            let rel_along = v;
            let delta = -rel_along;
            let turn = (1.0 + delta.abs() / (2.0 * cfg.speed)).min(WalkConfig::MAX_TURN_BOOST);
            let a = cfg.acceleration * turn;
            if delta.abs() <= a * DT {
                Motor {
                    accel: [0.0, 0.0],
                    boost: [axis[0] * delta, axis[1] * delta],
                }
            } else {
                let push = a * delta.signum();
                Motor {
                    accel: [axis[0] * push, axis[1] * push],
                    boost: [0.0, 0.0],
                }
            }
        };
        assert_eq!(neutral, before, "brake 1 nao pode mover um bit (v = {v})");
    }
}

/// **`0` NÃO freia** — e é um valor legítimo, não *"desligado"*.
///
/// ⚠️ **O CONTROLE está no mesmo gate:** com o eixo APERTADO o personagem ainda
/// acelera normalmente. Sem ele, um `brake_scale` que zerasse a caminhada inteira
/// passaria — e seria um personagem que não anda, não um que não freia.
///
/// **Mutação que deve sangrar:** devolver `1.0` sempre no [`brake_scale`].
#[test]
fn a_brake_of_zero_keeps_the_speed_and_still_lets_him_accelerate() {
    let mut cfg = WalkConfig::STARTING_POINT;
    cfg.brake_scale = 0.0;
    let ground = flat_at(0.5);

    let coasting = walk(
        &cfg,
        Some(&ground),
        [cfg.speed, 0.0],
        UP,
        0.0,
        [0.0, 0.0],
        DT,
    );
    assert_eq!(
        coasting,
        Motor::default(),
        "com o eixo solto e brake 0 nada e' removido: {coasting:?}"
    );

    let driving = walk(&cfg, Some(&ground), [0.0, 0.0], UP, 1.0, [0.0, 0.0], DT);
    assert!(
        driving.accel[0] > 0.0,
        "o CONTROLE: com o dedo no acelerador ele continua a arrancar: {driving:?}"
    );
}

/// **Um freio negativo não ACELERA** — o piso em zero do [`brake_scale`].
///
/// ⚠️ Sem ele `a` fica negativo e `push = a · delta.signum()` empurra para o lado
/// errado: largar o direcional passaria a ganhar velocidade, em silêncio.
///
/// **Mutação que deve sangrar:** tirar o `.max(0.0)` do [`brake_scale`].
#[test]
fn a_negative_brake_never_pushes_him_forward() {
    let mut cfg = WalkConfig::STARTING_POINT;
    let ground = flat_at(0.5);
    for b in [-0.5_f32, -4.0, f32::NEG_INFINITY, f32::NAN] {
        cfg.brake_scale = b;
        let m = walk(
            &cfg,
            Some(&ground),
            [cfg.speed, 0.0],
            UP,
            0.0,
            [0.0, 0.0],
            DT,
        );
        assert!(
            m.accel[0] <= 0.0 && m.boost[0] <= 0.0,
            "brake {b} nao pode empurrar para a frente: {m:?}"
        );
    }
}

/// ⚠️ **O AR é isento** — `air_acceleration` já é a resposta do ar à mesma
/// pergunta, e um segundo número sobre ela seria a falha de duas portas.
///
/// **Mutação que deve sangrar:** tirar o `grounded &&` do [`brake_scale`].
#[test]
fn the_brake_leaves_the_air_alone() {
    let mut cfg = WalkConfig::STARTING_POINT;
    let airborne = |c: &WalkConfig| walk(c, None, [4.0, -2.0], UP, 0.0, [0.0, 0.0], DT);

    cfg.brake_scale = 1.0;
    let neutral = airborne(&cfg);
    assert!(
        neutral.accel[0] < 0.0,
        "o CONTROLE: sem chao o controle aereo ja' freia sozinho: {neutral:?}"
    );
    for b in [0.0_f32, 0.25, 4.0] {
        cfg.brake_scale = b;
        assert_eq!(
            airborne(&cfg),
            neutral,
            "brake {b} nao pode mudar o que o AR faz"
        );
    }
}

/// ⚠️ **O freio é o eixo SOLTO, e não *"a velocidade está a cair"*** — com o dedo
/// no acelerador quem manda é o fator de viragem.
///
/// **Mutação que deve sangrar:** aplicar o freio também com o eixo apertado (a
/// mutação que o plano 10 §2 nomeia: *"estraga a viragem"*).
#[test]
fn a_pressed_axis_is_never_braking_even_while_slowing_down() {
    let mut cfg = WalkConfig::STARTING_POINT;
    let ground = flat_at(0.5);

    // Duas situações em que a velocidade CAI com o dedo apertado.
    let reversing =
        |c: &WalkConfig| walk(c, Some(&ground), [-c.speed, 0.0], UP, 1.0, [0.0, 0.0], DT);
    let contained = |c: &WalkConfig| {
        walk(
            c,
            Some(&ground),
            [c.speed * 3.0, 0.0],
            UP,
            1.0,
            [0.0, 0.0],
            DT,
        )
    };

    cfg.brake_scale = 1.0;
    let (turn_ref, hold_ref) = (reversing(&cfg), contained(&cfg));
    assert!(
        (turn_ref.accel[0] - cfg.acceleration * WalkConfig::MAX_TURN_BOOST).abs() < 1.0e-3,
        "o CONTROLE: virar continua a custar 2x: {turn_ref:?}"
    );
    assert!(
        hold_ref.accel[0] < 0.0,
        "o CONTROLE: acima do cruzeiro ele e' CONTIDO, e isso e' desaceleracao: {hold_ref:?}"
    );

    for b in [0.0_f32, 0.25, 4.0] {
        cfg.brake_scale = b;
        assert_eq!(
            reversing(&cfg),
            turn_ref,
            "brake {b} nao pode tocar a viragem"
        );
        assert_eq!(
            contained(&cfg),
            hold_ref,
            "brake {b} nao pode tocar a contencao"
        );
    }
}
