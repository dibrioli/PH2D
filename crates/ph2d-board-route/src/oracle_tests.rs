//! Gate do oráculo: a LIGAÇÃO das setas e a FORMA do cotovelo contra o Excalidraw.
//!
//! Oráculo: `@excalidraw/excalidraw` 0.18.1 (MIT), Chromium 153.0.8010.12, corrido por
//! `docs/MiroClone/ferramentas/excalidraw_oracle/corre.sh` (2026-10-05 e 2026-10-06). Os números são
//! os de `saidas/<n>.restored.json` (a normalização da ligação, `fixedPoint`) e `saidas/<n>.editor.json`
//! (a rota que o editor montado traça), transcritos aqui:
//!
//! | entrada | origem | destino | ligação (`fixedPoint`) | rota do editor (absoluta) |
//! |---|---|---|---|---|
//! | `seta_cotovelo_desalinhada` | 40,60 140×90 | 500,280 140×90 | `[1, 0.5001]` → `[0, 0.5001]` | (180,105) (340,105) (340,325) (500,325) |
//! | `seta_cotovelo_volta` | 300,60 140×90 | 40,300 140×90 | `[1, 0.5001]` → `[0, 0.5001]` | (440,105) (480,105) (480,225) (0,225) (0,345) (40,345) |
//! | `seta_cotovelo_volta_grande` | 600,60 280×180 | 40,520 280×180 | idem | (880,150) (920,150) (920,380) (0,380) (0,610) (40,610) |
//!
//! ⛔ Recusa medida: o `0.5001` (e o `105.009` da rota) é o desempate do oráculo, não uma lei — o
//! nosso meio de lado é `0.5` exacto. E a caixa do meio de `seta_cotovelo_obstaculo` (o oráculo
//! atravessa-a) é hoje a NOSSA lei também: a seta não reage a outras formas (ordem do dono, 06/10).

use super::*;
use ph2d_board_model::{
    Anchor, BoardDoc, BoardOp, Connector, Element, ElementId, End, Rgba, Route, Shape, ShapeType,
    Style,
};

/// O SVG/JSON do oráculo traz o `0.5001` do desempate: ×90 de altura = 0,009 de desvio.
const ORACLE_TIE: f64 = 0.01;

fn rect(doc: &mut BoardDoc, [x, y, w, h]: [f64; 4]) -> ElementId {
    let ink = Rgba([0, 0, 0, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, Some(ink), ink),
        text: Default::default(),
    };
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [x, y, w, h]);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

