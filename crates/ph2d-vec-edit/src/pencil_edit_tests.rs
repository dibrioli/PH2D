//! Gates do Edit Shape no lápis: a oferta ao soltar, abrir, agarrar um nó, afinar, soltar, o undo
//! que reencontra a forma, e cada porta que o fecha.

use super::*;
use crate::pencil_width::PenDynamics;
use crate::quickshape_hold::HOLD_NS;

fn dy(i: usize) -> PenDynamics {
    PenDynamics {
        pressure: 1.0,
        t_ns: i as u128 * 4_000_000,
    }
}

/// Um traço pelos `corners` (amostras a cada 2 un.), seguro meio segundo e solto.
fn quick(pencil: &mut Pencil, scene: &mut VecScene, corners: &[P2]) -> VecPathId {
    let mut pts = vec![corners[0]];
    for w in corners.windows(2) {
        let n = ((w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]) / 2.0).ceil() as usize;
        for k in 1..=n {
            let t = k as f64 / n as f64;
            pts.push([
                w[0][0] + (w[1][0] - w[0][0]) * t,
                w[0][1] + (w[1][1] - w[0][1]) * t,
            ]);
        }
    }
    let id = pencil.on_press(scene, pts[0], 1.0, dy(0));
    for (i, &p) in pts.iter().enumerate().skip(1) {
        pencil.on_drag(scene, p, dy(i));
    }
    let t = (pts.len() as u128 - 1) * 4_000_000;
    pencil.hold_tick(scene, t + HOLD_NS, false, true);
    assert!(pencil.is_quick(), "a forma nasceu");
    assert!(pencil.on_release(scene));
    id
}

const RECT: [P2; 5] = [
    [0.0, 0.0],
    [400.0, 0.0],
    [400.0, 250.0],
    [0.0, 250.0],
    [0.0, 0.0],
];

fn anchors(scene: &VecScene, id: VecPathId) -> Vec<P2> {
    scene.path(id).unwrap().verts.iter().map(|v| v.anchor).collect()
}

#[test]
fn a_released_quickshape_is_offered_and_the_button_opens_its_nodes() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    let id = quick(&mut pencil, &mut scene, &RECT);
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Offered);
    assert_eq!(pencil.shape_edit_path(), Some(id));
    assert!(pencil.shape_edit_nodes().0.is_empty(), "oferecido: nós escondidos");
    pencil.toggle_shape_edit();
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open);
    let (nodes, grabbed) = pencil.shape_edit_nodes();
    assert_eq!(nodes.len(), 4, "os cantos do rectângulo");
    assert_eq!(grabbed, None);
    pencil.toggle_shape_edit();
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Offered, "fecha; a oferta fica");
}

#[test]
fn a_free_stroke_offers_nothing_and_the_next_stroke_ends_the_offer() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    // Um traço livre (sem segurar).
    pencil.on_press(&mut scene, [0.0, 0.0], 1.0, dy(0));
    pencil.on_drag(&mut scene, [50.0, 30.0], dy(1));
    pencil.on_drag(&mut scene, [90.0, 10.0], dy(2));
    assert!(pencil.on_release(&mut scene));
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::None);
    quick(&mut pencil, &mut scene, &RECT);
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Offered);
    pencil.on_press(&mut scene, [900.0, 900.0], 1.0, dy(0));
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::None, "continuar a pintar fecha");
}

