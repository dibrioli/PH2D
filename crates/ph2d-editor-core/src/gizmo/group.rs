//! **Um membro de uma selecção a rodar ou escalar em grupo** — o alvo de MUNDO a partir do próprio
//! início, de volta a LOCAL contra o pai de agora. Saiu da shell (`ramo_gizmo_escrita_grupo`) para ter
//! gate contra o motor.
//!
//! ⭐ **O SKEW sob uma escala de mundo não uniforme** (onda 6, doc ObjectsAndComponents 01 §16): o
//! motor desenha `R·Sk·S`; com `R = I`, `D·Sk·S = Sk'·S'` exactamente quando `tan k'ₓ = tan kₓ·fₓ/f_y`
//! e `tan k'_y = tan k_y·f_y/fₓ`. Sem isto a imagem inclinada esticada na horizontal mantinha o ângulo
//! — e o desenho deixava de ser o que o grupo pediu. Rodada (`R ≠ I`), o chamador já força a escala
//! uniforme, e `fₓ = f_y` deixa o skew intacto.

use super::{TransformSnapshot, compose_snapshot, world_translation_to_local};

/// O que se escreve no `Transform` local de um membro.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MemberPose {
    pub translation: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
    pub skew: [f32; 2],
}

/// O que o arrasto do grupo pede a todos os membros.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GroupMotion {
    pub pivot: [f32; 2],
    pub delta_rot: f32,
    pub factor: [f32; 2],
    /// Gizmo GLOBAL (órbita em torno do pivô partilhado, escala nos eixos do MUNDO) ou cada um no seu.
    pub global: bool,
}

/// O skew de MUNDO depois de a escala de mundo passar de `now` a `new` nos eixos do mundo (`R = I`).
/// Skew nulo ⇒ intacto, bit a bit.
#[must_use]
pub fn skew_under_world_scale(skew: [f32; 2], now: [f32; 2], new: [f32; 2]) -> [f32; 2] {
    if skew == [0.0, 0.0] {
        return skew;
    }
    let razao = |a: f32, b: f32| if a.abs() > 1e-6 { b / a } else { 1.0 };
    let (fx, fy) = (razao(now[0], new[0]), razao(now[1], new[1]));
    [
        libm::atanf(libm::tanf(skew[0]) * fx / fy),
        libm::atanf(libm::tanf(skew[1]) * fy / fx),
    ]
}

/// A pose local de um membro: `start_*` o seu início, `live_parent`/`parent_skew` o pai de AGORA
/// (um antepassado seleccionado já pode ter sido escrito neste quadro), `live_local_*` o seu local de
/// agora (o skew lê-se do vivo: `tan k·s_y/sₓ` é invariante sob a escala de mundo, sem estado guardado).
#[must_use]
pub fn group_member_pose(
    start_local: TransformSnapshot,
    start_parent: TransformSnapshot,
    live_parent: TransformSnapshot,
    parent_skew: [f32; 2],
    live_local: (TransformSnapshot, [f32; 2]),
    m: GroupMotion,
) -> MemberPose {
    let start_world = compose_snapshot(start_parent, start_local);
    let target_scale = [
        start_world.scale[0] * m.factor[0],
        start_world.scale[1] * m.factor[1],
    ];
    let target_translation = if m.global {
        // T1.3.5 cross-OS bit-identical.
        let (sin_d, cos_d) = libm::sincosf(m.delta_rot);
        let rel_x = (start_world.translation[0] - m.pivot[0]) * m.factor[0];
        let rel_y = (start_world.translation[1] - m.pivot[1]) * m.factor[1];
        [
            m.pivot[0] + rel_x * cos_d - rel_y * sin_d,
            m.pivot[1] + rel_x * sin_d + rel_y * cos_d,
        ]
    } else {
        start_world.translation
    };
    let ps = |v: f32| if v.abs() > 1e-6 { v } else { 1.0 };
    let (live_scale, live_skew) = (live_local.0.scale, live_local.1);
    let skew = if m.global && live_skew != [0.0, 0.0] {
        let mundo = [parent_skew[0] + live_skew[0], parent_skew[1] + live_skew[1]];
        let agora = [
            live_parent.scale[0] * live_scale[0],
            live_parent.scale[1] * live_scale[1],
        ];
        let novo = skew_under_world_scale(mundo, agora, target_scale);
        [novo[0] - parent_skew[0], novo[1] - parent_skew[1]]
    } else {
        live_skew
    };
    MemberPose {
        translation: world_translation_to_local(live_parent, target_translation),
        rotation: start_world.rotation + m.delta_rot - live_parent.rotation,
        scale: [
            target_scale[0] / ps(live_parent.scale[0]),
            target_scale[1] / ps(live_parent.scale[1]),
        ],
        skew,
    }
}

#[cfg(test)]
#[path = "group_tests.rs"]
mod tests;
