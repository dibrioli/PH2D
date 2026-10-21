//! ⏱ **A régua do bloco 4 da W23** (B6 · B7, `docs/MiroClone/16_bloco4_velocidade.md`) — num só processo:
//!
//! - **B6** o rascunho de quadro grande: por tamanho da forma no ECRÃ, o custo de encode (CPU) e de
//!   placa do traço à mão contra o final, e se a diferença SE VÊ — a diferença de píxeis rascunho × final
//!   contra o CONTROLO: o mesmo final deslocado meio píxel (o que ninguém distingue).
//! - **B7** o chão das cenas de 100 mil: as maneiras de pôr os vivos na ordem de desenho, conferidas
//!   iguais à de hoje antes de cronometradas.
//!
//! Variantes no mesmo processo, intercaladas por rodada, vale o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`):
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --profile smoke --test it measure_bloco4 -- --ignored --nocapture
//! ```

use super::measure_encode_cost::{FRAMES, Mix, ROUNDS, board_with, one_frame, routes_of};
use super::measure_gpu_raster_cost::{SIZE, min_med, wall_frame};
use ph2d_board_model::{BoardOp, BoardSet, Element};
use ph2d_board_render::{Detail, RenderCache};
use ph2d_board_route::RouteCache;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::time::Instant;

/// O lado da forma da mistura, em unidades do mundo (`board_with`).
const SIDE: f64 = 24.0;
/// Os tamanhos da forma no ecrã medidos (px). Abaixo de 4 já é um ponto de cor (`DOT_PX`).
const PX: [f64; 9] = [5.0, 6.0, 8.0, 10.0, 11.0, 12.0, 16.0, 24.0, 32.0];
const LOOK: (u32, u32) = (640, 480);

fn at_px(mut set: BoardSet, px: f64) -> BoardSet {
    let id = set.boards()[0].id;
    set.get_mut(id).unwrap().camera.zoom = px / SIDE;
    set
}

fn cache_for(final_below: f64) -> RenderCache {
    let mut c = RenderCache::default();
    c.detail = Detail {
        sketch_final_below_px: final_below,
        ..Detail::default()
    };
    c
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_owned()
}

#[test]
#[ignore = "régua: corre-se à mão (ver o cabeçalho)"]
fn measure_bloco4() {
    println!("loadavg no início: {}", loadavg());
    if std::env::var_os("PH2D_BLOCO4_SO_OLHAR").is_none() {
        b6_cost();
    }
    b6_looks();
    if std::env::var_os("PH2D_BLOCO4_SO_OLHAR").is_none() {
        b7_order();
    }
    println!("loadavg no fim: {}", loadavg());
}

/// 10 mil formas em rascunho, a vista a pôr cada uma a `px` no ecrã: à mão (`0`) contra final.
fn b6_cost() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).ok();
    let mut pass = gpu
        .as_ref()
        .map(|g| VelloPass::new(g, wgpu::TextureFormat::Rgba8Unorm, SIZE).expect("VelloPass"));
    let sets: Vec<BoardSet> = PX
        .iter()
        .map(|&p| at_px(board_with((10_000, Mix::Sketch)), p))
        .collect();
    let routes: Vec<RouteCache> = sets.iter().map(routes_of).collect();
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    // [tamanho][0 = à mão, 1 = final]
    let mut caches: Vec<[RenderCache; 2]> = Vec::new();
    let mut scenes: Vec<[VectorScene; 2]> = Vec::new();
    for (s, r) in sets.iter().zip(&routes) {
        let mut pair = [cache_for(0.0), cache_for(f64::INFINITY)];
        let mut out = [VectorScene::new(), VectorScene::new()];
        for (c, o) in pair.iter_mut().zip(&mut out) {
            one_frame(s, o, &mut ts, c, r);
        }
        caches.push(pair);
        scenes.push(out);
    }
    let n = PX.len() * 2;
    let (mut cpu, mut gpu_ms) = (vec![Vec::new(); n], vec![Vec::new(); n]);
    for round in 0..ROUNDS {
        for k in 0..n {
            let i = (k + round) % n;
            let (p, v) = (i / 2, i % 2);
            let t = Instant::now();
            for _ in 0..FRAMES {
                one_frame(&sets[p], &mut scene, &mut ts, &mut caches[p][v], &routes[p]);
            }
            cpu[i].push(t.elapsed().as_secs_f64() * 1e3 / FRAMES as f64);
            if let (Some(g), Some(pass)) = (&gpu, &mut pass) {
                gpu_ms[i].push(wall_frame(g, pass, &scenes[p][v]));
            }
        }
    }
    println!("\n## B6 custo — 10 mil formas em rascunho (mínimo de {ROUNDS}×{FRAMES}; mediana)");
    println!("| forma no ecrã | CPU à mão | CPU de longe | placa à mão | placa de longe |");
    for (p, px) in PX.iter().enumerate() {
        let f = |v: &Vec<f64>| {
            if v.is_empty() {
                "—".to_owned()
            } else {
                let (a, b) = min_med(v);
                format!("{a:.2} ({b:.2})")
            }
        };
        println!(
            "| {px} px | {} | {} | {} | {} |",
            f(&cpu[2 * p]),
            f(&cpu[2 * p + 1]),
            f(&gpu_ms[2 * p]),
            f(&gpu_ms[2 * p + 1])
        );
    }
}