#[test]
fn grabbing_a_corner_reshapes_the_same_path_and_release_is_one_step() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    let id = quick(&mut pencil, &mut scene, &RECT);
    pencil.toggle_shape_edit();
    let n_paths = scene.paths().len();
    let corner = pencil.shape_edit_nodes().0[2];
    // Agarrado a 3 un. do canto (dentro dos 10 px): o canto NÃO salta para o ponteiro.
    let p = [corner[0] - 3.0, corner[1] + 2.0];
    assert!(pencil.shape_edit_press(&scene, p, 1.0, Xform::IDENTITY, false));
    assert!(pencil.is_active(), "o gesto é do Edit Shape");
    assert_eq!(pencil.active_path(), Some(id));
    assert_eq!(pencil.shape_edit_nodes().1, Some(2));
    assert!(pencil.on_drag(&mut scene, [p[0] + 40.0, p[1] + 25.0], dy(1)));
    assert!(pencil.on_release(&mut scene));
    assert_eq!(scene.paths().len(), n_paths, "nenhum caminho novo");
    assert_eq!(pencil.selected(), Some(id));
    let a = anchors(&scene, id);
    assert!(
        (a[2][0] - (corner[0] + 40.0)).abs() < 1e-9 && (a[2][1] - (corner[1] + 25.0)).abs() < 1e-9,
        "o canto andou o arrasto: {:?}",
        a[2]
    );
    assert!(crate::quickshape::edit::is_rectangle(&a), "continua rectângulo");
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open, "continua aberto");
    assert!(!pencil.is_active());
}

#[test]
fn a_press_away_from_the_nodes_closes_the_edit_and_is_a_stroke() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    quick(&mut pencil, &mut scene, &RECT);
    pencil.toggle_shape_edit();
    assert!(!pencil.shape_edit_press(&scene, [200.0, 125.0], 1.0, Xform::IDENTITY, false));
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::None);
}

#[test]
fn a_press_on_the_outline_scales_the_whole_shape() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    let id = quick(&mut pencil, &mut scene, &RECT);
    pencil.toggle_shape_edit();
    let before = anchors(&scene, id);
    // O meio do lado de cima, longe dos cantos: é o CORPO.
    assert!(pencil.shape_edit_press(&scene, [200.0, 0.0], 1.0, Xform::IDENTITY, false));
    assert_eq!(pencil.shape_edit_nodes().1, None);
    pencil.on_drag(&mut scene, [200.0, -125.0], dy(1));
    pencil.on_release(&mut scene);
    let after = anchors(&scene, id);
    let w = |a: &[P2]| a[1][0] - a[0][0];
    assert!((w(&after) / w(&before) - 2.0).abs() < 1e-9, "a distância ao centro dobrou");
}

#[test]
fn escape_mid_drag_puts_the_shape_back_and_keeps_the_edit_open() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    let id = quick(&mut pencil, &mut scene, &RECT);
    pencil.toggle_shape_edit();
    let before = anchors(&scene, id);
    let c = pencil.shape_edit_nodes().0[0];
    assert!(pencil.shape_edit_press(&scene, c, 1.0, Xform::IDENTITY, false));
    pencil.on_drag(&mut scene, [c[0] - 50.0, c[1] - 50.0], dy(1));
    pencil.cancel(&mut scene);
    assert_eq!(anchors(&scene, id), before, "voltou à de quando foi agarrada");
    assert!(scene.path(id).is_some(), "o cancel NÃO apaga a forma");
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open);
    assert!(pencil.shape_edit_escape(), "o Esc seguinte fecha os nós");
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Offered);
}

#[test]
fn shift_mid_drag_keeps_the_rectangles_proportion_live() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    let id = quick(&mut pencil, &mut scene, &RECT);
    pencil.toggle_shape_edit();
    let c = pencil.shape_edit_nodes().0[2];
    assert!(pencil.shape_edit_press(&scene, c, 1.0, Xform::IDENTITY, false));
    pencil.on_drag(&mut scene, [c[0] + 100.0, c[1]], dy(1));
    assert!(pencil.shape_edit_regular(&mut scene, true));
    let a = anchors(&scene, id);
    let (w, h) = (a[1][0] - a[0][0], a[3][1] - a[0][1]);
    assert!((w / h - 400.0 / 250.0).abs() < 1e-9, "{w}×{h}");
    assert!(!pencil.shape_edit_regular(&mut scene, true), "só quando muda");
}

