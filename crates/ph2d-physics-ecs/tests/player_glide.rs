//! **O PLANEIO, pela porta do produto** (`W-Glide`) — a lei, a ponte e o solver
//! juntos.
//!
//! ⚠️ **O oráculo é a POSE e o TEMPO, nunca um campo de estado:** o que o jogador
//! vê é o personagem descer devagar e atravessar um vão que não atravessava. Um
//! gate sobre um booleano `gliding` ficaria verde com o corpo a cair como sempre.

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use scene_fixture::{pose, scene};

/// De quantos metros se larga o personagem.
const DROP: f32 = 8.0;

/// Uma cena plana com o personagem a `DROP` metros do repouso, e o planeio
/// autorado (`0.0` = desligado).
fn dropped(glide: f32) -> (SimWorld, PhysicsBridge, Entity) {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
        p.glide_fall_speed = glide;
    }
    let y = pose(&sim).1;
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(player) {
        t.translation.y = y + DROP;
    }
    (sim, bridge, player)
}

/// Deixa cair com uma entrada fixa e devolve `(segundos no ar, alcance lateral)`.
fn fall(glide: f32, input: PlayerInput) -> (f32, f32) {
    let (mut sim, mut bridge, player) = dropped(glide);
    let (x0, y0) = pose(&sim);
    let mut tick = 0_u64;
    let mut prev = y0;
    for i in 1..=900_u64 {
        bridge.set_player_input(player, input);
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        let (x, y) = pose(&sim);
        let v = (y - prev) * 60.0;
        prev = y;
        if v > -0.05 && y < y0 - 1.0 {
            return (i as f32 / 60.0, (x - x0).abs());
        }
    }
    (15.0, (pose(&sim).0 - x0).abs())
}

/// O dedo no pulo, e mais nada.
fn holding() -> PlayerInput {
    PlayerInput {
        jump: true,
        ..PlayerInput::default()
    }
}

/// **Segurar o pulo faz a queda demorar** — e o CONTROLE é a mesma cena com a
/// capacidade desligada.
///
/// ⚠️ **Sem o controlo este gate não prova nada:** o personagem podia estar a
/// demorar por qualquer outra razão da cena, e o número sozinho não distingue.
#[test]
fn holding_the_button_makes_the_fall_last_longer() {
    let (t_off, _) = fall(0.0, holding());
    let (t_on, _) = fall(2.0, holding());
    assert!(
        t_on > t_off * 1.5,
        "planando, a queda de {DROP} m tem de demorar muito mais: {t_off:.2} s desligado \
         contra {t_on:.2} s ligado"
    );
}

/// **⚠️ O GATE DA WAVE: sem o DEDO não há planeio.**
///
/// ⚠️ **É este que separa a capacidade de um `fall_gravity` mais brando** — se
/// ele falhar, o que foi construído não é um planeio, é uma queda mais lenta
/// para toda gente.
#[test]
fn without_the_finger_the_fall_is_the_one_it_always_was() {
    let (t_off, _) = fall(0.0, PlayerInput::default());
    let (t_on, _) = fall(2.0, PlayerInput::default());
    assert!(
        (t_on - t_off).abs() < 0.05,
        "com a capacidade armada e o dedo SOLTO, a queda tem de ser a de sempre: \
         {t_off:.2} s contra {t_on:.2} s"
    );
}

/// **O planeio ATRAVESSA um vão** — é para isto que ele existe.
///
/// ⚠️ **O alcance é a única coluna que fala a língua do artista**, e é a que a
/// cena de smoke desenha: dois patamares à mesma distância, e só quem plana
/// chega ao segundo.
///
/// ⚠️ **O `drive` é parte da fixture, e a primeira versão deste gate não o
/// tinha:** ele mediu `0,00 m` nas DUAS colunas e falhou — não porque o planeio
/// não carrega, mas porque **ninguém estava a andar**. Um alcance lateral é
/// `velocidade × tempo`, e sem dedo no eixo horizontal a primeira metade é zero.
#[test]
fn the_glide_carries_him_across_a_gap() {
    let walking = PlayerInput {
        drive: 1.0,
        ..holding()
    };
    let (_, far_off) = fall(0.0, walking);
    let (_, far_on) = fall(2.0, walking);
    assert!(
        far_on > far_off * 1.5,
        "planando ele tem de viajar muito mais longe: {far_off:.2} m contra {far_on:.2} m"
    );
}

