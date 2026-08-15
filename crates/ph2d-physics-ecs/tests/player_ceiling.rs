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
