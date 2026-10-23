//! ⏱ **A régua da hachura de longe** (W24, `docs/MiroClone/16_bloco4_velocidade.md` §1, a recusa «levar o
//! caminho de longe às formas com preenchimento às riscas — NÃO medido»), num só processo:
//!
//! - **custo** — 10 mil formas a cada tamanho no ECRÃ: o sólido em rascunho (que já toma o caminho de
//!   longe), cada preenchimento às riscas À MÃO e a candidata DE LONGE (o final de contorno engrossado
//!   com o preenchimento liso num tom, [`Detail::hatch_final_below_px`]).
//! - **visibilidade** — 400 formas: |Δ luma| entre as riscas à mão e a candidata a cada TOM, dividida
//!   pelo controlo (a candidata contra ela própria deslocada ½ px, o que ninguém distingue). Razão ≤ 1
//!   ⇒ não se vê. O tom não se inventa: é o que esta tabela escolhe.
//!
//! Variantes no mesmo processo, intercaladas por rodada, vale o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`):
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --profile smoke --test it measure_hatch_far -- --ignored --nocapture
//! ```
//! `PH2D_HATCH_SO_OLHAR=1` salta o custo; `PH2D_HATCH_PNG=<pasta>` grava as imagens (PPM).

use super::measure_bloco4::{LOOK, at_px, diff, loadavg};
use super::measure_encode_cost::{FRAMES, Mix, ROUNDS, board_with, one_frame, routes_of};
use super::measure_gpu_raster_cost::{SIZE, min_med, wall_frame};
use ph2d_board_model::{BoardOp, BoardSet, Element, FillStyle};
use ph2d_board_render::{Detail, RenderCache};
use ph2d_board_route::RouteCache;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::time::Instant;

const PX: [f64; 7] = [5.0, 6.0, 8.0, 10.0, 11.0, 12.0, 16.0];
const HATCHES: [(&str, FillStyle); 3] = [
    ("riscas", FillStyle::Hachure),
    ("cruzadas", FillStyle::CrossHatch),
    ("ziguezague", FillStyle::Zigzag),
];
/// Os tons varridos pela visibilidade.
const TONES: [f32; 10] = [0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.5, 0.6, 0.7];

/// A mistura `n` em rascunho (`sketch`) com o preenchimento `fill` em todas as formas.
fn filled(n: usize, fill: FillStyle, sketch: bool) -> BoardSet {
    let mut set = board_with((n, Mix::Sketch));
    let id = set.boards()[0].id;
    let b = set.get_mut(id).unwrap();
    let els: Vec<Element> = b.doc.live().cloned().collect();
    for mut e in els {
        if let Some(s) = e.shape_mut() {
            s.style.look.fill_style = fill;
            s.style.sketch = sketch;
        }
        BoardOp::Put(e).apply(&mut b.doc);
    }
    set
}

/// O detalhe do produto com as riscas à mão (`None`) ou de longe no tom `k`.
fn detail(tone: Option<f32>) -> Detail {
    Detail {
        hatch_final_below_px: if tone.is_some() { f64::INFINITY } else { 0.0 },
        hatch_tone: [tone.unwrap_or(0.0); 3],
        ..Detail::default()
    }
}

fn cache(tone: Option<f32>) -> RenderCache {
    let mut c = RenderCache::default();
    c.detail = detail(tone);
    c
}

#[test]
#[ignore = "régua: corre-se à mão (ver o cabeçalho)"]
fn measure_hatch_far() {
    println!("loadavg no início: {}", loadavg());
    if std::env::var_os("PH2D_HATCH_SO_OLHAR").is_none() {
        cost();
    }
    looks();
    println!("loadavg no fim: {}", loadavg());
}

