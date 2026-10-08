//! **Guias e alinhamento de objectos rodados** (W5b) pelo caminho de quem usa — o estudo e as fontes
//! de cada lei estão em `docs/MiroClone/03_guias_e_alinhamento.md`.
//!
//! As quatro queixas do dono (08/10), cada uma reproduzida VERMELHA antes da cura:
//! 1. objectos inclinados alinham mal — dois rodados ao mesmo ângulo não ficam com as bordas na mesma
//!    linha inclinada;
//! 2. alinhar a rotação não existe — rodar perto do ângulo de outro objecto não encaixa nele;
//! 3. frames: o frame não é alvo, e arrastado alinha a união dos FILHOS (vazio, nada);
//! 4. dentro de um frame rodado as formas não se alinham entre si (nos eixos do frame).

use super::*;
use ph2d_board_model::Frame as BoardFrame;

const DEG: f64 = std::f64::consts::PI / 180.0;

/// Uma caixa `[x, y, w, h]` de tamanho `size` com o centro em `c`.
fn centred(c: [f64; 2], size: [f64; 2]) -> [f64; 4] {
    [c[0] - size[0] / 2.0, c[1] - size[1] / 2.0, size[0], size[1]]
}

/// O ponto do mundo a `(du, dv)` de `c` nos eixos rodados de `angle`.
fn local(c: [f64; 2], angle: f64, du: f64, dv: f64) -> [f64; 2] {
    let (s, k) = angle.sin_cos();
    [c[0] + du * k - dv * s, c[1] + du * s + dv * k]
}

fn rect(w: &mut World, bx: [f64; 4], angle: f64, home: Option<ElementId>) -> ElementId {
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: w.ed.style.clone(),
        text: Default::default(),
    };
    let mut el = Element::new_shape(w.doc.mint_id(), w.doc.z_on_top(), shape, bx);
    el.angle = angle;
    el.frame = home;
    let id = el.id;
    BoardOp::Put(el).apply(&mut w.doc);
    id
}

fn board_frame(w: &mut World, bx: [f64; 4], angle: f64) -> ElementId {
    let f = BoardFrame {
        name: "F".to_owned(),
        style: w.ed.style.clone(),
    };
    let mut el = Element::new_frame(w.doc.mint_id(), w.doc.z_on_top(), f, bx);
    el.angle = angle;
    let id = el.id;
    BoardOp::Put(el).apply(&mut w.doc);
    id
}

/// A caixa de `el` nos eixos rodados de `angle`: `[u0, v0, u1, v1]`.
fn along(el: &Element, angle: f64) -> [f64; 4] {
    let (s, k) = angle.sin_cos();
    el.corners().iter().fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |b, p| {
            let u = p[0] * k + p[1] * s;
            let v = -p[0] * s + p[1] * k;
            [b[0].min(u), b[1].min(v), b[2].max(u), b[3].max(v)]
        },
    )
}

/// Pré-condição: nenhuma paragem da caixa envolvente de `a` está a menos de `tol` de uma de `b` —
/// o encaixe que o teste espera não pode vir dos eixos do mundo.
fn far_in_world(a: &Element, b: &Element, tol: f64) -> bool {
    let (x, y) = (a.aabb(), b.aabb());
    (0..2).all(|axis| {
        let s = |q: [f64; 4]| [q[axis], (q[axis] + q[axis + 2]) / 2.0, q[axis + 2]];
        s(x).iter()
            .all(|p| s(y).iter().all(|q| (p - q).abs() > tol))
    })
}

fn guides(w: &mut World) -> Vec<Guide> {
    w.ed.overlay(&mut w.doc, &mut w.ts).guides
}

