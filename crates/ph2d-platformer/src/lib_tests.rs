//! Os gates da porta única [`crate::player_motor`].
//!
//! Irmão de `lib.rs` pelo teto de LOC, e o corte é o mesmo que o
//! `jump_tests.rs` ao lado já faz: o pai fica com **o que a lei É** (os
//! tipos, a composição, a porta), o filho com **o que ela responde**.
//!
//! ⚠️ Módulo FILHO (via `#[path]`), não um `tests/` de integração — é isso
//! que mantém `use super::*` a alcançar o que não é `pub`.

use super::*;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
const DT: f32 = 1.0 / 60.0;

fn at(distance: f32, normal: Vec2) -> GroundSample {
    GroundSample {
        distance,
        normal,
        ground_velocity: [0.0, 0.0],
        one_way: false,
    }
}

/// ⚠️ **O canal que cancela a gravidade é DECLARADO, e o que sobra é o
/// resto do motor AO BIT** (W11).
///
/// As duas metades num gate só porque a ponte depende das duas: ela subtrai
/// o `gravity_hold` do `accel` e paga cada um por um caminho diferente. Se a
/// declaração não fosse exactamente `− gravity`, a subtração deixaria um
/// resíduo no caminho agrupado e o cancelamento sairia errado nos dois.
///
/// **Mutação que deve sangrar:** devolver `[0, 0]` no `gravity_hold` com a
/// mola armada (o defeito volta inteiro, e o `measure_substep` mede-o).
#[test]
fn the_leg_declares_which_half_of_its_push_cancels_gravity() {
    let cfg = PlayerConfig::STARTING_POINT;
    let ground = at(cfg.ride.float_height, UP);
    let step = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    assert_eq!(
        step.gravity_hold,
        [-G[0], -G[1]],
        "com a perna a segurar, o canal declarado E' o peso cancelado"
    );
    // E o que sobra para o caminho agrupado é a MOLA, que na altura de
    // repouso é zero — a prova de que a subtração da ponte é exacta.
    let lumped = [
        step.motor.accel[0] - step.gravity_hold[0],
        step.motor.accel[1] - step.gravity_hold[1],
    ];
    assert!(
        lumped[0].abs() < 1.0e-5 && lumped[1].abs() < 1.0e-5,
        "na altura pedida o resto do motor e' zero: {lumped:?}"
    );
}

