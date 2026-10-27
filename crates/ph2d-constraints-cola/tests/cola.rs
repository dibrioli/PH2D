//! Gates da cola (05 §4): fecho analítico, conflito e o oráculo FreeCAD 1.1.4 Assembly por passo.

use std::collections::BTreeMap;

use ph2d_constraints_cola::{Cola, Corpo, Entrada, Estado, Pose2, Tipo, resolve};

fn pose(x: f64, y: f64, graus: f64) -> Pose2 {
    Pose2 { x, y, rot: graus.to_radians(), sx: 1.0, sy: 1.0 }
}

fn entrada(p: Pose2, condutor: bool) -> Entrada {
    Entrada { pose: p, condutor }
}

fn dist(p: [f64; 2], q: [f64; 2]) -> f64 {
    (p[0] - q[0]).hypot(p[1] - q[1])
}

#[test]
fn a_rigid_glue_keeps_the_relative_pose() {
    // Pai com escala uniforme: a `Pose2` não tem cisalhamento (o limite do Child Of, ADR-0192); o filho pode esticar.
    let a0 = Pose2 { x: 1.0, y: 2.0, rot: 0.3, sx: 1.5, sy: 1.5 };
    let b0 = Pose2 { x: 4.0, y: -1.0, rot: -0.8, sx: 2.0, sy: 0.5 };
    let cola = Cola::rigida(1, 10, 20, a0, b0).unwrap();
    let Tipo::Rigida { rel } = cola.tipo else { unreachable!() };
    let a1 = Pose2 { x: -3.0, y: 5.0, rot: 1.9, ..a0 };

    let s = resolve(&[cola], &BTreeMap::from([(10, entrada(a1, true)), (20, entrada(b0, false))]));
    assert_eq!(s.grupos[0].estado, Estado::Definido);
    assert_eq!(s.poses[&10], a1, "o condutor sai com a sua pose ao bit");
    assert_eq!(s.poses[&20], rel.under(a1), "B = A · rel, ao bit");

    let b1 = Pose2 { x: 7.0, y: 0.5, rot: 2.4, ..b0 };
    let s = resolve(&[cola], &BTreeMap::from([(10, entrada(a0, false)), (20, entrada(b1, true))]));
    let a = s.poses[&10];
    let b = rel.under(a);
    assert!(dist([b.x, b.y], [b1.x, b1.y]) < 1e-9 && (b.rot - b1.rot).abs() < 1e-9, "arrastar B leva A: {a:?}");
}

#[test]
fn a_pin_keeps_the_points_together_and_each_its_rotation() {
    let p = [pose(0.0, 0.0, 0.0), pose(2.0, 0.0, 10.0), pose(4.0, 0.5, -20.0)];
    let colas = [
        Cola::pino(1, 1, 2, p[0], p[1], [2.0, 0.0]).unwrap(),
        Cola::pino(2, 2, 3, p[1], p[2], [4.0, 0.3]).unwrap(),
    ];
    let mao = pose(0.0, 0.0, 30.0);
    let corpos = BTreeMap::from([(1, entrada(mao, true)), (2, entrada(p[1], false)), (3, entrada(p[2], false))]);
    let s = resolve(&colas, &corpos);
    assert_eq!(s.grupos[0].estado, Estado::Definido);
    for c in &colas {
        let Tipo::Pino { em_a, em_b } = c.tipo else { unreachable!() };
        let d = dist(s.poses[&c.a].ponto(em_a), s.poses[&c.b].ponto(em_b));
        assert!(d <= 1e-9, "pino {} aberto: {d:e}", c.id);
    }
    assert_eq!(s.poses[&2].rot, p[1].rot, "o seguidor mantém a sua rotação");
    assert_eq!(s.poses[&3].rot, p[2].rot);
}

#[test]
fn two_drivers_that_disagree_are_a_conflict_not_a_fight() {
    let (a0, b0) = (pose(0.0, 0.0, 0.0), pose(3.0, 0.0, 0.0));
    let rigida = Cola::rigida(1, 1, 2, a0, b0).unwrap();
    let s = resolve(&[rigida], &BTreeMap::from([(1, entrada(a0, true)), (2, entrada(pose(3.0, 1.0, 0.0), true))]));
    assert_eq!(s.grupos[0].estado, Estado::Conflito);
    assert!(s.poses.is_empty(), "ninguém se mexe");

    let pino = Cola::pino(2, 1, 2, a0, b0, [1.0, 0.0]).unwrap();
    let longe = BTreeMap::from([(1, entrada(a0, true)), (2, entrada(pose(9.0, 0.0, 0.0), true))]);
    let s = resolve(&[pino], &longe);
    assert_eq!(s.grupos[0].estado, Estado::Conflito);
    assert!(s.poses.is_empty());

    let parados = BTreeMap::from([(1, entrada(a0, false)), (2, entrada(b0, false))]);
    assert_eq!(resolve(&[pino], &parados).grupos[0].estado, Estado::Parado);
}

