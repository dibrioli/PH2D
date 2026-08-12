//! **O NADO na porta única** (W-Swim) — irmão de `lib_tests.rs` pelo teto de
//! 700 LOC, e o mesmo corte dos dois que já estão ao lado: o pai fica com *o que
//! a porta É*, este filho com *o que o regime faz a ela*.
//!
//! ⚠️ Módulo FILHO por `#[path]`, e um glob basta — privado quer dizer *visível
//! aos descendentes*, então o `super::*` traz as fixtures do pai (`at`, `UP`,
//! `G`, `DT`) e os tipos que ele já importou.

use super::*;

/// A capsula flutuante — o modo em que a perna existe, que é o que estes gates
/// precisam de ver calar.
const SPRING: Support = Support::Spring;

/// Ar seco — o CONTROLE de todos estes gates.
const DRY: Buoyed = Buoyed::DRY;

/// Fundo o bastante para armar com o limiar default (`1.0` = a linha de
/// flutuação; a tabela está em `measure_the_swim_threshold`).
const DEEP: Buoyed = Buoyed(2.0);

/// Uma config com o NADO ligado — a capacidade nasce desligada.
fn swimming_cfg() -> PlayerConfig {
    PlayerConfig {
        swim: SwimConfig {
            speed: 4.0,
            acceleration: 12.0,
            ..SwimConfig::STARTING_POINT
        },
        ..PlayerConfig::STARTING_POINT
    }
}

/// Um passo da porta única, com tudo o que estes gates variam.
fn step(
    cfg: &PlayerConfig,
    ground: Option<&GroundSample>,
    input: PlayerInput,
    state: PlayerState,
    velocity: Vec2,
    buoyed: Buoyed,
) -> PlayerStep {
    player_motor(
        cfg, ground, None, None, None, None, input, state, velocity, G, UP, DT, buoyed, SPRING,
    )
}

/// **A capacidade desligada é o mundo de antes desta wave, AO BIT** — a rede de
/// segurança de toda a wave, e o gate que a mutação de qualquer termo do regime
/// tem de deixar em paz.
#[test]
fn the_world_without_the_capability_is_untouched() {
    let off = PlayerConfig::STARTING_POINT;
    assert!(!off.swim.armed(), "a capacidade nasce desligada");
    let air = step(
        &off,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, -3.0],
        DEEP,
    );
    // A mesma chamada com a água a zero: a água só muda o que o `waterborne`
    // sempre mudou (a modelagem do arco), nunca o nado.
    let dry = step(
        &off,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, -3.0],
        DRY,
    );
    assert!(!air.state.swim.active, "desligado nunca nada");
    assert_eq!(
        air.motor.boost, dry.motor.boost,
        "sem a capacidade, o motor e' o de sempre"
    );
}

/// ⚠️ **AS TRÊS COISAS QUE O REGIME CALA, num gate só** — a perna, a caminhada e
/// a parede. Elas viajam juntas porque são **uma frase** (o argumento do
/// arranque): as três se apoiam num chão, e a lei só arma o nado onde não há
/// nenhum.
///
/// O oráculo é o que resta do motor: com o nado ligado e o dedo parado, o único
/// termo vivo é o freio da braçada — que numa velocidade zero é **zero**.
///
/// **Mutação que deve sangrar:** tirar o `|| swimming` do termo da caminhada (o
/// controle aéreo volta e escreve o eixo horizontal).
#[test]
fn the_regime_silences_the_walk_that_would_otherwise_be_air_control() {
    let cfg = swimming_cfg();
    let driving = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    // Parado dentro d'água, a empurrar para a direita.
    let wet = step(
        &cfg,
        None,
        driving,
        PlayerState::default(),
        [0.0, 0.0],
        DEEP,
    );
    assert!(wet.state.swim.active, "tem de estar a nadar");
    // A braçada empurra com o orçamento DELA, não com a soma dos dois servos.
    let expected = cfg.swim.acceleration;
    assert!(
        (wet.motor.accel[0] - expected).abs() < 1.0e-4,
        "o eixo horizontal e' so' a bracada ({expected}): {:?}",
        wet.motor.accel
    );
}

