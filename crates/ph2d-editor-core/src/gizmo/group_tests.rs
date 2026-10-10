//! Gates do membro de grupo contra o MOTOR (`ph2d_ecs::GlobalTransform::from_transform` sobre
//! `Transform::compose`): esticar o grupo nos eixos do mundo estica o DESENHO de cada membro.

use super::super::TransformSnapshot;
use super::*;
use ph2d_ecs::{GlobalTransform, Transform};

fn pose(t: [f32; 2], s: [f32; 2], skew: [f32; 2]) -> Transform {
    let mut x = Transform::IDENTITY;
    x.translation.x = t[0];
    x.translation.y = t[1];
    x.scale.x = s[0];
    x.scale.y = s[1];
    x.skew_x = skew[0];
    x.skew_y = skew[1];
    x
}

fn snap(t: &Transform) -> TransformSnapshot {
    TransformSnapshot {
        translation: [t.translation.x, t.translation.y],
        rotation: t.rotation,
        scale: [t.scale.x, t.scale.y],
    }
}

fn aplica(t: &Transform, p: MemberPose) -> Transform {
    let mut x = *t;
    x.translation.x = p.translation[0];
    x.translation.y = p.translation[1];
    x.rotation = p.rotation;
    x.scale.x = p.scale[0];
    x.scale.y = p.scale[1];
    x.skew_x = p.skew[0];
    x.skew_y = p.skew[1];
    x
}

/// Os quatro cantos `(±1, ±0.5)` desenhados.
fn cantos(t: Transform) -> [[f32; 2]; 4] {
    let a = GlobalTransform::from_transform(t).affine();
    [(-1.0, -0.5), (1.0, -0.5), (-1.0, 0.5), (1.0, 0.5)]
        .map(|(x, y): (f32, f32)| [a[0] * x + a[2] * y + a[4], a[1] * x + a[3] * y + a[5]])
}

fn esticado(c: [[f32; 2]; 4], m: GroupMotion) -> [[f32; 2]; 4] {
    c.map(|p| {
        [
            m.pivot[0] + (p[0] - m.pivot[0]) * m.factor[0],
            m.pivot[1] + (p[1] - m.pivot[1]) * m.factor[1],
        ]
    })
}

fn iguais(a: [[f32; 2]; 4], b: [[f32; 2]; 4], tol: f32) -> bool {
    a.iter()
        .zip(b)
        .all(|(p, q)| (p[0] - q[0]).abs() < tol && (p[1] - q[1]).abs() < tol)
}

const M: GroupMotion = GroupMotion {
    pivot: [0.5, -1.0],
    delta_rot: 0.0,
    factor: [1.6, 0.8],
    global: true,
};

/// ⭐⭐ Uma imagem inclinada num grupo esticado só na horizontal: o desenho dela é o desenho
/// esticado. (Controlo: com o skew de antes — o caminho velho — os cantos fogem.)
#[test]
fn a_skewed_member_of_a_stretched_group_is_the_stretched_drawing() {
    let t0 = pose([2.0, 1.0], [1.5, 0.8], [0.4, -0.25]);
    let s = snap(&t0);
    let id = TransformSnapshot::IDENTITY;
    let p = group_member_pose(s, id, id, [0.0, 0.0], (s, [t0.skew_x, t0.skew_y]), M);
    let alvo = esticado(cantos(t0), M);
    assert!(iguais(cantos(aplica(&t0, p)), alvo, 1e-4));
    let velho = MemberPose {
        skew: [t0.skew_x, t0.skew_y],
        ..p
    };
    assert!(
        !iguais(cantos(aplica(&t0, velho)), alvo, 1e-2),
        "o caso tem de medir o skew"
    );
}

/// Sob um pai escalado `(2, 1)`: o skew de mundo é o do filho, a escala de mundo é o produto.
#[test]
fn under_a_scaled_parent_the_drawing_is_stretched_too() {
    let pai = pose([1.0, 0.0], [2.0, 1.0], [0.0, 0.0]);
    let filho = pose([0.5, 0.5], [0.7, 1.2], [0.3, 0.2]);
    let (sp, sf) = (snap(&pai), snap(&filho));
    let p = group_member_pose(sf, sp, sp, [0.0, 0.0], (sf, [0.3, 0.2]), M);
    let alvo = esticado(cantos(Transform::compose(pai, filho)), M);
    let depois = Transform::compose(pai, aplica(&filho, p));
    assert!(iguais(cantos(depois), alvo, 1e-4));
}

