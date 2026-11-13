//! Os gates da cena 110 (`W-MultiJump`) — os NÚMEROS que a mensagem imprime,
//! afirmados antes de o artista os ler.
//!
//! ⚠️ **A cena inteira é um contraste**, então o gate corre os DOIS lados: um
//! gate que só afirmasse *"o da direita sobe"* passaria numa cena cuja
//! prateleira alta fosse baixa demais para separar coisa alguma.

use super::{
    FEET_GAP, FLOOR_LEN, HIGH_TOP, HIGH_X0, LANE_A, LANE_B, LANE_SPAN, LOW_TOP, LOW_X1,
    build_multi_jump_scene,
};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};

/// A metade da cápsula de `spawn_player` (0,3 + 0,2): centro − isto = os pés.
const HALF_H: f32 = 0.5;

/// **A aritmética que a mensagem imprime está certa** — derivada das alturas que a
/// LEI promete (`jump_height` é o ápice real desde `f3a92736ab`), nunca de um alcance
/// medido com a lei velha (1,903 / 4,028 deixavam de valer).
///
/// ⚠️ **É este gate que impede a cena de deixar de conter o próprio fenômeno.**
#[test]
fn the_scene_delivers_the_numbers_its_message_prints() {
    let p = PlatformPlayer::default();
    let one = FEET_GAP + p.jump_height;
    let two = one + p.air_jump_height;
    // A baixa cabe num pulo só — é ela que prova que os dois sabem pular.
    assert!(LOW_TOP < one, "baixa {LOW_TOP} × um pulo {one}");
    // A alta NÃO cabe num pulo, nem dado de cima da baixa, e cabe em dois do chão.
    assert!(
        HIGH_TOP > LOW_TOP + one,
        "alta {HIGH_TOP} × da baixa {}",
        LOW_TOP + one
    );
    assert!(HIGH_TOP < two, "alta {HIGH_TOP} × dois pulos {two}");
    // O repouso medido é o `FEET_GAP` da conta.
    let mut sim = SimWorld::new();
    let (_, b) = build_multi_jump_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    for t in 1..=30 {
        bridge.dispatch(&mut sim, true, t);
    }
    let rest = sim
        .world()
        .get::<Transform>(b)
        .expect("transform")
        .translation
        .y;
    assert!(
        (rest - HALF_H - FEET_GAP).abs() < 0.01,
        "pés em repouso {}",
        rest - HALF_H
    );
    // ⚠️ E as raias não se tocam.
    const _: () = assert!(LANE_SPAN >= FLOOR_LEN);
    const _: () = assert!(LANE_B - LANE_A == LANE_SPAN);
}

/// Dirige os dois com a MESMA entrada e devolve o `y` mais alto de cada um.
///
/// ⚠️ **A entrada é uma só, como no produto** (`hand_input_to_players` entrega
/// a todos): um harness que dirigisse cada um por sua vez mediria duas corridas
/// e perderia a propriedade que a cena existe para mostrar.
fn drive(script: &[(bool, bool)]) -> (f32, f32) {
    let mut sim = SimWorld::new();
    let (one, two) = build_multi_jump_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let peak = |sim: &SimWorld, e: Entity| {
        sim.world()
            .get::<Transform>(e)
            .expect("transform")
            .translation
            .y
    };
    // Assenta primeiro: a mola em repouso, senão a altura medida carrega a
    // oscilação inicial em vez do pulo.
    let mut tick = 0_u64;
    for _ in 0..30 {
        tick += 1;
        bridge.set_player_input(one, PlayerInput::default());
        bridge.set_player_input(two, PlayerInput::default());
        bridge.dispatch(&mut sim, true, tick);
    }
    let (mut hi_one, mut hi_two) = (peak(&sim, one), peak(&sim, two));
    for &(jump, right) in script {
        let input = PlayerInput {
            jump,
            drive: if right { 1.0 } else { 0.0 },
            ..PlayerInput::default()
        };
        bridge.set_player_input(one, input);
        bridge.set_player_input(two, input);
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        hi_one = hi_one.max(peak(&sim, one));
        hi_two = hi_two.max(peak(&sim, two));
    }
    (hi_one, hi_two)
}

/// **O contraste que a cena É: com o mesmo gesto, só um sobe.**
///
/// O gesto é o do passo 3 do roteiro — segura, solta, aperta de novo no alto.
#[test]
fn the_same_gesture_lifts_only_the_one_with_a_charge() {
    let mut script: Vec<(bool, bool)> = Vec::new();
    script.extend(std::iter::repeat_n((true, false), 25)); // segura o 1o pulo
    script.push((false, false)); // solta
    script.extend(std::iter::repeat_n((true, false), 90)); // aperta de novo
    let (one, two) = drive(&script);
    assert!(
        two - one > 1.0,
        "com a MESMA entrada o da direita tem de subir muito mais alto: \
         sem carga {one:.3}, com carga {two:.3}"
    );
    // E o alto que ele alcança tem de passar da prateleira ALTA, senão a cena
    // mostra uma diferença que não chega a lugar nenhum.
    assert!(
        two - HALF_H > HIGH_TOP,
        "o da direita tem de passar do topo da prateleira alta ({HIGH_TOP}): {two:.3}"
    );
    assert!(one - HALF_H < HIGH_TOP, "e o da esquerda NAO: {one:.3}");
}