/// Arrasta de `from` por `d`, devolve as guias ANTES de largar.
fn drag_by(w: &mut World, from: [f64; 2], d: [f64; 2]) -> Vec<Guide> {
    let to = [from[0] + d[0], from[1] + d[1]];
    let mid = [from[0] + d[0] / 2.0, from[1] + d[1] / 2.0];
    w.ed.pointer_down(
        &mut w.doc,
        &mut w.h,
        &mut w.ts,
        at(from[0], from[1], NONE),
        VIEW,
    );
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(mid[0], mid[1], NONE));
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(to[0], to[1], NONE));
    let g = guides(w);
    w.ed.pointer_up(&mut w.doc, &mut w.h, at(to[0], to[1], NONE));
    g
}

/// A guia vai na direcção de `angle` (mod 180°).
fn runs_along(g: &Guide, angle: f64) -> bool {
    let d = [g.b[0] - g.a[0], g.b[1] - g.a[1]];
    let (s, k) = angle.sin_cos();
    (d[0] * s - d[1] * k).abs() < 1e-6 * d[0].hypot(d[1]).max(1.0)
}

/// ⭐ Queixa 1: dois rectângulos a 30° — arrastar um para a borda do outro põe-nos na MESMA linha
/// inclinada, e a guia desenha-se nessa linha (não na caixa envolvente).
#[test]
fn two_shapes_at_the_same_angle_align_along_their_rotated_edges() {
    let mut w = world();
    let t = 30.0 * DEG;
    let ca = [0.0, 0.0];
    let a = rect(&mut w, centred(ca, [200.0, 100.0]), t, None);
    // A borda esquerda de B (meia largura 50) a 3 da de A (meia largura 100), bem abaixo dela.
    let cb = local(ca, t, -100.0 + 3.0 + 50.0, 210.0);
    let b = rect(&mut w, centred(cb, [100.0, 60.0]), t, None);
    assert!(far_in_world(w.el(a), w.el(b), 12.0), "pré-condição");
    let g = drag_by(&mut w, cb, [-4.0, 0.0]);
    let (ua, ub) = (along(w.el(a), t), along(w.el(b), t));
    assert!(
        (ua[0] - ub[0]).abs() < 1e-6,
        "a borda de B não foi para a linha da de A: {} contra {}",
        ub[0],
        ua[0]
    );
    assert!(
        g.iter().any(|g| runs_along(g, t + 90.0 * DEG)),
        "nenhuma guia na linha inclinada: {g:?}"
    );
}

/// ⭐ Queixa 2: rodar B perto do ângulo de A (30°) encaixa nesse ângulo, com a guia a mostrá-lo.
#[test]
fn rotating_near_the_angle_of_another_shape_snaps_to_it() {
    let mut w = world();
    rect(
        &mut w,
        centred([400.0, 0.0], [100.0, 60.0]),
        30.0 * DEG,
        None,
    );
    let b = rect(&mut w, centred([0.0, 0.0], [100.0, 60.0]), 0.0, None);
    w.ed.select(&w.doc, [b]);
    let knob = crate::Frame::of(w.el(b)).handle(Handle::Rotate, PX, &metrics());
    // A pega está por cima do centro; o ponteiro dá a volta a 100 do centro até 28,5° — 2,6 px de
    // arco do ângulo de A, dentro dos 6 da tolerância.
    let a0 = knob[1].atan2(knob[0]);
    let p = |deg: f64| {
        let a = a0 + deg * DEG;
        at(100.0 * a.cos(), 100.0 * a.sin(), NONE)
    };
    w.ed.pointer_down(
        &mut w.doc,
        &mut w.h,
        &mut w.ts,
        at(knob[0], knob[1], NONE),
        VIEW,
    );
    w.ed.pointer_move(&mut w.doc, &mut w.ts, p(14.0));
    w.ed.pointer_move(&mut w.doc, &mut w.ts, p(28.5));
    let g = guides(&mut w);
    w.ed.pointer_up(&mut w.doc, &mut w.h, p(28.5));
    assert!(
        (w.el(b).angle - 30.0 * DEG).abs() < 1e-9,
        "rodou para {}°",
        w.el(b).angle / DEG
    );
    assert!(!g.is_empty(), "o encaixe do ângulo não se mostrou");
}

