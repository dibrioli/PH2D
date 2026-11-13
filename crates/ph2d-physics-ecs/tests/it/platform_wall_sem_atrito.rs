//! **O personagem não cola na parede** (dono 04/11), com o rapier de verdade.
//!
//! Desde o Bug #14 o corpo de TODO player é sem atrito (`player_sem_atrito`: atrito 0 + `Min`,
//! imposto pela ponte), qualquer que seja o material autorado — antes o padrão (0,5 `Average`)
//! colava (0,08 m em 30 tiques contra 0,90 solto) e o atrito 0 com `Average` ainda freava.

use crate::platform_wall_rig::{Rig, START_Y, into_wall, pose, rig};
use ph2d_physics_ecs::{Collider, CombineRule, MaterialCombine, PlayerInput};

const QUEDA: u64 = 30;
const ASSENTAR: u64 = 180;
const ANDAR: u64 = 45;

fn com_material(friction: f32, regra: CombineRule) -> Rig {
    let mut r = rig(0.0, 0.0);
    let w = r.sim.world_mut();
    w.get_mut::<Collider>(r.player)
        .expect("o player tem collider")
        .friction = friction;
    w.entity_mut(r.player).insert(MaterialCombine {
        friction: regra,
        ..MaterialCombine::default()
    });
    r
}

fn corre(r: &mut Rig, tick: &mut u64, input: PlayerInput, ticks: u64) -> (f32, f32) {
    r.bridge.set_player_input(r.player, input);
    for _ in 0..ticks {
        *tick += 1;
        r.bridge.dispatch(&mut r.sim, true, *tick);
    }
    pose(&r.sim)
}

fn parado() -> PlayerInput {
    PlayerInput {
        drive: 0.0,
        ..into_wall()
    }
}

fn quanto_desce(mut r: Rig, input: PlayerInput) -> f32 {
    START_Y - corre(&mut r, &mut 0, input, QUEDA).1
}

/// Metros andados no chão, para o lado oposto ao da parede.
fn quanto_anda(mut r: Rig) -> f32 {
    let mut t = 0;
    let inicio = corre(&mut r, &mut t, parado(), ASSENTAR).0;
    let longe = PlayerInput {
        drive: -1.0,
        ..parado()
    };
    inicio - corre(&mut r, &mut t, longe, ANDAR).0
}

#[test]
fn sem_atrito_o_personagem_desce_a_parede_como_quem_cai_solto() {
    let padrao = || rig(0.0, 0.0);
    let meio = || com_material(0.0, CombineRule::Average);
    let zero = || com_material(0.0, CombineRule::Min);

    let solto = quanto_desce(zero(), parado());
    let colado = quanto_desce(padrao(), into_wall());
    let so_zero = quanto_desce(meio(), into_wall());
    let com_min = quanto_desce(zero(), into_wall());
    let (anda_padrao, anda_zero) = (quanto_anda(padrao()), quanto_anda(zero()));
    eprintln!(
        "em {QUEDA} tiques: solto {solto:.3} m · padrão {colado:.3} · atrito 0 {so_zero:.3} · \
         atrito 0 + Min {com_min:.3}\nandar {ANDAR} tiques: padrão {anda_padrao:.3} m · sem atrito {anda_zero:.3}"
    );

    for (qual, desce) in [
        ("padrão", colado),
        ("atrito 0 Average", so_zero),
        ("Min", com_min),
    ] {
        assert!(
            (desce - solto).abs() < solto * 0.01,
            "{qual}: o player não raspa — desce {desce} × solto {solto}"
        );
    }
    assert!(
        (anda_zero - anda_padrao).abs() < anda_padrao * 0.02,
        "o andar no chão não muda: {anda_zero} × {anda_padrao}"
    );
}
