//! ⭐ GEF 04 — o pico do pulo com o player REAL na ponte nunca desce quando a Jump Height sobe,
//! cheio ou cortado (botão solto nos tiques 1..5), nas quatro sensações. A lei divide o tique onde
//! a velocidade cruza um limiar de faixa; lida só no início do tique, o pulo cortado fazia serra
//! (1,15 → 0,591 m, 1,20 → 0,493 m). A lei pura tem a varredura a 1 cm (`jump_band_tests`).

use crate::platform_scene::{pose, scene};

use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use ph2d_platformer::feel::Feel;

/// O pico acima do repouso, com o botão preso `held` tiques (`None` = o pulo cheio).
fn pico(feel: Option<Feel>, h: f32, held: Option<u32>) -> f32 {
    let (mut sim, mut bridge, player) = scene(0.0, -20.0);
    {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        if let Some(f) = feel {
            p.set_feel(f);
        }
        p.jump_height = h;
    }
    let mut tick = 0_u64;
    let mut passo = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, jump: bool| {
        bridge.set_player_input(
            player,
            PlayerInput {
                jump,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
        pose(sim).1
    };
    for _ in 0..60 {
        passo(&mut sim, &mut bridge, false);
    }
    let y0 = pose(&sim).1;
    let (mut topo, mut antes) = (y0, y0);
    for k in 0..400_u32 {
        let y = passo(&mut sim, &mut bridge, held.is_none_or(|n| k < n));
        if k > 2 && y < antes {
            break;
        }
        topo = topo.max(y);
        antes = y;
    }
    topo - y0
}

#[test]
fn o_pico_do_jogo_nunca_desce_quando_a_altura_sobe() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        for held in [None, Some(1), Some(2), Some(3), Some(4), Some(5)] {
            let mut antes: Option<(f32, f32)> = None;
            for i in 0..=50 {
                let h = 0.5 + i as f32 * 0.05;
                let top = pico(feel, h, held);
                if let Some((ha, ta)) = antes
                    && top < ta - 1.0e-4
                {
                    falhas.push(format!(
                        "{feel:?} {held:?}: {ha:.2}→{h:.2} pico {ta:.4}→{top:.4}"
                    ));
                }
                antes = Some((h, top));
            }
        }
    }
    assert!(falhas.is_empty(), "o pico do jogo faz serra: {falhas:?}");
}
