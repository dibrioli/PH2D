//! Gate do oráculo: os cantos e a elipse do Quadro contra o Excalidraw.
//!
//! Oráculo: `@excalidraw/excalidraw` 0.18.1 (MIT), Chromium 153.0.8010.12, corrido por
//! `docs/MiroClone/ferramentas/excalidraw_oracle/corre.sh` (2026-10-06), `roughness 0`,
//! `strokeWidth 2`, sem preenchimento. Os números abaixo são os `d=` dos SVG em `saidas/`, em
//! coordenadas LOCAIS da caixa (o `translate` de cada `<g>` — a caixa + `exportPadding 10` — já
//! tirado); re-medir: `python3 docs/MiroClone/ferramentas/excalidraw_oracle/mede_cantos.py`.
//!
//! - `formas_canto_retangulo.{excalidraw,svg}` — rectângulo `roundness {type:3}` (o que o editor
//!   escreve ao desenhar um; sonda `formas_arredondamento_padrao` + `sonda_editor_arredondamento.mjs`).
//! - `formas_canto_losango.{excalidraw,svg}` — losango `roundness {type:2}` (o do editor).
//! - `formas_contorno_elipse.{excalidraw,svg}` — elipse 300×160 e círculo 120×120.
//!
//! ⛔ Recusa medida: o losango do oráculo tem os vértices em `(⌊w/2⌋+1, 0)`, `(w, ⌊h/2⌋+1)`… — um
//! desvio de +1 un. que o torna ASSIMÉTRICO (o lado esquerdo mede 41, o direito 39 numa caixa de
//! 80). Não o copiamos: o nosso losango é simétrico. O gate compara o canto depois de alinhar os
//! vértices e aceita o que esse +1 propaga (¼ × 1 un. por eixo).

use super::*;
use ph2d_board_model::{Rgba, Style};
use ph2d_vector::ParamCurve;

/// O SVG do oráculo traz 2 casas decimais: cada coordenada vem arredondada a ≤ 0,005.
const SVG_ROUNDING: f64 = 0.01;
/// O +1 do vértice do oráculo entra no corte multiplicado por ¼ ([`POLYGON_CORNER_CUT`]).
const DIAMOND_QUIRK: f64 = 0.25;

type Cubic = [(f64, f64); 4];

/// Rectângulo `{type:3}`: `(w, h, canto superior direito [p0, c1, c2, p3])`.
const RECT_TOP_RIGHT: &[(f64, f64, Cubic)] = &[
    (
        30.0,
        20.0,
        [(25.0, 0.0), (28.33, 0.0), (30.0, 1.67), (30.0, 5.0)],
    ),
    (
        60.0,
        40.0,
        [(50.0, 0.0), (56.67, 0.0), (60.0, 3.33), (60.0, 10.0)],
    ),
    (
        120.0,
        80.0,
        [(100.0, 0.0), (113.33, 0.0), (120.0, 6.67), (120.0, 20.0)],
    ),
    (
        150.0,
        100.0,
        [(125.0, 0.0), (141.67, 0.0), (150.0, 8.33), (150.0, 25.0)],
    ),
    (
        180.0,
        120.0,
        [(150.0, 0.0), (170.0, 0.0), (180.0, 10.0), (180.0, 30.0)],
    ),
    (
        192.0,
        128.0,
        [(160.0, 0.0), (181.33, 0.0), (192.0, 10.67), (192.0, 32.0)],
    ),
    (
        204.0,
        136.0,
        [(172.0, 0.0), (193.33, 0.0), (204.0, 10.67), (204.0, 32.0)],
    ),
    (
        240.0,
        160.0,
        [(208.0, 0.0), (229.33, 0.0), (240.0, 10.67), (240.0, 32.0)],
    ),
    (
        300.0,
        200.0,
        [(268.0, 0.0), (289.33, 0.0), (300.0, 10.67), (300.0, 32.0)],
    ),
    (
        600.0,
        400.0,
        [(568.0, 0.0), (589.33, 0.0), (600.0, 10.67), (600.0, 32.0)],
    ),
    (
        40.0,
        80.0,
        [(30.0, 0.0), (36.67, 0.0), (40.0, 3.33), (40.0, 10.0)],
    ),
    (
        128.0,
        256.0,
        [(96.0, 0.0), (117.33, 0.0), (128.0, 10.67), (128.0, 32.0)],
    ),
    (
        200.0,
        400.0,
        [(168.0, 0.0), (189.33, 0.0), (200.0, 10.67), (200.0, 32.0)],
    ),
];

