//! ⏱ **Sonda do TREMER do cotovelo** (6.º smoke do dono, 06/10: «as setas rectangulares são bem
//! nervosas e tremem ao serem ajustadas»): arrasta uma caixa de 1 em 1 unidade e conta os passos em
//! que a rota SALTA (muda de número de pontos, ou um vértice anda muito mais do que o dedo).

use super::*;
use ph2d_board_model::{
    BoardDoc, BoardOp, Connector, Element, ElementId, End, Rgba, Route, Shape, ShapeType, Style,
};

fn ink() -> Rgba {
    Rgba([0, 0, 0, 255])
}

fn rect(doc: &mut BoardDoc, bx: [f64; 4]) -> ElementId {
    let s = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, Some(ink()), ink()),
        text: Default::default(),
    };
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), s, bx);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

/// Quantos saltos ao longo do caminho de `b` (uma lista de cantos superiores esquerdos).
pub(crate) fn jumps(
    path: &[[f64; 2]],
    a_end: End,
    b_end: fn(ElementId) -> End,
    wp: &[[f64; 2]],
) -> (usize, usize, f64) {
    let mut doc = BoardDoc::default();
    let a = rect(&mut doc, [0.0, 0.0, 160.0, 100.0]);
    let b = rect(&mut doc, [path[0][0], path[0][1], 160.0, 100.0]);
    let a_end = match a_end {
        End::Bound { anchor, .. } => End::Bound { target: a, anchor },
        e => e,
    };
    let mut c = Connector::new(
        a_end,
        b_end(b),
        Route::Elbow,
        Style::new(None, Some(ink()), ink()),
    );
    c.heads = [ph2d_board_model::Head::None; 2];
    c.waypoints = wp.to_vec();
    let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
    let id = el.id;
    BoardOp::Put(el).apply(&mut doc);
    let mut cache = RouteCache::default();
    let (mut shape_jumps, mut far_jumps, mut worst) = (0, 0, 0.0f64);
    let mut prev: Option<Vec<[f64; 2]>> = None;
    for p in path {
        let mut el = doc.get(b).unwrap().clone();
        el.x = p[0];
        el.y = p[1];
        BoardOp::Put(el).apply(&mut doc);
        cache.sync(&doc);
        let now = cache.get(id).unwrap().polyline();
        if let Some(pr) = &prev {
            if pr.len() != now.len() {
                shape_jumps += 1;
                if std::env::var("PROBE_SHAPES").is_ok() {
                    println!(
                        "  forma b={p:?}: {} → {} pontos  {pr:?} → {now:?}",
                        pr.len(),
                        now.len()
                    );
                }
            } else {
                let m = pr
                    .iter()
                    .zip(&now)
                    .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                    .fold(0.0, f64::max);
                worst = worst.max(m);
                if m > 5.0 {
                    far_jumps += 1;
                    if far_jumps <= 3 && std::env::var("PROBE_SHOW").is_ok() {
                        println!("  b={p:?}\n   antes {pr:?}\n   agora {now:?}");
                    }
                }
            }
        }
        prev = Some(now);
    }
    (shape_jumps, far_jumps, worst)
}

fn center(t: ElementId) -> End {
    End::Bound {
        target: t,
        anchor: Anchor::Center,
    }
}

#[test]
#[ignore = "sonda: corre-se à mão"]
fn probe_elbow_jitter() {
    let circle: Vec<[f64; 2]> = (0..3600)
        .map(|k| {
            let a = f64::from(k) / 3600.0 * std::f64::consts::TAU;
            [80.0 + 300.0 * a.cos() - 80.0, 50.0 + 260.0 * a.sin() - 50.0]
        })
        .collect();
    let line_x: Vec<[f64; 2]> = (0..800).map(|k| [-400.0 + f64::from(k), 180.0]).collect();
    let near: Vec<[f64; 2]> = (0..600).map(|k| [-300.0 + f64::from(k), 120.0]).collect();
    let free = End::Free([0.0, 0.0]);
    let _ = free;
    for (name, path) in [
        ("círculo r≈300", &circle),
        ("linha y=180", &line_x),
        ("linha y=120 (perto)", &near),
    ] {
        let (s, f, w) = jumps(path, center(ElementId(0)), center, &[]);
        println!("centro→centro · {name}: mudanças de forma {s}, saltos >5 {f}, pior {w:.1}");
    }
    let wp = [[400.0, -150.0]];
    let (s, f, w) = jumps(&line_x, center(ElementId(0)), center, &wp);
    println!("com 1 ponto · linha y=180: mudanças de forma {s}, saltos >5 {f}, pior {w:.1}");
}

/// ⭐ **O cotovelo não treme** (6.º smoke do dono, 06/10). Medido antes das curas: a volta inteira dava
/// 846 saltos > 5 un. em 3 600 passos de 1 un. (a dobra pulava entre o meio do vão e o recuo por um
/// empate de custo decidido pelo arredondamento); o vão curto 8 mudanças de forma (os dois recuos de
/// 40 cruzavam-se); com um ponto de ajuste 24 saltos (o trecho contornava a forma que não tocava).
/// Depois: só as trocas de lado nas diagonais (inevitáveis, o Miro também as faz).
#[test]
fn the_elbow_does_not_jitter_while_a_box_is_dragged() {
    let circle: Vec<[f64; 2]> = (0..1800)
        .map(|k| {
            let a = f64::from(k) / 1800.0 * std::f64::consts::TAU;
            [300.0 * a.cos(), 260.0 * a.sin()]
        })
        .collect();
    let (_, far, _) = jumps(&circle, center(ElementId(0)), center, &[]);
    assert!(
        far <= 4,
        "a volta inteira saltou {far} vezes (só as 4 diagonais podem)"
    );
    let near: Vec<[f64; 2]> = (0..600).map(|k| [-300.0 + f64::from(k), 120.0]).collect();
    let (shape, _, _) = jumps(&near, center(ElementId(0)), center, &[]);
    assert!(shape <= 2, "o vão curto mudou de forma {shape} vezes");
    let line: Vec<[f64; 2]> = (0..800).map(|k| [-400.0 + f64::from(k), 180.0]).collect();
    let (_, far, worst) = jumps(&line, center(ElementId(0)), center, &[[400.0, -150.0]]);
    assert_eq!(
        far, 0,
        "com um ponto de ajuste a rota saltou (pior {worst})"
    );
}