/// ⭐ Queixa 3a: um frame é ALVO — uma forma arrastada cola na borda dele.
#[test]
fn a_frame_is_a_snap_target() {
    let mut w = world();
    board_frame(&mut w, [0.0, 0.0, 400.0, 300.0], 0.0);
    let r = rect(&mut w, [407.0, 110.0, 100.0, 50.0], 0.0, None);
    drag_by(&mut w, [457.0, 135.0], [-4.0, 0.0]);
    assert_eq!(w.el(r).x, 400.0, "não colou na borda do frame");
}

/// ⭐ Queixa 3b: arrastar um frame (vazio ou com filhos) alinha a caixa DO FRAME, não a dos filhos.
#[test]
fn dragging_a_frame_aligns_the_frame_box() {
    for with_child in [false, true] {
        let mut w = world();
        let f = board_frame(&mut w, [0.0, 0.0, 400.0, 300.0], 0.0);
        if with_child {
            rect(&mut w, [300.0, 100.0, 40.0, 40.0], 0.0, Some(f));
        }
        rect(&mut w, [500.0, 1000.0, 100.0, 50.0], 0.0, None);
        // O miolo vazio agarra o frame; a borda direita vai a 497, a 3 da esquerda do rectângulo.
        drag_by(&mut w, [100.0, 150.0], [97.0, 0.0]);
        assert_eq!(w.el(f).x, 100.0, "com filho: {with_child}");
    }
}

/// ⭐ Queixa 4: dentro de um frame rodado a 20°, os filhos alinham-se entre si nos eixos DO FRAME.
#[test]
fn inside_a_rotated_frame_children_align_in_the_frame_axes() {
    let mut w = world();
    let t = 20.0 * DEG;
    let cf = [0.0, 0.0];
    let f = board_frame(&mut w, centred(cf, [1000.0, 800.0]), t);
    let ca = local(cf, t, -200.0, -100.0);
    let a = rect(&mut w, centred(ca, [100.0, 60.0]), t, Some(f));
    // A esquerda de B (meia largura 40) a 3 da de A (meia largura 50).
    let cb = local(cf, t, -200.0 - 50.0 + 3.0 + 40.0, 150.0);
    let b = rect(&mut w, centred(cb, [80.0, 60.0]), t, Some(f));
    assert!(far_in_world(w.el(a), w.el(b), 12.0), "pré-condição");
    drag_by(&mut w, cb, [-4.0, 0.0]);
    let (ua, ub) = (along(w.el(a), t), along(w.el(b), t));
    assert!(
        (ua[0] - ub[0]).abs() < 1e-6,
        "B não alinhou com A nos eixos do frame: {} contra {}",
        ub[0],
        ua[0]
    );
    assert_eq!(w.el(b).frame, Some(f), "continua dentro");
}

/// Guarda (verde antes da W5b): num frame NÃO rodado os filhos alinham-se entre si.
#[test]
fn inside_a_straight_frame_children_align_with_each_other() {
    let mut w = world();
    let f = board_frame(&mut w, [0.0, 0.0, 1000.0, 800.0], 0.0);
    rect(&mut w, [200.0, 200.0, 100.0, 60.0], 0.0, Some(f));
    let b = rect(&mut w, [207.0, 500.0, 80.0, 60.0], 0.0, Some(f));
    drag_by(&mut w, [247.0, 530.0], [-4.0, 0.0]);
    assert_eq!(w.el(b).x, 200.0);
}

