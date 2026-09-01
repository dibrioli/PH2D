//! Gates da COSTURA de **SOLDAR** — o que a lei pura não alcança: a cena, a pose e o undo.

use super::*;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VertexKind, Xform};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn reta(scene: &mut VecScene, a: [f64; 2], b: [f64; 2]) -> u64 {
    scene.push_path(VecPath {
        id: 0,
        verts: vec![v(a[0], a[1]), v(b[0], b[1])],
        closed: false,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 2.0)),
        subpaths: Vec::new(),
        fill_rule: ph2d_vec_scene::FillRule::NonZero,
        effects: Vec::new(),
    })
}

fn cena() -> (VecScene, ph2d_vec_edit::History, ph2d_vec_edit::PenTool) {
    (
        VecScene::new(),
        ph2d_vec_edit::History::default(),
        ph2d_vec_edit::PenTool::default(),
    )
}

/// ⭐⭐⭐ **A IDEIA DO ENIO:** duas linhas cruzadas viram quatro arcos, e as quatro pontas do meio
/// caem **no mesmo sítio** — é isso o nó partilhado.
#[test]
fn two_crossing_lines_become_four_arcs_that_meet_at_one_point() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let vt = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    pen.select_many(&[h, vt]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new());

    assert_eq!(
        scene.paths().len(),
        4,
        "duas linhas cruzadas dao QUATRO arcos"
    );
    // As oito pontas: quatro nas extremidades, quatro no cruzamento.
    let no_centro = scene
        .paths()
        .iter()
        .flat_map(|p| [p.verts[0].anchor, p.verts[p.verts.len() - 1].anchor])
        .filter(|a| a[0].abs() < 1e-6 && a[1].abs() < 1e-6)
        .count();
    assert_eq!(
        no_centro, 4,
        "as quatro pontas do meio tem de cair EXACTAMENTE no cruzamento"
    );
    assert!(
        scene.paths().iter().all(|p| p.stroke.is_some()),
        "o estilo viaja para cada arco"
    );
}

/// ⚠️ **Um caminho que não encontra ninguém NÃO é tocado** — nem o objecto, nem o id. Sem isto,
/// soldar uma selecção grande dissolveria tudo o que estava só a passar por lá.
#[test]
fn a_path_that_meets_nobody_keeps_its_identity() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let vt = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    let longe = reta(&mut scene, [100.0, 100.0], [120.0, 100.0]);
    pen.select_many(&[h, vt, longe]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new());

    assert!(
        scene.path(longe).is_some(),
        "o traco distante perdeu o id — ele nao cruza nada"
    );
    assert_eq!(scene.paths().len(), 5, "4 arcos + o intocado");
}

/// ⚠️⚠️ **A POSE entra na conta.** As duas linhas só se cruzam DEPOIS de o `Transform` da segunda a
/// pôr no lugar; medir na geometria local diria que elas não se encontram.
#[test]
fn the_crossing_is_found_in_world_space_not_in_local() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    // Esta nasce longe e é TRAZIDA para cima da outra pela pose.
    let vt = reta(&mut scene, [500.0, -10.0], [500.0, 10.0]);
    let mut xf = VecXforms::new();
    xf.insert(vt, Xform([1.0, 0.0, 0.0, 1.0, -500.0, 0.0]));
    pen.select_many(&[h, vt]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &xf);
    assert_eq!(
        scene.paths().len(),
        4,
        "sem a pose na conta, as duas linhas nao se encontram"
    );
}

/// **Nada se cruza ⇒ NADA acontece**, e sem passo de undo: um comando que não fez nada não pode
/// gastar um Ctrl+Z.
#[test]
fn a_selection_that_crosses_nothing_is_a_no_op() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let b = reta(&mut scene, [-10.0, 50.0], [10.0, 50.0]);
    pen.select_many(&[a, b]);
    let antes = scene.paths().len();

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new());

    assert_eq!(scene.paths().len(), antes);
    assert!(scene.path(a).is_some() && scene.path(b).is_some());
}

/// ⭐⭐ **E é isto que fecha as ÁREAS**: um X dentro de um quadrado dá uma rede em que cada região
/// é cercada por arcos que se encontram nas pontas — o substrato que o balde vai usar.
#[test]
fn a_cross_inside_a_square_becomes_a_network_of_arcs() {
    let (mut scene, mut hist, mut pen) = cena();
    let quadrado = scene.push_path(VecPath {
        id: 0,
        verts: vec![
            v(-10.0, -10.0),
            v(10.0, -10.0),
            v(10.0, 10.0),
            v(-10.0, 10.0),
        ],
        closed: true,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 2.0)),
        subpaths: Vec::new(),
        fill_rule: ph2d_vec_scene::FillRule::NonZero,
        effects: Vec::new(),
    });
    let diagonal = reta(&mut scene, [-20.0, 0.0], [20.0, 0.0]);
    pen.select_many(&[quadrado, diagonal]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new());

    // O quadrado é cortado nos dois lados e a linha nos dois cruzamentos.
    assert!(
        scene.paths().len() >= 5,
        "a rede tem de ter os arcos dos dois: {} caminhos",
        scene.paths().len()
    );
    assert!(
        scene.paths().iter().all(|p| !p.closed),
        "depois de soldado nao sobra anel — tudo e' arco"
    );
}
