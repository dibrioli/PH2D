//! Os gates da **SUPERFÍCIE** (`W-Surface`) — e as sondas que os numeraram.
//!
//! Irmão de `walk.rs` pelo teto de LOC, e o corte é por ASSUNTO: o pai fica com
//! *o que a caminhada é*, este com *o que o chão tem a dizer sobre ela*.
//!
//! ⚠️ **Módulo FILHO** (via `#[path]`), como o do freio — é isso que mantém
//! `use super::*` a alcançar o [`surface_grip`] privado.

use super::sim_tests::{DT, UP, drive_for, flat_at};
use super::*;
use crate::PlayerConfig;

/// Chão plano com o `grip` autorado.
fn ice(grip: f32) -> GroundSample {
    GroundSample {
        grip,
        ..flat_at(PlayerConfig::STARTING_POINT.ride.float_height)
    }
}

/// Uma ESTEIRA: chão plano parado cuja SUPERFÍCIE anda a `belt` m/s ao longo da
/// tangente — exactamente o que a ponte monta ao somar a correia na
/// `ground_velocity` (ver o doc daquele campo).
fn belt(belt: f32, grip: f32) -> GroundSample {
    let g = ice(grip);
    let axis = perp_cw(g.normal);
    GroundSample {
        ground_velocity: [axis[0] * belt, axis[1] * belt],
        ..g
    }
}

/// **A SONDA que numerou os gates** — o que o `grip` faz com arrancar e parar.
///
/// `cargo test -p ph2d-platformer measure_the_grip -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_grip() {
    let cfg = PlayerConfig::STARTING_POINT;
    eprintln!("   grip   parar(m)  tiques   arrancar: v@0,5s   dist@0,5s");
    for g in [0.0, 0.1, 0.25, 0.5, 1.0, 2.0, 4.0] {
        let stop = drive_for(&cfg, &ice(g), cfg.walk.speed, 0.0, 600);
        let start = drive_for(&cfg, &ice(g), 0.0, 1.0, 30);
        let (d, t) = match stop.ticks_to_still {
            Some(t) => (
                format!("{:9.4}", stop.travelled_when_still),
                format!("{t:6}"),
            ),
            None => ("  (nunca)".to_string(), " >600".to_string()),
        };
        eprintln!(
            "  {g:5.2}  {d}  {t}          {:6.3}      {:6.3}",
            start.velocity, start.travelled
        );
    }
    // O ponto de saturação previsto pela lei, do mesmo jeito que o do freio: a
    // sobra cabe num tique quando `speed <= turn·accel·dt·grip`, com o fator de
    // viragem em 1,5 (a sobra vale um cruzeiro inteiro).
    let sat = cfg.walk.speed / (1.5 * cfg.walk.acceleration * DT);
    eprintln!("  saturacao prevista: grip = {sat:.4}");
}

/// **A SONDA da ESTEIRA** — quanto a correia leva quem está de pé sem tocar em
/// nada, e o que o `grip` faz com isso.
///
/// `cargo test -p ph2d-platformer measure_the_belt -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_belt() {
    let cfg = PlayerConfig::STARTING_POINT;
    eprintln!("  correia   grip   v@1s     dist@1s");
    for (b, g) in [
        (3.0, 1.0),
        (3.0, 0.25),
        (3.0, 0.0),
        (-3.0, 1.0),
        (12.0, 1.0),
    ] {
        let r = drive_for(&cfg, &belt(b, g), 0.0, 0.0, 60);
        eprintln!(
            "  {b:7.2}  {g:5.2}  {:6.3}   {:7.3}",
            r.velocity, r.travelled
        );
    }
}