/// Roda `b` (seleccionada) com o ponteiro a `radius` do centro dela, até `deg` graus da pega.
fn rotate_to(w: &mut World, b: ElementId, radius: f64, deg: f64, m: Mods) -> Vec<Guide> {
    w.ed.select(&w.doc, [b]);
    let c = w.el(b).center();
    let knob = crate::Frame::of(w.el(b)).handle(Handle::Rotate, PX, &metrics());
    let a0 = (knob[1] - c[1]).atan2(knob[0] - c[0]);
    let p = |d: f64| {
        let a = a0 + d * DEG;
        at(c[0] + radius * a.cos(), c[1] + radius * a.sin(), m)
    };
    w.ed.pointer_down(
        &mut w.doc,
        &mut w.h,
        &mut w.ts,
        at(knob[0], knob[1], m),
        VIEW,
    );
    w.ed.pointer_move(&mut w.doc, &mut w.ts, p(deg / 2.0));
    w.ed.pointer_move(&mut w.doc, &mut w.ts, p(deg));
    let g = guides(w);
    w.ed.pointer_up(&mut w.doc, &mut w.h, p(deg));
    g
}

/// A tolerância do ângulo é em px no ARCO do ponteiro: 1,5° a 300 do centro são 7,9 px (> 6) — não
/// encaixa; e o `Ctrl` suspende-o como suspende as guias.
#[test]
fn the_angle_snap_is_measured_on_the_arc_and_ctrl_suspends_it() {
    for (radius, m, snaps) in [
        (100.0, NONE, true),
        (300.0, NONE, false),
        (100.0, CTRL, false),
    ] {
        let mut w = world();
        rect(
            &mut w,
            centred([400.0, 0.0], [100.0, 60.0]),
            30.0 * DEG,
            None,
        );
        let b = rect(&mut w, centred([0.0, 0.0], [100.0, 60.0]), 0.0, None);
        let g = rotate_to(&mut w, b, radius, 28.5, m);
        let want = if snaps { 30.0 } else { 28.5 };
        assert!(
            (w.el(b).angle / DEG - want).abs() < 1e-6,
            "raio {radius}, ctrl {}: {}°",
            m.ctrl,
            w.el(b).angle / DEG
        );
        assert_eq!(!g.is_empty(), snaps);
    }
}

/// O ângulo dos alvos conta mod 90°: um alvo a 30° encaixa quem roda até 120°; e um alvo DIREITO
/// encaixa em 90° (um rodado volta a ficar direito sem `Shift`).
#[test]
fn the_angle_snap_counts_the_target_angle_modulo_a_right_angle() {
    for (target, deg, want) in [(30.0, 118.5, 120.0), (0.0, 88.0, 90.0)] {
        let mut w = world();
        rect(
            &mut w,
            centred([400.0, 0.0], [100.0, 60.0]),
            target * DEG,
            None,
        );
        let b = rect(&mut w, centred([0.0, 0.0], [100.0, 60.0]), 0.0, None);
        rotate_to(&mut w, b, 100.0, deg, NONE);
        assert!(
            (w.el(b).angle / DEG - want).abs() < 1e-6,
            "{}°",
            w.el(b).angle / DEG
        );
    }
}

/// `Shift` continua a rodar em passos de 15° (e não procura o ângulo dos outros).
#[test]
fn shift_still_steps_fifteen_degrees() {
    let mut w = world();
    rect(
        &mut w,
        centred([400.0, 0.0], [100.0, 60.0]),
        31.0 * DEG,
        None,
    );
    let b = rect(&mut w, centred([0.0, 0.0], [100.0, 60.0]), 0.0, None);
    rotate_to(&mut w, b, 100.0, 29.0, SHIFT);
    assert!((w.el(b).angle / DEG - 30.0).abs() < 1e-6);
}

