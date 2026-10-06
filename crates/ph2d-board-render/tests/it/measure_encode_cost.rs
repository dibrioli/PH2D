//! ⏱ **A RÉGUA do desenho do quadro, lado CPU** (W0/W1, `docs/MiroClone/02_plano.md` §2) — quanto
//! custa ENCODAR um quadro de N formas numa `VectorScene` (o que `paint` faz a cada quadro): só
//! rectângulos (a cena da W0) e a mistura da W1 (contorno, cantos, rotação, texto).
//!
//! Régua do dono (05/10): as variantes correm no MESMO processo, intercaladas em rodadas curtas com
//! ordem rodada, e vale o MÍNIMO (mediana ao lado como controlo). ⚠️ Mede só a CPU: o tempo da placa
//! (raster do Vello) é do perfilador da GPU, na W1. Corre à mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it -- --ignored --nocapture
//! ```

use ph2d_board_model::{BoardOp, BoardSet, Camera, Element, Rgba, Shape, ShapeType, Style};
use ph2d_board_render::RenderCache;
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::time::Instant;

/// O que enche o quadro: só rectângulos cheios (a cena da W0, para comparar) ou a mistura da W1 —
/// rectângulo, elipse e losango com contorno, um em cada cinco rodado, e uma palavra em cada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mix {
    Rects,
    /// As formas da W1 com a vista a enquadrar o quadro inteiro (a letra é pequena demais para
    /// ler: desenha-se como traço).
    Shapes,
    /// As MESMAS formas com a vista aproximada até a letra se ler (14 px): só uma parte está à
    /// vista — o regime de quem trabalha num quadro grande.
    ShapesNear,
    /// O fluxograma da W2: N formas 160×100 com o vão de nascença e N/10 setas CURVAS (a de nascença) entre
    /// vizinhas, presas ao centro (`measure_route_cost`), com a vista a enquadrar tudo.
    Flow,
}

pub(super) const SCENES: [(usize, Mix); 12] = [
    (1_000, Mix::Rects),
    (10_000, Mix::Rects),
    (100_000, Mix::Rects),
    (1_000, Mix::Shapes),
    (10_000, Mix::Shapes),
    (100_000, Mix::Shapes),
    (1_000, Mix::ShapesNear),
    (10_000, Mix::ShapesNear),
    (100_000, Mix::ShapesNear),
    (1_000, Mix::Flow),
    (10_000, Mix::Flow),
    (100_000, Mix::Flow),
];
/// Passo da grelha do fluxograma: a caixa 160×100 e o vão de um fluxograma (o `NEXT_GAP` do
/// editor, 80).
const FLOW_STEP: [f64; 2] = [240.0, 180.0];
/// O tamanho da letra das formas da mistura (mundo) e o do ecrã a que a vista de perto a põe.
const FONT: f64 = 6.0;
const NEAR_FONT_PX: f64 = 14.0;
pub(super) const ROUNDS: usize = 7;
pub(super) const FRAMES: usize = 20;
pub(super) const AREA: [f64; 4] = [0.0, 0.0, 1920.0, 1080.0];
/// O laranja dos rectângulos (o controlo de cobertura da régua da placa conta-o).
pub(super) const ORANGE: Rgba = Rgba([200, 120, 40, 255]);

/// N formas numa grelha quadrada, com a vista a enquadrar a grelha inteira (todas visíveis).
pub(super) fn board_with((n, mix): (usize, Mix)) -> BoardSet {
    if mix == Mix::Flow {
        return flow(n);
    }
    let mut set = BoardSet::default();
    let id = set.create(format!("{n}"));
    let b = set.get_mut(id).unwrap();
    let side = (n as f64).sqrt().ceil();
    let ink = Rgba([30, 30, 30, 255]);
    for i in 0..n {
        let (row, col) = ((i as f64 / side).floor(), (i as f64 % side));
        let shape = match mix {
            Mix::Rects => Shape {
                kind: ShapeType::Rectangle,
                style: Style::new(Some(ORANGE), None, ink),
                text: String::new(),
            },
            Mix::Shapes | Mix::ShapesNear | Mix::Flow => {
                let kind = [ShapeType::Rectangle, ShapeType::Ellipse, ShapeType::Diamond][i % 3];
                let mut style = Style::new(Some(ORANGE), Some(ink), ink);
                style.round = i % 2 == 0;
                style.stroke_width = 1.0;
                style.font_size = FONT;
                Shape {
                    kind,
                    style,
                    text: format!("ideia {i}"),
                }
            }
        };
        let mut el = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            shape,
            [col * 30.0, row * 30.0, 24.0, 24.0],
        );
        if mix != Mix::Rects && i % 5 == 0 {
            el.angle = 0.3;
        }
        BoardOp::Put(el).apply(&mut b.doc);
    }
    b.camera = Camera {
        center_x: side * 15.0,
        center_y: side * 15.0,
        zoom: if mix == Mix::ShapesNear {
            NEAR_FONT_PX / FONT
        } else {
            1000.0 / (side * 30.0)
        },
    };
    set
}

