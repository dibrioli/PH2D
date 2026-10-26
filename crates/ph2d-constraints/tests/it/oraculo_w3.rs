//! As leis da onda 3 contra o Blender 5.2.3 CORRIDO (casos `pivot`, `floor`, `transformation`,
//! `armature`, `spline_ik`, `vertice` da fixture). As recusas medidas (o Replace que zera os canais
//! vizinhos, a mistura de matrizes que encolhe) estão em `docs/BlenderConstraints/03_W3_poderosas.md`
//! §1 e têm aqui o gate do lado contrário.

use ph2d_constraints::{
    Ajuste, Canal, ChildOf, Ctx, EixoDoChao, Floor, Kind, Mix, Pivot, PontoDeForma, Pose2, Space,
    Transformation, apply, blend, lay_on_curve, wrap_pi,
};
use serde_json::Value;

const FIXTURE: &str =
    include_str!("../../../../docs/BlenderConstraints/fixtures/blender_5_2_3_constraints_2d.json");
const TOL: f64 = 1e-4;

fn oraculo() -> Value {
    serde_json::from_str(FIXTURE).expect("a fixture é JSON")
}

fn n(v: &Value, k: &str) -> f64 {
    v[k].as_f64().expect(k)
}

fn pose(v: &Value) -> Pose2 {
    Pose2 {
        x: n(v, "x"),
        y: n(v, "y"),
        rot: n(v, "rot_deg").to_radians(),
        sx: n(v, "sx"),
        sy: n(v, "sy"),
    }
}

fn at(x: f64, y: f64, rot_deg: f64) -> Pose2 {
    Pose2 {
        x,
        y,
        rot: rot_deg.to_radians(),
        ..Pose2::IDENTITY
    }
}

#[track_caller]
fn igual(got: Pose2, want: Pose2) {
    let d = [
        got.x - want.x,
        got.y - want.y,
        wrap_pi(got.rot - want.rot),
        got.sx - want.sx,
        got.sy - want.sy,
    ];
    assert!(
        d.iter().all(|v| v.abs() < TOL),
        "got {got:?}\nwant {want:?}"
    );
}

fn ctx(owner: Pose2, target: Option<Pose2>) -> Ctx {
    Ctx {
        owner,
        parent: Pose2::IDENTITY,
        target,
        target_parent: Pose2::IDENTITY,
    }
}

#[test]
fn pivot_turns_the_owner_around_the_target() {
    let o = oraculo();
    let k = Kind::Pivot(Pivot::default());
    for (caso, rot) in [("noventa", 90.0), ("zero", 0.0), ("menos_45", -45.0)] {
        let c = ctx(at(2.0, 0.0, rot), Some(at(1.0, 1.0, 0.0)));
        igual(apply(&k, &c, 1.0).unwrap(), pose(&o["pivot"][caso]));
    }
    // Sem alvo, o pivô é o próprio dono + o desvio.
    let sem = Kind::Pivot(Pivot {
        offset: [-1.0, 1.0],
    });
    igual(
        apply(&sem, &ctx(at(2.0, 0.0, 90.0), None), 1.0).unwrap(),
        pose(&o["pivot"]["noventa"]),
    );
}

#[test]
fn floor_keeps_the_owner_on_its_side() {
    let o = oraculo();
    for (caso, dono, eixo, rot_alvo, usa_rotacao) in [
        ("abaixo", (1.0, -1.0), EixoDoChao::MaisY, 0.0, false),
        ("acima", (1.0, 2.0), EixoDoChao::MaisY, 0.0, false),
        ("tecto", (1.0, 2.0), EixoDoChao::MenosY, 0.0, false),
        ("rodado", (1.0, -1.0), EixoDoChao::MaisY, 30.0, true),
        (
            "rodado_sem_rot",
            (1.0, -1.0),
            EixoDoChao::MaisY,
            30.0,
            false,
        ),
    ] {
        let k = Kind::Floor(Floor {
            eixo,
            desvio: 0.5,
            usa_rotacao,
        });
        let c = ctx(at(dono.0, dono.1, 15.0), Some(at(0.0, 0.0, rot_alvo)));
        igual(apply(&k, &c, 1.0).unwrap(), pose(&o["floor"][caso]));
    }
}