/// Sem skew a conta é a de sempre (a que vivia na shell), bit a bit, e o skew fica `[0, 0]`.
#[test]
fn without_skew_the_member_is_the_old_one_bit_for_bit() {
    let t0 = pose([2.0, 1.0], [1.5, 0.8], [0.0, 0.0]);
    let s = snap(&t0);
    let id = TransformSnapshot::IDENTITY;
    let m = GroupMotion {
        delta_rot: 0.3,
        ..M
    };
    let p = group_member_pose(s, id, id, [0.0, 0.0], (s, [0.0, 0.0]), m);
    let (sin_d, cos_d) = libm::sincosf(0.3);
    let (rx, ry) = ((2.0 - 0.5) * 1.6_f32, (1.0 + 1.0) * 0.8_f32);
    assert_eq!(
        p.translation[0].to_bits(),
        (0.5 + rx * cos_d - ry * sin_d).to_bits()
    );
    assert_eq!(p.rotation.to_bits(), (0.0_f32 + 0.3 - 0.0).to_bits());
    assert_eq!(p.scale, [1.5 * 1.6, 0.8 * 0.8]);
    assert_eq!(p.skew, [0.0, 0.0]);
}

/// O skew lê-se do VIVO: quadro a quadro (1,2 e depois 1,6 a partir do início) dá o mesmo que direto
/// a 1,6 — o invariante não acumula erro de um quadro para o outro.
#[test]
fn frame_by_frame_is_the_same_as_one_jump() {
    let t0 = pose([2.0, 1.0], [1.5, 0.8], [0.4, -0.25]);
    let s = snap(&t0);
    let id = TransformSnapshot::IDENTITY;
    let direto = group_member_pose(s, id, id, [0.0, 0.0], (s, [0.4, -0.25]), M);
    let meio = GroupMotion {
        factor: [1.2, 0.9],
        ..M
    };
    let q1 = group_member_pose(s, id, id, [0.0, 0.0], (s, [0.4, -0.25]), meio);
    let t1 = aplica(&t0, q1);
    let q2 = group_member_pose(s, id, id, [0.0, 0.0], (snap(&t1), q1.skew), M);
    for k in 0..2 {
        assert!(
            (q2.skew[k] - direto.skew[k]).abs() < 1e-5,
            "{q2:?} ≠ {direto:?}"
        );
    }
}

/// Fora do gizmo GLOBAL (cada um no seu pivô, eixos locais) o skew não muda.
#[test]
fn a_local_group_scale_keeps_the_skew() {
    let t0 = pose([2.0, 1.0], [1.5, 0.8], [0.4, -0.25]);
    let s = snap(&t0);
    let id = TransformSnapshot::IDENTITY;
    let m = GroupMotion { global: false, ..M };
    let p = group_member_pose(s, id, id, [0.0, 0.0], (s, [0.4, -0.25]), m);
    assert_eq!(p.skew, [0.4, -0.25]);
}

/// Com um pai INCLINADO o skew de mundo é a soma (o `compose` soma-os): o que a lei estica é o de
/// MUNDO, e o local escrito é esse menos o do pai — o skew composto depois é o da lei.
#[test]
fn under_a_skewed_parent_the_world_skew_is_the_one_the_law_stretches() {
    let pai = pose([0.0, 0.0], [2.0, 1.0], [0.1, 0.05]);
    let filho = pose([0.5, 0.5], [0.7, 1.2], [0.3, 0.2]);
    let (sp, sf) = (snap(&pai), snap(&filho));
    let p = group_member_pose(sf, sp, sp, [0.1, 0.05], (sf, [0.3, 0.2]), M);
    let mundo = Transform::compose(pai, filho);
    let lei = skew_under_world_scale(
        [mundo.skew_x, mundo.skew_y],
        [mundo.scale.x, mundo.scale.y],
        [mundo.scale.x * M.factor[0], mundo.scale.y * M.factor[1]],
    );
    let depois = Transform::compose(pai, aplica(&filho, p));
    assert!((depois.skew_x - lei[0]).abs() < 1e-6 && (depois.skew_y - lei[1]).abs() < 1e-6);
}
