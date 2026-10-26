//! As leis da onda 2 contra o Blender 5.2.3 CORRIDO (casos `ik_*`, `stretch_to`, `follow_path` da
//! fixture). As recusas medidas (o stretch que encolhe, a tangente suavizada da poligonal) estão em
//! `docs/BlenderConstraints/02_W2_ossos.md` §1 e têm aqui o gate do lado contrário.

use ph2d_constraints::{
    BendSide, Ctx, FollowPath, Ik, Kind, PathBy, Pose2, StretchTo, Volume, apply_with, solve_chain,
    stretch_factor, wrap_pi,
};
use ph2d_skeleton::bend_side_of;
use serde_json::Value;

const FIXTURE: &str =
    include_str!("../../../../docs/BlenderConstraints/fixtures/blender_5_2_3_constraints_2d.json");
/// O Blender guarda em `f32`: a tolerância das leis fechadas.
const TOL: f64 = 2e-4;
/// O solver IK dele ITERA e deixa resíduo (medido na fixture): a cauda a 1,06e-4 do alvo (`baixo`) e
/// 0,3° de dobra que sobra fora de alcance (`fora`: o cotovelo a 2,7e-3 da recta). A nossa lei é a
/// fechada — por isso, ao alcance, exige-se também a ponta NO alvo.
const TOL_IK: f64 = 3e-3;

fn oraculo() -> Value {
    serde_json::from_str(FIXTURE).expect("a fixture é JSON")
}

fn n(v: &Value, k: &str) -> f64 {
    v[k].as_f64().expect(k)
}

fn ponto(v: &Value, pre: &str) -> [f64; 2] {
    [n(v, &format!("{pre}_x")), n(v, &format!("{pre}_y"))]
}

#[track_caller]
fn perto(a: [f64; 2], b: [f64; 2], tol: f64) {
    assert!(
        (a[0] - b[0]).abs() < tol && (a[1] - b[1]).abs() < tol,
        "{a:?} ≠ {b:?}"
    );
}

/// O braço do oráculo: b1 (0,0)→(1,0), b2 dobrado 30° para cima.
fn braco(sx: f64) -> Vec<[f64; 2]> {
    let c = 30f64.to_radians();
    vec![[0.0, 0.0], [sx, 0.0], [sx * (1.0 + c.cos()), c.sin()]]
}

#[test]
fn ik_matches_the_blender_two_bone_reach() {
    let o = oraculo();
    for caso in ["perto", "baixo", "fora", "colado"] {
        let v = &o["ik_dois_ossos"][caso];
        let alvo = [
            v["alvo"][0].as_f64().unwrap(),
            v["alvo"][1].as_f64().unwrap(),
        ];
        let mut j = braco(1.0);
        let k = solve_chain(&mut j, &[1.0, 1.0], alvo, &Ik::default(), false);
        assert_eq!(k, 1.0, "{caso}: sem stretch não estica");
        perto(j[1], ponto(&v["b1"], "cauda"), TOL_IK);
        perto(j[2], ponto(&v["b2"], "cauda"), TOL_IK);
        if alvo[0].hypot(alvo[1]) < 2.0 {
            perto(j[2], alvo, 1e-12);
        }
    }
}

#[test]
fn the_forced_side_is_the_chains_and_flips_with_the_mirror() {
    let o = oraculo();
    // O lado capturado no rig direito, como o «add» o grava.
    let j0 = braco(1.0);
    let lado = bend_side_of(&j0, j0[2], 2.0);
    assert_ne!(lado, BendSide::Keep);
    let ik = Ik {
        bend: lado,
        ..Ik::default()
    };
    for caso in ["perto", "baixo", "colado"] {
        let v = &o["ik_dois_ossos"][&format!("espelho.{caso}")];
        let alvo = [
            v["alvo"][0].as_f64().unwrap(),
            v["alvo"][1].as_f64().unwrap(),
        ];
        let mut j = braco(-1.0);
        solve_chain(&mut j, &[1.0, 1.0], alvo, &ik, true);
        perto(j[1], ponto(&v["b1"], "cauda"), TOL_IK);
        perto(j[2], ponto(&v["b2"], "cauda"), TOL_IK);
        // Sem dizer que o referencial é espelhado, o lado forçado põe o joelho do lado errado.
        let mut errado = braco(-1.0);
        solve_chain(&mut errado, &[1.0, 1.0], alvo, &ik, false);
        assert!(
            (errado[1][1] - j[1][1]).abs() > 0.1,
            "{caso}: o joelho tem de trocar de lado"
        );
    }
}

