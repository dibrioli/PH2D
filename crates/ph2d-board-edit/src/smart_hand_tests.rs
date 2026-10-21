//! ⭐ O QuickShape do quadro com traços de MÃO (smoke do dono 20/10, o «D»): o lápis do vetor é a lei
//! (o dono: «use ESSE») — a mesma forma reconhecida, e o caminho vivo É a forma exacta, que soltar não
//! muda. Os gates do B13 usavam polígonos perfeitos e só olhavam o elemento DEPOIS de soltar.

use super::smart_test_hand::{hand_ellipse, hand_rect, hand_strokes, hand_triangle, resting};
use super::*;
use ph2d_board_model::{Pen, ShapeType};
use ph2d_vec_edit::pencil_width::PenDynamics;
use ph2d_vec_edit::quickshape::QuickShape;

const T0: u128 = 1_000_000_000_000;
/// Desenhar: um evento a cada 4 ms (250 Hz); pousada: um a cada 8 ms.
const DT: u128 = 4_000_000;
const DT_REST: u128 = 8_000_000;
/// Depois do último evento, o quadro de pintura que chega quando a mão ficou parada.
const STILL: u128 = 600_000_000;

type Outline = Option<(bool, Vec<[f64; 2]>)>;

/// A caneta pousada: tremor de ±1 px (abaixo do passo do lápis) durante 600 ms, ou nada.
fn rests(end: [f64; 2]) -> [Vec<[f64; 2]>; 2] {
    [resting(end, 75, 1.0, 7), Vec::new()]
}

/// O traço `pts` e a pousada `rest` com a caneta do quadro, como a shell (o relógio de cada evento;
/// sem eventos, um quadro de pintura `STILL` depois). Devolve o relógio do último evento.
fn board_draw(w: &mut World, pts: &[[f64; 2]], rest: &[[f64; 2]]) -> u128 {
    w.ed.tool = Tool::Pen(Pen::Pen);
    let mut t = T0;
    w.ed.at_time(t);
    let down = at(pts[0][0], pts[0][1], NONE);
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, down, VIEW);
    for (k, p) in pts[1..].iter().chain(rest).enumerate() {
        t += if k + 1 < pts.len() { DT } else { DT_REST };
        w.ed.at_time(t);
        w.ed.pointer_move(&mut w.doc, &mut w.ts, at(p[0], p[1], NONE));
    }
    if rest.is_empty() {
        w.ed.at_time(t + STILL);
        w.ed.ink_hold(&mut w.doc, false);
    }
    t
}

fn outline(s: &QuickShape) -> (bool, Vec<[f64; 2]>) {
    let p = s.to_path();
    (p.closed, p.verts.iter().map(|v| v.anchor).collect())
}

/// O MESMO gesto pelo lápis do vetor (a porta que a shell do vetor usa: `on_drag` + `hold_tick` a cada
/// evento) e pela caneta do quadro: o caminho vivo de cada um.
fn both_ways(pts: &[[f64; 2]], rest: &[[f64; 2]]) -> [Outline; 2] {
    let mut scene = Default::default();
    let mut pencil = ph2d_vec_edit::Pencil::default();
    let dy = |t_ns| PenDynamics {
        pressure: 1.0,
        t_ns,
    };
    let mut t = T0;
    pencil.on_press(&mut scene, pts[0], PX, dy(t));
    for (k, &p) in pts[1..].iter().chain(rest).enumerate() {
        t += if k + 1 < pts.len() { DT } else { DT_REST };
        pencil.on_drag(&mut scene, p, dy(t));
        pencil.hold_tick(&mut scene, t, false, true);
    }
    if rest.is_empty() {
        pencil.hold_tick(&mut scene, t + STILL, false, true);
    }
    let vector = pencil.is_quick().then(|| {
        let p = scene
            .path(pencil.active_path().expect("o traço"))
            .expect("vivo");
        (p.closed, p.verts.iter().map(|v| v.anchor).collect())
    });
    let mut w = world();
    board_draw(&mut w, pts, rest);
    let board = match &w.ed.ink {
        Some(crate::ink::InkGesture::Draw { smart, .. }) => smart.hold.shape().map(|s| outline(&s)),
        _ => None,
    };
    [vector, board]
}

/// ⭐ (a) O quadro reconhece o MESMO que o lápis do vetor sobre traços de mão (tipo e vértices a menos
/// de 1 px de ecrã), também com a caneta pousada a tremer abaixo do passo do lápis.
#[test]
fn hand_strokes_recognise_on_the_board_as_in_the_vector_pencil() {
    for (name, pts) in hand_strokes() {
        for (mode, rest) in ["pousada a tremer", "parada"]
            .into_iter()
            .zip(rests(pts[pts.len() - 1]))
        {
            let [vector, board] = both_ways(&pts, &rest);
            let (Some((vc, va)), Some((bc, ba))) = (&vector, &board) else {
                panic!("{name} · {mode}: vetor {vector:?} · quadro {board:?}");
            };
            assert!(
                vc == bc && va.len() == ba.len(),
                "{name} · {mode}: vetor {va:?} · quadro {ba:?}"
            );
            for (v, b) in va.iter().zip(ba) {
                let d = (v[0] - b[0]).hypot(v[1] - b[1]);
                assert!(
                    d <= PX,
                    "{name} · {mode}: {d:.2} px · vetor {va:?} · quadro {ba:?}"
                );
            }
        }
    }
}

