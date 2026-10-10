//! Gates do Edit Shape (F7e) no QUADRO do lápis: o clique da ④ que o quadro consome, o press que
//! agarra um nó com a mão crua, o `Esc`, a outra ferramenta e o overlay de nós.

use crate::state::VecState;
use ph2d_vec_edit::pencil::ShapeEditState;
use ph2d_vec_scene::VecScene;

/// Um rectângulo desenhado, seguro e solto com o lápis na mão — selecionado como a shell faz.
fn held_rectangle() -> (VecState, VecScene) {
    let mut vec = VecState::default();
    vec.draw_config.mode = ph2d_tool_vector::DrawMode::Pencil;
    let mut scene = VecScene::default();
    let d = |i: u128| ph2d_vec_edit::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: i * 4_000_000,
    };
    let c = [
        [0.0, 0.0],
        [400.0, 0.0],
        [400.0, 250.0],
        [0.0, 250.0],
        [0.0, 0.0],
    ];
    let mut n = 0u128;
    vec.pencil.on_press(&mut scene, c[0], 1.0, d(0));
    for w in c.windows(2) {
        for k in 1..=50 {
            let u = f64::from(k) / 50.0;
            n += 1;
            let p = [
                w[0][0] + (w[1][0] - w[0][0]) * u,
                w[0][1] + (w[1][1] - w[0][1]) * u,
            ];
            vec.pencil.on_drag(&mut scene, p, d(n));
        }
    }
    let held = n * 4_000_000 + ph2d_vec_edit::quickshape_hold::HOLD_NS;
    assert!(
        super::frame(&mut vec, &mut scene, held, false),
        "a forma nasce"
    );
    assert!(vec.pencil.on_release(&mut scene));
    vec.pen.select(vec.pencil.selected());
    (vec, scene)
}

fn open(vec: &mut VecState, scene: &mut VecScene) {
    super::frame(vec, scene, 0, false);
    assert_eq!(vec.pencil.shape_edit_state(), ShapeEditState::Offered);
    ph2d_panel_vector::request_shape_edit_toggle();
    super::frame(vec, scene, 0, false);
    assert_eq!(
        vec.pencil.shape_edit_state(),
        ShapeEditState::Open,
        "o clique da ④ abre"
    );
}

#[test]
fn the_button_click_opens_the_nodes_and_a_press_on_one_grabs_it_with_the_raw_hand() {
    let (mut vec, mut scene) = held_rectangle();
    open(&mut vec, &mut scene);
    assert_eq!(
        crate::edit_marks::node_overlay_selected(&vec),
        None,
        "as alças genéricas saem enquanto os nós próprios estão à vista"
    );
    let corner = vec.pencil.shape_edit_nodes().0[2];
    let d = ph2d_vec_edit::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: 0,
    };
    crate::pencil_input::press(&mut vec, &mut scene, corner, 1.0, d, false);
    assert!(vec.pencil.is_active() && vec.pencil.shape_edit_nodes().1 == Some(2));
    // A mão: o estabilizador forte atrasaria o ponteiro; com o nó agarrado ele é CRU.
    vec.pencil_hand.begin((0.0, 0.0));
    assert_eq!(vec.pencil_hand.filter((100.0, 50.0), 0.9), (100.0, 50.0));
    vec.pencil.on_release(&mut scene);
    // Um press longe dos nós: o Edit Shape acaba e é um traço — com a mão estabilizada de novo.
    crate::pencil_input::press(&mut vec, &mut scene, [900.0, 900.0], 1.0, d, false);
    assert_eq!(vec.pencil.shape_edit_state(), ShapeEditState::None);
    assert!(vec.pencil.is_active(), "um traço novo");
    vec.pencil_hand.begin((0.0, 0.0));
    assert_ne!(vec.pencil_hand.filter((100.0, 50.0), 0.9), (100.0, 50.0));
}

#[test]
fn escape_closes_the_nodes_and_another_tool_ends_the_offer() {
    let (mut vec, mut scene) = held_rectangle();
    open(&mut vec, &mut scene);
    assert!(super::escape(&mut vec, &mut scene), "o Esc é do Edit Shape");
    assert_eq!(vec.pencil.shape_edit_state(), ShapeEditState::Offered);
    assert_eq!(
        crate::edit_marks::node_overlay_selected(&vec),
        vec.pen.selected()
    );
    assert!(!super::escape(&mut vec, &mut scene), "o 2.º Esc é de outro");
    vec.draw_config.mode = ph2d_tool_vector::DrawMode::Node;
    super::frame(&mut vec, &mut scene, 0, false);
    assert_eq!(vec.pencil.shape_edit_state(), ShapeEditState::None);
}

#[test]
fn shift_held_mid_drag_reaches_the_shape_through_the_frame() {
    let (mut vec, mut scene) = held_rectangle();
    open(&mut vec, &mut scene);
    let id = vec.pencil.selected().unwrap();
    let corner = vec.pencil.shape_edit_nodes().0[2];
    let d = ph2d_vec_edit::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: 0,
    };
    crate::pencil_input::press(&mut vec, &mut scene, corner, 1.0, d, false);
    vec.pencil
        .on_drag(&mut scene, [corner[0] + 200.0, corner[1]], d);
    super::frame(&mut vec, &mut scene, 0, true);
    let v: Vec<[f64; 2]> = scene
        .path(id)
        .unwrap()
        .verts
        .iter()
        .map(|v| v.anchor)
        .collect();
    let (w, h) = (v[1][0] - v[0][0], v[3][1] - v[0][1]);
    assert!((w / h - 1.6).abs() < 1e-9, "a proporção do Shift: {w}×{h}");
}