/// ⭐ Espaçamento igual (o Miro): ao lado de um par, o vão do par (43 → 40); entre dois vizinhos, ao
/// meio. As guias são os vãos iguais.
#[test]
fn equal_spacing_beside_a_pair_and_between_two_neighbours() {
    let mut w = world();
    rect(&mut w, [0.0, 0.0, 100.0, 60.0], 0.0, None);
    rect(&mut w, [140.0, 0.0, 100.0, 60.0], 0.0, None);
    let c = rect(&mut w, [283.0, 10.0, 100.0, 60.0], 0.0, None);
    let g = drag_by(&mut w, [333.0, 40.0], [-4.0, 0.0]);
    assert_eq!(w.el(c).x, 280.0, "o vão do par é 40");
    let gaps: Vec<f64> = g
        .iter()
        .map(|g| g.b[0] - g.a[0])
        .filter(|d| *d > 0.0)
        .collect();
    assert_eq!(gaps, vec![40.0, 40.0], "{g:?}");

    let mut w = world();
    rect(&mut w, [0.0, 0.0, 100.0, 60.0], 0.0, None);
    rect(&mut w, [300.0, 0.0, 100.0, 60.0], 0.0, None);
    let c = rect(&mut w, [153.0, 10.0, 100.0, 60.0], 0.0, None);
    drag_by(&mut w, [203.0, 40.0], [-4.0, 0.0]);
    assert_eq!(w.el(c).x, 150.0, "ao meio: 50 de cada lado");
}

/// ⭐ Dentro de um frame rodado a 20° o espaçamento igual vale nos eixos do frame.
#[test]
fn equal_spacing_inside_a_rotated_frame_runs_along_the_frame_axes() {
    let mut w = world();
    let t = 20.0 * DEG;
    let f = board_frame(&mut w, centred([0.0, 0.0], [1400.0, 800.0]), t);
    let kid = |w: &mut World, du: f64, dv: f64| {
        let c = local([0.0, 0.0], t, du, dv);
        rect(w, centred(c, [100.0, 60.0]), t, Some(f))
    };
    let a = kid(&mut w, -400.0, 0.0);
    kid(&mut w, -400.0 + 140.0, 0.0);
    let c = kid(&mut w, -400.0 + 283.0, 10.0);
    let cc = w.el(c).center();
    drag_by(&mut w, cc, [-4.0 * t.cos(), -4.0 * t.sin()]);
    let gap = along(w.el(c), t)[0] - along(w.el(a), t)[0];
    assert!((gap - 280.0).abs() < 1e-6, "{gap}");
}

/// ⭐ Redimensionar um RODADO encaixa a pega nos eixos dele: a borda direita de B (30°) vai para a
/// linha da de A.
#[test]
fn resizing_a_rotated_shape_snaps_in_its_own_axes() {
    let mut w = world();
    let t = 30.0 * DEG;
    let a = rect(&mut w, centred([0.0, 0.0], [200.0, 100.0]), t, None);
    let b = rect(
        &mut w,
        centred(local([0.0, 0.0], t, -20.0, 200.0), [100.0, 60.0]),
        t,
        None,
    );
    w.ed.select(&w.doc, [b]);
    let knob = crate::Frame::of(w.el(b)).handle(Handle::Resize(Dir::E), PX, &metrics());
    let to = local([0.0, 0.0], t, 97.0, 200.0);
    w.drag(knob, to, NONE);
    let (ua, ub) = (along(w.el(a), t), along(w.el(b), t));
    assert!((ua[2] - ub[2]).abs() < 1e-6, "{} contra {}", ub[2], ua[2]);
}

/// O frame onde um filho vive é alvo dele (o «snap to parent» do Godot): a borda de dentro.
#[test]
fn a_child_snaps_to_the_edge_of_its_own_frame() {
    let mut w = world();
    let f = board_frame(&mut w, [0.0, 0.0, 600.0, 400.0], 0.0);
    let c = rect(&mut w, [7.0, 107.0, 50.0, 50.0], 0.0, Some(f));
    drag_by(&mut w, [32.0, 132.0], [-4.0, 0.0]);
    assert_eq!(w.el(c).x, 0.0);
    assert_eq!(w.el(c).frame, Some(f));
}
