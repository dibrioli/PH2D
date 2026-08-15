//! **O TETO É UM FATO** (`W-Ceiling`) — o `is_on_ceiling` do Godot, a última das
//! três consultas que a auditoria 09 §3.A deixou como cauda.
//!
//! ⚠️ **Estes gates moram na PONTE e não na lei, e não é arbitrário:** o
//! `head_blocked` é medido contra o mundo (uma varredura da forma), então um
//! teste que o soletrasse à mão numa fixture estaria a afirmar o que ele próprio
//! escreveu. O oráculo é uma cena com um teto de verdade e um personagem a subir
//! contra ele.
//!
//! ⚠️ **E o gate que carrega a wave é o do ASSIST DESLIGADO.** O fato nasceu
//! porque os dois sensores de cima que já existiam são assistências
//! CONDICIONAIS — o `Headroom` só é sondado agachado, o `CeilingProbe` só com a
//! quina armada —, e derivar o teto de qualquer um deles daria um readout falso
//! na maioria dos tiques, em silêncio. Um gate que só medisse a configuração
//! padrão (onde `corner_reach = 0,12` e as duas condições coincidem) seria
//! **verde exactamente sobre o defeito que esta wave existe para não ter**.

#[path = "platform_water_scene.rs"]
mod scene;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::PlayerInput;
use scene::{FLOAT, floor, subject_tuned};

/// Um teto — caixa larga e estática cuja face de BAIXO fica em `bottom`.
fn ceiling(sim: &mut SimWorld, bottom: f32) {
    sim.world_mut().spawn((
        Name::new("Ceiling"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, bottom + 0.5)),
    ));
}

/// O personagem destes gates: pula alto o bastante para alcançar o teto.
fn jumper(corner_reach: f32) -> PlatformPlayer {
    PlatformPlayer {
        float_height: FLOAT,
        jump_height: 1.2,
        corner_reach,
        ..PlatformPlayer::default()
    }
}