fn render_look(
    g: &mut (GpuContext, VelloPass),
    set: &BoardSet,
    final_below: f64,
    dx: f64,
) -> Vec<u8> {
    let mut set = set.clone();
    let id = set.boards()[0].id;
    let cam = &mut set.get_mut(id).unwrap().camera;
    cam.center_x += dx / cam.zoom;
    let board = &set.boards()[0];
    let mut routes = RouteCache::default();
    routes.sync(&board.doc);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let mut cache = cache_for(final_below);
    let area = [0.0, 0.0, f64::from(LOOK.0), f64::from(LOOK.1)];
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
    g.1.render_and_readback(&g.0, scene.inner(), LOOK)
        .expect("readback")
}

/// A média e o p99,9 de |Δ luma| (0–255) entre duas imagens.
fn diff(a: &[u8], b: &[u8]) -> (f64, f64) {
    let luma =
        |p: &[u8]| 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]);
    let mut d: Vec<f64> = a
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .map(|(x, y)| (luma(x) - luma(y)).abs())
        .collect();
    let mean = d.iter().sum::<f64>() / d.len() as f64;
    d.sort_by(f64::total_cmp);
    (mean, d[d.len() * 999 / 1000])
}

/// O mesmo quadro em FINAL com o contorno `k` vezes mais grosso (o rascunho passa a linha duas vezes).
fn thicker(set: &BoardSet, k: f64) -> BoardSet {
    let mut set = set.clone();
    let id = set.boards()[0].id;
    let b = set.get_mut(id).unwrap();
    let els: Vec<Element> = b.doc.live().cloned().collect();
    for mut e in els {
        if let Some(s) = e.shape_mut() {
            s.style.sketch = false;
            s.style.stroke_width *= k;
        }
        BoardOp::Put(e).apply(&mut b.doc);
    }
    set
}

/// Se o rascunho pequeno SE VÊ: rascunho × final contra o controlo final × final deslocado ½ px.
fn b6_looks() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        println!("sem adaptador: a visibilidade do B6 não corre");
        return;
    };
    let pass = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, LOOK).expect("VelloPass");
    let mut g = (gpu, pass);
    let dir = std::env::var_os("PH2D_BLOCO4_PNG").map(std::path::PathBuf::from);
    println!(
        "\n## B6 visibilidade — |Δ luma| média · p99,9 (0–255); razão = rascunho × X ÷ controlo (de longe × ele ½ px)"
    );
    println!(
        "| forma no ecrã | rascunho × de longe | controlo ½ px | razão de longe | razão final simples (o candidato do W5) · contorno ×2 · ×3 |"
    );
    for px in PX {
        let set = at_px(board_with((400, Mix::Sketch)), px);
        let hand = render_look(&mut g, &set, 0.0, 0.0);
        let fin = render_look(&mut g, &set, f64::INFINITY, 0.0);
        let half = render_look(&mut g, &set, f64::INFINITY, 0.5);
        let (a, ap) = diff(&hand, &fin);
        let (c, cp) = diff(&fin, &half);
        let thick: Vec<Vec<u8>> = [1.0, 2.0, 3.0]
            .iter()
            .map(|&k| render_look(&mut g, &thicker(&set, k), f64::INFINITY, 0.0))
            .collect();
        let r: Vec<String> = thick
            .iter()
            .map(|t| format!("{:.2}", diff(&hand, t).0 / c.max(1e-9)))
            .collect();
        println!(
            "| {px} px | {a:.3} · {ap:.0} | {c:.3} · {cp:.0} | {:.2} | {} |",
            a / c.max(1e-9),
            r.join(" · ")
        );
        if let Some(d) = &dir {
            for (name, px_buf) in [("mao", &hand), ("longe", &fin), ("final", &thick[0])] {
                let mut ppm = format!("P6\n{} {}\n255\n", LOOK.0, LOOK.1).into_bytes();
                ppm.extend(
                    px_buf
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .flat_map(|p| [p[0], p[1], p[2]]),
                );
                std::fs::write(d.join(format!("b6_{px:02}px_{name}.ppm")), ppm).expect("ppm");
            }
        }
    }
}

