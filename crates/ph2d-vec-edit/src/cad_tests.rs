//! O CAD na caneta (W4.4): a guia acesa vira regra pelo MESMO snap que a shell corre; o arrasto obedece às regras;
//! a regra a mais não entra (medida nasce REF).

use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VecXforms, VertId, rectangle};

use crate::snap::{SnapConfig, SnapResult, collect_targets, snap};
use crate::{PenTool, junta_regra};

const PTW: f64 = 0.01;

fn v(p: VecPathId, i: u32) -> PontoCad {
    PontoCad::Vert(p, VertId(i))
}

/// A caneta com o snap da shell (`despacho_clique_vetor_premido.rs`): alvos recolhidos da cena, limiar de 1.
fn clica(pen: &mut PenTool, scene: &mut VecScene, p: [f64; 2]) {
    let alvos = collect_targets(scene, &VecXforms::default(), &[], &[], false);
    let cfg = SnapConfig {
        threshold: 1.0,
        ..SnapConfig::default()
    };
    pen.on_press_encaixe(scene, p, PTW, false, &mut |q| snap(&[q], &alvos, cfg, None));
    pen.on_release();
}

#[test]
fn draw_with_a_guide_creates_the_rule() {
    let mut scene = VecScene::new();
    let a = scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([100.0, 0.0]),
        ],
        ..VecPath::default()
    });
    let mut pen = PenTool::new();
    clica(&mut pen, &mut scene, [200.0, 0.4]); // a guia horizontal do 1.º vértice de A
    let b = pen.selected().expect("a caneta começou um path");
    clica(&mut pen, &mut scene, [100.5, 30.0]); // a guia vertical do 2.º vértice de A
    assert_eq!(
        scene.regras(),
        &[
            RegraCad::nova(Lei::Horizontal(SegCad(v(a, 1), v(b, 1)))),
            RegraCad::nova(Lei::Vertical(SegCad(v(a, 2), v(b, 2)))),
        ]
    );
    assert_eq!(scene.path(b).unwrap().verts[1].anchor, [100.0, 30.0]);
}

/// ⛔ CONTROLO: sem guia acesa (longe de tudo), nenhuma regra nasce.
#[test]
fn a_point_placed_far_from_every_guide_creates_no_rule() {
    let mut scene = VecScene::new();
    scene.push_path(rectangle([0.0, 0.0], [10.0, 5.0]));
    let mut pen = PenTool::new();
    clica(&mut pen, &mut scene, [40.0, 40.0]);
    clica(&mut pen, &mut scene, [60.0, 70.0]);
    assert!(scene.regras().is_empty());
}

fn retangulo_preso() -> (VecScene, VecPathId) {
    let mut scene = VecScene::new();
    let r = scene.push_path(rectangle([0.0, 0.0], [10.0, 5.0]));
    *scene.regras_mut() = vec![
        RegraCad::nova(Lei::Horizontal(SegCad(v(r, 1), v(r, 2)))),
        RegraCad::nova(Lei::Vertical(SegCad(v(r, 2), v(r, 3)))),
        RegraCad::nova(Lei::Horizontal(SegCad(v(r, 3), v(r, 4)))),
        RegraCad::nova(Lei::Vertical(SegCad(v(r, 4), v(r, 1)))),
        RegraCad::nova(Lei::Fixo(v(r, 1), [0.0, 0.0])),
    ];
    (scene, r)
}

#[test]
fn dragging_a_corner_keeps_the_rectangle() {
    let (mut scene, r) = retangulo_preso();
    let canto = scene.path(r).unwrap().verts[2].anchor;
    let mut pen = PenTool::new();
    pen.on_press_node(&mut scene, canto, PTW, false);
    for passo in 1..=6 {
        let t = f64::from(passo);
        let alvo = [canto[0] + t, canto[1] + 0.5 * t];
        assert!(pen.on_drag_encaixe(&mut scene, alvo, &mut |_| SnapResult::default()));
        let p: Vec<[f64; 2]> = scene
            .path(r)
            .unwrap()
            .verts
            .iter()
            .map(|v| v.anchor)
            .collect();
        let perto =
            |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6;
        assert!(perto(p[2], alvo), "o canto agarrado vai ao cursor: {p:?}");
        assert!(perto(p[0], [0.0, 0.0]), "o fixo fica");
        assert!(
            perto(p[1], [alvo[0], 0.0]) && perto(p[3], [0.0, alvo[1]]),
            "passo {passo}: {p:?}"
        );
    }
    assert_eq!(pen.cad_em_gesto(), &[r]);
    pen.on_release();
    assert!(pen.cad_em_gesto().is_empty());
    assert_eq!(
        scene.regras().len(),
        5,
        "o arrasto sem guia não junta regra"
    );
}