/// Corre a cena e devolve `(bateu alguma vez, subiu alguma vez)`.
///
/// O segundo é o **CONTROLE**: sem ele um gate que mede *"bateu"* fica verde num
/// personagem que nunca saiu do chão, e a razão da falha seria invisível.
fn run(corner_reach: f32, ceiling_bottom: Option<f32>) -> (bool, bool) {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    if let Some(b) = ceiling_bottom {
        ceiling(&mut sim, b);
    }
    let who = subject_tuned(&mut sim, true, FLOAT, Some(jumper(corner_reach)));
    let mut bridge = PhysicsBridge::new();

    let mut bonked = false;
    let mut rose = false;
    let start = scene::y_of(&sim, "Subject");
    for t in 1..=120u64 {
        // Segura o pulo nos primeiros tiques: o buffer/coyote resolve o resto.
        bridge.set_player_input(
            who,
            PlayerInput {
                jump: t <= 8,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        if let Some(v) = bridge.player_view(who) {
            bonked |= v.ceiling;
        }
        rose |= scene::y_of(&sim, "Subject") > start + 0.2;
    }
    (bonked, rose)
}

/// **O FATO SOBREVIVE À ASSISTÊNCIA DE QUINA ESTAR DESLIGADA** — o gate da wave.
///
/// ⚠️ **Mutação que deve sangrar:** dar um knob ao [`ceiling_fact_wanted`] (por
/// exemplo, fazê-lo delegar ao `corner_probe_wanted`). O braço de cima continua
/// verde — é a configuração que shipa, onde as duas condições coincidem — e o de
/// baixo fica vermelho, que é exactamente o *"funciona às vezes"* que o campo
/// existe para não ser.
#[test]
fn the_ceiling_fact_survives_the_corner_assist_being_off() {
    // Com a quina ARMADA (o ponto de partida do produto).
    let (bonked_on, rose_on) = run(0.12, Some(1.9));
    assert!(rose_on, "o controle: ele tem de sair do chao");
    assert!(bonked_on, "com a quina armada o teto tem de ser relatado");

    // Com a quina DESARMADA — o mesmo mundo, a mesma subida.
    let (bonked_off, rose_off) = run(0.0, Some(1.9));
    assert!(rose_off, "o controle: ele tem de sair do chao");
    assert!(
        bonked_off,
        "um FATO nao e' opt-in: desligar a assistencia de quina nao pode apagar \
         o `is_on_ceiling`"
    );
}

/// **Sem teto não há batida** — o controle que impede o bit de nascer `true`.
#[test]
fn a_clear_sky_is_never_a_ceiling() {
    let (bonked, rose) = run(0.12, None);
    assert!(rose, "o controle: ele tem de sair do chao");
    assert!(!bonked, "sem teto nenhum o bit nao pode acender");
}

/// **De pé sob uma marquise NÃO é teto** — a fronteira contra o [`Headroom`].
///
/// ⚠️ Esta é a metade que separa as duas perguntas. *Falta de espaço para
/// levantar* é do agachar e é verdadeira o tempo todo num corredor baixo;
/// *bateu com a cabeça* é um evento da SUBIDA. Colapsá-las daria um
/// `is_on_ceiling` aceso permanentemente, que não é um evento de batida.
#[test]
fn standing_under_an_overhang_is_not_a_ceiling() {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    // Baixo o bastante para o `Headroom` reclamar, e o personagem NUNCA pula.
    ceiling(&mut sim, FLOAT + 0.6);
    let who = subject_tuned(&mut sim, true, FLOAT, Some(jumper(0.12)));
    let mut bridge = PhysicsBridge::new();

    for t in 1..=90u64 {
        bridge.set_player_input(who, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
        if let Some(v) = bridge.player_view(who) {
            assert!(
                !v.ceiling,
                "parado no chao sob uma marquise ele nao esta' a bater em nada \
                 (tique {t})"
            );
        }
    }
}

// ── E O QUE O TETO PUBLICA: `PlayerEvent::Bonked` ────────────────────────────
//
// ⚠️ **Estes gates moram AQUI, na ponte, e não ao lado do `events_between`:** o
// `speed` que o evento carrega é a velocidade de subida no tique em que o
// SENSOR relatou o bloqueio, e um teste que soletrasse as duas vistas à mão
// estaria a afirmar o número que ele próprio escreveu. O oráculo é um teto de
// verdade e uma cabeça a subir contra ele.

/// **Uma cabeça que bate publica UM evento, com a velocidade que ela trazia.**
///
/// Medido na cena: o topo do collider chega a 1,4 mm do teto no tique 8 com
/// `vy = 3,8712`, e no tique seguinte percorre **0,6 mm** em vez dos 65 mm que a
/// velocidade pedia — é uma batida, não um quase.
///
/// ⚠️ **Mutação que deve sangrar:** derivar o evento do NÍVEL do bit em vez da
/// BORDA (`after.ceiling` sem o `&& !before.ceiling`). O bit fica de pé nos
/// tiques 8 **e** 9 — ele só se apaga quando a subida morre —, então o nível
/// publica DOIS eventos para uma batida, e um consumidor de som toca duas vezes.
#[test]
fn the_head_that_hits_publishes_one_bonk_carrying_the_speed_it_climbed_with() {
    let (bonks, rose) = bonks(0.12, Some(1.9));
    assert!(rose, "o controle: ele tem de sair do chao");
    assert_eq!(bonks.len(), 1, "uma batida, um evento: {bonks:?}");
    assert!(
        (3.0..5.0).contains(&bonks[0]),
        "a velocidade publicada nao e' a subida que ele trazia: {:?}",
        bonks[0]
    );
}

/// **Sem teto não há batida** — o controle que impede o evento de nascer de um
/// bit que acende sozinho.
#[test]
fn a_clear_sky_never_publishes_a_bonk() {
    let (bonks, rose) = bonks(0.12, None);
    assert!(rose, "o controle: ele tem de sair do chao");
    assert!(bonks.is_empty(), "sem teto nenhum: {bonks:?}");
}

/// **A batida é SILENCIOSA na margem, e isso é medição, não desenho.**
///
/// O sensor olha um tique à frente (`rel_up * dt`), então há uma janela em que
/// ele diz *bloqueado* e o contato é marginal — a pergunta honesta é **quão
/// alto** o evento grita ali. A varredura de altura de teto responde: a
/// velocidade publicada desce continuamente até o teto sair do alcance, e o
/// último evento antes do corte carrega ~9% de uma batida cheia.
///
/// | teto | m/s   |   | teto | m/s   |
/// |------|-------|---|------|-------|
/// | 1,80 | 4,035 |   | 2,05 | 1,746 |
/// | 1,90 | 3,871 |   | 2,10 | 0,847 |
/// | 2,00 | 3,054 |   | 2,15 | 0,438 |
/// |      |       |   | 2,20 | —     |
///
/// ⚠️ **E o corte cai onde a GEOMETRIA manda:** o ápice livre põe o topo do
/// collider em 2,1715, e o primeiro teto sem batida é 2,20 — não há evento com
/// o teto fora de alcance. Reproduzir: `measure_the_bonk_across_ceiling_heights`.
#[test]
fn the_bonk_fades_to_nothing_as_the_ceiling_leaves_reach() {
    let hard = bonks(0.12, Some(1.8)).0;
    let graze = bonks(0.12, Some(2.15)).0;
    let clear = bonks(0.12, Some(2.20)).0;
    assert_eq!(hard.len(), 1, "{hard:?}");
    assert_eq!(graze.len(), 1, "{graze:?}");
    assert!(clear.is_empty(), "teto fora de alcance: {clear:?}");
    assert!(
        graze[0] < hard[0] * 0.25,
        "a margem tinha de ser silenciosa: {graze:?} contra {hard:?}"
    );
}

/// A SONDA da tabela acima — varre a altura do teto e imprime o que sai.
///
/// `cargo test -p ph2d-physics-ecs --test player_ceiling measure_the_bonk -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_bonk_across_ceiling_heights() {
    println!("teto | bonk m/s");
    let mut b = 1.80f32;
    while b <= 2.40 {
        match bonks(0.12, Some(b)).0.first() {
            Some(s) => println!("{b:.2} | {s:8.4}"),
            None => println!("{b:.2} |        -"),
        }
        b += 0.05;
    }
}

/// Corre a cena e devolve `(velocidades das batidas, subiu alguma vez)`.
fn bonks(corner_reach: f32, ceiling_bottom: Option<f32>) -> (Vec<f32>, bool) {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    if let Some(b) = ceiling_bottom {
        ceiling(&mut sim, b);
    }
    let who = subject_tuned(&mut sim, true, FLOAT, Some(jumper(corner_reach)));
    let mut bridge = PhysicsBridge::new();

    let mut out = Vec::new();
    let mut rose = false;
    let start = scene::y_of(&sim, "Subject");
    for t in 1..=120u64 {
        bridge.set_player_input(
            who,
            PlayerInput {
                jump: t <= 8,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        for (_, ev) in bridge.player_events() {
            if let ph2d_platformer::PlayerEvent::Bonked { speed } = ev {
                out.push(*speed);
            }
        }
        rose |= scene::y_of(&sim, "Subject") > start + 0.2;
    }
    (out, rose)
}

/// **E a batida chega ao BARRAMENTO com nome** — a quarta condição da política
/// de UI do plano (*a sequência leva a algum lugar*).
///
/// ⚠️ Sem este gate a wave entregaria um evento que ninguém pode ouvir: o
/// `player_signal_name` é um `match` exaustivo, então o compilador obriga a
/// nomear o variant novo — e um nome que ninguém verifica pode estar escrito e
/// não sair, porque a saída é **opt-in por-player** (`PlayerSignals`) e o opt-in
/// é a metade que um `match` não pode garantir.
#[test]
fn the_bonk_reaches_the_signal_bus_with_a_name() {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    ceiling(&mut sim, 1.9);
    let who = subject_tuned(&mut sim, true, FLOAT, Some(jumper(0.12)));
    sim.world_mut()
        .entity_mut(who)
        .insert(ph2d_physics_ecs::PlayerSignals);
    let mut bridge = PhysicsBridge::new();

    let mut names: Vec<String> = Vec::new();
    for t in 1..=120u64 {
        bridge.set_player_input(
            who,
            PlayerInput {
                jump: t <= 8,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        names.extend(bridge.signal_events(&sim).into_iter().map(|s| s.name));
    }
    assert_eq!(
        names.iter().filter(|n| *n == "player.bonked").count(),
        1,
        "a batida tem de sair pelo barramento, uma vez: {names:?}"
    );
}
