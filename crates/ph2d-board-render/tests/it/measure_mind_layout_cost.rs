//! ⏱ **A RÉGUA do layout do mapa mental** (W6, `docs/MiroClone/04_planear_mapa_e_modelos.md` §4):
//! um mapa de N nós (a raiz, até seis filhos por nó, larguras variadas, metade de cada lado) e, por N
//! e por variante:
//!
//! - **arrumar**: `mind_settle` sobre o mapa já arrumado — a conta inteira do layout, o que cada lote
//!   que toca o mapa paga (uma letra a escrever num nó, por exemplo);
//! - **Tab**: a tecla inteira pelo editor (`Editor::mind_child` num nó do meio: o nó novo, o lote, o
//!   layout e os vizinhos que se mexem);
//! - **taffy**: o MESMO layout pelo motor de caixas da casa (`ph2d-vec-layout`: uma linha `[nó,
//!   coluna dos filhos]` centrada por subárvore) — a pergunta do briefing «a composição já o
//!   exprime?». O teste prova também que as posições são as mesmas (N = 100).
//!
//! Variantes no MESMO processo, rodadas intercaladas com a ordem rodada, vale o MÍNIMO (mediana ao
//! lado). Corre à mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it measure_mind -- --ignored --nocapture
//! ```

use std::collections::BTreeSet;
use std::time::Instant;

use ph2d_board_edit::{Editor, Metrics};
use ph2d_board_model::{
    BoardDoc, BoardOp, Element, ElementId, FracKey, History, MIND_FONT, MIND_LEVEL_GAP,
    MIND_SIBLING_GAP, Mind, Rgba, Shape, ShapeType, Side, Style,
};
use ph2d_text::TextSystem;
use ph2d_vec_layout::{Align, Dir, FrameStyle, ItemStyle, Len, Node, solve};

const NS: [usize; 3] = [100, 1_000, 10_000];
const ROUNDS: usize = 7;
/// A altura de um nó de uma linha (o nosso moldado a 14 + as folgas).
const LINE_H: f64 = 28.939_453;

fn metrics() -> Metrics {
    Metrics {
        handle: 8.0,
        rotate_offset: 24.0,
        snap: 8.0,
        drag: 3.0,
        click_size: [160.0, 100.0],
        paste_offset: 10.0,
        bind: 8.0,
        dot: 40.0,
        eraser: 10.0,
        frame_title: 20.0,
    }
}

fn style() -> Style {
    let ink = Rgba([20, 20, 20, 255]);
    let mut s = Style::new(None, None, ink);
    s.font_size = MIND_FONT;
    s
}

/// O mapa de `n` nós, já arrumado; devolve a raiz e um nó do meio (o do `Tab`).
fn map(n: usize) -> (BoardDoc, ElementId, ElementId) {
    let mut doc = BoardDoc::default();
    let put = |doc: &mut BoardDoc, parent: Option<ElementId>, i: usize, side| {
        let id = doc.mint_id();
        let shape = Shape {
            kind: if parent.is_none() {
                ShapeType::Pill
            } else {
                ShapeType::Rectangle
            },
            style: style(),
            text: Default::default(),
        };
        let w = 60.0 + (i % 5) as f64 * 30.0;
        let mut el = Element::new_shape(id, doc.z_on_top(), shape, [0.0, 0.0, w, LINE_H]);
        let order = FracKey::between(None, None);
        el.set_mind(Some(Mind::new(parent, order, side, Rgba([1, 2, 3, 255]))));
        BoardOp::Put(el).apply(doc);
        id
    };
    let root = put(&mut doc, None, 0, Side::After);
    let mut queue = std::collections::VecDeque::from([(root, Side::After)]);
    let mut count = 1;
    let mut last_order: std::collections::BTreeMap<ElementId, FracKey> = Default::default();
    while let Some((p, side)) = queue.pop_front() {
        for k in 0..6 {
            if count >= n {
                break;
            }
            let s = if p == root {
                if k % 2 == 0 {
                    Side::After
                } else {
                    Side::Before
                }
            } else {
                side
            };
            let id = put(&mut doc, Some(p), count, s);
            // A ordem dos irmãos pela ordem de nascença.
            let o = FracKey::between(last_order.get(&p), None);
            last_order.insert(p, o.clone());
            let mut el = doc.get(id).unwrap().clone();
            el.mind_mut().unwrap().order = o;
            BoardOp::Put(el).apply(&mut doc);
            queue.push_back((id, s));
            count += 1;
        }
    }
    let _ = ph2d_board_model::mind_settle(&mut doc, &BTreeSet::from([root]));
    let mid = doc.mind_children(doc.mind_children(root)[0])[0];
    (doc, root, mid)
}