fn mapa(de: Canal, de_faixa: [f64; 2], para: Canal, para_faixa: [f64; 2]) -> Transformation {
    Transformation {
        de,
        de_faixa,
        para,
        para_faixa,
        extrapola: false,
        mix: Mix::Replace,
        space: Space::World,
    }
}

#[test]
fn transformation_maps_one_channel_range_to_another() {
    let o = oraculo();
    let t = &o["transformation"];
    let loc_rot = mapa(Canal::X, [0.0, 2.0], Canal::Rot, [0.0, 90f64.to_radians()]);
    for extrapola in [false, true] {
        let nome = if extrapola { "extrapola" } else { "apara" };
        let k = Kind::Transformation(Transformation {
            extrapola,
            ..loc_rot
        });
        for ax in ["-1.0", "0.0", "0.5", "1.0", "2.0", "3.0"] {
            let c = ctx(at(1.0, 0.5, 10.0), Some(at(ax.parse().unwrap(), 0.0, 0.0)));
            igual(
                apply(&k, &c, 1.0).unwrap(),
                pose(&t[format!("loc_rot.{nome}.{ax}")]),
            );
        }
    }
    let soma = Kind::Transformation(Transformation {
        mix: Mix::Add,
        ..loc_rot
    });
    for ax in ["0.5", "1.0"] {
        let c = ctx(at(1.0, 0.5, 10.0), Some(at(ax.parse().unwrap(), 0.0, 0.0)));
        igual(
            apply(&soma, &c, 1.0).unwrap(),
            pose(&t[format!("loc_rot.soma.{ax}")]),
        );
    }
    let esc = Kind::Transformation(mapa(Canal::Sx, [1.0, 2.0], Canal::Sy, [1.0, 3.0]));
    for asx in ["0.5", "1.5", "2.0"] {
        let alvo = Pose2 {
            sx: asx.parse().unwrap(),
            ..Pose2::IDENTITY
        };
        let c = ctx(Pose2::IDENTITY, Some(alvo));
        igual(
            apply(&esc, &c, 1.0).unwrap(),
            pose(&t[format!("esc_esc.{asx}")]),
        );
    }
}

/// ⛔ Recusa medida: o Replace do Blender zera os OUTROS canais de posição (x autorado 1 → 0). Aqui
/// o canal y recebe o mapa e o x fica o do dono.
#[test]
fn transformation_replace_leaves_the_unmapped_channels_alone() {
    let o = oraculo();
    let k = Kind::Transformation(mapa(
        Canal::Rot,
        [(-90f64).to_radians(), 90f64.to_radians()],
        Canal::Y,
        [0.0, 3.0],
    ));
    for ar in ["-170.0", "-90.0", "0.0", "45.0", "120.0"] {
        let blender = pose(&o["transformation"][format!("rot_loc.{ar}")]);
        assert_eq!(blender.x, 0.0, "a recusa medida: o Blender zera o x");
        let c = ctx(at(1.0, 0.5, 0.0), Some(at(0.0, 0.0, ar.parse().unwrap())));
        igual(apply(&k, &c, 1.0).unwrap(), Pose2 { x: 1.0, ..blender });
    }
}

