//! Gates do GESTO da seta, pelo [`crate::Pencil`] (F7d) — irmão de `quickshape_arrow_tests.rs` (a
//! lei) pelo tecto de 700 LOC: segurar põe a ponta, soltar guarda-a, `Esc` tira-a, o arrasto escala
//! e roda a seta inteira, o `Shift` a meio mantém-na, a ponta armada manda (a ponta da seta não
//! passa ao traço seguinte: as pontas ARMADAS da tool, `ph2d-tool-vector/src/tool_armed_tests.rs`).

use super::arrow_tests::{Kind, arrow_poly, held_with};
use super::pencil_tests::draw;
use super::*;
use crate::quickshape_hold::{HOLD_NS, HoldTick};
use ph2d_vec_scene::Marker;

/// Uma seta de mão de cabo recto de 300 px a 37° e farpas de 60 px: a ponta desenhada.
fn arrow_hand() -> (Vec<P2>, P2) {
    let a = 37f64.to_radians();
    let tip = [300.0 * a.cos(), 300.0 * a.sin()];
    let p = arrow_poly(&[[0.0, 0.0], tip], 60.0, 30.0, Kind::Open, false);
    (held_with(&p, 1.0, 0, 0.5, 1.0), tip)
}

fn held_pencil(style: crate::PenStyle) -> (ph2d_vec_scene::VecScene, crate::Pencil, P2, u128) {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut pencil = crate::Pencil::default();
    pencil.set_style(style);
    let (pts, tip) = arrow_hand();
    let t = draw(&mut pencil, &mut scene, &pts);
    let id = pencil.active_path().unwrap();
    assert_eq!(
        scene.path(id).unwrap().stroke.as_ref().unwrap().marker_end,
        style.marker_end,
        "o traço livre tem a ponta do estilo"
    );
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS, false, true),
        HoldTick::Recognised
    );
    (scene, pencil, tip, t)
}

/// ⭐ **Segurar a seta põe a ponta no traço vivo, e soltar guarda-a** — a `Open` (a «arrow» de
/// nascença do Excalidraw) com o lápis sem ponta armada; o tamanho é o das farpas (o alcance da
/// `Open` é √20 larguras); o caminho acaba na ponta desenhada.
#[test]
fn holding_an_arrow_puts_the_head_on_the_live_path_and_release_keeps_it() {
    let (mut scene, mut pencil, tip, _) = held_pencil(crate::PenStyle::default());
    let id = pencil.active_path().unwrap();
    let p = scene.path(id).unwrap();
    let s = p.stroke.as_ref().unwrap();
    assert_eq!((s.marker_start, s.marker_end), (Marker::None, Marker::Open));
    let drawn = 60.0 / (3.0 * 20f64.sqrt());
    assert!(
        (s.marker_scale(false) / drawn - 1.0).abs() < 0.35,
        "tamanho {} vs {drawn}",
        s.marker_scale(false)
    );
    // F16c: o tamanho é POR PONTA — o início, sem farpa, fica com o do estilo.
    assert_eq!(s.marker_scale(true), 1.0, "o inicio nao tem farpa");
    assert_eq!(p.verts.len(), 2, "o cabo é uma recta: {:?}", p.verts);
    assert!(
        dist(p.verts[1].anchor, tip) < 4.0,
        "{:?} vs {tip:?}",
        p.verts[1].anchor
    );
    assert!(pencil.on_release(&mut scene));
    let s = scene.path(id).unwrap().stroke.as_ref().unwrap();
    assert_eq!(
        s.marker_end,
        Marker::Open,
        "soltar comete a seta com a ponta"
    );
}

/// **`Esc` devolve o traço livre SEM ponta** (e com o tamanho de ponta do estilo).
#[test]
fn esc_gives_back_the_free_stroke_without_the_head() {
    let (mut scene, mut pencil, _, _) = held_pencil(crate::PenStyle::default());
    let id = pencil.active_path().unwrap();
    assert!(pencil.quick_dismiss(&mut scene));
    let p = scene.path(id).unwrap();
    let s = p.stroke.as_ref().unwrap();
    assert_eq!(
        (
            s.marker_start,
            s.marker_end,
            [s.marker_scale_start, s.marker_scale_end]
        ),
        (Marker::None, Marker::None, [1.0; 2])
    );
    assert!(
        p.verts.len() > 2,
        "o traço livre volta: {} nós",
        p.verts.len()
    );
}