/// ⚠️ **A gravidade NÃO é cancelada** — o que segura um nadador é o empuxo, e o
/// canal do `gravity_hold` existe para a perna, que aqui está calada.
///
/// **Mutação que deve sangrar:** fazer o regime declarar `-gravity` no
/// `gravity_hold` (a água seria paga duas vezes: uma pelo empuxo do solver,
/// outra por este cancelamento).
#[test]
fn the_swim_does_not_cancel_gravity() {
    let cfg = swimming_cfg();
    let wet = step(
        &cfg,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, 0.0],
        DEEP,
    );
    assert!(wet.state.swim.active);
    assert_eq!(
        wet.gravity_hold,
        [0.0, 0.0],
        "quem segura o nadador e' o empuxo, nao um cancelamento"
    );
    assert!(
        wet.reaction.is_none(),
        "sem chao nao ha' em quem empurrar: {:?}",
        wet.reaction
    );
}

/// **O botão de pulo vira BRAÇADA** — e o oráculo é o que ele deixa de fazer.
///
/// Com o coyote cheio (acabou de sair do chão) e o botão apertado, o mundo seco
/// PULA; o mesmo estado dentro d'água não pula, e o que aparece no lugar é a
/// braçada para CIMA.
///
/// **Mutação que deve sangrar:** passar `input.jump` cru ao `jump_step`.
#[test]
fn the_jump_button_becomes_a_stroke() {
    let cfg = swimming_cfg();
    let pressing = PlayerInput {
        jump: true,
        ..PlayerInput::default()
    };
    // Coyote cheio e SEM chão: é o estado de quem acabou de sair de uma borda,
    // e o único em que um aperto ainda pula fora do chão.
    //
    // ⚠️ `airborne` fica FALSO de propósito — ele significa *"eu pulei"*, não
    // *"estou no ar"*, e a decolagem pergunta por ele (`!next.airborne`).
    let coyote = PlayerState {
        jump: JumpState {
            coyote: cfg.jump.coyote_time,
            ..JumpState::default()
        },
        ..PlayerState::default()
    };

    let dry = step(&cfg, None, pressing, coyote, [0.0, 0.0], DRY);
    assert!(
        dry.motor.boost[1] > 1.0,
        "seco, com coyote, o aperto PULA: {:?}",
        dry.motor
    );

    let wet = step(&cfg, None, pressing, coyote, [0.0, 0.0], DEEP);
    assert!(wet.state.swim.active);
    assert!(
        wet.state.jump.coyote > 0.0,
        "o coyote nao pode ser GASTO por um pulo que nao houve: {:?}",
        wet.state.jump
    );
    // E o que sobe é a braçada, com o orçamento dela — não um impulso de pulo.
    assert!(
        (wet.motor.accel[1] - cfg.swim.acceleration).abs() < 1.0e-4,
        "a subida e' a bracada: {:?}",
        wet.motor
    );
    assert_eq!(
        wet.motor.boost[1], 0.0,
        "e nao um impulso de decolagem: {:?}",
        wet.motor
    );
}

/// **O ARRANQUE vence o nado** — *durante o arranque o personagem é uma
/// velocidade*, e somar os dois daria dois donos do mesmo eixo.
///
/// **Mutação que deve sangrar:** tirar o `&& !dashing` do termo da braçada.
#[test]
fn a_dash_wins_over_the_stroke() {
    let cfg = PlayerConfig {
        dash: DashConfig {
            speed: 18.0,
            ..DashConfig::STARTING_POINT
        },
        ..swimming_cfg()
    };
    // Um arranque EM CURSO, dentro d'água.
    let mid_dash = PlayerState {
        dash: DashState {
            left: 0.1,
            dir: 1.0,
            facing: 1.0,
            ..DashState::default()
        },
        ..PlayerState::default()
    };
    let driving = PlayerInput {
        drive: -1.0,
        ..PlayerInput::default()
    };
    let wet = step(&cfg, None, driving, mid_dash, [0.0, 0.0], DEEP);
    assert!(wet.state.swim.active, "a trava do nado continua armada");
    assert!(
        wet.motor.boost[0] > 0.0,
        "o arranque leva a velocidade AO alvo dele, para a direita: {:?}",
        wet.motor
    );
    // A braçada pedia o oposto (`drive = -1`) e não escreveu nada.
    assert_eq!(
        wet.motor.accel[0], 0.0,
        "a bracada nao pode disputar o eixo com o arranque: {:?}",
        wet.motor
    );
}