/// ⭐ Multi-parent é a pilha: *Child Of A* a 100 % + *Child Of B* à força `w_B`. A posição é a do
/// `armature.linear` do Blender; a escala NÃO encolhe (a recusa: 0,707 a meio).
#[test]
fn multi_parent_is_two_child_of_items() {
    let o = oraculo();
    let dono = at(2.0, 0.0, 0.0);
    let (a_repouso, b) = (Pose2::IDENTITY, Pose2::IDENTITY);
    let a = at(0.0, 0.0, 90.0);
    let nasce = |alvo: Pose2, owner: Pose2| {
        let mut k = Kind::ChildOf(ChildOf::default());
        k.keep_current(&ctx(owner, Some(alvo))).unwrap();
        k
    };
    let (filho_a, filho_b) = (nasce(a_repouso, dono), nasce(b, dono));
    // Cada Child Of nasce sem salto.
    let parado = apply(&filho_a, &ctx(dono, Some(a_repouso)), 1.0).unwrap();
    igual(apply(&filho_b, &ctx(parado, Some(b)), 0.5).unwrap(), dono);
    for w_a in ["0.5", "0.3"] {
        let w_a: f64 = w_a.parse().unwrap();
        let blender = pose(&o["armature"][format!("linear.{w_a}")]);
        let meio = apply(&filho_a, &ctx(dono, Some(a)), 1.0).unwrap();
        let got = apply(&filho_b, &ctx(meio, Some(b)), 1.0 - w_a).unwrap();
        igual(got, blend(at(0.0, 2.0, 90.0), dono, 1.0 - w_a));
        assert!((got.x - blender.x).abs() < TOL && (got.y - blender.y).abs() < TOL);
        assert!(
            (got.sx - 1.0).abs() < 1e-12 && blender.sx < 0.8,
            "a recusa: o LBS encolhe"
        );
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

/// O meio círculo do oráculo: Bézier de (2,0) por (0,2) a (−2,0), alças 0,5523·r.
fn meio_circulo() -> Vec<[f64; 2]> {
    let (r, k) = (2.0, 0.552_284_749_8 * 2.0);
    let mut a = bezier([r, 0.0], [r, k], [k, r], [0.0, r], 512);
    a.extend(
        bezier([0.0, r], [-k, r], [-r, k], [-r, 0.0], 512)
            .into_iter()
            .skip(1),
    );
    a
}

#[test]
fn spline_ik_lays_the_chain_on_the_curve() {
    let o = oraculo();
    let curvas: [(&str, Vec<[f64; 2]>, f64); 3] = [
        ("recta", vec![[0.0, 1.0], [5.0, 1.0]], TOL),
        ("curta", vec![[0.0, 1.0], [1.0, 1.0], [1.0, 2.0]], TOL),
        // A poligonal do Blender (resolução 64) e a nossa (512) diferem no 4.º dígito.
        ("meio_circulo", meio_circulo(), 2e-3),
    ];
    for (forma, curva, tol) in &curvas {
        for (modo, ajuste) in [
            ("NONE", Ajuste::Nenhum),
            ("BONE_ORIGINAL", Ajuste::Nenhum),
            ("FIT_CURVE", Ajuste::Esticar),
        ] {
            let juntas = lay_on_curve(&[1.0, 1.0, 1.0], curva, ajuste).expect("a curva serve");
            assert_eq!(juntas.len(), 4);
            let caso = &o["spline_ik"][format!("{forma}.{modo}")];
            for (i, osso) in ["s1", "s2", "s3"].iter().enumerate() {
                let v = &caso[osso];
                let (a, b) = (juntas[i], juntas[i + 1]);
                for (got, want) in [
                    (a[0], n(v, "cabeca_x")),
                    (a[1], n(v, "cabeca_y")),
                    (b[0], n(v, "cauda_x")),
                    (b[1], n(v, "cauda_y")),
                    ((b[0] - a[0]).hypot(b[1] - a[1]), n(v, "comprimento")),
                ] {
                    assert!(
                        (got - want).abs() < *tol,
                        "{forma}.{modo}.{osso}: {got} ≠ {want}"
                    );
                }
            }
        }
    }
    assert!(lay_on_curve(&[1.0], &[[0.0, 0.0]], Ajuste::Nenhum).is_none());
}

#[test]
fn a_point_of_a_shape_is_a_target() {
    let o = oraculo();
    let v = &o["vertice"];
    let local = [
        v["vertice_local"][0].as_f64().unwrap(),
        v["vertice_local"][1].as_f64().unwrap(),
    ];
    let forma = Pose2 {
        x: 3.0,
        y: -1.0,
        rot: 30f64.to_radians(),
        sx: 2.0,
        sy: 2.0,
    };
    let [x, y] = forma.ponto(local);
    let k = Kind::PontoDeForma(PontoDeForma { indice: 1 });
    let alvo = Pose2 {
        x,
        y,
        ..Pose2::IDENTITY
    };
    igual(
        apply(&k, &ctx(Pose2::IDENTITY, Some(alvo)), 1.0).unwrap(),
        pose(&v["dono"]),
    );
}
