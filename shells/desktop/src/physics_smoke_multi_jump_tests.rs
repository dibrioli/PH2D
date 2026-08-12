//! Os gates da cena 110 (`W-MultiJump`) — os NÚMEROS que a mensagem imprime,
//! afirmados antes de o artista os ler.
//!
//! ⚠️ **A cena inteira é um contraste**, então o gate corre os DOIS lados: um
//! gate que só afirmasse *"o da direita sobe"* passaria numa cena cuja
//! prateleira alta fosse baixa demais para separar coisa alguma.

use super::{HIGH_TOP, LANE_A, LANE_B, LANE_SPAN, LOW_TOP, build_multi_jump_scene};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// O alcance MEDIDO de um pulo segurado (`measure_the_reachable_height`).
const REACH_ONE: f32 = 1.903;
/// O alcance MEDIDO de dois pulos segurados.
const REACH_TWO: f32 = 4.028;

/// **A aritmética que a mensagem imprime está certa** — em tempo de compilação.
///
/// ⚠️ **É este gate que impede a cena de deixar de conter o próprio fenômeno:**
/// baixar a prateleira alta para dentro do alcance de um pulo faria os DOIS
/// personagens subirem, e o smoke passaria a aprovar uma cena que não distingue
/// nada.
#[test]
fn the_scene_delivers_the_numbers_its_message_prints() {
    // A baixa cabe num pulo só — é ela que prova que os dois sabem pular.
    const _: () = assert!(LOW_TOP < REACH_ONE);
    // A alta NÃO cabe num, e cabe em dois: é exatamente o vão desta wave.
    const _: () = assert!(HIGH_TOP > REACH_ONE);
    const _: () = assert!(HIGH_TOP < REACH_TWO);
    // ⚠️ E as raias não se tocam: a raia mede 12 m de chão, então um vão menor
    // deixaria um personagem alcançar a geometria do vizinho e o contraste
    // deixaria de ser entre DOIS mundos iguais.
    const _: () = assert!(LANE_SPAN >= 12.0);
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
        two > HIGH_TOP,
        "o da direita tem de passar do topo da prateleira alta ({HIGH_TOP}): {two:.3}"
    );
    assert!(one < HIGH_TOP, "e o da esquerda NAO: {one:.3}");
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
            y > LOW_TOP,
            "o {who} tem de passar do topo da prateleira baixa ({LOW_TOP}): {y:.3}"
        );
    }
}

/// **E o passo 3 do roteiro é VERDADE: ele POUSA em cima da prateleira alta.**
///
/// ⚠️ **O pico não é o mesmo que pousar**, e é por isso que este gate existe ao
/// lado do de cima: um personagem pode passar do topo e cair de volta do lado
/// errado. A mensagem promete um pouso, e uma promessa de roteiro que a cena não
/// cumpre é o que faz o artista concluir que a feature está quebrada.
#[test]
fn the_charged_one_lands_on_the_high_shelf() {
    let mut sim = SimWorld::new();
    let (one, two) = build_multi_jump_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let y = |sim: &SimWorld, e: Entity| {
        sim.world()
            .get::<Transform>(e)
            .expect("transform")
            .translation
            .y
    };
    let mut tick = 0_u64;
    let mut step = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, jump: bool, drive: f32| {
        let input = PlayerInput {
            jump,
            drive,
            ..PlayerInput::default()
        };
        bridge.set_player_input(one, input);
        bridge.set_player_input(two, input);
        tick += 1;
        bridge.dispatch(sim, true, tick);
    };
    for _ in 0..30 {
        step(&mut sim, &mut bridge, false, 0.0);
    }
    // Anda para a direita até encostar na parede da prateleira alta.
    for _ in 0..150 {
        step(&mut sim, &mut bridge, false, 1.0);
    }
    // O gesto do passo 3: segura, solta, aperta de novo — sempre andando.
    for _ in 0..22 {
        step(&mut sim, &mut bridge, true, 1.0);
    }
    step(&mut sim, &mut bridge, false, 1.0);
    for _ in 0..40 {
        step(&mut sim, &mut bridge, true, 1.0);
    }
    // E deixa assentar onde quer que tenha parado.
    for _ in 0..90 {
        step(&mut sim, &mut bridge, false, 1.0);
    }
    let (a, b) = (y(&sim, one), y(&sim, two));
    assert!(
        b > HIGH_TOP,
        "o da direita tinha de POUSAR em cima da prateleira (topo {HIGH_TOP}): {b:.3}"
    );
    assert!(
        a < HIGH_TOP,
        "e o da esquerda tinha de ficar embaixo dela: {a:.3}"
    );
}