/// **O `grip` mexe nos DOIS sentidos, e é isso que faz gelo ser gelo.**
///
/// ⚠️ **É a metade que um gate só de travagem não veria:** uma lei que mexesse
/// apenas no freio daria um personagem que arranca como sempre e escorrega ao
/// parar — patins, não gelo. A tabela mede as duas colunas
/// (`PlayerConfig::STARTING_POINT`, cruzeiro 6 m/s, `dt = 1/60`):
///
/// | grip | parar (m) | tiques | v @ 0,5 s | dist @ 0,5 s |
/// |-----:|----------:|-------:|----------:|-------------:|
/// | 0,00 |  (nunca)  |  >600  |     0,000 |        0,000 |
/// | 0,10 |    2,2096 |     49 |     3,996 |        1,074 |
/// | 0,25 |    0,8486 |     20 |     6,000 |        2,151 |
/// | 1,00 |    0,1700 |      5 |     6,000 |        2,830 |
/// | 4,00 |    0,0000 |      1 |     6,000 |        3,000 |
///
/// ⚠️ **E a coluna do arranque é MODESTA de propósito, não fraca:** com uma
/// aceleração de 60 m/s² até um quarto dela chega ao cruzeiro dentro de meio
/// segundo, e o que sobra é a distância. Quem quiser gelo VISÍVEL baixa a
/// aceleração junto — foi o que a cena da wave mediu.
///
/// **Mutação que deve sangrar:** ignorar o `s.grip` no [`surface_grip`] (as duas
/// colunas colapsam nos números do `1,00`).
#[test]
fn less_grip_is_both_slower_to_start_and_longer_to_stop() {
    let cfg = PlayerConfig::STARTING_POINT;
    let slick = drive_for(&cfg, &ice(0.25), cfg.walk.speed, 0.0, 600);
    let normal = drive_for(&cfg, &ice(1.0), cfg.walk.speed, 0.0, 600);

    let slick_stop = slick.travelled_when_still;
    let normal_stop = normal.travelled_when_still;
    assert!(
        (normal_stop - 0.1700).abs() < 5.0e-3,
        "o mundo de antes desta wave pára em 0,1700 m: {normal_stop:.4}"
    );
    assert!(
        slick_stop > normal_stop * 3.0,
        "com um quarto do grip ele derrapa MUITO mais: {slick_stop:.4} vs {normal_stop:.4}"
    );

    let slick_start = drive_for(&cfg, &ice(0.25), 0.0, 1.0, 30).travelled;
    let normal_start = drive_for(&cfg, &ice(1.0), 0.0, 1.0, 30).travelled;
    assert!(
        slick_start < normal_start,
        "e ARRANCA mais devagar, que é a metade que um gate de travagem nao ve: \
         {slick_start:.4} vs {normal_start:.4}"
    );
}