/// O fluxograma: N formas e uma seta de cada 10.ª forma para a vizinha da direita (a última da
/// linha liga à de baixo).
pub(super) fn flow(n: usize) -> BoardSet {
    use ph2d_board_model::{Anchor, Connector, End, Route};
    let mut set = BoardSet::default();
    let id = set.create(format!("{n}"));
    let b = set.get_mut(id).unwrap();
    let side = (n as f64).sqrt().ceil() as usize;
    let ink = Rgba([30, 30, 30, 255]);
    let style = Style::new(None, Some(ink), ink);
    let mut ids = Vec::with_capacity(n);
    for i in 0..n {
        let (row, col) = ((i / side) as f64, (i % side) as f64);
        let shape = Shape {
            kind: [ShapeType::Rectangle, ShapeType::Ellipse, ShapeType::Diamond][i % 3],
            style: style.clone(),
            text: String::new(),
        };
        let bx = [col * FLOW_STEP[0], row * FLOW_STEP[1], 160.0, 100.0];
        let el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), shape, bx);
        ids.push(el.id);
        BoardOp::Put(el).apply(&mut b.doc);
    }
    let center = |target| End::Bound {
        target,
        anchor: Anchor::Center,
    };
    for i in (0..n).step_by(10) {
        let j = if (i + 1) % side == 0 { i + side } else { i + 1 };
        if j >= n {
            continue;
        }
        let c = Connector::new(center(ids[i]), center(ids[j]), Route::Curved, style.clone());
        let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
        BoardOp::Put(el).apply(&mut b.doc);
    }
    let w = side as f64 * FLOW_STEP[0];
    b.camera = Camera {
        center_x: w / 2.0,
        center_y: w / 2.0,
        zoom: 1000.0 / w,
    };
    set
}

/// As rotas de um quadro, já em dia (o que o editor faz uma vez, fora do quadro do ecrã).
pub(super) fn routes_of(set: &BoardSet) -> RouteCache {
    let mut r = RouteCache::default();
    r.sync(&set.boards()[0].doc);
    r
}

/// Encoda um quadro inteiro (o que o `paint` faz a cada quadro do ecrã).
pub(super) fn one_frame(
    set: &BoardSet,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    routes: &RouteCache,
) {
    scene.reset();
    let board = &set.boards()[0];
    ph2d_board_render::paint(board, AREA, scene, Theme::Forge, ts, cache, routes);
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_board_encode_cost() {
    let boards: Vec<BoardSet> = SCENES.iter().map(|&sc| board_with(sc)).collect();
    let routes: Vec<RouteCache> = boards.iter().map(routes_of).collect();
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    // Uma cache por cena (o ecrã de cada uma), aquecida fora da régua: o regime é o de um quadro
    // PARADO, em que o texto já está moldado.
    let mut caches: Vec<RenderCache> = boards
        .iter()
        .zip(&routes)
        .map(|(b, r)| {
            let mut c = RenderCache::default();
            one_frame(b, &mut scene, &mut ts, &mut c, r);
            c
        })
        .collect();
    let mut per: Vec<Vec<f64>> = vec![Vec::new(); SCENES.len()];
    for round in 0..ROUNDS {
        for k in 0..SCENES.len() {
            let i = (k + round) % SCENES.len(); // ordem rodada
            let t = Instant::now();
            for _ in 0..FRAMES {
                one_frame(&boards[i], &mut scene, &mut ts, &mut caches[i], &routes[i]);
            }
            per[i].push(t.elapsed().as_secs_f64() * 1e3 / FRAMES as f64);
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", load.trim());
    println!("| N | cena | mínimo ms/quadro | mediana |");
    for (k, (n, mix)) in SCENES.iter().enumerate() {
        let mut v = per[k].clone();
        v.sort_by(f64::total_cmp);
        println!("| {n} | {mix:?} | {:.3} | {:.3} |", v[0], v[v.len() / 2]);
    }
}
