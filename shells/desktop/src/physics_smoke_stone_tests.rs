//! Os gates da cena 107 (`W-ShapeCast`), e os números que a mensagem dela
//! imprime.
//!
//! ⚠️ **Uma cena cuja mensagem cita NÚMEROS tem de medir os dela**, e a lição
//! desta linha estende isso aos **GESTOS** que ela manda fazer: o passo 4 diz
//! *"pare debaixo de uma pedra e solte: ele não se levanta"* e o passo 5 diz
//! *"entre duas pedras ele levanta-se na hora"*. As duas metades são afirmações
//! sobre o produto, e as duas correm aqui — pela **cena real**, não por uma
//! reconstrução — antes de o artista as ler.
//!
//! ⚠️ **E os gates aqui julgam a CENA, não a lei.** A lei tem os dela
//! (`ph2d-platformer::crouch_tests`), a porta tem os dela
//! (`ph2d-physics`, `world::sweep`) e o produto tem os dele
//! (`ph2d-physics-ecs/tests/platform_headroom_sweep.rs`).

use super::{CROUCH_HEIGHT, NARROW_HALF, NARROW_X, STONE_BOTTOM, WIDE_X, build_stone_scene};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// Meia-altura da caixa do personagem das cenas (`half_height + radius`).
const BODY_HALF: f32 = 0.5;
/// A altura de flutuação DE PÉ das cenas de player.
const FLOAT: f32 = 0.9;
/// O topo da cabeça de pé e agachado — os dois pólos que a cena separa.
const STANDING_TOP: f32 = FLOAT + BODY_HALF;
const CROUCHED_TOP: f32 = CROUCH_HEIGHT + BODY_HALF;

/// **Põe o personagem em `x`, agacha-o, e solta.** Devolve o topo da cabeça.
///
/// ⚠️ **Ele é POSTO em `x` em vez de caminhar até lá**, e isso é fixture: andar
/// até uma pedra de 8 cm exigiria calibrar tiques de caminhada contra uma janela
/// de centímetros, e o gate falharia por deriva de fixture em vez de por
/// defeito. O que a mensagem afirma é o que acontece **parado** debaixo dela,
/// que é exactamente o que isto mede.
fn crouch_and_release_at(x: f32) -> f32 {
    let mut sim = SimWorld::new();
    let player = build_stone_scene(sim.world_mut());
    {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(player)
            .expect("o player tem Transform");
        t.translation.x = x;
        t.translation.y = CROUCH_HEIGHT;
    }

    let mut bridge = PhysicsBridge::new();
    let down = PlayerInput {
        down: true,
        ..PlayerInput::default()
    };
    let mut tick = 0;
    for _ in 0..90 {
        bridge.set_player_input(player, down);
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    for _ in 0..150 {
        bridge.set_player_input(player, PlayerInput::default());
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    top_of(&sim)
}

fn top_of(sim: &SimWorld) -> f32 {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, tr) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some(tr.translation.y + BODY_HALF);
        }
    }
    found.expect("o player tem de existir")
}

/// **A cena entrega os números que a mensagem dela imprime.**
///
/// O topo de pé e o topo agachado são os dois pólos que a face das pedras tem de
/// separar — e se ela sair desse intervalo a cena passa a medir outra coisa
/// (mais alta: ele passa de pé; mais baixa: nem agachado cabe).
#[test]
fn the_scene_delivers_the_numbers_its_message_prints() {
    assert!(
        (STANDING_TOP - 1.40).abs() < 1.0e-6,
        "a mensagem diz que o topo de pe' mede 1.40: {STANDING_TOP}"
    );
    assert!(
        (CROUCHED_TOP - 1.05).abs() < 1.0e-6,
        "a mensagem diz que o topo agachado mede 1.05: {CROUCHED_TOP}"
    );
    // ⚠️ Em tempo de COMPILAÇÃO: os três são constantes, então um assert de
    // runtime não pode falhar por nada que não seja alguém tê-las mexido — e o
    // compilador responde isso melhor, e antes.
    const _: () = assert!(CROUCHED_TOP < STONE_BOTTOM && STONE_BOTTOM < STANDING_TOP);
    assert!(
        (NARROW_HALF * 2.0 - 0.08).abs() < 1.0e-6,
        "a mensagem diz 'pedras de 8 cm': {}",
        NARROW_HALF * 2.0
    );
}

