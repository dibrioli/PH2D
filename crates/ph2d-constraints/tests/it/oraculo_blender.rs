//! As leis contra o Blender 5.2.3 CORRIDO (fixture com cabeçalho, gerada por
//! `docs/BlenderConstraints/ferramentas/oraculo_blender.py`). O Blender guarda em `f32`: a tolerância
//! é a dele, e o ângulo compara-se no círculo.

use ph2d_constraints::{
    Canais, ChildOf, CopyTransform, Ctx, Distance, DistanceMode, Faixa, Kind, Limit, LookAt, Pose2,
    apply, wrap_pi,
};
use serde_json::Value;

const FIXTURE: &str =
    include_str!("../../../../docs/BlenderConstraints/fixtures/blender_5_2_3_constraints_2d.json");
const TOL: f64 = 1e-4;

fn oraculo() -> Value {
    serde_json::from_str(FIXTURE).expect("a fixture é JSON")
}

fn pose(v: &Value) -> Pose2 {
    let n = |k: &str| v[k].as_f64().expect(k);
    Pose2 {
        x: n("x"),
        y: n("y"),
        rot: n("rot_deg").to_radians(),
        sx: n("sx"),
        sy: n("sy"),
    }
}

fn deg(d: f64) -> f64 {
    d.to_radians()
}

fn at(x: f64, y: f64, rot_deg: f64, s: f64) -> Pose2 {
    Pose2 {
        x,
        y,
        rot: deg(rot_deg),
        sx: s,
        sy: s,
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
fn the_blend_takes_the_short_arc() {
    let o = oraculo();
    let copia = Kind::CopyTransform(CopyTransform {
        canais: Canais::ROTATION,
        ..Default::default()
    });
    let c = ctx(at(0.0, 0.0, 170.0, 1.0), Some(at(0.0, 0.0, -170.0, 1.0)));
    for t in ["0.0", "0.25", "0.5", "0.75", "1.0"] {
        let influencia: f64 = t.parse().unwrap();
        igual(
            apply(&copia, &c, influencia).unwrap(),
            pose(&o["mistura_rotacao_copy_rotation"][t]),
        );
    }
}

#[test]
fn child_of_never_jumps_and_follows_like_blender() {
    let o = oraculo();
    let avo = at(1.0, -2.0, -20.0, 1.2);
    let alvo = at(5.0, 0.0, 37.0, 1.5).under(avo);
    let dono = at(2.0, 1.0, 30.0, 1.0);
    let mut k = Kind::ChildOf(ChildOf::default());
    assert!(k.keeps_current_on_create());
    k.keep_current(&ctx(dono, Some(alvo))).unwrap();
    for caso in ["sem_inverse", "set_inverse"] {
        let c = &o["child_of"][caso];
        igual(
            apply(&k, &ctx(dono, Some(alvo)), 1.0).unwrap(),
            pose(&c["ao_ligar"]),
        );
        let movido = at(6.0, 0.0, 47.0, 1.5).under(avo);
        igual(
            apply(&k, &ctx(dono, Some(movido)), 1.0).unwrap(),
            pose(&c["alvo_movido"]),
        );
    }
}

#[test]
fn one_look_at_is_the_three_blender_tracks() {
    let o = oraculo();
    let k = Kind::LookAt(LookAt::default());
    let got = apply(
        &k,
        &ctx(at(0.0, 0.0, 10.0, 1.0), Some(at(3.0, 4.0, 0.0, 1.0))),
        1.0,
    )
    .unwrap();
    for tipo in ["DAMPED_TRACK", "TRACK_TO", "LOCKED_TRACK"] {
        igual(got, pose(&o["apontar_3_tipos"][tipo]));
    }
}

#[test]
fn limit_rotation_goes_to_the_nearest_edge_on_the_circle() {
    let o = oraculo();
    let k = Kind::Limit(Limit {
        rot: Some([deg(-45.0), deg(45.0)]),
        ..Default::default()
    });
    for r in ["30.0", "60.0", "120.0", "170.0", "-170.0", "-100.0"] {
        let dono = at(0.0, 0.0, r.parse().unwrap(), 1.0);
        igual(
            apply(&k, &ctx(dono, None), 1.0).unwrap(),
            pose(&o["limit_rotation_z_-45_45"][r]),
        );
    }
}

#[test]
fn limit_distance_projects_radially() {
    let o = oraculo();
    let k = Kind::Distance(Distance {
        distance: 2.0,
        mode: DistanceMode::Inside,
    });
    let c = ctx(at(5.0, 0.0, 0.0, 1.0), Some(Pose2::IDENTITY));
    igual(
        apply(&k, &c, 1.0).unwrap(),
        pose(&o["limit_distance_inside_2"]),
    );
}

#[test]
fn the_stack_order_is_the_evaluation_order() {
    let o = oraculo();
    let alvo = at(8.0, 1.0, 0.0, 1.0);
    let copia = Kind::CopyTransform(CopyTransform {
        canais: Canais::LOCATION,
        ..Default::default()
    });
    let limite = Kind::Limit(Limit {
        x: Faixa {
            min: None,
            max: Some(3.0),
        },
        ..Default::default()
    });
    let pilha = |ordem: [&Kind; 2]| {
        ordem.iter().fold(Pose2::IDENTITY, |dono, k| {
            apply(k, &ctx(dono, Some(alvo)), 1.0).unwrap()
        })
    };
    let r = &o["ordem_da_pilha"];
    igual(
        pilha([&copia, &limite]),
        pose(&r["COPY_LOCATION > LIMIT_LOCATION"]),
    );
    igual(
        pilha([&limite, &copia]),
        pose(&r["LIMIT_LOCATION > COPY_LOCATION"]),
    );
}
