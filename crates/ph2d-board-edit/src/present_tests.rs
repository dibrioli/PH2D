//! A apresentação (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 P1–P5). Números transcritos do GIF
//! oficial do Miro `21019593022866`: margem 7 px de 853 (0,8 %); a viagem chega a 83 % aos 0,50 s e a
//! 99 % aos 0,94 s.

use super::*;
use ph2d_board_model::{BoardOp, BoardSet, Element, Frame, Rgba, Style};

fn frame(b: &mut Board, name: &str, bx: [f64; 4], parent: Option<ElementId>) -> ElementId {
    let ink = Rgba([0, 0, 0, 255]);
    let mut f = Element::new_frame(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Frame {
            name: name.into(),
            style: Style::new(Some(Rgba([255; 4])), None, ink),
        },
        bx,
    );
    f.frame = parent;
    let id = f.id;
    BoardOp::Put(f).apply(&mut b.doc);
    id
}

fn board() -> (BoardSet, [ElementId; 4]) {
    let mut set = BoardSet::default();
    let a = set.create("Show".into());
    let b = set.get_mut(a).unwrap();
    let one = frame(b, "1", [0.0, 0.0, 1600.0, 900.0], None);
    let inner = frame(b, "1a", [100.0, 100.0, 200.0, 100.0], Some(one));
    let two = frame(b, "2", [2000.0, 0.0, 800.0, 800.0], None);
    let three = frame(b, "3", [4000.0, 0.0, 400.0, 400.0], None);
    (set, [one, inner, two, three])
}

/// ⭐ P1: os frames de TOPO, na ordem do painel; o de dentro não; o escondido não.
#[test]
fn slides_are_the_top_frames_in_panel_order_minus_the_hidden() {
    let (mut set, [one, _inner, two, three]) = board();
    let id = set.boards()[0].id;
    let b = set.get_mut(id).unwrap();
    assert_eq!(slides(b), [one, two, three]);
    b.team.set_slide_hidden(two, true);
    assert_eq!(slides(b), [one, three]);
}

/// P2: começa no seleccionado, ou no que contém o seleccionado, senão no 1.º.
#[test]
fn the_show_starts_at_the_selected_frame_or_its_container() {
    let (set, [one, inner, two, three]) = board();
    let b = &set.boards()[0];
    let s = slides(b);
    assert_eq!(first_slide(b, &s, &[]), 0);
    assert_eq!(first_slide(b, &s, &[three]), 2);
    assert_eq!(first_slide(b, &s, &[inner]), 0, "o de dentro → o de topo");
    assert_eq!(
        first_slide(b, &s, &[two, one]),
        1,
        "o 1.º da selecção manda"
    );
}

/// ⭐ P3: o diapositivo enquadra-se com 0,8 % do lado limitante — um 16:9 numa área 16:9 ocupa
/// 98,4 % da altura (o GIF: 839 de 853).
#[test]
fn a_slide_fits_with_the_measured_margin() {
    let (set, [one, _, two, _]) = board();
    let b = &set.boards()[0];
    let area = [0.0, 0.0, 1727.0, 853.0];
    let c = slide_camera(b, one, area).unwrap();
    let h = 900.0 * c.zoom;
    assert!(
        (h - (853.0 - 2.0 * 853.0 * SLIDE_MARGIN)).abs() < 1e-9,
        "{h}"
    );
    assert!((h / 853.0 - 0.984).abs() < 1e-3);
    assert_eq!([c.center_x, c.center_y], [800.0, 450.0]);
    let sq = slide_camera(b, two, area).unwrap();
    assert!(
        (800.0 * sq.zoom - 839.352).abs() < 1e-3,
        "o quadrado encaixa pela altura"
    );
}

/// ⭐ P5: a viagem é a curva ajustada aos dois pontos medidos, e acaba em `TRAVEL_SECS`.
#[test]
fn the_travel_matches_the_measured_points() {
    assert!((travel(0.50) - 0.83).abs() < 0.01, "{}", travel(0.50));
    assert!((travel(0.94) - 0.99).abs() < 0.005, "{}", travel(0.94));
    assert_eq!(travel(0.0), 0.0);
    assert_eq!(travel(-1.0), 0.0);
    assert_eq!(travel(TRAVEL_SECS), 1.0);
    assert_eq!(travel(10.0), 1.0);
    let a = Camera {
        center_x: 0.0,
        center_y: 0.0,
        zoom: 1.0,
    };
    let b = Camera {
        center_x: 100.0,
        center_y: -50.0,
        zoom: 100.0,
    };
    assert_eq!(camera_at(a, b, 0.0), a);
    assert_eq!(camera_at(a, b, 5.0), b);
    let mid = camera_at(a, b, TRAVEL_SECS / 2.0);
    let k = travel(TRAVEL_SECS / 2.0);
    assert!(
        (mid.zoom - 100f64.powf(k)).abs() < 1e-9,
        "zoom em escala geométrica"
    );
    assert!((mid.center_x - 100.0 * k).abs() < 1e-9);
}

/// ⭐ P3 — a margem é do lado que LIMITA: um frame largo numa área alta encaixa pela largura, com
/// 0,8 % dela (auditoria W8).
#[test]
fn the_margin_is_of_the_limiting_side() {
    let (set, [_, _, _, three]) = board();
    let b = &set.boards()[0];
    let area = [0.0, 0.0, 1000.0, 2000.0];
    let c = slide_camera(b, three, area).unwrap();
    let w = 400.0 * c.zoom;
    assert!((w - (1000.0 - 2.0 * 1000.0 * SLIDE_MARGIN)).abs() < 1e-9, "{w}");
    assert_eq!(travel(f64::NAN), 0.0);
}