#[test]
fn a_chain_of_three_reaches_with_rigid_bones() {
    let o = oraculo();
    let v = &o["ik_cadeia_3"];
    let mut j: Vec<[f64; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.7, 0.3], [2.2, 0.6]];
    let comps: Vec<f64> = j
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .collect();
    let alvo = [1.0, 1.4];
    solve_chain(
        &mut j,
        &comps,
        alvo,
        &Ik {
            chain: 3,
            ..Ik::default()
        },
        false,
    );
    perto(j[3], ponto(&v["c3"], "cauda"), 1e-3);
    for (w, c) in j.windows(2).zip(&comps) {
        assert!(((w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]) - c).abs() < 1e-9);
    }
}

#[test]
fn the_stretch_only_grows_past_the_knee_and_reaches_the_target() {
    let o = oraculo();
    let estica = Ik {
        stretch: true,
        ..Ik::default()
    };
    // Ao alcance o Blender ENCOLHE o braço (0,631 / 0,952): recusado — aqui nada estica.
    let v = &o["ik_estica"]["perto"];
    assert!(
        n(&v["b1"], "comprimento") < 0.7,
        "a recusa medida continua na fixture"
    );
    let mut com = braco(1.0);
    let mut sem = braco(1.0);
    assert_eq!(
        solve_chain(&mut com, &[1.0, 1.0], [1.2, 0.9], &estica, false),
        1.0
    );
    solve_chain(&mut sem, &[1.0, 1.0], [1.2, 0.9], &Ik::default(), false);
    assert_eq!(com, sem);
    // Fora do alcance: a ponta chega e a corrente é recta (o Blender dobra-a 3,7° / 32,6°).
    let alvo = [3.0, 1.0];
    let mut j = braco(1.0);
    let k = solve_chain(&mut j, &[1.0, 1.0], alvo, &estica, false);
    assert!((k - alvo[0].hypot(alvo[1]) / 2.0).abs() < 1e-12);
    perto(j[2], alvo, 1e-9);
    let (a1, a2) = (
        (j[1][1]).atan2(j[1][0]),
        (j[2][1] - j[1][1]).atan2(j[2][0] - j[1][0]),
    );
    assert!(wrap_pi(a1 - a2).abs() < 1e-9);
    // Com suavidade estica desde o joelho, e é contínuo nele.
    let s = 0.2 * 2.0;
    assert_eq!(stretch_factor(2.0 - s, 2.0, s), 1.0);
    assert!(stretch_factor(2.0 - s + 1e-9, 2.0, s) - 1.0 < 1e-8);
    let suave = Ik {
        softness: 0.2,
        ..estica
    };
    let alvo = [1.75, 0.0];
    let mut j = braco(1.0);
    assert!(solve_chain(&mut j, &[1.0, 1.0], alvo, &suave, false) > 1.0);
    perto(j[2], alvo, 1e-9);
}

#[test]
fn stretch_to_matches_blender_volume() {
    let o = oraculo();
    let modos = [
        ("VOLUME_XZX", Volume::Xz),
        ("VOLUME_X", Volume::Area),
        ("NO_VOLUME", Volume::Nenhum),
        ("VOLUME_Z", Volume::Nenhum),
    ];
    for (nome, volume) in modos {
        for dist in [2.0f64, 0.5] {
            let v = &o["stretch_to"][&format!("{nome}.{dist:?}")];
            let a = 25f64.to_radians();
            let ctx = Ctx {
                owner: Pose2::IDENTITY,
                parent: Pose2::IDENTITY,
                target: Some(Pose2 {
                    x: dist * a.cos(),
                    y: dist * a.sin(),
                    ..Pose2::IDENTITY
                }),
                target_parent: Pose2::IDENTITY,
            };
            let k = Kind::StretchTo(StretchTo { rest: 1.0, volume });
            let p = apply_with(&k, &ctx, None, 1.0).unwrap();
            assert!(
                wrap_pi(p.rot - n(v, "rumo_deg").to_radians()).abs() < TOL,
                "{nome} {dist}"
            );
            assert!((p.sx - n(v, "comprimento")).abs() < TOL, "{nome} {dist}");
            // A largura no plano: o eixo X do osso do Blender (o `fundo` é o Z, fora do plano).
            let largura = if nome == "VOLUME_Z" {
                1.0
            } else {
                n(v, "largura")
            };
            assert!(
                (p.sy - largura).abs() < TOL,
                "{nome} {dist}: {} ≠ {largura}",
                p.sy
            );
        }
    }
}