/// **O TETO é o número autorado**, e a descida assenta nele.
///
/// ⚠️ **É esta a propriedade que a medição escolheu** (`docs` do
/// `ph2d_platformer::glide`): sob uma ESCALA de gravidade a velocidade nunca
/// assenta — ela cresce com a profundidade —, e é por isso que o gate mede a
/// descida em DOIS pontos e exige que sejam o mesmo número.
///
/// ⚠️ **A descida assenta UM TIQUE DE GRAVIDADE ABAIXO do teto, e o número é
/// DERIVADO, não escolhido:** a lei corre uma vez por tique e põe `rel_up` no
/// teto; a gravidade corre durante o tique e acrescenta
/// `g · dt · fall_gravity` = `9,81 × (1/60) × 2` = **0,327 m/s** antes de a lei
/// voltar a agir. A velocidade média de um tique fica entre o teto e o teto mais
/// isso — medido, **−2,25 para um teto de 2,00**.
///
/// ⚠️ **A primeira versão deste gate tinha uma tolerância de `0,15` que eu
/// escolhi**, e ela reprovou o produto correto. Uma barra escolhida esconde o
/// resíduo; uma barra DERIVADA o explica — e recusa tanto um teto que não
/// segura como um que segura de mais.
#[test]
fn the_descent_settles_on_the_authored_ceiling() {
    // ⚠️ **3,0 e não 2,0, de propósito:** 2,0 é o número que a cena de smoke
    // autora, e um gate que usasse o mesmo valor não distinguiria *"a lei lê o
    // número"* de *"a lei tem esse número escrito dentro"*. Medido: com o teto
    // cravado em 2,0 na lei, este gate fica **VERDE** com `CEILING = 2.0` e
    // **sangra** com 3,0.
    const CEILING: f32 = 3.0;
    /// O que a gravidade acrescenta entre duas passagens da lei, m/s.
    const TICK_OF_GRAVITY: f32 = 9.81 / 60.0 * 2.0;
    let (mut sim, mut bridge, player) = dropped(CEILING);
    let y0 = pose(&sim).1;
    let mut tick = 0_u64;
    let mut prev = y0;
    let mut marks = [0.0_f32; 2];
    let mut next = 0_usize;
    for _ in 0..900 {
        bridge.set_player_input(player, holding());
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        let y = pose(&sim).1;
        let v = (y - prev) * 60.0;
        prev = y;
        let fallen = y0 - y;
        while next < 2 && fallen >= [2.0, 6.0][next] {
            marks[next] = v;
            next += 1;
        }
        if next == 2 {
            break;
        }
    }
    assert_eq!(next, 2, "a queda tem de alcancar os dois pontos de medicao");
    for (i, v) in marks.iter().enumerate() {
        let over = v.abs() - CEILING;
        assert!(
            (0.0..=TICK_OF_GRAVITY).contains(&over),
            "a descida tem de assentar entre o teto {CEILING} e um tique de gravidade \
             acima dele ({:.3}): no ponto {i} media {v:.3} (excesso {over:.3})",
            CEILING + TICK_OF_GRAVITY
        );
    }
    assert!(
        (marks[1] - marks[0]).abs() < 0.05,
        "e tem de ser o MESMO numero nos dois: {:.3} e {:.3}",
        marks[0],
        marks[1]
    );
}

/// **⚠️ A lei nunca acelera uma queda — nem no ápice de um pulo.**
///
/// ⚠️ **É a propriedade que descartou o ALVO**, e ela é observável no produto:
/// pular com o dedo preso e um teto autorado tem de dar **a mesma altura** que
/// pular sem a capacidade. Um alvo teria travado a subida em `−2 m/s` no
/// primeiro tique.
#[test]
fn a_jump_with_the_finger_held_reaches_the_same_height() {
    let apex = |glide: f32| {
        let (mut sim, mut bridge, player) = scene(0.0, 0.0);
        if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
            p.glide_fall_speed = glide;
        }
        let mut tick = 0_u64;
        for _ in 0..30 {
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
        }
        let rest = pose(&sim).1;
        let mut top = rest;
        for _ in 0..120 {
            bridge.set_player_input(player, holding());
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
            top = top.max(pose(&sim).1);
        }
        top - rest
    };
    let (off, on) = (apex(0.0), apex(2.0));
    assert!(
        (on - off).abs() < 0.02,
        "o planeio nao pode encolher um pulo: {off:.4} m sem ele, {on:.4} m com ele"
    );
}
