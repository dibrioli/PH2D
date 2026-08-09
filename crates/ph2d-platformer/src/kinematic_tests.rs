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

fn resting() -> KinematicState {
    KinematicState {
        grounded: true,
        ..KinematicState::default()
    }
}

/// Chão PLANO que se move — a única fixture que estas leis precisam do sensor.
fn floor_moving(gv: Vec2) -> GroundSample {
    GroundSample {
        distance: 0.5,
        normal: [0.0, 1.0],
        ground_velocity: gv,
        one_way: false,
    }
}

/// **No ar, a gravidade é aplicada AQUI** — e é a assimetria central: o solver
/// não a aplica a um corpo cinemático.
#[test]
fn gravity_is_integrated_by_this_law_when_airborne() {
    let (st, wanted) = kinematic_advance(still(), Motor::default(), None, G, UP, DT);
    assert!(
        (st.velocity[1] - G[1] * DT).abs() < 1e-6,
        "um tique de queda livre tem de dar {} e deu {}",
        G[1] * DT,
        st.velocity[1]
    );
    assert!(wanted[1] < 0.0, "e o deslocamento pedido aponta para baixo");
}

/// **No chão, a componente que aponta PARA o chão é absorvida** — e sem ela um
/// personagem parado numa rampa desliza (ver o aviso do módulo, com o número).
///
/// ⚠️ **A metade oposta é o pulo:** subir com o raio ainda a ver o chão é
/// exactamente o tique da decolagem, e zerar o eixo inteiro mataria o salto.
#[test]
fn the_ground_absorbs_only_what_points_into_it() {
    let mut st = resting();
    for _ in 0..600 {
        st = kinematic_advance(st, Motor::default(), None, G, UP, DT).0;
    }
    assert_eq!(
        st.velocity[1], 0.0,
        "dez segundos parado no chao nao podem acumular queda"
    );

    let takeoff = KinematicState {
        velocity: [0.0, 5.0],
        grounded: true,
    };
    let (up_st, _) = kinematic_advance(takeoff, Motor::default(), None, G, UP, DT);
    assert!(
        up_st.velocity[1] > 4.8,
        "o tique da decolagem ve' o chao e NAO pode ser zerado: {}",
        up_st.velocity[1]
    );
}

/// **E no AR ela não age** — o controle que separa *"pousei"* de *"a perna
/// alcança"*, e a razão de a régua do `grounded` ser corrigida sob Snap.
#[test]
fn nothing_is_absorbed_while_airborne() {
    let (st, _) = kinematic_advance(still(), Motor::default(), None, G, UP, DT);
    assert!(
        (st.velocity[1] - G[1] * DT).abs() < 1.0e-6,
        "no ar a gravidade tem de sobreviver: {}",
        st.velocity[1]
    );
}

