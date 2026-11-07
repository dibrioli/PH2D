//! «Em uso no jogo» (ficha f1, onda 4): no nível de `PH2D_GAME_SMOKE=4`, a plataforma alargada por Width continua a
//! segurar o Player, pela mesma porta que o Inspector usa; o Player cai onde ela não chega sem o alargamento.

use ph2d_app_sprite_anim::tiled::set_tiled_side;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
use ph2d_render::HERO_SHEET_SOLE_BELOW_CENTRE;

/// «Platform Low» vai de 2,3 a 4,7 m, com o topo a 1,1 m; o chão da direita começa aos 5 m.
const FORA_DA_ORIGINAL: f32 = 5.5;
const TOPO: f32 = 1.1;

fn nivel_4(width: Option<f32>) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let jogador = crate::comp::montar(&mut sim, 4);
    let plataforma = ph2d_app_game::smoke::por_nome(&mut sim, "Platform Low").expect("no nível");
    if let Some(w) = width {
        assert!(set_tiled_side(sim.world_mut(), plataforma, 0, w));
    }
    sim.world_mut()
        .get_mut::<Transform>(jogador)
        .expect("pose")
        .translation = Vec2::new(FORA_DA_ORIGINAL, 3.0);
    (sim, jogador)
}

fn corre(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    jogador: Entity,
    de: u64,
    n: u64,
    drive: f32,
) -> Vec2 {
    for tick in de..de + n {
        bridge.set_player_input(
            jogador,
            PlayerInput {
                drive,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(sim, true, tick);
    }
    sim.world()
        .get::<Transform>(jogador)
        .expect("pose")
        .translation
}

#[test]
fn a_plataforma_alargada_por_width_segura_o_player_e_ele_anda_nela() {
    let em_cima = TOPO + HERO_SHEET_SOLE_BELOW_CENTRE;
    let (mut sem, jogador) = nivel_4(None);
    let p = corre(&mut sem, &mut PhysicsBridge::new(), jogador, 1, 180, 0.0);
    assert!(
        p.y < em_cima - 0.5,
        "controlo: sem alargar, cai ao chão: {p:?}"
    );

    let (mut sim, jogador) = nivel_4(Some(6.0));
    let mut bridge = PhysicsBridge::new();
    let p = corre(&mut sim, &mut bridge, jogador, 1, 180, 0.0);
    assert!((p.y - em_cima).abs() < 0.1, "pousou: {p:?}");
    // Anda para a esquerda até ao que já era plataforma, sem cair na emenda.
    let p = corre(&mut sim, &mut bridge, jogador, 181, 40, -1.0);
    assert!(p.x < 4.7, "andou até à plataforma de antes: {p:?}");
    assert!((p.y - em_cima).abs() < 0.1, "continua em cima: {p:?}");
}