/// O custo: [tamanho] × (sólido, cada riscas à mão, cada riscas de longe).
fn cost() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).ok();
    let mut pass = gpu
        .as_ref()
        .map(|g| VelloPass::new(g, wgpu::TextureFormat::Rgba8Unorm, SIZE).expect("VelloPass"));
    let mut cols: Vec<(String, FillStyle, Option<f32>)> =
        vec![("sólido".to_owned(), FillStyle::Solid, None)];
    for (name, f) in HATCHES {
        cols.push((format!("{name} à mão"), f, None));
        cols.push((format!("{name} de longe"), f, Some(0.3)));
    }
    let mut sets = Vec::new();
    let mut caches = Vec::new();
    for &p in &PX {
        for (_, f, tone) in &cols {
            sets.push(at_px(filled(10_000, *f, true), p));
            caches.push(cache(*tone));
        }
    }
    let routes: Vec<RouteCache> = sets.iter().map(routes_of).collect();
    let mut ts = TextSystem::without_system_fonts();
    let mut scenes: Vec<VectorScene> = Vec::new();
    for ((s, r), c) in sets.iter().zip(&routes).zip(&mut caches) {
        let mut out = VectorScene::new();
        one_frame(s, &mut out, &mut ts, c, r);
        scenes.push(out);
    }
    let n = sets.len();
    let mut scene = VectorScene::new();
    let (mut cpu, mut gpu_ms) = (vec![Vec::new(); n], vec![Vec::new(); n]);
    for round in 0..ROUNDS {
        for k in 0..n {
            let i = (k + round) % n;
            let t = Instant::now();
            for _ in 0..FRAMES {
                one_frame(&sets[i], &mut scene, &mut ts, &mut caches[i], &routes[i]);
            }
            cpu[i].push(t.elapsed().as_secs_f64() * 1e3 / FRAMES as f64);
            if let (Some(g), Some(pass)) = (&gpu, &mut pass) {
                gpu_ms[i].push(wall_frame(g, pass, &scenes[i]));
            }
        }
    }
    let f = |v: &Vec<f64>| {
        if v.is_empty() {
            "—".to_owned()
        } else {
            min_med(v).0.to_string().chars().take(5).collect()
        }
    };
    println!("\n## Custo — 10 mil formas em rascunho, ms (mínimo de {ROUNDS}×{FRAMES}): CPU / placa");
    let head: Vec<&str> = cols.iter().map(|c| c.0.as_str()).collect();
    println!("| forma no ecrã | {} |", head.join(" | "));
    for (p, px) in PX.iter().enumerate() {
        let cells: Vec<String> = (0..cols.len())
            .map(|k| {
                let i = p * cols.len() + k;
                format!("{} / {}", f(&cpu[i]), f(&gpu_ms[i]))
            })
            .collect();
        println!("| {px} px | {} |", cells.join(" | "));
    }
}

fn render(g: &mut (GpuContext, VelloPass), set: &BoardSet, tone: Option<f32>, dx: f64) -> Vec<u8> {
    let mut set = set.clone();
    let id = set.boards()[0].id;
    let cam = &mut set.get_mut(id).unwrap().camera;
    cam.center_x += dx / cam.zoom;
    let board = &set.boards()[0];
    let mut routes = RouteCache::default();
    routes.sync(&board.doc);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let mut c = cache(tone);
    let area = [0.0, 0.0, f64::from(LOOK.0), f64::from(LOOK.1)];
    ph2d_board_render::paint(board, area, &mut scene, Theme::Forge, &mut ts, &mut c, &routes, None);
    g.1.render_and_readback(&g.0, scene.inner(), LOOK)
        .expect("readback")
}

fn ppm(dir: &std::path::Path, name: &str, buf: &[u8]) {
    let mut out = format!("P6\n{} {}\n255\n", LOOK.0, LOOK.1).into_bytes();
    out.extend(buf.as_chunks::<4>().0.iter().flat_map(|p| [p[0], p[1], p[2]]));
    std::fs::write(dir.join(name), out).expect("ppm");
}

/// A visibilidade: por riscas e tamanho, a razão de cada tom (rascunho; e o final às riscas importado).
fn looks() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        println!("sem adaptador: a visibilidade não corre");
        return;
    };
    let pass = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, LOOK).expect("VelloPass");
    let mut g = (gpu, pass);
    let dir = std::env::var_os("PH2D_HATCH_PNG").map(std::path::PathBuf::from);
    let tones: Vec<String> = TONES.iter().map(|k| format!("tom {k}")).collect();
    for (name, fill, sketch) in HATCHES
        .iter()
        .map(|(n, f)| (n.to_string(), *f, true))
        .chain([("riscas, final importado".to_owned(), FillStyle::Hachure, false)])
    {
        println!(
            "\n## Visibilidade — {name}: razão = |Δ luma|(à mão × de longe no tom) ÷ controlo ½ px (≤ 1 não se vê)"
        );
        println!("| forma no ecrã | {} |", tones.join(" | "));
        for px in PX {
            let set = at_px(filled(400, fill, sketch), px);
            let hand = render(&mut g, &set, None, 0.0);
            let cells: Vec<String> = TONES
                .iter()
                .map(|&k| {
                    let far = render(&mut g, &set, Some(k), 0.0);
                    let half = render(&mut g, &set, Some(k), 0.5);
                    let ratio = diff(&hand, &far).0 / diff(&far, &half).0.max(1e-9);
                    if let Some(d) = &dir {
                        ppm(d, &format!("{name}_{px:02}px_tom{k}.ppm"), &far);
                    }
                    format!("{ratio:.2}")
                })
                .collect();
            if let Some(d) = &dir {
                ppm(d, &format!("{name}_{px:02}px_mao.ppm"), &hand);
            }
            println!("| {px} px | {} |", cells.join(" | "));
        }
    }
}