/// O que o documento DESENHA: os elementos vivos sem o contador de versão da colaboração (cada `Put`
/// o sobe; soltar regista o passo do traço cru e o da troca).
fn drawn(doc: &BoardDoc) -> Vec<Element> {
    doc.live_in_z_order()
        .into_iter()
        .map(|e| Element {
            version: 0,
            ..e.clone()
        })
        .collect()
}

/// ⭐ (b) VIVO = FINAL: com a forma reconhecida, o documento do segurar já é o de depois de soltar — o
/// objecto do quadro (rectângulo, elipse) ou, onde o quadro não o tem (o triângulo), a tinta limpa.
#[test]
fn the_live_shape_is_what_release_leaves() {
    for (name, pts, want) in [
        ("rect", hand_rect(), Some(ShapeType::Rectangle)),
        ("elipse", hand_ellipse(), Some(ShapeType::Ellipse)),
        ("triângulo", hand_triangle(), None),
    ] {
        for rest in rests(pts[pts.len() - 1]) {
            let mut w = world();
            let t = board_draw(&mut w, &pts, &rest);
            assert!(w.ed.is_quick(), "{name}: a forma nasceu");
            let held = drawn(&w.doc);
            assert_eq!(held.len(), 1, "{name}");
            assert_eq!(held[0].shape().map(|s| s.kind), want, "{name}: o vivo");
            assert_eq!(held[0].ink().is_some(), want.is_none(), "{name}: o vivo");
            let end = rest.last().unwrap_or(&pts[pts.len() - 1]);
            w.ed.at_time(t);
            w.ed.pointer_up(&mut w.doc, &mut w.h, at(end[0], end[1], NONE));
            let after = drawn(&w.doc);
            assert_eq!(held, after, "{name}: soltar não muda o documento");
        }
    }
}

/// A tinta limpa (o que nenhum objecto do quadro desenha igual) tem um ponto a cada px de ECRÃ: o
/// pintor da caneta alisa (perfect-freehand), e com só os cantos ele os cortava — o «D».
#[test]
fn the_clean_ink_has_a_point_every_screen_pixel() {
    let tri = QuickShape::Polygon {
        corners: vec![[0.0, 0.0], [300.0, 0.0], [60.0, -220.0]],
        closed: true,
    };
    for px in [0.5, 1.0, 2.5] {
        let pts = crate::smart::shape_points(&tri, px);
        let gap = pts
            .windows(2)
            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
            .fold(0.0, f64::max);
        assert!(gap <= px + 1e-9, "px {px}: {gap:.2}");
        for c in [[0.0, 0.0], [300.0, 0.0], [60.0, -220.0]] {
            assert!(pts.contains(&c), "o canto {c:?} fica exacto");
        }
    }
}

/// ⭐ O `Shift` SEM mexer, com a forma viva (smoke 21/10): o lápis do vetor relê-o a cada quadro
/// (`hold_tick(.., regular)`) e o quadro também (`ink_hold(.., shift)`) — a MESMA forma regular e,
/// largado, a MESMA forma livre.
#[test]
fn shift_without_moving_regularises_as_the_vector_pencil_does() {
    let steps = [true, false, true];
    for (name, pts) in [
        ("caixa", hand_rect()),
        ("elipse", hand_ellipse()),
        ("triângulo", hand_triangle()),
    ] {
        let mut scene = Default::default();
        let mut pencil = ph2d_vec_edit::Pencil::default();
        let dy = |t_ns| PenDynamics {
            pressure: 1.0,
            t_ns,
        };
        let mut t = T0;
        pencil.on_press(&mut scene, pts[0], PX, dy(t));
        for &p in &pts[1..] {
            t += DT;
            pencil.on_drag(&mut scene, p, dy(t));
            pencil.hold_tick(&mut scene, t, false, true);
        }
        t += STILL;
        pencil.hold_tick(&mut scene, t, false, true);
        let mut vector = Vec::new();
        for shift in steps {
            t += DT;
            pencil.hold_tick(&mut scene, t, shift, true);
            let p = scene
                .path(pencil.active_path().expect("o traço"))
                .expect("vivo");
            vector.push(p.verts.iter().map(|v| v.anchor).collect::<Vec<_>>());
        }
        let mut w = world();
        board_draw(&mut w, &pts, &[]);
        let mut board = Vec::new();
        for shift in steps {
            w.ed.ink_hold(&mut w.doc, shift);
            let Some(crate::ink::InkGesture::Draw { smart, .. }) = &w.ed.ink else {
                panic!("{name}: o gesto");
            };
            board.push(outline(&smart.hold.shape().expect("a forma viva")).1);
        }
        assert_ne!(vector[0], vector[1], "{name}: o vetor regulariza");
        for (k, (v, b)) in vector.iter().zip(&board).enumerate() {
            assert_eq!(v.len(), b.len(), "{name} · passo {k}");
            for (p, q) in v.iter().zip(b) {
                let d = (p[0] - q[0]).hypot(p[1] - q[1]);
                assert!(
                    d <= PX,
                    "{name} · passo {k}: {d:.2} px · vetor {v:?} · quadro {b:?}"
                );
            }
        }
    }
}
