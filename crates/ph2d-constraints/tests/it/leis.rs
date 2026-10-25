//! As leis que o oráculo não cobre: o espelho, o desvio guardado, o espaço local e as pontas da força.

use ph2d_constraints::{
    Canais, CopyTransform, Ctx, Inerte, Kind, LookAt, Mix, Pose2, Space, apply, blend,
};

fn at(x: f64, y: f64, rot_deg: f64, sx: f64, sy: f64) -> Pose2 {
    Pose2 {
        x,
        y,
        rot: rot_deg.to_radians(),
        sx,
        sy,
    }
}

#[track_caller]
fn perto(a: Pose2, b: Pose2) {
    let d = [
        a.x - b.x,
        a.y - b.y,
        ph2d_constraints::wrap_pi(a.rot - b.rot),
        a.sx - b.sx,
        a.sy - b.sy,
    ];
    assert!(d.iter().all(|v| v.abs() < 1e-9), "{a:?}\n{b:?}");
}

#[test]
fn the_ends_of_the_influence_are_exact_bits() {
    let a = at(1.3, -2.7, 33.0, 1.1, -0.9);
    let b = at(-4.0, 9.1, -151.0, 2.5, -3.0);
    assert_eq!(blend(a, b, 0.0), a);
    assert_eq!(blend(a, b, 1.0), b);
    assert_eq!(blend(a, b, f64::NAN), a);
}

#[test]
fn a_mirrored_owner_aims_without_turning_half_a_circle() {
    let espelhado = Pose2 {
        sx: -1.0,
        ..Pose2::IDENTITY
    };
    let dono = Pose2::IDENTITY.under(espelhado);
    assert!(dono.sy < 0.0, "o espelho vai para o sinal de sy: {dono:?}");
    let alvo = at(3.0, 4.0, 0.0, 1.0, 1.0);
    let c = Ctx {
        owner: dono,
        parent: espelhado,
        target: Some(alvo),
        target_parent: Pose2::IDENTITY,
    };
    let r = apply(&Kind::LookAt(LookAt::default()), &c, 1.0).unwrap();
    let eixo = r.to_xform().apply_vec([1.0, 0.0]);
    assert!((eixo[1].atan2(eixo[0]) - 4.0_f64.atan2(3.0)).abs() < 1e-12);
    assert!(r.to_xform().det() < 0.0, "continua espelhado");
}

#[test]
fn keep_current_makes_every_targeted_kind_a_no_op() {
    let pai = at(2.0, 1.0, 25.0, 1.5, 1.5);
    let dono = at(-1.0, 3.0, 70.0, 0.8, 1.2);
    let alvo = at(4.0, -2.0, -40.0, 2.0, 0.5);
    let c = Ctx {
        owner: dono,
        parent: pai,
        target: Some(alvo),
        target_parent: pai,
    };
    let tipos = [
        Kind::ChildOf(Default::default()),
        Kind::CopyTransform(CopyTransform {
            space: Space::Local,
            ..Default::default()
        }),
        Kind::CopyTransform(CopyTransform {
            mix: Mix::Add,
            canais: Canais::ROTATION,
            ..Default::default()
        }),
        Kind::LookAt(LookAt::default()),
        Kind::Distance(Default::default()),
    ];
    for mut k in tipos {
        k.keep_current(&c).unwrap();
        perto(apply(&k, &c, 1.0).unwrap(), dono);
    }
}

#[test]
fn a_kind_without_its_target_says_so_and_moves_nothing() {
    let c = Ctx {
        owner: Pose2::IDENTITY,
        parent: Pose2::IDENTITY,
        target: None,
        target_parent: Pose2::IDENTITY,
    };
    assert_eq!(
        apply(&Kind::LookAt(LookAt::default()), &c, 1.0),
        Err(Inerte::SemAlvo)
    );
}

#[test]
fn local_copy_reads_the_targets_own_parent() {
    let pai_alvo = at(10.0, 0.0, 90.0, 1.0, 1.0);
    let alvo = at(1.0, 0.0, 15.0, 1.0, 1.0).under(pai_alvo);
    let c = Ctx {
        owner: Pose2::IDENTITY,
        parent: Pose2::IDENTITY,
        target: Some(alvo),
        target_parent: pai_alvo,
    };
    let k = Kind::CopyTransform(CopyTransform {
        space: Space::Local,
        ..Default::default()
    });
    perto(apply(&k, &c, 1.0).unwrap(), at(1.0, 0.0, 15.0, 1.0, 1.0));
}