#[test]
fn a_rule_the_drawing_already_implies_is_not_added_and_an_extra_dimension_is_born_ref() {
    let (mut scene, r) = retangulo_preso();
    let xf = VecXforms::default();
    assert_eq!(
        junta_regra(&mut scene, &xf, Lei::Horizontal(SegCad(v(r, 2), v(r, 1)))),
        None,
        "a mesma H, de trás para a frente"
    );
    assert!(junta_regra(&mut scene, &xf, Lei::Distancia(v(r, 1), v(r, 2), 10.0)).is_some());
    assert!(junta_regra(&mut scene, &xf, Lei::Distancia(v(r, 2), v(r, 3), 5.0)).is_some());
    // O retângulo está definido: a diagonal já não sobe o posto.
    let diagonal = Lei::Distancia(v(r, 1), v(r, 3), 125f64.sqrt());
    assert_eq!(
        junta_regra(&mut scene, &xf, diagonal),
        Some(RegraCad {
            lei: diagonal,
            referencia: true
        })
    );
    // A mesma medida de trás para a frente é repetida, não uma 2.ª REF.
    let n = scene.regras().len();
    assert_eq!(
        junta_regra(&mut scene, &xf, Lei::Distancia(v(r, 2), v(r, 1), 10.0)),
        None
    );
    assert_eq!(scene.regras().len(), n);
    // Uma lei geométrica que o desenho já impõe, sem ser cópia de nenhuma: as H fazem as arestas paralelas.
    let paralelas = Lei::Paralelo(SegCad(v(r, 1), v(r, 2)), SegCad(v(r, 4), v(r, 3)));
    assert_eq!(junta_regra(&mut scene, &xf, paralelas), None);
    // Uma lei geométrica a mais não entra: as duas H já fazem a 3.ª.
    let n = scene.regras().len();
    assert_eq!(
        junta_regra(&mut scene, &xf, Lei::Vertical(SegCad(v(r, 1), v(r, 4)))),
        None
    );
    assert_eq!(scene.regras().len(), n);
}

/// No arrasto a guia muda: a regra que fica é a da guia acesa ao SOLTAR, e nenhuma das anteriores.
#[test]
fn a_drag_keeps_only_the_guide_lit_at_release() {
    let mut scene = VecScene::new();
    let a = scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([50.0, 0.0]),
        ],
        ..VecPath::default()
    });
    let b = scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([100.0, 30.0]),
            VecVertex::corner([150.0, 60.0]),
        ],
        ..VecPath::default()
    });
    let mut pen = PenTool::new();
    pen.on_press_node(&mut scene, [100.0, 30.0], PTW, false);
    let cfg = SnapConfig {
        threshold: 1.0,
        ..SnapConfig::default()
    };
    let skip = pen.dragging_anchors().expect("âncora agarrada");
    let alvos = collect_targets(&scene, &VecXforms::default(), &[], &skip, false);
    let arrasta = |pen: &mut PenTool, scene: &mut VecScene, p: [f64; 2]| {
        pen.on_drag_encaixe(scene, p, &mut |q| snap(&[q], &alvos, cfg, None))
    };
    arrasta(&mut pen, &mut scene, [120.0, 0.3]);
    assert_eq!(
        scene.regras(),
        &[RegraCad::nova(Lei::Horizontal(SegCad(v(a, 1), v(b, 1))))]
    );
    arrasta(&mut pen, &mut scene, [80.0, 20.0]);
    assert!(scene.regras().is_empty(), "a guia apagou-se");
    arrasta(&mut pen, &mut scene, [50.2, 40.0]);
    pen.on_release();
    assert_eq!(
        scene.regras(),
        &[RegraCad::nova(Lei::Vertical(SegCad(v(a, 2), v(b, 1))))]
    );
}

/// O teste de posto só olha a componente ligada à regra nova: outra forma com as suas regras não muda a resposta.
#[test]
fn another_shape_with_its_own_rules_does_not_change_the_rank_answer() {
    let (mut scene, r) = retangulo_preso();
    let s = scene.push_path(rectangle([50.0, 50.0], [60.0, 58.0]));
    scene.regras_mut().extend([
        RegraCad::nova(Lei::Horizontal(SegCad(v(s, 1), v(s, 2)))),
        RegraCad::nova(Lei::Fixo(v(s, 1), [50.0, 50.0])),
    ]);
    let xf = VecXforms::default();
    junta_regra(&mut scene, &xf, Lei::Distancia(v(r, 1), v(r, 2), 10.0));
    junta_regra(&mut scene, &xf, Lei::Distancia(v(r, 2), v(r, 3), 5.0));
    let diagonal = Lei::Distancia(v(r, 1), v(r, 3), 125f64.sqrt());
    assert_eq!(
        junta_regra(&mut scene, &xf, diagonal).map(|g| g.referencia),
        Some(true)
    );
    let outra = Lei::Vertical(SegCad(v(s, 2), v(s, 3)));
    assert_eq!(
        junta_regra(&mut scene, &xf, outra),
        Some(RegraCad::nova(outra)),
        "a outra forma ainda tem liberdade"
    );
}
