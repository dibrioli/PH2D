//! Os gates da integração cinemática (W-KinMove).
//!
//! ⚠️ Módulo FILHO por `#[path]`: é isso que mantém `use super::*` a alcançar o
//! que não é `pub`.
use super::*;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
const DT: f32 = 1.0 / 60.0;

fn still() -> KinematicState {
    KinematicState::default()
}

/// **No ar, a gravidade é aplicada AQUI** — e é a assimetria central: o solver
/// não a aplica a um corpo cinemático.
#[test]
fn gravity_is_integrated_by_this_law_when_airborne() {
    let (st, wanted) = kinematic_advance(still(), Motor::default(), false, [0.0, 0.0], G, UP, DT);
    assert!(
        (st.velocity[1] - G[1] * DT).abs() < 1e-6,
        "um tique de queda livre tem de dar {} e deu {}",
        G[1] * DT,
        st.velocity[1]
    );
    assert!(wanted[1] < 0.0, "e o deslocamento pedido aponta para baixo");
}

/// **No chão, a componente que aponta PARA o chão é absorvida** — sem isto o
/// personagem parado acumula velocidade para baixo para sempre.
///
/// ⚠️ **E a metade oposta é o pulo:** subir com o raio ainda a ver o chão é
/// exactamente o tique da decolagem, e zerar o eixo inteiro mataria o salto.
#[test]
fn the_ground_absorbs_only_what_points_into_it() {
    let mut st = still();
    for _ in 0..600 {
        st = kinematic_advance(st, Motor::default(), true, [0.0, 0.0], G, UP, DT).0;
    }
    assert_eq!(
        st.velocity[1], 0.0,
        "dez segundos parado no chao nao podem acumular queda"
    );

    let takeoff = KinematicState {
        velocity: [0.0, 5.0],
    };
    let (up_st, _) = kinematic_advance(takeoff, Motor::default(), true, [0.0, 0.0], G, UP, DT);
    assert!(
        up_st.velocity[1] > 4.8,
        "o tique da decolagem ve' o chao e NAO pode ser zerado: {}",
        up_st.velocity[1]
    );
}

/// **A plataforma leva o personagem sem o CONTAMINAR** (K7).
///
/// ⚠️ As duas metades são a decisão: o deslocamento deste tique inclui o vagão,
/// e a velocidade guardada **não** — senão ele continuaria a voar para o lado
/// depois de saltar para o chão firme.
#[test]
fn a_moving_platform_carries_without_being_owned() {
    let gv = [3.0, 0.0];
    let (st, wanted) = kinematic_advance(still(), Motor::default(), true, gv, G, UP, DT);
    assert!(
        (wanted[0] - gv[0] * DT).abs() < 1e-6,
        "o deslocamento tem de levar o vagao: {}",
        wanted[0]
    );
    assert_eq!(
        st.velocity[0], 0.0,
        "e a velocidade PROPRIA nao pode herda-lo"
    );
}

/// **Contra uma parede, a velocidade própria PARA — e não inverte.**
///
/// ⚠️ Sem esta lei o personagem soma `+v` tique após tique e sai disparado no
/// instante em que a parede acaba; com uma subtração sem teto ele saltaria para
/// trás. O gate mede as duas.
#[test]
fn a_blocked_body_stops_and_does_not_bounce() {
    let st = KinematicState {
        velocity: [4.0, 0.0],
    };
    let wanted = [4.0 * DT, 0.0];
    let settled = kinematic_settle(st, wanted, [0.0, 0.0], DT);
    assert_eq!(
        settled.velocity[0], 0.0,
        "encostado numa parede a velocidade propria tem de ir a ZERO"
    );

    // E um bloqueio maior que a velocidade não a inverte.
    let hard = kinematic_settle(st, [10.0 * DT, 0.0], [0.0, 0.0], DT);
    assert_eq!(hard.velocity[0], 0.0, "e nunca trocar de sinal");
}

/// **Um bloqueio que impede a PLATAFORMA não é uma velocidade minha a
/// corrigir** — o artefato que a regra ingênua produz.
///
/// ⚠️ Nasceu deste raciocínio e não de um relatório: parado sobre um vagão e
/// prensado contra uma parede, `v −= (pedido − efetivo)/dt` deixa
/// `v = −velocidade_do_vagão`, e o personagem **dispara para trás** ao sair
/// dele. É a mesma classe de defeito que o `−gv` de um `settle` cego produz em
/// qualquer controlador cinemático escrito sem o teste de sinal.
#[test]
fn a_platform_blocked_by_a_wall_does_not_owe_the_character_velocity() {
    let gv = [3.0, 0.0];
    let (st, wanted) = kinematic_advance(still(), Motor::default(), true, gv, G, UP, DT);
    // A parede impede tudo.
    let settled = kinematic_settle(st, wanted, [0.0, 0.0], DT);
    assert_eq!(
        settled.velocity[0], 0.0,
        "parado sobre um vagao prensado, a velocidade propria continua ZERO -- \
         a regra ingenua daria {}",
        -gv[0]
    );
}

/// **Deslizar não é ser bloqueado** — a componente onde a velocidade própria é
/// zero passa intacta.
///
/// ⚠️ Sem isto, uma rampa que converte queda em movimento lateral seria lida
/// como *"a parede me parou"* e o deslize morreria no primeiro tique.
#[test]
fn sliding_along_a_slope_is_not_absorbed() {
    let st = KinematicState {
        velocity: [0.0, -1.0],
    };
    let wanted = [0.0, -1.0 * DT];
    // A rampa desviou metade da queda para o lado.
    let effective = [0.5 * DT, -0.5 * DT];
    let settled = kinematic_settle(st, wanted, effective, DT);
    assert_eq!(
        settled.velocity[0], 0.0,
        "o eixo em que ele nao empurrava nao pode ganhar correcao"
    );
    assert!(
        (settled.velocity[1] + 0.5).abs() < 1e-6,
        "e a queda tem de sobrar so' o que de facto aconteceu: {}",
        settled.velocity[1]
    );
}
