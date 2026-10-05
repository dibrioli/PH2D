//! ⏱ **A RÉGUA do desenho do quadro, lado CPU** (W0, `docs/MiroClone/02_plano.md` §2) — quanto
//! custa ENCODAR um quadro de N rectângulos numa `VectorScene` (o que `paint` faz a cada quadro).
//!
//! Régua do dono (05/10): as variantes correm no MESMO processo, intercaladas em rodadas curtas com
//! ordem rodada, e vale o MÍNIMO (mediana ao lado como controlo). ⚠️ Mede só a CPU: o tempo da placa
//! (raster do Vello) é do perfilador da GPU, na W1. Corre à mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it -- --ignored --nocapture
//! ```

use ph2d_board_model::{BoardOp, BoardSet, Camera, Element, ElementKind, Rgba};
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::time::Instant;

const SIZES: [usize; 3] = [1_000, 10_000, 100_000];
const ROUNDS: usize = 7;
const FRAMES: usize = 20;
const AREA: [f64; 4] = [0.0, 0.0, 1920.0, 1080.0];

/// N rectângulos numa grelha quadrada, com a vista a enquadrar a grelha inteira (todos visíveis).
fn board_with(n: usize) -> BoardSet {
    let mut set = BoardSet::default();
    let id = set.create(format!("{n}"));
    let b = set.get_mut(id).unwrap();
    let side = (n as f64).sqrt().ceil();
    for i in 0..n {
        let (row, col) = ((i as f64 / side).floor(), (i as f64 % side));
        let el = Element {
            id: b.doc.mint_id(),
            kind: ElementKind::Rect {
                fill: Rgba([200, 120, 40, 255]),
            },
            x: col * 30.0,
            y: row * 30.0,
            w: 24.0,
            h: 24.0,
            z: b.doc.z_on_top(),
            version: 0,
            nonce: 0,
            deleted: false,
        };
        BoardOp::Put(el).apply(&mut b.doc);
    }
    b.camera = Camera {
        center_x: side * 15.0,
        center_y: side * 15.0,
        zoom: 1000.0 / (side * 30.0),
    };
    set
}

fn one_frame(set: &BoardSet, scene: &mut VectorScene) {
    scene.reset();
    ph2d_board_render::paint(&set.boards()[0], AREA, scene, Theme::Forge);
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_board_encode_cost() {
    let boards: Vec<BoardSet> = SIZES.iter().map(|&n| board_with(n)).collect();
    let mut scene = VectorScene::new();
    let mut per: Vec<Vec<f64>> = vec![Vec::new(); SIZES.len()];
    for round in 0..ROUNDS {
        for k in 0..SIZES.len() {
            let i = (k + round) % SIZES.len(); // ordem rodada
            let t = Instant::now();
            for _ in 0..FRAMES {
                one_frame(&boards[i], &mut scene);
            }
            per[i].push(t.elapsed().as_secs_f64() * 1e3 / FRAMES as f64);
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", load.trim());
    println!("| N | mínimo ms/quadro | mediana |");
    for (k, n) in SIZES.iter().enumerate() {
        let mut v = per[k].clone();
        v.sort_by(f64::total_cmp);
        println!("| {n} | {:.3} | {:.3} |", v[0], v[v.len() / 2]);
    }
}
