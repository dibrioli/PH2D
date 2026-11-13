//! **O PLAYER SNAP ASSENTA ONDE A LEI JÁ O VÊ DE PÉ** (Bug #13, `docs/Physics/BUGS_physics.md`).
//!
//! Sob Snap a lei diz *«no chão»* pelo `footing` (a perna, `float_height + cling_distance`) e o
//! integrador pelo toque do controlador (`was.kin.grounded`). Na faixa entre os dois o corpo caía em
//! queda livre com a lei a vê-lo de pé: o pulo levava `v0 − rel_up(0)` somado a uma velocidade que
//! ainda descia (H = 1,0 premido no tique 20: 0,357 m). A cura tem duas metades que só valem juntas:
//! o integrador parte da velocidade que a lei julgou (`kin_in`) e o corpo ENCAIXA no chão nesse tique
//! (`seat`). O encaixe é o `apply_floor_snap` do Godot (`floor_snap_length` + `floor_stop_on_slope`):
//! um SEGUNDO movimento, depois do pedido, só na direção da gravidade, sem autostep, sem empurrão, que
//! só conta se bateu em chão caminhável, e do qual só fica a componente em `up`. Os três vermelhos
//! do primeiro desenho (o vão somado ao PRÓPRIO pedido) eram as três coisas que o segundo movimento
//! não faz: deslizar ao longo da rampa, cancelar o autostep e empurrar o chão.

use ph2d_physics::{CharacterHit, CharacterParams, PhysicsWorld};
use ph2d_platformer::{GroundSample, KinematicState, RideConfig};

use super::kinmove::KinMove;
use crate::PhysicsSettings;

/// **Encaixa este tique?** Ao NASCER, ou num POUSO: a lei vê chão agora (`stand`), não me via de
/// pé no tique anterior (`was.stood`) e o controlador não tocou. Uma elevação do próprio controlador
/// (o lábio de um degrau) chega com `was.stood` e não encaixa — puxá-la de volta ao chão de baixo a
/// cada tique era o degrau que nunca se subia. Nem o coyote serve de régua: com `coyote = 0` não há
/// janela e o pouso ficaria mudo.
pub(super) fn lands(born: bool, was: KinematicState, stand: Option<&GroundSample>) -> bool {
    born || (stand.is_some() && !was.grounded && !was.stood)
}

/// A velocidade de onde o integrador parte: no tique do encaixe, a da lei (`vel`, já sem a queda que
/// o chão segura); senão a dele. Só nesse tique — repô-la a cada tique da faixa deixava o corpo a
/// descer `g·dt²` por tique até o controlador tocar (2,7 mm, ~75 tiques numa rampa de 30°).
pub(super) fn kin_in(was: KinematicState, vel: [f32; 2], lands: bool) -> KinematicState {
    if lands {
        KinematicState {
            velocity: vel,
            ..was
        }
    } else {
        was
    }
}

/// Quanto o encaixe pode descer (o `floor_snap_length`), ou nada; nunca a subir. Ao nascer, a perna
/// AUTORADA (`born` = `float_height + cling_distance`, a da Dynamic) menos o pé. No pouso, o vão que
/// o sensor viu MAIS a `cling_distance`: os raios dos pés medem o lado MAIS ALTO de uma rampa e o
/// centro da cápsula fica mais fundo (30°: o sensor dizia 0,204 m, o corpo descia 0,266); e é a
/// `cling` autorada — não a perna da Dynamic — que diz quanto ele cola (com `cling` 0,02 o salto
/// da descida da rampa tem de continuar a existir). O movimento pára no primeiro chão.
pub(super) fn reach(
    born: Option<f32>,
    lands: bool,
    stand: Option<&GroundSample>,
    ride: &RideConfig,
    wanted: [f32; 2],
) -> Option<f32> {
    let up = super::UP;
    if !lands || wanted[0] * up[0] + wanted[1] * up[1] > 0.0 {
        return None;
    }
    let foot = ride.float_height; // sob Snap o `float_height` é o pé
    let gap = match (born, stand) {
        (Some(leg), _) => leg - foot,
        (None, Some(s)) => s.distance - foot + ride.cling_distance,
        (None, None) => return None,
    };
    (gap > 0.0).then_some(gap)
}

/// O encaixe, depois do pedido (`from` = o que o controlador deu): desce `m.seat` na direção da
/// gravidade e devolve o que acrescentar, só em `up`, se bateu em chão que a lei aceita.
pub(super) fn seat(
    world: &PhysicsWorld,
    m: &KinMove,
    from: [f32; 2],
    hits: &mut Vec<CharacterHit>,
) -> Option<[f32; 2]> {
    let len = m.seat?;
    let up = m.params.up;
    let params = CharacterParams {
        snap_distance: 0.0,
        step_height: 0.0,
        ..m.params
    };
    let got = world.move_character_from(
        m.handle,
        from,
        [-up[0] * len, -up[1] * len],
        params,
        m.passing,
        m.layer,
        hits,
    );
    // O sentido da normal não é contrato (`CharacterHit::normal`): a LINHA dela; a descer, é chão.
    let floor = libm::cosf(m.params.max_slope_deg.to_radians());
    hits.iter()
        .any(|h| (h.normal[0] * up[0] + h.normal[1] * up[1]).abs() >= floor)
        .then(|| {
            let d = got.translation[0] * up[0] + got.translation[1] * up[1];
            [up[0] * d, up[1] * d]
        })
}

/// O encaixe achou chão que a LEI não via (`seated` = encaixou e `!stood`: o nascido fora da faixa
/// da perna): a velocidade contra ele acaba ali (o `velocity.slide(floor_normal)` do
/// `move_and_slide`), só a componente que desce em `up`. Sem isto o nascido a 0,9 pousava com a
/// queda do tique 1 guardada, e o pulo premido no tique 2 subia 0,966 a H = 1. Onde a lei já via o
/// chão a velocidade dela já o descontou (`kin_in`).
pub(super) fn rest(kin: KinematicState, seated: bool, up: [f32; 2]) -> KinematicState {
    let along = kin.velocity[0] * up[0] + kin.velocity[1] * up[1];
    if !seated || along >= 0.0 {
        return kin;
    }
    KinematicState {
        velocity: [
            kin.velocity[0] - up[0] * along,
            kin.velocity[1] - up[1] * along,
        ],
        ..kin
    }
}

/// Os passos de Euler por tique de quem integra o pulo: o mundo VIVO no dinâmico; UM sob Snap, porque
/// ali quem integra é o `kinematic_advance` (`v += g·dt; x += v·dt`, uma vez por tique) — com os 16 do
/// mundo o `launch_speed` modelava outro integrador e o ápice saía 0,956 m a H = 1,0.
pub(super) fn world_steps(writes_own_pose: bool, vivo: &PhysicsSettings) -> u32 {
    if writes_own_pose {
        1
    } else {
        crate::world_steps(vivo)
    }
}
