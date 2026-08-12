//! Os gates da cena 109 (`W-Probes2`) — os NÚMEROS que a mensagem imprime,
//! afirmados antes de o artista os ler.
//!
//! ⚠️ **A cena inteira é um contraste**, então o gate tem de correr os DOIS
//! lados: um gate que só afirmasse *"o da direita fica de pé"* passaria numa
//! cena sem fenda nenhuma.

use super::{FAN_X, FLOAT, GAP_NARROW, GAP_WIDE, ONE_RAY_X, build_foot_fan_scene};
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// Meia-largura do corpo das cenas de player (o `radius` da cápsula).
const BODY_HALF_WIDTH: f32 = 0.2;

/// Onde os dois assentam depois de `ticks` tiques sem entrada nenhuma.
fn settle(ticks: u64) -> (f32, f32) {
    let mut sim = SimWorld::new();
    let (one, fan) = build_foot_fan_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    for i in 1..=ticks {
        bridge.set_player_input(one, PlayerInput::default());
        bridge.set_player_input(fan, PlayerInput::default());
        bridge.dispatch(&mut sim, true, i);
    }
    let y = |e| {
        sim.world()
            .get::<Transform>(e)
            .expect("transform")
            .translation
            .y
    };
    (y(one), y(fan))
}

/// **A aritmética que a mensagem imprime está certa** — em tempo de compilação,
/// que é onde constantes se conferem melhor, e antes.
#[test]
fn the_scene_delivers_the_numbers_its_message_prints() {
    // A fenda estreita é atravessada pelo corpo: os pés de fora acham chão.
    const _: () = assert!(GAP_NARROW * 0.5 < BODY_HALF_WIDTH);
    // A larga não: nem os pés de fora alcançam.
    const _: () = assert!(GAP_WIDE * 0.5 > BODY_HALF_WIDTH);
    // ⚠️ E os dois personagens ficam em fendas SEPARADAS — se as posições se
    // aproximassem, os trechos de chão entre elas desapareceriam e a cena
    // deixaria de conter o contraste que ela existe para mostrar.
    assert!(
        FAN_X - ONE_RAY_X > GAP_NARROW * 2.0,
        "as duas fendas tem de ter chao entre elas: {ONE_RAY_X} .. {FAN_X}"
    );
}

/// **O da direita fica de pé; o da esquerda afunda** — o contraste que a cena É.
#[test]
fn the_fan_stands_where_the_single_ray_sinks() {
    let (one, fan) = settle(240);
    let rest = FLOAT;
    let fan_dip = rest - fan;
    let one_dip = rest - one;
    assert!(
        fan_dip < 0.01,
        "o de TRES raios tem de ficar de pe' sobre uma fenda que o corpo \
         atravessa: afundou {fan_dip:.3} m (y = {fan:.3})"
    );
    assert!(
        one_dip > 0.3,
        "o de UM raio e' o CONTROLE da cena, e ele tem de afundar: \
         {one_dip:.3} m (y = {one:.3})"
    );
}