/// Losango `{type:2}`: `(w, h, canto do topo, canto da direita)` — cúbicas com c1 = c2 = vértice.
const DIAMOND_CORNERS: &[(f64, f64, Cubic, Cubic)] = &[
    (
        80.0,
        60.0,
        [(30.75, 7.75), (41.0, 0.0), (41.0, 0.0), (51.25, 7.75)],
        [(69.75, 23.25), (80.0, 31.0), (80.0, 31.0), (69.75, 38.75)],
    ),
    (
        160.0,
        120.0,
        [(60.75, 15.25), (81.0, 0.0), (81.0, 0.0), (101.25, 15.25)],
        [
            (139.75, 45.75),
            (160.0, 61.0),
            (160.0, 61.0),
            (139.75, 76.25),
        ],
    ),
    (
        400.0,
        300.0,
        [(150.75, 37.75), (201.0, 0.0), (201.0, 0.0), (251.25, 37.75)],
        [
            (349.75, 113.25),
            (400.0, 151.0),
            (400.0, 151.0),
            (349.75, 188.75),
        ],
    ),
    (
        200.0,
        200.0,
        [(75.75, 25.25), (101.0, 0.0), (101.0, 0.0), (126.25, 25.25)],
        [
            (174.75, 75.75),
            (200.0, 101.0),
            (200.0, 101.0),
            (174.75, 126.25),
        ],
    ),
    (
        400.0,
        80.0,
        [(150.75, 10.25), (201.0, 0.0), (201.0, 0.0), (251.25, 10.25)],
        [
            (349.75, 30.75),
            (400.0, 41.0),
            (400.0, 41.0),
            (349.75, 51.25),
        ],
    ),
    (
        81.0,
        61.0,
        [(30.75, 7.75), (41.0, 0.0), (41.0, 0.0), (51.25, 7.75)],
        [(70.75, 23.25), (81.0, 31.0), (81.0, 31.0), (70.75, 38.75)],
    ),
];

fn shape(kind: ShapeType, round: bool) -> Shape {
    let ink = Rgba([0, 0, 0, 255]);
    let mut style = Style::new(Some(ink), None, ink);
    style.round = round;
    Shape {
        kind,
        style,
        text: String::new(),
    }
}

/// Cada segmento curvo do contorno, como cúbica (as quadráticas elevadas: é assim que o SVG as traz).
/// `[p0, c1, c2, p3]`.
fn curves(p: &BezPath) -> Vec<[Point; 4]> {
    p.segments()
        .filter_map(|s| match s {
            PathSeg::Line(_) => None,
            PathSeg::Quad(q) => {
                let c = q.raise();
                Some([c.p0, c.p1, c.p2, c.p3])
            }
            PathSeg::Cubic(c) => Some([c.p0, c.p1, c.p2, c.p3]),
        })
        .collect()
}

fn assert_cubic_close(ours: &[Point; 4], oracle: &Cubic, shift: (f64, f64), tol: f64, what: &str) {
    for (k, (p, o)) in ours.iter().zip(oracle).enumerate() {
        let (dx, dy) = ((p.x + shift.0 - o.0).abs(), (p.y + shift.1 - o.1).abs());
        assert!(
            dx <= tol && dy <= tol,
            "{what}: ponto {k} nosso {p:?} (+{shift:?}) vs oráculo {o:?} (Δ {dx:.4}, {dy:.4} > {tol})"
        );
    }
}

#[test]
fn the_round_rectangle_corner_is_the_oracles_radius_and_quadratic() {
    for &(w, h, ref oracle) in RECT_TOP_RIGHT {
        let r = corner_radius(w.min(h));
        let measured = oracle[3].1 - oracle[0].1;
        assert!(
            (r - measured).abs() <= SVG_ROUNDING,
            "{w}×{h}: raio {r} vs oráculo {measured}"
        );
        let o = outline(&shape(ShapeType::Rectangle, true), w, h);
        let cs = curves(&o.fill);
        assert_eq!(cs.len(), 4, "{w}×{h}: quatro cantos");
        let tr = cs
            .iter()
            .find(|c| c[0].y.abs() < 1e-9 && (c[3].x - w).abs() < 1e-9)
            .unwrap_or_else(|| panic!("{w}×{h}: sem canto superior direito"));
        assert_cubic_close(
            tr,
            oracle,
            (0.0, 0.0),
            SVG_ROUNDING,
            &format!("rect {w}×{h}"),
        );
    }
}

