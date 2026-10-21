//! ⭐ **O rascunho pela TINTA** (smoke do dono 21/10, «sketch não tem aspecto de rascunho»): as formas
//! que a caneta dá (desenhar e segurar) pintadas pelo `VelloPass` do produto e lidas de volta, contra
//! as da FERRAMENTA no mesmo quadro em Rascunho. A régua de campos (rascunho, `Look`, semente iguais)
//! passava e o dono via liso: o tremor do rough é de MUNDO (`max_randomness_offset · rugosidade`, ~3
//! px a zoom 1) e não cresce com a espessura — à espessura da caneta (2) as passagens fundem-se num
//! traço só; à da ferramenta (0,5) vêem-se duas.
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --test it sketch_paint`
//! (sem adaptador o teste di-lo e não corre, como `frames_paint`).

use ph2d_board_edit::{Editor, Metrics, Mods, Pointer, Tool};
use ph2d_board_model::{BoardSet, Element, History, Pen, Rgba, ShapeType, Style};
use ph2d_board_render::RenderCache;
use ph2d_board_route::RouteCache;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

#[path = "../../../ph2d-board-edit/src/smart_test_hand.rs"]
#[allow(dead_code)]
mod hand;

const SIZE: (u32, u32) = (1100, 820);
const VIEW: [f64; 4] = [-1e4, -1e4, 1e4, 1e4];

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

fn at(p: [f64; 2]) -> Pointer {
    Pointer {
        world: p,
        px: 1.0,
        mods: Mods::default(),
    }
}

/// Um quadro em Rascunho (mundo = ecrã) com três rectângulos da FERRAMENTA à esquerda e três da
/// CANETA + segurar à direita (a caneta de nascença: espessura `PEN_WIDTHS[0]`).
fn board() -> BoardSet {
    let ink = Rgba(ph2d_board_model::DEFAULT_INK);
    let mut set = BoardSet::default();
    let id = set.create("rascunho".into());
    let b = set.get_mut(id).expect("o quadro");
    b.sketch = true;
    b.camera.center_x = f64::from(SIZE.0) / 2.0;
    b.camera.center_y = f64::from(SIZE.1) / 2.0;
    let doc = &mut b.doc;
    let mut ed = Editor::new(Style::new(None, Some(ink), ink), metrics());
    let (mut h, mut ts) = (History::default(), TextSystem::without_system_fonts());
    let mut t: u128 = 1_000_000_000_000;
    for row in 0..3 {
        let y = 40.0 + 260.0 * f64::from(row);
        ed.tool = Tool::Shape(ShapeType::Rectangle);
        ed.style.sketch = true;
        let (a, z) = ([60.0, y], [360.0, y + 200.0]);
        t += 1_000_000_000;
        ed.at_time(t);
        ed.pointer_down(doc, &mut h, &mut ts, at(a), VIEW);
        ed.pointer_move(doc, &mut ts, at([210.0, y + 100.0]));
        ed.pointer_move(doc, &mut ts, at(z));
        ed.pointer_up(doc, &mut h, at(z));
        ed.tool = Tool::Pen(Pen::Pen);
        let pts: Vec<[f64; 2]> = hand::hand_rect()
            .into_iter()
            .map(|p| [p[0] + 540.0, p[1] - 60.0 + 260.0 * f64::from(row)])
            .collect();
        t += 1_000_000_000;
        ed.at_time(t);
        ed.pointer_down(doc, &mut h, &mut ts, at(pts[0]), VIEW);
        for &p in &pts[1..] {
            t += 4_000_000;
            ed.at_time(t);
            ed.pointer_move(doc, &mut ts, at(p));
        }
        t += 600_000_000;
        ed.at_time(t);
        let end = pts[pts.len() - 1];
        ed.pointer_move(doc, &mut ts, at(end));
        assert!(ed.is_quick(), "a forma nasceu");
        ed.pointer_up(doc, &mut h, at(end));
    }
    set
}

fn render(set: &BoardSet) -> Option<Vec<u8>> {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        println!("sem adaptador: o rascunho pela tinta não corre");
        return None;
    };
    let mut pass = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, SIZE).expect("VelloPass");
    let board = &set.boards()[0];
    let mut routes = RouteCache::default();
    routes.sync(&board.doc);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let mut cache = RenderCache::default();
    let area = [0.0, 0.0, f64::from(SIZE.0), f64::from(SIZE.1)];
    ph2d_board_render::paint(
        board,
        area,
        &mut scene,
        Theme::Forge,
        &mut ts,
        &mut cache,
        &routes,
        None,
    );
    Some(
        pass.render_and_readback(&gpu, scene.inner(), SIZE)
            .expect("readback"),
    )
}

/// Ao longo das quatro arestas (os 60 % do meio de cada), a fracção das linhas perpendiculares em que
/// a tinta mostra DUAS passagens separadas (duas corridas de píxeis numa faixa de ±15 px).
fn doubled(px: &[u8], el: &Element) -> f64 {
    let lum = |x: i64, y: i64| {
        let i = ((y as u32 * SIZE.0 + x as u32) * 4) as usize;
        0.2126 * f64::from(px[i]) + 0.7152 * f64::from(px[i + 1]) + 0.0722 * f64::from(px[i + 2])
    };
    let bg = lum(0, 0);
    let ink = |x: i64, y: i64| (lum(x, y) - bg).abs() > 60.0;
    let (x0, y0) = (el.x.round() as i64, el.y.round() as i64);
    let (x1, y1) = ((el.x + el.w).round() as i64, (el.y + el.h).round() as i64);
    let (mut lines, mut two) = (0.0, 0.0);
    let edges: [(i64, i64, i64, bool); 4] = [
        (y0, x0, x1, true),
        (y1, x0, x1, true),
        (x0, y0, y1, false),
        (x1, y0, y1, false),
    ];
    for (fixed, a, b, horizontal) in edges {
        let span = b - a;
        for s in (a + span / 5)..(b - span / 5) {
            let on: Vec<i64> = (-15..=15)
                .filter(|&o| {
                    if horizontal {
                        ink(s, fixed + o)
                    } else {
                        ink(fixed + o, s)
                    }
                })
                .collect();
            if on.is_empty() {
                continue;
            }
            lines += 1.0;
            if on.windows(2).any(|w| w[1] - w[0] > 1) {
                two += 1.0;
            }
        }
    }
    two / f64::max(lines, 1.0)
}

/// ⭐ Num quadro em Rascunho, o rectângulo da caneta + segurar lê-se à mão como o da ferramenta: as
/// duas passagens do rough VÊEM-SE. Medido (21/10, `docs/MiroClone/17_bloco6_plano.md` §9): a
/// ferramenta (0,5) mostra 22–65 % das linhas com duas passagens; a caneta à espessura dela (2), 0 %.
#[test]
fn the_pens_held_box_reads_as_sketched_as_the_tools() {
    let set = board();
    let Some(px) = render(&set) else {
        return;
    };
    let els: Vec<Element> = set.boards()[0]
        .doc
        .live_in_z_order()
        .into_iter()
        .cloned()
        .collect();
    let (mut tool, mut pen) = (Vec::new(), Vec::new());
    for el in &els {
        assert_eq!(el.shape().map(|s| s.kind), Some(ShapeType::Rectangle));
        let d = doubled(&px, el);
        if el.x < 500.0 { &mut tool } else { &mut pen }.push(d);
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (tool, pen) = (mean(&tool), mean(&pen));
    assert!(
        tool > 0.2,
        "o controlo: a ferramenta lê-se à mão ({tool:.2})"
    );
    assert!(
        pen >= 0.5 * tool,
        "a caneta lê-se lisa: {:.0} % das linhas com duas passagens, a ferramenta {:.0} %",
        pen * 100.0,
        tool * 100.0
    );
}