/// O mapa como árvore de caixas do `ph2d-vec-layout`; devolve os nós e, por nó do mapa, o índice
/// da folha dele.
fn taffy_tree(doc: &BoardDoc, root: ElementId) -> (Vec<Node>, Vec<(ElementId, usize)>) {
    let mut nodes = Vec::new();
    let mut leaf_of = Vec::new();
    let flow = |dir, gap: f64, align| Node {
        parent: None,
        frame: Some(FrameStyle {
            dir,
            gap: [gap, 0.0],
            pad: [0.0; 4],
            align,
            justify: Default::default(),
        }),
        item: ItemStyle::default(),
        size: [Len::Hug, Len::Hug],
        min: [None; 2],
        max: [None; 2],
    };
    let leaf = |el: &Element| Node {
        parent: None,
        frame: None,
        item: ItemStyle::default(),
        size: [Len::Fixed(el.w), Len::Fixed(el.h)],
        min: [None; 2],
        max: [None; 2],
    };
    fn push(nodes: &mut Vec<Node>, mut n: Node, parent: Option<usize>) -> usize {
        n.parent = parent;
        nodes.push(n);
        nodes.len() - 1
    }
    // Uma subárvore do lado `side` debaixo de `parent` (índice da caixa onde ela entra).
    #[allow(clippy::too_many_arguments)]
    fn subtree(
        doc: &BoardDoc,
        id: ElementId,
        side: Side,
        parent: usize,
        nodes: &mut Vec<Node>,
        leaf_of: &mut Vec<(ElementId, usize)>,
        flow: &dyn Fn(Dir, f64, Align) -> Node,
        leaf: &dyn Fn(&Element) -> Node,
    ) {
        let el = doc.get(id).unwrap();
        let kids = doc.mind_children(id);
        if kids.is_empty() {
            let i = push(nodes, leaf(el), Some(parent));
            leaf_of.push((id, i));
            return;
        }
        let row = push(
            nodes,
            flow(Dir::Row, MIND_LEVEL_GAP, Align::Center),
            Some(parent),
        );
        let col_align = if side == Side::After {
            Align::Start
        } else {
            Align::End
        };
        let col = |nodes: &mut Vec<Node>, leaf_of: &mut Vec<(ElementId, usize)>| {
            let c = push(
                nodes,
                flow(Dir::Column, MIND_SIBLING_GAP, col_align),
                Some(row),
            );
            for k in &kids {
                subtree(doc, *k, side, c, nodes, leaf_of, flow, leaf);
            }
        };
        if side == Side::Before {
            col(nodes, leaf_of);
        }
        let i = push(nodes, leaf(el), Some(row));
        leaf_of.push((id, i));
        if side == Side::After {
            col(nodes, leaf_of);
        }
    }
    let top = push(
        &mut nodes,
        flow(Dir::Row, MIND_LEVEL_GAP, Align::Center),
        None,
    );
    for side in [Side::Before, Side::After] {
        let kids: Vec<ElementId> = doc
            .mind_children(root)
            .into_iter()
            .filter(|k| doc.get(*k).unwrap().mind().unwrap().side == side)
            .collect();
        if side == Side::After {
            let i = push(&mut nodes, leaf(doc.get(root).unwrap()), Some(top));
            leaf_of.push((root, i));
        }
        if kids.is_empty() {
            continue;
        }
        let align = if side == Side::After {
            Align::Start
        } else {
            Align::End
        };
        let c = push(
            &mut nodes,
            flow(Dir::Column, MIND_SIBLING_GAP, align),
            Some(top),
        );
        for k in kids {
            subtree(doc, k, side, c, &mut nodes, &mut leaf_of, &flow, &leaf);
        }
    }
    (nodes, leaf_of)
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn summary(v: &mut [f64]) -> String {
    v.sort_by(f64::total_cmp);
    format!("{:>8.3} (med {:>8.3})", v[0], v[v.len() / 2])
}

/// ⭐ O `taffy` exprime a lei: as posições dele, relativas à raiz, são as do nosso layout.
#[test]
fn taffy_expresses_the_mind_layout() {
    let (doc, root, _) = map(100);
    let (nodes, leaf_of) = taffy_tree(&doc, root);
    let solved = solve(&nodes).expect("a árvore resolve");
    let r = doc.get(root).unwrap();
    let ri = leaf_of.iter().find(|(id, _)| *id == root).unwrap().1;
    let [rx, ry, ..] = solved[ri];
    for (id, i) in leaf_of {
        let el = doc.get(id).unwrap();
        let [x, y, ..] = solved[i];
        let (dx, dy) = (x - rx - (el.x - r.x), y - ry - (el.y - r.y));
        // O `taffy` conta em `f32`: 2e-6 a 300 da raiz é o arredondamento dele, não outra lei.
        assert!(dx.abs() < 1e-3 && dy.abs() < 1e-3, "{id:?}: {dx} {dy}");
    }
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_mind_layout_cost() {
    let mut ts = TextSystem::without_system_fonts();
    let maps: Vec<_> = NS.iter().map(|&n| map(n)).collect();
    let mut per: Vec<[Vec<f64>; 3]> = vec![Default::default(); NS.len()];
    for round in 0..ROUNDS {
        for k in 0..NS.len() * 3 {
            let j = (k + round) % (NS.len() * 3);
            let (i, var) = (j / 3, j % 3);
            let (doc0, root, mid) = &maps[i];
            let mut doc = doc0.clone();
            let t = match var {
                0 => {
                    let t = Instant::now();
                    let inv = ph2d_board_model::mind_settle(&mut doc, &BTreeSet::from([*root]));
                    assert!(inv.is_empty(), "o mapa já estava arrumado");
                    ms(t)
                }
                1 => {
                    let mut h = History::default();
                    let mut ed = Editor::new(style(), metrics());
                    ed.select(&doc, [*mid]);
                    let t = Instant::now();
                    assert!(ed.mind_child(&mut doc, &mut h, &mut ts, None));
                    ms(t)
                }
                _ => {
                    let t = Instant::now();
                    let (nodes, _) = taffy_tree(&doc, *root);
                    let _ = std::hint::black_box(solve(&nodes).expect("resolve"));
                    ms(t)
                }
            };
            per[i][var].push(t);
        }
    }
    eprintln!("régua do layout do mapa mental — mínimo de {ROUNDS} rodadas (ms)");
    for (i, n) in NS.iter().enumerate() {
        let [a, b, c] = &mut per[i];
        eprintln!(
            "N {n:>6}: arrumar {} · Tab {} · taffy {}",
            summary(a),
            summary(b),
            summary(c)
        );
    }
}