/// **O PASSO 4: debaixo de CADA pedra, soltar não levanta.**
///
/// ⚠️ *"em TODAS, e não só nalgumas"* é a metade que a fileira existe para
/// afirmar — a grade antiga acompanhava o corpo, então uma pedra só teria uma
/// janela de `x` em que falhava.
#[test]
fn under_every_stone_releasing_the_button_does_not_stand_him_up() {
    for x in NARROW_X {
        let top = crouch_and_release_at(x);
        assert!(
            top < STONE_BOTTOM,
            "sob a pedra em {x}: topo {top:.3} contra face em {STONE_BOTTOM}"
        );
        assert!(
            (top - CROUCHED_TOP).abs() < 0.05,
            "sob a pedra em {x} ele fica na altura de agachado: {top:.3}"
        );
    }
}

/// **O PASSO 5: ENTRE duas pedras ele levanta-se.**
///
/// Sem isto, o passo 4 é satisfeito por um sensor cravado em *bloqueado* — e o
/// roteiro mandaria o artista procurar uma diferença que a cena não tem.
#[test]
fn between_two_stones_he_stands_right_up() {
    let mid = (NARROW_X[0] + NARROW_X[1]) * 0.5;
    let top = crouch_and_release_at(mid);
    assert!(
        (top - STANDING_TOP).abs() < 0.05,
        "entre as pedras ({mid}) ele levanta-se inteiro: {top:.3}"
    );
}

/// **O PASSO 6: a laje larga responde o MESMO** — o controle da cena.
///
/// Se ela se comportasse diferente da pedra, o sensor estaria a medir o tamanho
/// do teto e não o corpo.
#[test]
fn the_wide_slab_answers_exactly_what_the_narrow_stone_answers() {
    let narrow = crouch_and_release_at(NARROW_X[0]);
    let wide = crouch_and_release_at(WIDE_X + 3.0);
    assert!(
        wide < STONE_BOTTOM,
        "sob a laje larga ele fica agachado: {wide:.3}"
    );
    assert!(
        (wide - narrow).abs() < 0.02,
        "a laje e a pedra tem de dar a MESMA altura: {wide:.3} contra {narrow:.3}"
    );
}

/// **A PEDRA CABE NO VÃO ENTRE DUAS AMOSTRAS** — a propriedade que torna esta
/// cena o repro, e não uma cena de agachar qualquer.
///
/// ⚠️ Sem isto, alguém que engordasse a pedra deixaria a cena verde e **muda**:
/// ela passaria a mostrar o que a cena 94 já mostra, e a wave ficaria sem
/// demonstração. Os deslocamentos são os do sensor ANTIGO — a grade que morreu —
/// e é por isso que eles estão escritos aqui como literais e não importados de
/// lado nenhum: não há de onde os importar, e é esse o ponto.
#[test]
fn the_stone_fits_between_two_of_the_old_samples() {
    const OLD_RAYS: [f32; 3] = [-0.2, 0.0, 0.2];
    let gap = OLD_RAYS[1] - OLD_RAYS[0];
    assert!(
        NARROW_HALF * 2.0 < gap,
        "uma pedra de {} tem de caber no vao de {gap} entre duas amostras",
        NARROW_HALF * 2.0
    );
}

/// **AS PEDRAS ESTÃO SEPARADAS O BASTANTE PARA HAVER UM "ENTRE"** — o passo 5.
///
/// ⚠️ Se elas ficassem juntas demais, *"entre duas pedras ele levanta-se"* seria
/// um gesto impossível, e o roteiro mandaria o artista procurar uma coisa que a
/// cena não tem.
#[test]
fn there_is_room_to_stand_between_the_stones() {
    const BODY_HALF_WIDTH: f32 = 0.2;
    for pair in NARROW_X.windows(2) {
        let gap = pair[1] - pair[0] - 2.0 * NARROW_HALF;
        assert!(
            gap > 4.0 * BODY_HALF_WIDTH,
            "entre {} e {} sobram {gap:.2} m de ceu -- pouco para o passo 5",
            pair[0],
            pair[1]
        );
    }
    assert!(
        WIDE_X > NARROW_X[NARROW_X.len() - 1] + 1.0,
        "a laje de controle vem DEPOIS das pedras, com espaco entre elas"
    );
}