/// ⚠️ **A PAREDE cala dentro d'água** — agarrar-se é apoiar-se, e a lei só arma
/// o nado onde não há em que se apoiar.
///
/// Sem o silêncio, o escorregamento de parede escreveria o eixo vertical ao
/// lado da braçada: dois donos do mesmo número, e o nadador desceria a parede
/// submersa enquanto pede para subir.
///
/// **Mutação que deve sangrar:** tirar o `if swimming { None }` do `clinging`.
#[test]
fn the_wall_is_silent_under_water() {
    let cfg = PlayerConfig {
        wall: WallConfig {
            slide_speed: 3.0,
            reach: 0.1,
            ..WallConfig::STARTING_POINT
        },
        ..swimming_cfg()
    };
    // Uma parede à direita, encostada, com o dedo a empurrar contra ela.
    let wall = WallProbe::from_hits(
        1.0,
        &[
            Some(WallHit {
                distance: 0.05,
                normal: [-1.0, 0.0],
            }),
            None,
            None,
        ],
    );
    let pushing = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    let call = |buoyed| {
        player_motor(
            &cfg,
            None,
            None,
            Some(&wall),
            None,
            None,
            pushing,
            PlayerState::default(),
            // A cair: é a condição em que o escorregamento age.
            [0.0, -6.0],
            G,
            UP,
            DT,
            buoyed,
            SPRING,
        )
    };
    // CONTROLE — em ar seco a parede pega, e o escorregamento escreve o eixo.
    let dry = call(DRY);
    assert!(
        dry.motor.boost[1] > 0.0,
        "seco, a parede tem de FREAR a queda: {:?}",
        dry.motor
    );
    // Dentro d'água: o eixo vertical é só da braçada, que aqui trava a queda
    // por FORÇA (`accel`) e nunca por um boost de parede.
    let wet = call(DEEP);
    assert!(wet.state.swim.active);
    assert_eq!(
        wet.motor.boost[1], 0.0,
        "a parede nao pode escrever o eixo da bracada: {:?}",
        wet.motor
    );
    assert!(
        (wet.motor.accel[1] - cfg.swim.acceleration).abs() < 1.0e-4,
        "e o que sobe e' a bracada a travar a queda: {:?}",
        wet.motor
    );
}

/// ⚠️ **Vadear é ANDAR** — com o fundo ao alcance dos pés a perna continua a
/// segurar e a caminhada continua a valer, por mais funda que a água esteja.
///
/// É este gate que sustenta a decisão de **não** escrever `&& !swimming` na
/// `standing` do `player_motor`: a trava já garante a frase.
#[test]
fn a_wader_still_walks() {
    let cfg = swimming_cfg();
    let ground = at(cfg.ride.float_height, UP);
    let driving = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    let wading = step(
        &cfg,
        Some(&ground),
        driving,
        PlayerState::default(),
        [0.0, 0.0],
        DEEP,
    );
    assert!(!wading.state.swim.active, "com chao, anda-se");
    assert!(
        wading.reaction.is_some(),
        "e o peso continua a voltar para o chao"
    );
    // ⚠️ **O oráculo é a MESMA cena em ar seco**, e não o `walk.acceleration`
    // cru: a caminhada tem um bônus de arranque (`MAX_TURN_BOOST`), então um
    // número escrito à mão aqui pinaria o bônus em vez da frase — e mudaria de
    // sentido no dia em que alguém o afinasse.
    let dry_walk = step(
        &cfg,
        Some(&ground),
        driving,
        PlayerState::default(),
        [0.0, 0.0],
        DRY,
    );
    assert_eq!(
        wading.motor, dry_walk.motor,
        "vadear tem de dar o MESMO motor que andar em seco"
    );
}
