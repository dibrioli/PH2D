//! ⏱ **A RÉGUA das setas** (W2, `docs/MiroClone/02_plano.md` §2: «rota em cache, refeita só quando
//! uma ponta ou um obstáculo próximo muda») — um fluxograma de N formas 160×100 em grelha com N/10
//! setas em cotovelo entre vizinhas, presas ao centro. Mede, por N:
//!
//! - **frio**: a primeira sincronização (todas as rotas ao A\*);
//! - **parado**: a sincronização de um quadro em que nada mudou (a revisão é a mesma);
//! - **arrasto**: um quadro de arrastar UMA forma com setas (mexe 1 un. e sincroniza) — o que se paga
//!   por quadro enquanto o dedo arrasta, e quantas setas voltaram ao roteador;
//! - **desenho**: encodar o quadro inteiro com as setas (vista a enquadrar tudo).
//!
//! Régua do dono (05/10): variantes no MESMO processo, rodadas intercaladas com ordem rodada, vale o
//! MÍNIMO (mediana ao lado). Corre à mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it measure_route -- --ignored --nocapture
//! ```

use ph2d_board_model::{BoardOp, BoardSet, Element, ElementId};
use ph2d_board_render::RenderCache;
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::time::Instant;

use super::measure_encode_cost::{AREA, FRAMES, Mix, ROUNDS, board_with};

const NS: [usize; 3] = [1_000, 10_000, 100_000];

/// O fluxograma da régua (`Mix::Flow`) e a forma a arrastar: a origem da 1.ª seta.
fn flow(n: usize) -> (BoardSet, ElementId) {
    let set = board_with((n, Mix::Flow));
    let doc = &set.boards()[0].doc;
    let drag = doc
        .live()
        .find_map(|el| el.connector()?.targets().next())
        .expect("há setas");
    (set, drag)
}

/// Mexe a forma `id` uma unidade (o que um quadro de arrasto faz ao documento).
fn nudge(set: &mut BoardSet, id: ElementId, k: usize) {
    let bid = set.boards()[0].id;
    let doc = &mut set.get_mut(bid).unwrap().doc;
    let mut el = doc.get(id).unwrap().clone();
    el.translate([if k % 2 == 0 { 1.0 } else { -1.0 }, 0.0]);
    BoardOp::Put(el).apply(doc);
}

/// Quantos obstáculos cada rota enxerga: com a lei do vectorial (o ponto fixo SEM tecto) e com o
/// tecto do quadro (`DETOUR_K`) — o número que motivou o tecto (só onde a lei sem tecto cabe).
fn obstacle_counts(doc: &ph2d_board_model::BoardDoc, routes: &RouteCache) -> (String, String) {
    use ph2d_vec_connect::{Aabb, ROI_PAD_K, obstacles_in_play};
    let bx = |el: &Element| {
        let [x0, y0, x1, y1] = el.aabb();
        Aabb::new([x0, y0], [x1, y1])
    };
    let shapes: Vec<Aabb> = doc
        .live()
        .filter(|el| el.shape().is_some())
        .map(bx)
        .collect();
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for el in doc.live() {
        let Some(c) = el.connector() else { continue };
        let ends: Vec<Aabb> = c.targets().filter_map(|t| doc.get(t)).map(bx).collect();
        let pad = ROI_PAD_K * ph2d_board_route::JETTY;
        a.push(obstacles_in_play(&shapes, ends[0], ends[1], pad).len());
        b.push(routes.obstacles(el.id).unwrap_or(0));
    }
    let fmt = |v: &[usize]| {
        let mean = v.iter().sum::<usize>() as f64 / v.len().max(1) as f64;
        format!("{mean:.1} / {}", v.iter().max().copied().unwrap_or(0))
    };
    (fmt(&a), fmt(&b))
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_route_cost() {
    let mut boards: Vec<(BoardSet, ElementId)> = NS.iter().map(|&n| flow(n)).collect();
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let mut caches: Vec<(RouteCache, RenderCache)> = boards
        .iter()
        .map(|(b, _)| {
            let mut r = RouteCache::default();
            r.sync(&b.boards()[0].doc);
            (r, RenderCache::default())
        })
        .collect();
    // [frio, parado, arrasto, desenho] por N.
    let mut per: Vec<[Vec<f64>; 4]> = vec![Default::default(); NS.len()];
    let mut rerouted = vec![0usize; NS.len()];
    let mut revisited = vec![0usize; NS.len()];
    for round in 0..ROUNDS {
        for k in 0..NS.len() {
            let i = (k + round) % NS.len();
            let (set, drag) = &mut boards[i];
            let doc = &set.boards()[0].doc;
            let t = Instant::now();
            let mut cold = RouteCache::default();
            cold.sync(doc);
            per[i][0].push(ms(t));

            let (routes, render) = &mut caches[i];
            let t = Instant::now();
            for _ in 0..FRAMES {
                routes.sync(&set.boards()[0].doc);
            }
            per[i][1].push(ms(t) / FRAMES as f64);

            let t = Instant::now();
            for f in 0..FRAMES {
                nudge(set, *drag, f);
                routes.sync(&set.boards()[0].doc);
            }
            per[i][2].push(ms(t) / FRAMES as f64);
            rerouted[i] = routes.rerouted();
            revisited[i] = routes.revisited();

            let t = Instant::now();
            for _ in 0..FRAMES {
                scene.reset();
                let b = &set.boards()[0];
                ph2d_board_render::paint(
                    b,
                    AREA,
                    &mut scene,
                    Theme::Forge,
                    &mut ts,
                    render,
                    routes,
                );
            }
            per[i][3].push(ms(t) / FRAMES as f64);
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", load.trim());
    println!(
        "| N formas | obstáculos por rota SEM tecto (média / máx) | COM tecto (média / máx) |"
    );
    for (k, n) in NS.iter().enumerate().take(2) {
        let doc = &boards[k].0.boards()[0].doc;
        let (without, with) = obstacle_counts(doc, &caches[k].0);
        println!("| {n} | {without} | {with} |");
    }
    println!(
        "| N formas | setas | frio ms | parado ms | arrasto ms/quadro (revistas / refeitas) | desenho ms/quadro |"
    );
    for (k, n) in NS.iter().enumerate() {
        let min_med = |v: &Vec<f64>| {
            let mut v = v.clone();
            v.sort_by(f64::total_cmp);
            format!("{:.3} ({:.3})", v[0], v[v.len() / 2])
        };
        let setas = boards[k].0.boards()[0]
            .doc
            .live()
            .filter(|el| el.connector().is_some())
            .count();
        println!(
            "| {n} | {setas} | {} | {} | {} ({} / {}) | {} |",
            min_med(&per[k][0]),
            min_med(&per[k][1]),
            min_med(&per[k][2]),
            revisited[k],
            rerouted[k],
            min_med(&per[k][3]),
        );
    }
}