/// **E os dois sobem a prateleira BAIXA** — o controle que torna a falha acima
/// uma falta de pulo, e não um personagem quebrado.
#[test]
fn both_of_them_clear_the_low_shelf() {
    let mut script: Vec<(bool, bool)> = Vec::new();
    script.extend(std::iter::repeat_n((true, false), 25));
    script.extend(std::iter::repeat_n((false, false), 20));
    let (one, two) = drive(&script);
    for (who, y) in [("sem carga", one), ("com carga", two)] {
        assert!(
            y - HALF_H > LOW_TOP,
            "o {who} tem de passar do topo da prateleira baixa ({LOW_TOP}): {y:.3}"
        );
    }
}

/// Põe os dois em `x0 + at` (`y` pela altura), dirige a MESMA entrada e devolve o
/// centro final de cada um.
fn run_from(at: f32, y0: f32, script: &[(bool, f32)]) -> (f32, f32) {
    let mut sim = SimWorld::new();
    let (one, two) = build_multi_jump_scene(sim.world_mut());
    for (e, x0) in [(one, LANE_A), (two, LANE_B)] {
        let mut t = sim.world_mut().get_mut::<Transform>(e).expect("transform");
        t.translation.x = x0 + at;
        t.translation.y = y0;
    }
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    for &(jump, drive) in std::iter::repeat_n(&(false, 0.0), 30).chain(script) {
        let input = PlayerInput {
            jump,
            drive,
            ..PlayerInput::default()
        };
        bridge.set_player_input(one, input);
        bridge.set_player_input(two, input);
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    let y = |e: Entity| {
        sim.world()
            .get::<Transform>(e)
            .expect("transform")
            .translation
            .y
    };
    (y(one), y(two))
}

/// O gesto do passo 3, correndo: segura, solta, aperta de novo — e assenta parado.
fn step_three() -> Vec<(bool, f32)> {
    let mut s = vec![(true, 1.0); 22];
    s.push((false, 1.0));
    s.extend(std::iter::repeat_n((true, 1.0), 40));
    s.extend(std::iter::repeat_n((false, 0.0), 90));
    s
}

/// **E o passo 3 do roteiro é VERDADE: ele POUSA em cima da prateleira alta**, de
/// qualquer ponto da pista em que o artista comece a correr.
///
/// ⚠️ **Pousar = os PÉS em cima do topo, parados.** O gate de antes comparava o
/// CENTRO com o topo e passava com o corpo a escorregar pela parede (pés 0,4 m
/// abaixo do topo) — e o gesto começava encostado na parede da baixa.
#[test]
fn the_charged_one_lands_on_the_high_shelf() {
    let rest = FEET_GAP + HALF_H;
    for lead in [0.3_f32, 0.8, 1.5] {
        let (a, b) = run_from(LOW_X1 + lead, rest, &step_three());
        assert!(
            (b - HALF_H - FEET_GAP - HIGH_TOP).abs() < 0.02,
            "da pista +{lead} o da direita tinha de POUSAR na prateleira (topo {HIGH_TOP}): \
             pés {:.3}",
            b - HALF_H
        );
        assert!(
            a - HALF_H < HIGH_TOP,
            "e o da esquerda tinha de ficar embaixo dela: pés {:.3}",
            a - HALF_H
        );
    }
}

/// **E o gesto NATURAL também: correr até à parede da alta e pular ENCOSTADO nela**, a seta
/// premida contra a face. Até o Bug #14 o atrito comia 61 % do pulo e a cena pedia o pulo na pista.
#[test]
fn the_charged_one_lands_on_the_high_shelf_jumping_against_its_wall() {
    let rest = FEET_GAP + HALF_H;
    for run in [20_usize, 60] {
        let mut s = vec![(false, 1.0); run];
        s.extend(step_three());
        let (a, b) = run_from(HIGH_X0 - 1.5, rest, &s);
        assert!(
            (b - HALF_H - FEET_GAP - HIGH_TOP).abs() < 0.02,
            "encostado ({run} tiques de corrida) o da direita tinha de POUSAR na alta: pés {:.3}",
            b - HALF_H
        );
        assert!(
            a - HALF_H < HIGH_TOP,
            "e o da esquerda tinha de ficar embaixo dela: pés {:.3}",
            a - HALF_H
        );
    }
}

/// **E nem da beira da prateleira BAIXA um pulo só chega à alta** — o atalho que a
/// cena de 3,5 m de pista deixava aberto (o da esquerda aprovado no lugar do da direita).
#[test]
fn a_single_jump_from_the_low_shelf_does_not_reach_the_high_one() {
    let on_low = LOW_TOP + FEET_GAP + HALF_H;
    for (start, run) in [(2.5_f32, 15_usize), (2.5, 12), (3.0, 6), (3.6, 0)] {
        let mut s = vec![(false, 1.0); run];
        s.extend(std::iter::repeat_n((true, 1.0), 60));
        s.extend(std::iter::repeat_n((false, 0.0), 60));
        let (a, _) = run_from(start, on_low, &s);
        assert!(
            a - HALF_H < HIGH_TOP,
            "da baixa (x {start}, corrida {run}) o da esquerda chegou à alta: pés {:.3}",
            a - HALF_H
        );
    }
    const _: () = assert!(HIGH_X0 > LOW_X1);
}