fn bezier(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], n: usize) -> Vec<[f64; 2]> {
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            [
                w[0] * p0[0] + w[1] * p1[0] + w[2] * p2[0] + w[3] * p3[0],
                w[0] * p0[1] + w[1] * p1[1] + w[2] * p2[1] + w[3] * p3[1],
            ]
        })
        .collect()
}

#[test]
fn follow_path_goes_by_arc_length_and_faces_the_tangent() {
    let o = oraculo();
    let r = 2.0;
    let k = 0.552_284_749_8 * r;
    let mut meio = bezier([r, 0.0], [r, k], [k, r], [0.0, r], 4096);
    meio.extend(
        bezier([0.0, r], [-k, r], [-r, k], [-r, 0.0], 4096)
            .into_iter()
            .skip(1),
    );
    let l = vec![[0.0, 0.0], [4.0, 0.0], [4.0, 3.0]];
    for f in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let fp = Kind::FollowPath(FollowPath {
            by: PathBy::Fraction,
            value: f,
            ..FollowPath::default()
        });
        let ctx = Ctx {
            owner: Pose2::IDENTITY,
            parent: Pose2::IDENTITY,
            target: None,
            target_parent: Pose2::IDENTITY,
        };
        let v = &o["follow_path"][&format!("meio_circulo.{f:?}")];
        let p = apply_with(&fp, &ctx, Some(&meio), 1.0).unwrap();
        perto([p.x, p.y], [n(v, "x"), n(v, "y")], 1e-4);
        assert!(
            wrap_pi(p.rot - n(v, "rot_deg").to_radians()).abs() < 2e-3,
            "{f}: {}",
            p.rot.to_degrees()
        );
        let v = &o["follow_path"][&format!("L.{f:?}")];
        let p = apply_with(&fp, &ctx, Some(&l), 1.0).unwrap();
        perto([p.x, p.y], [n(v, "x"), n(v, "y")], 1e-6);
        // Num trecho recto o rumo é o do trecho (o Blender roda 19,69° a meio do primeiro: recusado).
        let rumo = if f < 4.0 / 7.0 {
            0.0
        } else {
            90f64.to_radians()
        };
        assert!(
            wrap_pi(p.rot - rumo).abs() < 1e-12,
            "L {f}: {}",
            p.rot.to_degrees()
        );
    }
    // Por distância, e num caminho fechado dá a volta.
    let quadrado = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]];
    let por_dist = |d: f64| FollowPath {
        by: PathBy::Distance,
        value: d,
        ..FollowPath::default()
    };
    let ctx = Ctx {
        owner: Pose2::IDENTITY,
        parent: Pose2::IDENTITY,
        target: None,
        target_parent: Pose2::IDENTITY,
    };
    let a = apply_with(&Kind::FollowPath(por_dist(1.5)), &ctx, Some(&quadrado), 1.0).unwrap();
    let b = apply_with(&Kind::FollowPath(por_dist(5.5)), &ctx, Some(&quadrado), 1.0).unwrap();
    perto([a.x, a.y], [1.0, 0.5], 1e-12);
    perto([b.x, b.y], [1.0, 0.5], 1e-12);
    // Sem caminho não faz nada (e diz porquê).
    assert!(apply_with(&Kind::FollowPath(por_dist(1.0)), &ctx, None, 1.0).is_err());
}
