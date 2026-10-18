//! W23 B3 (d) — no rascunho o RAMO do mapa sai à mão, como a pílula do nó (antes saía liso).

use ph2d_board_model::{
    Board, BoardOp, BoardSet, Element, FracKey, Mind, Rgba, Shape, ShapeType, Side, Style,
};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Brush, DrawOp, VectorScene};

use crate::{RenderCache, doc_color};

const BRANCH: Rgba = Rgba([200, 40, 120, 255]);

fn map(sketch: bool) -> Board {
    let mut set = BoardSet::default();
    let id = set.create("m".into());
    let b = set.get_mut(id).unwrap();
    let ink = Rgba([20, 20, 20, 255]);
    let node = |b: &mut Board, kind, bx, parent| {
        // Como os nós do mapa: só texto, sem contorno (o ramo continua a ter traço).
        let mut st = Style::new(None, None, ink);
        st.sketch = sketch;
        let mut el = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Shape {
                kind,
                style: st,
                text: Default::default(),
            },
            bx,
        );
        el.set_mind(Some(Mind::new(
            parent,
            FracKey::between(None, None),
            Side::After,
            BRANCH,
        )));
        let id = el.id;
        BoardOp::Put(el).apply(&mut b.doc);
        id
    };
    let root = node(b, ShapeType::Pill, [40.0, 200.0, 166.0, 66.0], None);
    node(
        b,
        ShapeType::Rectangle,
        [320.0, 120.0, 100.0, 30.0],
        Some(root),
    );
    set.get(id).unwrap().clone()
}

/// Os trechos do traço do ramo (o pintado na cor dele).
fn branch_segments(b: &Board) -> usize {
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut scene = VectorScene::new();
    scene.record();
    let mut ts = TextSystem::without_system_fonts();
    let mut cache = RenderCache::default();
    crate::paint(
        b,
        [0.0, 0.0, 640.0, 480.0],
        &mut scene,
        Theme::Forge,
        &mut ts,
        &mut cache,
        &routes,
        None,
    );
    let want = Brush::Solid(doc_color(BRANCH));
    scene
        .take_record()
        .iter()
        .filter_map(|o| match o {
            DrawOp::Stroke { brush, path, .. } if *brush == want => Some(path.segments().count()),
            _ => None,
        })
        .sum()
}

#[test]
fn a_sketched_map_draws_its_branches_by_hand() {
    assert_eq!(
        branch_segments(&map(false)),
        1,
        "liso: a cúbica medida do Miro"
    );
    let hand = branch_segments(&map(true));
    assert!(hand > 2, "no rascunho o ramo sai à mão: {hand} trechos");
}