/// **No AR o canal é zero**, e a metade que importa é a do tique da
/// DECOLAGEM: ali o raio ainda vê o chão e a mola já está calada.
///
/// ⚠️ É por isso que a lei declara isto e a ponte não o deduz de *"há
/// amostra de chão?"* — as duas perguntas divergem exactamente num tique, e
/// é o tique em que o personagem sai do chão.
#[test]
fn nothing_is_held_in_the_air() {
    let cfg = PlayerConfig::STARTING_POINT;
    let air = player_motor(
        &cfg,
        None,
        None,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    assert_eq!(
        air.gravity_hold,
        [0.0, 0.0],
        "sem chao nao ha' peso a cancelar"
    );

    // A DECOLAGEM: o raio ainda vê o chão, o botão está premido, e a mola é
    // desarmada pelo pulo — o canal tem de acompanhar a mola, não o raio.
    let ground = at(cfg.ride.float_height, UP);
    let takeoff = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        PlayerInput {
            jump: true,
            ..PlayerInput::default()
        },
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    assert_eq!(
        takeoff.gravity_hold,
        [0.0, 0.0],
        "no tique da decolagem a mola esta' calada, logo nao ha' peso a cancelar"
    );
}

/// Uma rampa RASA é chão; uma parede não é. O limite é o autorado.
#[test]
fn a_wall_is_not_ground() {
    let cfg = PlayerConfig::STARTING_POINT; // 45°
    let shallow = at(0.5, [-0.5, 0.866_025_4]); // 30°
    let steep = at(0.5, [-0.866_025_4, 0.5]); // 60°
    assert!(is_grounded(&cfg, Some(&shallow), UP), "30° e' chao");
    assert!(!is_grounded(&cfg, Some(&steep), UP), "60° nao e' chao");
    assert!(!is_grounded(&cfg, None, UP));
}

/// ⚠️ **A recusa da rampa alcança a MOLA, não só a caminhada.**
///
/// É o gate da porta única: com a `footing` respondendo só à caminhada, a
/// mola seguraria o personagem colado a uma parede — parado no ar, porque
/// ela cancela a gravidade enquanto segura.
#[test]
fn the_spring_lets_go_of_a_wall() {
    let cfg = PlayerConfig::STARTING_POINT;
    let steep = at(cfg.ride.float_height, [-0.866_025_4, 0.5]); // 60°
    let at_wall = player_motor(
        &cfg,
        Some(&steep),
        None,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    // ⚠️ **O oráculo é a INDISTINGUIBILIDADE, não um zero.** A primeira
    // versão deste gate afirmava `Motor::default()` — ele QUERIA dizer *"a
    // mola se cala"* e DIZIA *"o motor inteiro é zero"*, e as duas frases só
    // coincidiam enquanto não havia pulo. Com a W4 o termo de gravidade por
    // fase existe no ar (e num |v| ≈ 0 ele é o do ápice), então o zero
    // passou a ser falso sobre um produto correto. O que a lei afirma é que
    // estar ao lado de uma parede íngreme é o MESMO que estar no ar.
    let in_the_air = player_motor(
        &cfg,
        None,
        None,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    assert_eq!(
        at_wall, in_the_air,
        "numa parede o motor tem de ser o do AR, termo a termo: {at_wall:?}"
    );
}

/// A mesma rampa vira chão quando o artista sobe o limite — o número é dele.
#[test]
fn raising_the_limit_makes_the_ramp_walkable() {
    let mut cfg = PlayerConfig::STARTING_POINT;
    let steep = at(cfg.ride.float_height, [-0.866_025_4, 0.5]); // 60°
    assert!(!is_grounded(&cfg, Some(&steep), UP));
    cfg.walk.max_slope_deg = 70.0;
    assert!(is_grounded(&cfg, Some(&steep), UP));
}

/// Normal degenerada (raio nascido dentro da geometria) conta como chão
/// plano — a suposição que deixa a mola empurrar o personagem para fora.
#[test]
fn a_degenerate_normal_counts_as_flat() {
    let cfg = PlayerConfig::STARTING_POINT;
    let inside = at(0.0, [0.0, 0.0]);
    assert!(is_grounded(&cfg, Some(&inside), UP));
    let m = player_motor(
        &cfg,
        Some(&inside),
        None,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    assert!(
        m.motor.accel[1] > 0.0,
        "a mola tem de empurrar para FORA: {:?}",
        m.motor.accel
    );
}

/// A porta única SOMA as duas leis, sem uma comer a outra.
#[test]
fn the_door_sums_both_laws() {
    let cfg = PlayerConfig::STARTING_POINT;
    let ground = at(cfg.ride.float_height, [0.0, 1.0]);
    let input = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    let whole = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        input,
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    let spring = ride_spring(&cfg.ride, Some(&ground), [0.0, 0.0], G, UP);
    let step = walk(
        &cfg.walk,
        Some(&ground),
        [0.0, 0.0],
        UP,
        1.0,
        [0.0, 0.0],
        DT,
    );
    assert_eq!(whole.motor, spring.plus(step));
    assert!(whole.motor.accel[0] > 0.0, "anda");
    assert!(whole.motor.accel[1] > 0.0, "e paira ao mesmo tempo");
}

// ── W9: O NÚMERO QUE O ARTISTA ESCREVE ───────────────────────────────────

/// **O sensor tem TRÊS respostas, e as duas que colapsavam pedem coisas
/// opostas da caminhada.**
///
/// **Mutação que deve sangrar:** fazer `footing_verdict` devolver
/// `Footing::Airborne` no braço da inclinação.
#[test]
fn the_verdict_separates_the_air_from_a_steep_slope() {
    let cfg = PlayerConfig::STARTING_POINT; // 45°
    let steep = at(cfg.ride.float_height, [-0.866_025_4, 0.5]); // 60°
    let shallow = at(cfg.ride.float_height, [-0.5, 0.866_025_4]); // 30°
    let far = at(
        cfg.ride.float_height + cfg.ride.cling_distance + 1.0,
        [0.0, 1.0],
    );

    assert_eq!(footing_verdict(&cfg, None, UP), Footing::Airborne);
    assert_eq!(footing_verdict(&cfg, Some(&far), UP), Footing::Airborne);
    assert_eq!(
        footing_verdict(&cfg, Some(&steep), UP),
        Footing::Steep(&steep),
        "uma rampa ao alcance e ingreme demais NAO e' o ar: ha' em que se \
         apoiar, e e' isso que a fazia ser escalada"
    );
    assert_eq!(
        footing_verdict(&cfg, Some(&shallow), UP),
        Footing::Ground(&shallow)
    );
    // As duas VISTAS saem da MESMA classificação — nunca de dois testes.
    assert!(footing(&cfg, Some(&steep), UP).is_none());
    assert!(
        footing_verdict(&cfg, Some(&steep), UP).steep().is_some(),
        "o que a perna recusa e' o que o empurrao tem de respeitar"
    );
}

/// **Morro acima some; morro abaixo passa inteiro.**
///
/// **Mutação que deve sangrar:** trocar o `> 0.0` do `kill` por `< 0.0` —
/// o personagem passaria a não conseguir DESCER a ladeira que não sobe.
#[test]
fn a_push_uphill_on_a_refused_slope_is_removed_and_downhill_is_not() {
    // Rampa de 60° subindo para a DIREITA: a normal tomba para a esquerda.
    let steep = at(0.5, [-0.866_025_4, 0.5]);
    let uphill = Motor {
        accel: [40.0, 0.0],
        boost: [0.0, 0.0],
    };
    let downhill = Motor {
        accel: [-40.0, 0.0],
        boost: [0.0, 0.0],
    };
    assert_eq!(
        no_uphill(uphill, Some(&steep), UP),
        Motor::default(),
        "empurrar contra a ladeira nao carrega ninguem para cima"
    );
    assert_eq!(
        no_uphill(downhill, Some(&steep), UP),
        downhill,
        "descer e' movimento legitimo, e a lei nao o toca"
    );
    // O canal de BOOST tem a mesma lei — ele escreve velocidade DIRETO, e
    // deixá-lo passar seria a mesma subida por outra porta.
    let boosted = Motor {
        accel: [0.0, 0.0],
        boost: [1.0, 0.0],
    };
    assert_eq!(no_uphill(boosted, Some(&steep), UP), Motor::default());
}

/// **Sem superfície recusada a lei não existe** — bit a bit.
///
/// É ela que mantém tudo o que a W3..W8 shipou byte-idêntico: no ar e no
/// chão o motor sai pelo mesmo valor que saía.
#[test]
fn without_a_refused_slope_the_motor_is_untouched() {
    let m = Motor {
        accel: [40.0, -3.0],
        boost: [0.25, 0.0],
    };
    assert_eq!(no_uphill(m, None, UP), m);
}

/// ⚠️ **"Morro acima" sai do `up`, não do eixo Y.**
///
/// O módulo assume `up = [0,1]` hoje (a ponte tem a const `UP`), mas a lei
/// recebe o vetor — e uma lei que lesse o Y literal seria a segunda resposta
/// que diverge no dia em que a gravidade lateral chegar ao player.
///
/// **Mutação que deve sangrar:** trocar `t[0]*up[0] + t[1]*up[1]` por `t[1]`.
#[test]
fn the_uphill_direction_comes_from_up_not_from_the_y_axis() {
    // Mundo girado 90°: o "alto" é +X. Uma rampa cuja normal aponta para
    // (+x, +y) tem o morro acima em... a tangente `perp_cw(n) = [n.y, -n.x]`.
    let up: Vec2 = [1.0, 0.0];
    let n: Vec2 = [0.5, 0.866_025_4];
    let steep = at(0.5, n);
    let t = perp_cw(n); // [0.866, -0.5] — `t · up = +0.866` ⇒ este É o morro acima
    let push_up = Motor {
        accel: [t[0] * 10.0, t[1] * 10.0],
        boost: [0.0, 0.0],
    };
    let push_down = Motor {
        accel: [-t[0] * 10.0, -t[1] * 10.0],
        boost: [0.0, 0.0],
    };
    assert_eq!(no_uphill(push_up, Some(&steep), up), Motor::default());
    assert_eq!(no_uphill(push_down, Some(&steep), up), push_down);
}

/// **Na porta:** a rampa recusada mata a CAMINHADA e deixa o PULO em paz.
///
/// ⚠️ É o escopo declarado da lei, e ele é uma decisão: capar o pulo faria o
/// personagem perder o salto por encostar numa ladeira.
///
/// **Mutação que deve sangrar:** passar o motor SOMADO pelo `no_uphill` em
/// vez de só o termo de caminhada.
#[test]
fn at_the_door_a_steep_slope_kills_the_walk_but_not_the_jump() {
    let cfg = PlayerConfig::STARTING_POINT;
    let steep = at(cfg.ride.float_height, [-0.866_025_4, 0.5]); // 60°
    let pushing = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    // Sem pulo: o empurrão morro acima não sobrevive.
    let walking = player_motor(
        &cfg,
        Some(&steep),
        None,
        None,
        pushing,
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    let idle = player_motor(
        &cfg,
        Some(&steep),
        None,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    assert_eq!(
        walking.motor, idle.motor,
        "com a ladeira recusada, empurrar contra ela tem de ser o mesmo que \
         nao empurrar: {:?} vs {:?}",
        walking.motor, idle.motor
    );

    // Com o pulo armado pelo coyote, o salto SAI — a lei não o alcança.
    let armed = JumpState {
        coyote: cfg.jump.coyote_time,
        ..JumpState::default()
    };
    let jumping = player_motor(
        &cfg,
        Some(&steep),
        None,
        None,
        PlayerInput {
            drive: 1.0,
            jump: true,
            ..PlayerInput::default()
        },
        PlayerState {
            jump: armed,
            ..PlayerState::default()
        },
        [0.0, 0.0],
        G,
        UP,
        DT,
    );
    assert!(
        jumping.motor.boost[1] > 0.0,
        "o pulo e' um gesto do artista e nao e' capado pela ladeira: {:?}",
        jumping.motor
    );
}

/// Os gates do ARRANQUE na porta única — irmão por `#[path]` pelo teto de 700
/// LOC, e o corte é o MESMO do par `jump_tests`/`jump_forgive_tests`: o pai fica
/// com *o que a porta É* (a composição, o canal da gravidade, a ladeira), e o
/// filho com *o que o arranque faz a ela* (calar a perna, calar a caminhada, e
/// ceder a um pulo).
#[path = "lib_dash_tests.rs"]
mod dash;