#[test]
fn the_tick_ends_the_edit_on_another_selection_another_tool_or_another_writer() {
    for case in 0..3 {
        let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
        let id = quick(&mut pencil, &mut scene, &RECT);
        pencil.toggle_shape_edit();
        pencil.shape_edit_tick(&scene, Xform::IDENTITY, Some(id), true);
        assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open, "nada mudou");
        match case {
            0 => pencil.shape_edit_tick(&scene, Xform::IDENTITY, None, true),
            1 => pencil.shape_edit_tick(&scene, Xform::IDENTITY, Some(id), false),
            _ => {
                scene.path_mut(id).unwrap().verts[0].anchor[0] += 7.0;
                pencil.shape_edit_tick(&scene, Xform::IDENTITY, Some(id), true);
            }
        }
        assert_eq!(pencil.shape_edit_state(), ShapeEditState::None, "caso {case}");
    }
}

#[test]
fn undo_inside_the_edit_finds_the_shape_that_wrote_that_geometry() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    let id = quick(&mut pencil, &mut scene, &RECT);
    pencil.toggle_shape_edit();
    let born = scene.path(id).unwrap().clone();
    let c = pencil.shape_edit_nodes().0[2];
    pencil.shape_edit_press(&scene, c, 1.0, Xform::IDENTITY, false);
    pencil.on_drag(&mut scene, [c[0] + 60.0, c[1] + 40.0], dy(1));
    pencil.on_release(&mut scene);
    // O undo da shell repõe a geometria do nascimento: o Edit Shape reencontra a forma dela.
    *scene.path_mut(id).unwrap() = born;
    pencil.shape_edit_tick(&scene, Xform::IDENTITY, Some(id), true);
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open);
    assert!((pencil.shape_edit_nodes().0[2][0] - c[0]).abs() < 1e-9, "o canto de antes");
    // E a afinação seguinte parte DESSA forma.
    pencil.shape_edit_press(&scene, c, 1.0, Xform::IDENTITY, false);
    pencil.on_drag(&mut scene, [c[0], c[1] + 10.0], dy(1));
    pencil.on_release(&mut scene);
    assert!((anchors(&scene, id)[2][0] - c[0]).abs() < 1e-9);
}

#[test]
fn a_click_on_a_node_without_moving_changes_nothing_and_a_cancel_after_it_returns_to_birth() {
    // Um clique sem arrasto empilha uma forma IGUAL — invisível: o undo procura pela geometria
    // (a prova de mutação de 10/10 mediu-o: o descarte do degrau repetido saiu do código).
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    let id = quick(&mut pencil, &mut scene, &RECT);
    pencil.toggle_shape_edit();
    let born = scene.path(id).unwrap().clone();
    let c = pencil.shape_edit_nodes().0[1];
    pencil.shape_edit_press(&scene, c, 1.0, Xform::IDENTITY, false);
    assert!(pencil.on_release(&mut scene));
    assert_eq!(scene.path(id).unwrap().verts, born.verts);
    pencil.shape_edit_press(&scene, c, 1.0, Xform::IDENTITY, false);
    pencil.on_drag(&mut scene, [c[0] + 30.0, c[1]], dy(1));
    pencil.cancel(&mut scene);
    assert_eq!(scene.path(id).unwrap().verts, born.verts);
    pencil.shape_edit_tick(&scene, Xform::IDENTITY, Some(id), true);
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open);
}