/// **`grip = 1` é o mundo de antes desta wave, AO BIT** — e o `grip` do AR é
/// esse mesmo neutro.
///
/// ⚠️ **É ele que torna a wave inteira gratuita:** nenhuma cena já autorada anda
/// diferente, e o `physics_ecs_c9` não se move. A redução é literal
/// (`x * 1.0` é `x` em IEEE-754), então o oráculo é igualdade EXATA.
///
/// ⚠️ **A segunda metade — o AR — não é redundante:** o `grip` do ar não vem de
/// um `if` mas do braço `None` do `match`, e é exactamente esse braço que uma
/// mutação distraída faz ler a superfície de que o personagem acabou de sair.
///
/// **Mutação que deve sangrar:** dar ao braço `None` do `match` um `grip`
/// diferente de `NEUTRAL_GRIP`; ou multiplicar por `s.grip` fora do braço.
#[test]
fn a_grip_of_one_is_the_world_before_this_wave_and_the_air_is_that_neutral() {
    let cfg = WalkConfig::STARTING_POINT;
    let ground = flat_at(0.5);
    assert_eq!(ground.grip, 1.0, "a fixture de partida é o neutro");

    for v in [0.0_f32, 0.5, 2.0, 6.0, -6.0, 11.0] {
        for drive in [-1.0_f32, 0.0, 1.0] {
            // A rota SEM o fator: o produto de antes desta wave, re-derivado.
            let before = {
                let axis = perp_cw(ground.normal);
                let delta = drive * cfg.speed - v;
                let turn = (1.0 + delta.abs() / (2.0 * cfg.speed)).min(WalkConfig::MAX_TURN_BOOST);
                let a = cfg.acceleration * turn * super::brake_scale(&cfg, true, drive);
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
            let now = walk(&cfg, Some(&ground), [v, 0.0], UP, drive, [0.0, 0.0], DT);
            assert_eq!(now, before, "v={v} drive={drive}");
        }
    }

    // ⚠️ **O AR gasta o orçamento DELE, inteiro.** O braço `None` do `match` é o
    // único lugar onde o `grip` do ar é escolhido, e a única coisa observável
    // dele é o tamanho da força: se alguém lá escrever outro número, esta conta
    // deixa de bater. Um laço sobre `grip`s não diria nada — no ar não há
    // amostra que os carregue, então as duas chamadas seriam a MESMA e o gate
    // não poderia falhar.
    let v = 1.0_f32;
    let airborne = walk(&cfg, None, [v, 0.0], UP, 1.0, [0.0, 0.0], DT);
    let delta = cfg.speed - v;
    let turn = (1.0 + delta.abs() / (2.0 * cfg.speed)).min(WalkConfig::MAX_TURN_BOOST);
    let want = cfg.air_acceleration * turn;
    assert!(
        (airborne.accel[0] - want).abs() < 1.0e-4,
        "o ar gasta o orçamento dele inteiro ({want:.4}): {:?}",
        airborne.accel
    );
}

/// **`grip = 0` é gelo PERFEITO — não arranca e não pára — e não é um caso
/// especial no código.**
///
/// ⚠️ **A ausência do `if` é o ponto:** o orçamento zera e a aritmética entrega
/// força nula sozinha. Um `if grip == 0 { return }` seria um segundo lugar onde a
/// lei diz o que já diz, e o primeiro a envelhecer no dia em que um outro termo
/// entrar no produto.
///
/// **Mutação que deve sangrar:** um piso positivo no [`surface_grip`] (o
/// personagem passa a arrancar e a parar sobre gelo perfeito).
#[test]
fn perfect_ice_neither_starts_him_nor_stops_him() {
    let cfg = PlayerConfig::STARTING_POINT;

    let coasting = drive_for(&cfg, &ice(0.0), cfg.walk.speed, 0.0, 600);
    assert!(
        coasting.ticks_to_still.is_none(),
        "gelo perfeito nao pára: parou no tique {:?}",
        coasting.ticks_to_still
    );
    assert!(
        (coasting.velocity - cfg.walk.speed).abs() < 1.0e-4,
        "ele conserva a velocidade que tinha: {:.4}",
        coasting.velocity
    );

    let pushing = drive_for(&cfg, &ice(0.0), 0.0, 1.0, 600);
    assert!(
        pushing.travelled.abs() < 1.0e-6 && pushing.velocity.abs() < 1.0e-6,
        "e nao arranca: v={:.6} d={:.6}",
        pushing.velocity,
        pushing.travelled
    );
}

/// **O `grip` e o `brake_scale` são UM orçamento, não dois mecanismos.**
///
/// ⚠️ **É o gate que prova a dependência que o plano declarou** (a `W-Surface`
/// vem DEPOIS da `W-Brake`): os dois multiplicam o mesmo `a`, então metade de um
/// com metade do outro dá exactamente um quarto — e é essa composição que deixa o
/// artista escolher *quanto* do escorregadio é "não consigo parar" contra "não
/// consigo arrancar".
///
/// **Mutação que deve sangrar:** aplicar o `grip` só ao ramo do `boost`, ou
/// somá-lo em vez de multiplicar.
#[test]
fn grip_and_brake_multiply_into_one_budget() {
    let quarter_by_grip = {
        let mut cfg = PlayerConfig::STARTING_POINT;
        cfg.walk.brake_scale = 1.0;
        drive_for(&cfg, &ice(0.25), cfg.walk.speed, 0.0, 600)
    };
    let quarter_by_both = {
        let mut cfg = PlayerConfig::STARTING_POINT;
        cfg.walk.brake_scale = 0.5;
        drive_for(&cfg, &ice(0.5), cfg.walk.speed, 0.0, 600)
    };
    assert_eq!(
        quarter_by_grip.ticks_to_still, quarter_by_both.ticks_to_still,
        "meio grip com meio freio é um quarto de orçamento, como um quarto de grip"
    );
    assert!(
        (quarter_by_grip.travelled_when_still - quarter_by_both.travelled_when_still).abs()
            < 1.0e-6,
        "e a distancia é a MESMA: {:.6} vs {:.6}",
        quarter_by_grip.travelled_when_still,
        quarter_by_both.travelled_when_still
    );
}

/// **A ESTEIRA leva por TRAÇÃO — e uma correia sem `grip` não leva nada.**
///
/// ⚠️ **Esta é a propriedade EMERGENTE da wave, e ela cai da composição:** a
/// correia chega como `ground_velocity`, a lei mede tudo relativo ao chão, e o
/// que fecha a distância entre o corpo e o chão é o orçamento — que o `grip`
/// multiplica. Com `grip = 0` a correia **não tem por onde puxar**, que é
/// exactamente o que uma esteira sem atrito faz no mundo.
///
/// ⚠️ **E a correia decide a velocidade de repouso, acima ou abaixo do
/// cruzeiro:** ficar parado *em relação à correia* é andar à velocidade dela no
/// mundo. Medido a 12 m/s sobre um cruzeiro de 6: o personagem assenta em 12.
///
/// | correia | grip | v @ 1 s | dist @ 1 s |
/// |--------:|-----:|--------:|-----------:|
/// |    3,00 | 1,00 |   3,000 |      2,961 |
/// |    3,00 | 0,25 |   3,000 |      2,769 |
/// |    3,00 | 0,00 |   0,000 |      0,000 |
/// |   −3,00 | 1,00 |  −3,000 |     −2,961 |
/// |   12,00 | 1,00 |  12,000 |     11,393 |
///
/// **Mutação que deve sangrar:** ignorar o `s.grip` (a linha do `0,00` passa a
/// levar o personagem).
#[test]
fn a_belt_carries_by_traction_so_a_frictionless_one_carries_nothing() {
    let cfg = PlayerConfig::STARTING_POINT;

    let carried = drive_for(&cfg, &belt(3.0, 1.0), 0.0, 0.0, 60);
    assert!(
        (carried.velocity - 3.0).abs() < 1.0e-3,
        "a correia leva quem esta de pé até a velocidade dela: {:.4}",
        carried.velocity
    );
    assert!(
        carried.travelled > 2.5,
        "e ele ANDA: {:.4}",
        carried.travelled
    );

    let slick = drive_for(&cfg, &belt(3.0, 0.0), 0.0, 0.0, 60);
    assert!(
        slick.velocity.abs() < 1.0e-6 && slick.travelled.abs() < 1.0e-6,
        "uma correia sem grip nao tem por onde puxar: v={:.6} d={:.6}",
        slick.velocity,
        slick.travelled
    );

    let backwards = drive_for(&cfg, &belt(-3.0, 1.0), 0.0, 0.0, 60);
    assert!(
        (backwards.velocity + 3.0).abs() < 1.0e-3,
        "o sinal é o sentido, ao longo da tangente: {:.4}",
        backwards.velocity
    );

    let fast = drive_for(&cfg, &belt(12.0, 1.0), 0.0, 0.0, 60);
    assert!(
        (fast.velocity - 12.0).abs() < 1.0e-3,
        "e uma correia rápida leva ACIMA do cruzeiro de {:.1}: {:.4}",
        cfg.walk.speed,
        fast.velocity
    );
}

/// **A correia empurra ao longo da SUPERFÍCIE, não de um eixo de mundo.**
///
/// ⚠️ **É a razão de a correia ser um ESCALAR e não um `Vec2`:** um vetor
/// autorado em eixos de mundo teria componente ao longo da NORMAL numa rampa — e
/// uma superfície não empurra ninguém para dentro nem para fora de si mesma;
/// quem faz isso é a perna. Como escalar sobre a tangente, o caso degenerado é
/// **inexprimível**, e uma esteira em rampa sobe sozinha.
///
/// **Mutação que deve sangrar:** montar a correia em `[belt, 0]` (mundo) em vez
/// de `belt · perp_cw(normal)` — numa rampa de 30° o personagem passa a ser
/// empurrado contra o chão em vez de subir por ele.
#[test]
fn a_belt_on_a_ramp_runs_along_the_ramp() {
    let cfg = PlayerConfig::STARTING_POINT;
    // Uma rampa de 30° subindo para a direita.
    let (s, c) = (0.5_f32, 0.75_f32.sqrt());
    let ramp = GroundSample {
        normal: [-s, c],
        ..ice(1.0)
    };
    let axis = perp_cw(ramp.normal);
    let uphill = GroundSample {
        ground_velocity: [axis[0] * 3.0, axis[1] * 3.0],
        ..ramp
    };
    // A correia autorada tem de ser PARALELA à rampa: zero ao longo da normal.
    let along_normal =
        uphill.ground_velocity[0] * ramp.normal[0] + uphill.ground_velocity[1] * ramp.normal[1];
    assert!(
        along_normal.abs() < 1.0e-6,
        "uma correia nao empurra para dentro do chao: {along_normal:.6}"
    );
    // E o motor a persegue ao longo da rampa, subindo.
    let m = walk(
        &cfg.walk,
        Some(&uphill),
        [0.0, 0.0],
        UP,
        0.0,
        [0.0, 0.0],
        DT,
    );
    assert!(
        m.accel[0] > 0.0 && m.accel[1] > 0.0,
        "a esteira em rampa SOBE: {:?}",
        m.accel
    );
}

/// ⚠️ **A ausência de teto é MEDIDA, e o piso em zero é uma GARANTIA** — os dois
/// irmãos exactos dos do freio.
///
/// A saturação `speed / (turn·accel·dt)` é **função da config** (4,0000 no perfil
/// de partida) e não cabe num `MAX_*`; e um `grip` negativo ou `NaN` nunca pode
/// empurrar para o lado errado.
///
/// **Mutação que deve sangrar:** tirar o `.max(0.0)` (o negativo inverte o
/// empurrão) ou o ramo do `is_finite` (o `NaN` envenena a força).
#[test]
fn a_huge_grip_saturates_and_a_broken_one_never_reverses() {
    let cfg = PlayerConfig::STARTING_POINT;
    let saturation = cfg.walk.speed / (1.5 * cfg.walk.acceleration * DT);
    assert!(
        (saturation - 4.0).abs() < 1.0e-4,
        "a saturacao prevista pela lei: {saturation:.4}"
    );
    for g in [4.0_f32, 12.0, 1.0e6] {
        let run = drive_for(&cfg, &ice(g), cfg.walk.speed, 0.0, 600);
        assert_eq!(
            run.ticks_to_still,
            Some(1),
            "grip {g} pára no primeiro tique"
        );
        assert!(
            run.travelled_when_still.abs() < 1.0e-6,
            "e sem andar mais nada: {:.6}",
            run.travelled_when_still
        );
    }

    // O piso: um `grip` negativo nunca empurra para o lado errado.
    for g in [-1.0_f32, -1.0e6] {
        let m = walk(
            &cfg.walk,
            Some(&ice(g)),
            [0.0, 0.0],
            UP,
            1.0,
            [0.0, 0.0],
            DT,
        );
        assert!(
            m.accel[0] >= 0.0 && m.boost[0] >= 0.0,
            "grip {g} nunca empurra contra o dedo: {m:?}"
        );
    }

    // ⚠️ **E um `NaN` cai no NEUTRO, não em gelo perfeito** — a metade que a
    // primeira versão deste gate NÃO via, e que a mutação encontrou.
    //
    // `s.grip.max(0.0)` sozinho já não envenena a força — em Rust
    // `NaN.max(0.0)` devolve `0.0`, o operando não-NaN —, então "nunca reverte,
    // nunca vira NaN" era satisfeito com a guarda REMOVIDA. Só que `0.0`
    // significa **gelo perfeito**: um número corrompido no arquivo deixaria o
    // personagem sem conseguir andar, em silêncio. A escolha é que um valor
    // quebrado não é uma intenção autorada ⇒ a superfície *não diz nada*.
    //
    // ⚠️ **A conta subtil é justamente essa**, e é por isso que a guarda fica
    // explícita: quem a "simplificar" para `if g < 0.0 { 0.0 } else { g }`
    // propaga o `NaN` inteiro, e quem a apagar troca neutro por gelo.
    let broken = walk(
        &cfg.walk,
        Some(&ice(f32::NAN)),
        [0.0, 0.0],
        UP,
        1.0,
        [0.0, 0.0],
        DT,
    );
    let neutral = walk(
        &cfg.walk,
        Some(&ice(GroundSample::NEUTRAL_GRIP)),
        [0.0, 0.0],
        UP,
        1.0,
        [0.0, 0.0],
        DT,
    );
    assert_eq!(
        broken, neutral,
        "um grip NaN e' lido como superficie que nao diz nada, nao como gelo"
    );
}