/// **O integrador paga o CONTATO e não o ATRITO** (K7) — e as duas metades são
/// o par que impede a plataforma de ser contada duas vezes.
///
/// ⚠️ A metade do VAGÃO nasceu deste gate ao contrário: ele afirmava que o
/// deslocamento *"tem de levar o vagão"*, o que é verdade sobre esta função
/// isolada e **falso sobre o produto** — a [`crate::walk`] já leva o personagem
/// ao referencial do chão pela tangente, e somar aqui dava 1,98× (a tabela está
/// no doc do [`ground_carry`]). Um gate que mede uma função sem a lei que a
/// alimenta pina a metade errada de um par.
///
/// A metade do ELEVADOR é a que sobrevive intacta: nenhuma tração empurra ao
/// longo da normal, então o contato é dívida do integrador — e ele a paga no
/// DESLOCAMENTO sem a escrever na velocidade guardada.
#[test]
fn the_integrator_owes_the_contact_and_not_the_traction() {
    // VAGÃO: velocidade tangente ao chão -- a caminhada é quem a paga.
    let wagon = floor_moving([3.0, 0.0]);
    let (st, wanted) = kinematic_advance(resting(), Motor::default(), Some(&wagon), G, UP, DT);
    assert!(
        wanted[0].abs() < 1e-9,
        "a tangente e' da caminhada; o integrador nao pode soma-la de novo: {}",
        wanted[0]
    );

    // ELEVADOR: velocidade ao longo da normal -- ninguem mais a paga.
    let lift = floor_moving([0.0, 3.0]);
    let (st_lift, w_lift) = kinematic_advance(resting(), Motor::default(), Some(&lift), G, UP, DT);
    assert!(
        (w_lift[1] - 3.0 * DT).abs() < 1e-6,
        "o contato tem de levantar o personagem: {}",
        w_lift[1]
    );
    assert_eq!(
        st.velocity[0], 0.0,
        "e a velocidade PROPRIA nao pode herdar o vagao"
    );
    assert!(
        st_lift.velocity[1] <= 0.0,
        "nem o elevador: a subida e' deslocamento deste tique, nao posse ({})",
        st_lift.velocity[1]
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
        grounded: true,
    };
    let wanted = [4.0 * DT, 0.0];
    let settled = kinematic_settle(st, wanted, [0.0, 0.0], true, DT);
    assert_eq!(
        settled.velocity[0], 0.0,
        "encostado numa parede a velocidade propria tem de ir a ZERO"
    );

    // E um bloqueio maior que a velocidade não a inverte.
    let hard = kinematic_settle(st, [10.0 * DT, 0.0], [0.0, 0.0], true, DT);
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
    // ⚠️ A fixture é um ELEVADOR e não um vagão desde que o `ground_carry`
    // passou a pagar só o contato: com a plataforma tangente o integrador não
    // pede deslocamento nenhum, e um bloqueio de zero não contém o fenômeno.
    let gv = [0.0, 3.0];
    let lift = floor_moving(gv);
    let (st, wanted) = kinematic_advance(resting(), Motor::default(), Some(&lift), G, UP, DT);
    assert!(
        wanted[1] > 0.0,
        "a fixture tem de CONTER o fenomeno: o pedido subiu {}",
        wanted[1]
    );
    // O teto impede tudo.
    let settled = kinematic_settle(st, wanted, [0.0, 0.0], true, DT);
    assert!(
        settled.velocity[1] <= 0.0,
        "prensado contra o teto, o elevador nao pode DEVER velocidade -- \
         a regra ingenua daria {} e deu {}",
        -gv[1],
        settled.velocity[1]
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
        grounded: false,
    };
    let wanted = [0.0, -DT];
    // A rampa desviou metade da queda para o lado.
    let effective = [0.5 * DT, -0.5 * DT];
    let settled = kinematic_settle(st, wanted, effective, true, DT);
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

/// **A porta da absorção — parada, e nos três casos que ela distingue.**
///
/// Ela é `pub` porque tem DOIS consumidores (o integrador e a ponte, que a chama
/// antes da lei); este gate pina o que ela responde a cada um.
#[test]
fn the_supported_velocity_drops_only_what_the_ground_holds() {
    // No chão, a caminho do chão: some a componente ao longo de `up`.
    let held = supported_velocity([2.0, -5.0], true, UP);
    assert!(
        (held[1]).abs() < 1e-6,
        "a queda tem de sair inteira: {held:?}"
    );
    assert!(
        (held[0] - 2.0).abs() < 1e-6,
        "e o eixo do chao passa intacto: {held:?}"
    );
    // No AR o valor é verbatim — é ali que a queda de facto acontece.
    assert_eq!(
        supported_velocity([2.0, -5.0], false, UP),
        [2.0, -5.0],
        "no ar nada e' absorvido"
    );
    // A SAIR do chão (um pulo) também é verbatim, senão a decolagem morre.
    assert_eq!(
        supported_velocity([2.0, 7.0], true, UP),
        [2.0, 7.0],
        "subir nao e' cair"
    );
}