#[test]
fn an_armed_head_that_recedes_the_path_keeps_receding_it_while_the_tip_is_dragged() {
    // A ponta TRIÂNGULO (armada no painel) avança para lá do nó: o caminho recua (`Marker::inset`)
    // para o bico cair onde a mão o pôs — e a afinação tem de recuar o MESMO, senão o caminho
    // escrito deixa de ser o da forma e o quadro seguinte fecha o Edit Shape.
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    pencil.set_style(crate::PenStyle {
        marker_end: ph2d_vec_scene::Marker::Triangle,
        stroke_w_px: 4.0,
        ..crate::PenStyle::default()
    });
    let id = quick(
        &mut pencil,
        &mut scene,
        &[[0.0, 0.0], [400.0, 0.0], [340.0, -40.0], [400.0, 0.0], [340.0, 40.0]],
    );
    let tip = crate::quickshape::edit::nodes(pencil.shape_edit_shape().unwrap())[1];
    let end = *anchors(&scene, id).last().unwrap();
    let inset = tip[0] - end[0];
    assert!(inset > 1.0, "o triângulo recua o caminho: {inset}");
    pencil.shape_edit_tick(&scene, Xform::IDENTITY, Some(id), true);
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Offered, "o caminho recuado É o da forma");
    pencil.toggle_shape_edit();
    pencil.shape_edit_press(&scene, tip, 1.0, Xform::IDENTITY, false);
    pencil.on_drag(&mut scene, [tip[0], tip[1] + 100.0], dy(1));
    pencil.on_release(&mut scene);
    let end = *anchors(&scene, id).last().unwrap();
    let to_tip = ((tip[0] - end[0]).powi(2) + (tip[1] + 100.0 - end[1]).powi(2)).sqrt();
    assert!((to_tip - inset).abs() < 1e-6, "o mesmo recuo: {to_tip} ≠ {inset}");
    pencil.shape_edit_tick(&scene, Xform::IDENTITY, Some(id), true);
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open);
}

#[test]
fn a_settled_path_is_written_in_its_local_space() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    let id = quick(&mut pencil, &mut scene, &RECT);
    pencil.toggle_shape_edit();
    // O assentamento da origem: a geometria passa a LOCAL, o afim leva-a de volta ao mundo.
    let xf = Xform([1.0, 0.0, 0.0, 1.0, 200.0, 125.0]);
    for v in &mut scene.path_mut(id).unwrap().verts {
        v.anchor = [v.anchor[0] - 200.0, v.anchor[1] - 125.0];
        v.in_handle = v.anchor;
        v.out_handle = v.anchor;
    }
    pencil.shape_edit_tick(&scene, xf, Some(id), true);
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open, "a mesma forma, em local");
    let c = pencil.shape_edit_nodes().0[2];
    assert_eq!(c, [400.0, 250.0], "os nós são de MUNDO");
    pencil.shape_edit_press(&scene, c, 1.0, xf, false);
    pencil.on_drag(&mut scene, [440.0, 275.0], dy(1));
    pencil.on_release(&mut scene);
    assert_eq!(anchors(&scene, id)[2], [240.0, 150.0], "escrito em LOCAL");
    pencil.shape_edit_tick(&scene, xf, Some(id), true);
    assert_eq!(pencil.shape_edit_state(), ShapeEditState::Open);
}

#[test]
fn the_arrow_keeps_its_heads_and_its_tip_follows_the_node() {
    let (mut scene, mut pencil) = (VecScene::default(), Pencil::default());
    // Uma linha com um V aberto no fim, segura.
    let id = quick(
        &mut pencil,
        &mut scene,
        &[[0.0, 0.0], [400.0, 0.0], [340.0, -40.0], [400.0, 0.0], [340.0, 40.0]],
    );
    assert!(matches!(
        pencil.shape_edit_shape(),
        Some(crate::quickshape::QuickShape::Arrow(_))
    ));
    let markers = |s: &VecScene| {
        let st = s.path(id).unwrap().stroke.clone().unwrap();
        (st.marker_start, st.marker_end, st.marker_scale)
    };
    let before = markers(&scene);
    let end_before = *anchors(&scene, id).last().unwrap();
    pencil.toggle_shape_edit();
    let tip = pencil.shape_edit_nodes().0[1];
    assert!(pencil.shape_edit_press(&scene, tip, 1.0, Xform::IDENTITY, false));
    pencil.on_drag(&mut scene, [tip[0], tip[1] + 100.0], dy(1));
    pencil.on_release(&mut scene);
    assert_eq!(markers(&scene), before, "as pontas ficam");
    let end = *anchors(&scene, id).last().unwrap();
    // O caminho recua o mesmo da ponta: o fim anda com o bico, 100 para baixo ao longo do cabo novo.
    assert!(end[1] > end_before[1] + 90.0, "{end:?}");
}
