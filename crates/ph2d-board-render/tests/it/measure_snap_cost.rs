//! ⏱ **A RÉGUA do encaixe** (W5b, `docs/MiroClone/03_guias_e_alinhamento.md` §5): N rectângulos
//! 120×80 em grelha, TODOS à vista, um em cada três rodado a 30°, e arrasta-se um deles pelo caminho
//! de quem usa (`Editor::pointer_down` / `pointer_move`). Mede, por N e por variante:
//!
//! - **carregar**: o `pointer_down` inteiro — recolher os alvos (o `Snapper` nasce aqui, uma vez
//!   por gesto) e o resto;
//! - **acertar**: só o `Editor::hit` do mesmo ponto (o clique que escolhe a forma, de cima para
//!   baixo) — o chão do carregar que NÃO é do encaixe;
//! - **mexer**: um movimento do rato com o encaixe (mundo + eixos próprios + espaçamento igual) e
//!   o documento mudado ao vivo — o que se paga por quadro enquanto o dedo arrasta.
//!
//! Variantes no MESMO processo: arrastar um DIREITO (só o referencial do mundo) e um RODADO (o dele
//! também); e o `Ctrl` (encaixe suspenso) como chão. Rodadas intercaladas com ordem rodada, vale o
//! MÍNIMO (mediana ao lado). Corre à mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it measure_snap -- --ignored --nocapture
//! ```

use std::time::Instant;

use ph2d_board_edit::{Editor, Metrics, Mods, Pointer};
use ph2d_board_model::{
    BoardDoc, BoardOp, Element, ElementId, History, Rgba, Shape, ShapeType, Style,
};
use ph2d_text::TextSystem;

const NS: [usize; 3] = [1_000, 10_000, 100_000];
const ROUNDS: usize = 7;
const MOVES: usize = 30;
const STEP: [f64; 2] = [200.0, 150.0];

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
    Style::new(Some(Rgba([255, 255, 255, 255])), Some(ink), ink)
}

/// A grelha de `n` e os dois a arrastar: um direito e um rodado.
fn board(n: usize) -> (BoardDoc, [ElementId; 2], [f64; 4]) {
    let mut doc = BoardDoc::default();
    let cols = (n as f64).sqrt().ceil() as usize;
    let mut ids = Vec::new();
    for i in 0..n {
        let (cx, cy) = ((i % cols) as f64, (i / cols) as f64);
        let shape = Shape {
            kind: ShapeType::Rectangle,
            style: style(),
            text: Default::default(),
        };
        let bx = [cx * STEP[0], cy * STEP[1], 120.0, 80.0];
        let mut el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, bx);
        if i % 3 == 1 {
            el.angle = 30f64.to_radians();
        }
        ids.push(el.id);
        BoardOp::Put(el).apply(&mut doc);
    }
    let rows = n.div_ceil(cols) as f64;
    let view = [
        -1e3,
        -1e3,
        cols as f64 * STEP[0] + 1e3,
        rows * STEP[1] + 1e3,
    ];
    (
        doc,
        [ids[n / 2 - n / 2 % 3], ids[n / 2 - n / 2 % 3 + 1]],
        view,
    )
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn at(p: [f64; 2], ctrl: bool) -> Pointer {
    Pointer {
        world: p,
        px: 1.0,
        mods: Mods {
            shift: false,
            ctrl,
            alt: false,
        },
    }
}

fn summary(v: &mut [f64]) -> String {
    v.sort_by(f64::total_cmp);
    format!("{:>8.3} (med {:>8.3})", v[0], v[v.len() / 2])
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_snap_cost() {
    let mut ts = TextSystem::without_system_fonts();
    let boards: Vec<_> = NS.iter().map(|&n| board(n)).collect();
    // [carregar, mexer] por (N, variante): 0 direito · 1 rodado · 2 direito com Ctrl.
    let mut per: Vec<[[Vec<f64>; 3]; 3]> = vec![Default::default(); NS.len()];
    let mut guides = vec![[0usize; 3]; NS.len()];
    for round in 0..ROUNDS {
        for k in 0..NS.len() * 3 {
            let j = (k + round) % (NS.len() * 3);
            let (i, var) = (j / 3, j % 3);
            let (doc0, ids, view) = &boards[i];
            let mut doc = doc0.clone();
            let mut h = History::default();
            let mut ed = Editor::new(style(), metrics());
            let id = ids[var % 2];
            ed.select(&doc, [id]);
            let c = doc.get(id).unwrap().center();
            let ctrl = var == 2;
            let t = Instant::now();
            let _ = std::hint::black_box(ed.hit(&doc, at(c, ctrl)));
            per[i][var][2].push(ms(t));
            let t = Instant::now();
            ed.pointer_down(&mut doc, &mut h, &mut ts, at(c, ctrl), *view);
            per[i][var][0].push(ms(t));
            for m in 0..MOVES {
                // Um passeio de vaivém pela vizinhança (encaixa e desencaixa).
                let d = 5.0 + (m % 10) as f64 * 9.7;
                let p = [c[0] + d, c[1] + d * 0.6];
                let t = Instant::now();
                ed.pointer_move(&mut doc, &mut ts, at(p, ctrl));
                per[i][var][1].push(ms(t));
                guides[i][var] += ed.overlay(&mut doc, &mut ts).guides.len();
            }
            ed.pointer_up(&mut doc, &mut h, at(c, ctrl));
        }
    }
    eprintln!("régua do encaixe — mínimo de {ROUNDS} rodadas (ms), {MOVES} movimentos por gesto");
    for (i, n) in NS.iter().enumerate() {
        for (var, name) in ["direito", "rodado ", "Ctrl   "].iter().enumerate() {
            let [down, mv, hit] = &mut per[i][var];
            eprintln!(
                "N {n:>7} {name}: carregar {} · acertar {} · mexer {} · guias/mov {:.1}",
                summary(down),
                summary(hit),
                summary(mv),
                guides[i][var] as f64 / (ROUNDS * MOVES) as f64
            );
        }
    }
}