// --- oráculo -----------------------------------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct Fixture {
    casos: Vec<Caso>,
}

#[derive(serde::Deserialize)]
struct Caso {
    nome: String,
    corpos: Vec<CorpoF>,
    juntas: Vec<Junta>,
    condutores: BTreeMap<String, [f64; 3]>,
    freecad: Freecad,
}

#[derive(serde::Deserialize)]
struct CorpoF {
    nome: String,
    pontos: Vec<[f64; 2]>,
    autorada: [f64; 3],
}

#[derive(serde::Deserialize)]
struct Junta {
    tipo: String,
    a: String,
    ia: usize,
    b: String,
    ib: usize,
}

#[derive(serde::Deserialize)]
struct Freecad {
    residuo: f64,
    poses: BTreeMap<String, [f64; 3]>,
}

fn fixture() -> Fixture {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/BlenderConstraints/fixtures/freecad_1_1_4_assembly_cola.json"
    );
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn p3(v: [f64; 3]) -> Pose2 {
    pose(v[0], v[1], v[2])
}

/// Corre um caso da fixture: as MESMAS entradas, o mesmo palpite (a pose autorada).
fn corre(c: &Caso) -> (Estado, BTreeMap<String, Pose2>) {
    let id: BTreeMap<&str, Corpo> = c.corpos.iter().zip(1..).map(|(k, i)| (k.nome.as_str(), i)).collect();
    let corpo = |n: &str| c.corpos.iter().find(|k| k.nome == n).unwrap();
    let colas: Vec<Cola> = c
        .juntas
        .iter()
        .zip(1..)
        .map(|(j, n)| {
            let (pa, pb) = (p3(corpo(&j.a).autorada), p3(corpo(&j.b).autorada));
            let tipo = match j.tipo.as_str() {
                "pino" => Tipo::Pino { em_a: corpo(&j.a).pontos[j.ia], em_b: corpo(&j.b).pontos[j.ib] },
                _ => Tipo::Rigida { rel: pb.relative_to(pa).unwrap() },
            };
            Cola { id: n, a: id[j.a.as_str()], b: id[j.b.as_str()], tipo }
        })
        .collect();
    let corpos = c
        .corpos
        .iter()
        .map(|k| {
            let cond = c.condutores.get(&k.nome);
            (id[k.nome.as_str()], entrada(p3(*cond.unwrap_or(&k.autorada)), cond.is_some()))
        })
        .collect();
    let s = resolve(&colas, &corpos);
    let poses = c.corpos.iter().filter_map(|k| Some((k.nome.clone(), *s.poses.get(&id[k.nome.as_str()])?)));
    (s.grupos[0].estado, poses.collect())
}

/// O FreeCAD é 3D/`f32` nas juntas e o MbD converge a ~1e-15 no resíduo; o nosso é 2D/`f64`. 1e-6 nas unidades
/// da cena e em graus (o pior desvio medido fica escrito na falha).
const TOL_ORACULO: f64 = 1e-6;

#[test]
fn the_four_bar_matches_freecad_step_by_step() {
    let f = fixture();
    let mut vistos = 0;
    for c in f.casos.iter().filter(|c| c.nome.starts_with("quadrilatero_") || c.nome == "rigida") {
        let (estado, nossas) = corre(c);
        assert_eq!(estado, Estado::Definido, "{}", c.nome);
        for (nome, fc) in &c.freecad.poses {
            let n = nossas[nome];
            let da = (n.rot.to_degrees() - fc[2] + 180.0).rem_euclid(360.0) - 180.0;
            let pior = (n.x - fc[0]).abs().max((n.y - fc[1]).abs()).max(da.abs());
            assert!(pior <= TOL_ORACULO, "{} / {nome}: nosso {n:?} × FreeCAD {fc:?} (desvio {pior:e})", c.nome);
        }
        vistos += 1;
    }
    assert_eq!(vistos, 9, "os 4 passos da manivela, os 4 do balancim e a rígida");
}

#[test]
fn a_loop_that_does_not_close_is_a_conflict() {
    let f = fixture();
    let c = f.casos.iter().find(|c| c.nome == "laco_impossivel").unwrap();
    assert!(c.freecad.residuo > 1.0, "o FreeCAD diz 0 mas deixa a junta aberta: {}", c.freecad.residuo);
    assert_eq!(corre(c).0, Estado::Conflito);
}