fn by_z(a: &Element, b: &Element) -> std::cmp::Ordering {
    a.z.cmp(&b.z).then(a.id.cmp(&b.id))
}

fn embeds_last(v: Vec<&Element>) -> Vec<&Element> {
    let (mut out, top): (Vec<&Element>, Vec<&Element>) =
        v.into_iter().partition(|e| e.embed().is_none());
    out.extend(top);
    out
}

fn order(doc: &ph2d_board_model::BoardDoc, variant: usize) -> Vec<&Element> {
    match variant {
        0 => doc.live_in_paint_order(),
        2 => {
            let mut v: Vec<&Element> = doc.live().collect();
            v.sort_by(|a, b| by_z(a, b));
            v.sort_by_key(|e| e.embed().is_some());
            v
        }
        1 => {
            let mut v: Vec<&Element> = doc.live().collect();
            v.sort_unstable_by(|a, b| by_z(a, b));
            embeds_last(v)
        }
        _ => doc.live().collect(), // o chão: a passagem pelo `BTreeMap` e o `collect`
    }
}

/// Os z de um quadro baralhados (o quadro depois de muitos «para a frente»): a ordem de z deixa de ser a
/// dos ids.
fn shuffled(mut set: BoardSet) -> BoardSet {
    let id = set.boards()[0].id;
    let b = set.get_mut(id).unwrap();
    let mut els: Vec<Element> = b.doc.live().cloned().collect();
    let mut zs: Vec<_> = els.iter().map(|e| e.z.clone()).collect();
    let mut s = 0x9E37_79B9_7F4A_7C15_u64;
    for i in (1..zs.len()).rev() {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        zs.swap(i, (s % (i as u64 + 1)) as usize);
    }
    for (e, z) in els.iter_mut().zip(zs) {
        e.z = z;
        BoardOp::Put(e.clone()).apply(&mut b.doc);
    }
    set
}

const ORDER: [&str; 4] = [
    "o produto (prefixo u64 + partição)",
    "instável + partição",
    "o de antes (estável ×2)",
    "chão: passagem + collect",
];

fn b7_order() {
    let cases: Vec<(&str, BoardSet)> = vec![
        ("10 mil à vista", board_with((10_000, Mix::Shapes))),
        (
            "100 mil, z na ordem dos ids",
            board_with((100_000, Mix::Rects)),
        ),
        (
            "100 mil, z baralhado",
            shuffled(board_with((100_000, Mix::Rects))),
        ),
    ];
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    println!("\n## B7 ordem de desenho (ms por imagem; mínimo de {ROUNDS}×{FRAMES}; mediana)");
    for (name, set) in &cases {
        let doc = &set.boards()[0].doc;
        let want: Vec<_> = order(doc, 0).iter().map(|e| e.id).collect();
        for (v, variant) in ORDER.iter().enumerate().take(3).skip(1) {
            let got: Vec<_> = order(doc, v).iter().map(|e| e.id).collect();
            assert_eq!(got, want, "{name}: a variante {variant} muda a ordem");
        }
        let routes = routes_of(set);
        let mut cache = RenderCache::default();
        one_frame(set, &mut scene, &mut ts, &mut cache, &routes);
        let n = ORDER.len() + 1;
        let mut per = vec![Vec::new(); n];
        for round in 0..ROUNDS {
            for k in 0..n {
                let i = (k + round) % n;
                let t = Instant::now();
                for _ in 0..FRAMES {
                    if i == ORDER.len() {
                        one_frame(set, &mut scene, &mut ts, &mut cache, &routes);
                    } else {
                        std::hint::black_box(order(doc, i).len());
                    }
                }
                per[i].push(t.elapsed().as_secs_f64() * 1e3 / FRAMES as f64);
            }
        }
        println!("### {name}");
        for (i, v) in per.iter().enumerate() {
            let (a, b) = min_med(v);
            let label = ORDER.get(i).copied().unwrap_or("a imagem inteira");
            println!("| {label} | {a:.3} | {b:.3} |");
        }
    }
}