fn arrow(doc: &mut BoardDoc, a: ElementId, b: ElementId, route: Route) -> ElementId {
    let ink = Rgba([0, 0, 0, 255]);
    let c = Connector::new(
        End::Bound {
            target: a,
            anchor: Anchor::Fixed([1.0, 0.5]),
        },
        End::Bound {
            target: b,
            anchor: Anchor::Fixed([0.0, 0.5]),
        },
        route,
        Style::new(None, Some(ink), ink),
    );
    let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

fn close(a: [f64; 2], b: [f64; 2], tol: f64) -> bool {
    (a[0] - b[0]).abs() <= tol && (a[1] - b[1]).abs() <= tol
}

/// ⭐ A normalização: a ponta largada no meio do lado direito da origem prende-se ao ponto fixo
/// `[1, 0.5]` (o `fixedPoint` do oráculo) e sai por esse lado; a do destino, `[0, 0.5]`, a oeste.
#[test]
fn a_dropped_end_normalizes_like_the_oracle_fixed_point() {
    let mut doc = BoardDoc::default();
    let o = rect(&mut doc, [40.0, 60.0, 140.0, 90.0]);
    let d = rect(&mut doc, [500.0, 280.0, 140.0, 90.0]);
    let band = 8.0;
    for (id, drop, oracle, side) in [
        (o, [180.0, 105.0], [1.0, 0.5001], Dir::East),
        (d, [500.0, 325.0], [0.0, 0.5001], Dir::West),
    ] {
        let el = doc.get(id).unwrap();
        let Some(Anchor::Fixed(uv)) = anchor_for(el, drop, band) else {
            panic!("a ponta no contorno prende-se a um PONTO FIXO");
        };
        assert!(
            close(uv, oracle, ORACLE_TIE),
            "{uv:?} contra o oráculo {oracle:?}"
        );
        assert_eq!(
            fixed_side(el, uv),
            side,
            "o lado sai da coordenada 0/1 do fixedPoint"
        );
    }
}

/// ⭐ A forma do cotovelo: sem nada no meio, o Z dobra a MEIO do vão (x = 340), como o oráculo.
#[test]
fn the_elbow_bends_in_the_middle_of_the_gap_like_the_oracle() {
    let mut doc = BoardDoc::default();
    let o = rect(&mut doc, [40.0, 60.0, 140.0, 90.0]);
    let d = rect(&mut doc, [500.0, 280.0, 140.0, 90.0]);
    let a = arrow(&mut doc, o, d, Route::Elbow);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let pts = &cache.get(a).unwrap().polyline();
    let oracle = [
        [180.0, 105.0],
        [340.0, 105.0],
        [340.0, 325.0],
        [500.0, 325.0],
    ];
    assert_eq!(pts.len(), oracle.len(), "{pts:?}");
    for (p, q) in pts.iter().zip(oracle) {
        assert!(
            close(*p, q, ORACLE_TIE),
            "{pts:?} contra o oráculo {oracle:?}"
        );
    }
}

/// ⭐ A volta: a rota sai 40 (o [`JETTY`] medido), dá a volta pelo meio do vão vertical e entra 40
/// antes do destino — 6 pontos, como o oráculo, nas duas escalas.
#[test]
fn the_u_turn_leaves_by_the_measured_jetty_like_the_oracle() {
    let cases = [
        (
            [300.0, 60.0, 140.0, 90.0],
            [40.0, 300.0, 140.0, 90.0],
            [
                [440.0, 105.0],
                [480.0, 105.0],
                [480.0, 225.0],
                [0.0, 225.0],
                [0.0, 345.0],
                [40.0, 345.0],
            ],
        ),
        (
            [600.0, 60.0, 280.0, 180.0],
            [40.0, 520.0, 280.0, 180.0],
            [
                [880.0, 150.0],
                [920.0, 150.0],
                [920.0, 380.0],
                [0.0, 380.0],
                [0.0, 610.0],
                [40.0, 610.0],
            ],
        ),
    ];
    for (ob, db, oracle) in cases {
        let mut doc = BoardDoc::default();
        let o = rect(&mut doc, ob);
        let d = rect(&mut doc, db);
        let a = arrow(&mut doc, o, d, Route::Elbow);
        let mut cache = RouteCache::default();
        cache.sync(&doc);
        let pts = &cache.get(a).unwrap().polyline();
        assert_eq!(pts.len(), oracle.len(), "{pts:?}");
        for (p, q) in pts.iter().zip(oracle) {
            assert!(
                close(*p, q, ORACLE_TIE),
                "{pts:?} contra o oráculo {oracle:?}"
            );
        }
    }
}

/// Uma caixa no MEIO não muda a seta (o oráculo e o Miro: as setas não se reajustam sozinhas —
/// ordem do dono, 06/10): a recta de `(180,205)` a `(500,205)`, como o `seta_cotovelo_obstaculo`.
#[test]
fn a_box_in_the_middle_does_not_bend_the_elbow_like_the_oracle() {
    let mut doc = BoardDoc::default();
    let o = rect(&mut doc, [40.0, 160.0, 140.0, 90.0]);
    rect(&mut doc, [280.0, 120.0, 120.0, 170.0]);
    let d = rect(&mut doc, [500.0, 160.0, 140.0, 90.0]);
    let a = arrow(&mut doc, o, d, Route::Elbow);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let pts = cache.get(a).unwrap().polyline();
    let oracle = [[180.0, 205.0], [500.0, 205.0]];
    assert_eq!(pts.len(), 2, "{pts:?}");
    for (p, q) in pts.iter().zip(oracle) {
        assert!(
            close(*p, q, ORACLE_TIE),
            "{pts:?} contra o oráculo {oracle:?}"
        );
    }
}