#[test]
fn the_round_diamond_corner_is_the_oracles_cut_and_cubic() {
    for &(w, h, ref top, ref right) in DIAMOND_CORNERS {
        // O desvio recusado, conferido: o vértice do oráculo é o nosso + (⌊·/2⌋ + 1 − ·/2).
        assert_eq!(top[1].0, (w / 2.0).floor() + 1.0, "{w}×{h}: x do topo");
        assert_eq!(right[1].1, (h / 2.0).floor() + 1.0, "{w}×{h}: y da direita");
        let o = outline(&shape(ShapeType::Diamond, true), w, h);
        let cs = curves(&o.fill);
        assert_eq!(cs.len(), 4, "{w}×{h}: quatro cantos");
        for c in &cs {
            assert!(
                c[1] == c[2],
                "{w}×{h}: canto não é cúbica com c1 = c2 = vértice"
            );
        }
        let find = |v: Point| {
            cs.iter()
                .find(|c| (c[1] - v).hypot() < 1e-9)
                .unwrap_or_else(|| panic!("{w}×{h}: sem canto em {v:?}"))
        };
        let (our_top, our_right) = (Point::new(w / 2.0, 0.0), Point::new(w, h / 2.0));
        let tol = DIAMOND_QUIRK + SVG_ROUNDING;
        let top_shift = (top[1].0 - our_top.x, 0.0);
        assert_cubic_close(
            find(our_top),
            top,
            top_shift,
            tol,
            &format!("losango {w}×{h} topo"),
        );
        let right_shift = (0.0, right[1].1 - our_right.y);
        assert_cubic_close(
            find(our_right),
            right,
            right_shift,
            tol,
            &format!("losango {w}×{h} direita"),
        );
    }
}

/// O triângulo não existe no oráculo: herda a lei do losango (¼ de cada aresta, cúbica no vértice).
#[test]
fn the_round_triangle_follows_the_diamond_law() {
    let (w, h) = (200.0, 120.0);
    let cs = curves(&outline(&shape(ShapeType::Triangle, true), w, h).fill);
    assert_eq!(cs.len(), 3);
    for c in &cs {
        let v = c[1];
        assert!(c[1] == c[2]);
        // as marcas estão a ¼ das arestas: o vértice oposto de cada uma fica a 4× o corte.
        for end in [c[0], c[3]] {
            let far = v + (end - v) * (1.0 / POLYGON_CORNER_CUT);
            let on_vertex = [[w / 2.0, 0.0], [w, h], [0.0, h]]
                .iter()
                .any(|q| (far - Point::new(q[0], q[1])).hypot() < 1e-9);
            assert!(
                on_vertex,
                "a marca {end:?} do canto {v:?} não está a ¼ da aresta"
            );
        }
    }
}

/// O oráculo desenha uma elipse VERDADEIRA a `roughness 0` (73 cúbicas, desvio máx 0,075 un. da
/// analítica em 300×160; 0,004 no círculo 120×120). A nossa tem de o ser também, com a mesma folga.
#[test]
fn the_ellipse_is_a_true_ellipse_like_the_oracles() {
    const ORACLE_MAX_DEVIATION: f64 = 0.075;
    for (w, h) in [(300.0, 160.0), (120.0, 120.0)] {
        let o = outline(&shape(ShapeType::Ellipse, false), w, h);
        let (a, b) = (w / 2.0, h / 2.0);
        let mut worst = 0.0_f64;
        for seg in o.fill.segments() {
            for k in 0..=32 {
                let p = seg.eval(f64::from(k) / 32.0);
                let f = ((p.x - a) / a).powi(2) + ((p.y - b) / b).powi(2) - 1.0;
                let g = (2.0 * (p.x - a) / (a * a)).hypot(2.0 * (p.y - b) / (b * b));
                worst = worst.max(f.abs() / g);
            }
        }
        assert!(
            worst <= ORACLE_MAX_DEVIATION + SVG_ROUNDING,
            "{w}×{h}: desvio {worst} da elipse analítica"
        );
    }
}