/// ⭐ **Arrastar escala e roda a seta INTEIRA** — o cabo à volta do início e a ponta com ele (o
/// tamanho dobra quando a seta dobra).
#[test]
fn the_drag_scales_and_turns_the_whole_arrow() {
    let (mut scene, mut pencil, _, t) = held_pencil(crate::PenStyle::default());
    let id = pencil.active_path().unwrap();
    let (s0, v0) = {
        let p = scene.path(id).unwrap();
        (p.stroke.as_ref().unwrap().marker_scale(false), p.verts.clone())
    };
    let end = *pencil.samples().last().unwrap();
    let pivot = v0[0].anchor;
    let far = add(pivot, mul(rotate(sub(end, pivot), FRAC_PI_2), 2.0));
    let dy = crate::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: t + HOLD_NS,
    };
    assert!(pencil.on_drag(&mut scene, far, dy));
    let p = scene.path(id).unwrap();
    let s = p.stroke.as_ref().unwrap();
    assert_eq!(s.marker_end, Marker::Open);
    assert!(
        (s.marker_scale(false) / s0 - 2.0).abs() < 1e-9,
        "a ponta dobra: {s0} → {}",
        s.marker_scale(false)
    );
    let want = add(pivot, mul(rotate(sub(v0[1].anchor, pivot), FRAC_PI_2), 2.0));
    assert!(dist(p.verts[0].anchor, pivot) < 1e-9, "o início fica");
    assert!(
        dist(p.verts[1].anchor, want) < 1e-6,
        "{:?} vs {want:?}",
        p.verts[1].anchor
    );
}

/// **O `Shift` a meio** regulariza o cabo e a seta continua seta, com a ponta.
#[test]
fn the_shift_mid_hold_keeps_the_arrow_and_its_head() {
    let (mut scene, mut pencil, _, t) = held_pencil(crate::PenStyle::default());
    let id = pencil.active_path().unwrap();
    assert_eq!(
        pencil.hold_tick(&mut scene, t + 2 * HOLD_NS, true, true),
        HoldTick::Reshaped
    );
    let p = scene.path(id).unwrap();
    assert_eq!(p.stroke.as_ref().unwrap().marker_end, Marker::Open);
    let deg = angle(sub(p.verts[1].anchor, p.verts[0].anchor)).to_degrees();
    assert!(
        (deg - 30.0).abs() < 1e-6 || (deg - 45.0).abs() < 1e-6,
        "{deg}°"
    );
}

/// **A ponta ARMADA no lápis é a da seta, e o bico cai na ponta desenhada** — o triângulo cresce
/// para lá do nó (`Marker::inset`), por isso o caminho recua o que ele avança.
#[test]
fn an_armed_head_is_the_arrows_and_its_tip_lands_on_the_drawn_tip() {
    let style = crate::PenStyle {
        marker_end: Marker::Triangle,
        ..crate::PenStyle::default()
    };
    let (scene, pencil, tip, _) = held_pencil(style);
    let p = scene.path(pencil.active_path().unwrap()).unwrap();
    let s = p.stroke.as_ref().unwrap();
    assert_eq!(s.marker_end, Marker::Triangle);
    let (node, dir) = ph2d_vec_scene::end_tangent(p, false).unwrap();
    let bico = add(
        node,
        mul(dir, Marker::Triangle.inset(s.marker_scale(false)) * s.width),
    );
    assert!(
        dist(bico, tip) < 4.0,
        "o bico em {bico:?}, a mão em {tip:?}"
    );
    assert!(dist(node, tip) > 20.0, "o caminho recuou: {node:?}");
}

/// **Quando a forma deixa de ser seta, as pontas voltam às do estilo** — a função que o `rebuild`
/// chama a cada quadro: com farpas, as da seta; sem, as do estilo, tamanho incluído.
#[test]
fn the_ends_go_back_to_the_style_when_the_shape_stops_being_an_arrow() {
    use crate::pencil::arrow_ends;
    let style = crate::PenStyle {
        marker_start: Marker::Bar,
        marker_end: Marker::Triangle,
        marker_scale_start: 1.7,
        marker_scale_end: 2.3,
        ..crate::PenStyle::default()
    };
    let arrow = arrow_ends(&style, [None, Some(30.0)], 3.0);
    assert_eq!(
        arrow.markers,
        [Marker::None, Marker::Triangle],
        "só onde a mão desenhou"
    );
    assert!(arrow.inset[1] > 0.0 && arrow.inset[0] == 0.0);
    // F16c: o tamanho é POR PONTA — o início, sem farpa, fica com o do estilo.
    assert_eq!(arrow.scale[0], 1.7, "o inicio sem farpa guarda o seu tamanho");
    let back = arrow_ends(&style, [None; 2], 3.0);
    assert_eq!(back.markers, [Marker::Bar, Marker::Triangle]);
    assert_eq!((back.scale, back.inset), ([1.7, 2.3], [0.0; 2]));
    let plain = arrow_ends(&crate::PenStyle::default(), [Some(30.0), Some(30.0)], 3.0);
    assert_eq!(
        plain.markers,
        [Marker::Open; 2],
        "sem ponta armada: a Open nas duas"
    );
    // ⭐ F16c — **cada ponta alcança a SUA farpa**: farpas de 30 e 60 dão tamanhos 1:2, e não a
    // média das duas (a lei de antes, com UM tamanho para as duas pontas).
    let desiguais = arrow_ends(&crate::PenStyle::default(), [Some(30.0), Some(60.0)], 3.0);
    assert!(
        (desiguais.scale[1] / desiguais.scale[0] - 2.0).abs() < 1e-9,
        "cada ponta tem o tamanho da sua farpa: {:?}",
        desiguais.scale
    );
    assert!(
        (desiguais.scale[0] - plain.scale[0]).abs() < 1e-9,
        "a ponta de partida nao mudou com a farpa do fim: {:?} vs {:?}",
        desiguais.scale,
        plain.scale
    );
}
