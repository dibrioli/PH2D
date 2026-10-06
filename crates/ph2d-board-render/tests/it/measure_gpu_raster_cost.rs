//! ⏱ **A RÉGUA do desenho do quadro, lado PLACA** (`docs/MiroClone/02_plano.md` §2) — quanto custa
//! o Vello RASTERIZAR um quadro de N rectângulos a 1920×1080, pelo `VelloPass` do produto.
//!
//! A cena é a MESMA da régua de CPU (`measure_encode_cost::board_with`), encodada uma vez por N.
//! Régua do dono (05/10): variantes no MESMO processo, intercaladas em rodadas com ordem rodada;
//! vale o MÍNIMO (mediana ao lado). Um render de aquecimento por N fica de fora (compilação dos
//! pipelines e primeiro upload). Colunas:
//! - **placa**: `TIMESTAMP_QUERY` à volta das submissões do Vello (marcadores como os do
//!   `pass_profiler`). ⚠️ O Vello encoda na CPU ANTES de submeter, e um marcador submetido antes
//!   disso contaria a placa ociosa: por isso a fila leva antes um ENCHIMENTO (cópias de buffer) mais
//!   longo que esse encode, e o quadro só vale se o enchimento o cobriu (`descobertos` = 0).
//! - **Vello CPU**: a chamada `render_to_intermediate` (resolve + gravação + submit).
//! - **parede**: outro render, sem marcadores, da chamada até a fila esvaziar — o que o artista sente.
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it measure_gpu -- --ignored --nocapture
//! ```

use super::measure_encode_cost::{AREA, FRAMES, Mix, ROUNDS, SCENES, board_with, one_frame};
use ph2d_board_render::RenderCache;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;
use ph2d_vector::{Color, VectorScene};
use std::time::Instant;

/// O alvo em pixels — a `AREA` da régua de CPU (o teste confere).
const SIZE: (u32, u32) = (1920, 1080);
/// O enchimento: `FILL_COPIES` cópias de `FILL_BYTES` na placa antes do marcador de início.
const FILL_BYTES: u64 = 128 << 20;
const FILL_COPIES: usize = 16;

/// Carimbos: 0 = início do enchimento, 1 = início do Vello, 2 = fim do Vello.
struct Stamps {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    read: wgpu::Buffer,
    fill: [wgpu::Buffer; 2],
    period_ns: f64,
}

struct GpuFrame {
    vello_ms: f64,
    fill_ms: f64,
    cpu_ms: f64,
}

fn marker(enc: &mut wgpu::CommandEncoder, set: &wgpu::QuerySet, idx: u32) {
    drop(enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("board gpu ruler marker"),
        timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
            query_set: set,
            beginning_of_pass_write_index: Some(idx),
            end_of_pass_write_index: None,
        }),
    }));
}

impl Stamps {
    fn new(gpu: &GpuContext) -> Option<Self> {
        let d = &gpu.device;
        if !d.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let buf = |label, size, usage| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let fill_usage = wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;
        Some(Self {
            set: d.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("board gpu ruler"),
                ty: wgpu::QueryType::Timestamp,
                count: 3,
            }),
            resolve: buf(
                "board gpu ruler resolve",
                24,
                wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            ),
            read: buf(
                "board gpu ruler read",
                24,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            fill: [
                buf("board gpu ruler fill a", FILL_BYTES, fill_usage),
                buf("board gpu ruler fill b", FILL_BYTES, fill_usage),
            ],
            period_ns: f64::from(gpu.queue.get_timestamp_period()),
        })
    }

    fn frame(&self, gpu: &GpuContext, pass: &mut VelloPass, scene: &VectorScene) -> GpuFrame {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        marker(&mut enc, &self.set, 0);
        for c in 0..FILL_COPIES {
            let (a, b) = (&self.fill[c % 2], &self.fill[(c + 1) % 2]);
            enc.copy_buffer_to_buffer(a, 0, b, 0, FILL_BYTES);
        }
        marker(&mut enc, &self.set, 1);
        gpu.queue.submit([enc.finish()]);
        let t = Instant::now();
        pass.render_to_intermediate(gpu, scene.inner(), SIZE, Color::TRANSPARENT)
            .expect("o Vello desenha");
        let cpu_ms = t.elapsed().as_secs_f64() * 1e3;
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        marker(&mut enc, &self.set, 2);
        enc.resolve_query_set(&self.set, 0..3, &self.resolve, 0);
        enc.copy_buffer_to_buffer(&self.resolve, 0, &self.read, 0, 24);
        gpu.queue.submit([enc.finish()]);

        let slice = self.read.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map dos carimbos"));
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
        let ts: Vec<u64> = slice
            .get_mapped_range()
            .as_chunks::<8>()
            .0
            .iter()
            .map(|b| u64::from_le_bytes(*b))
            .collect();
        self.read.unmap();
        let ms = |a: u64, b: u64| b.wrapping_sub(a) as f64 * self.period_ns / 1e6;
        GpuFrame {
            vello_ms: ms(ts[1], ts[2]),
            fill_ms: ms(ts[0], ts[1]),
            cpu_ms,
        }
    }
}

/// Um render sem marcadores, da chamada até a fila esvaziar (ms de parede).
fn wall_frame(gpu: &GpuContext, pass: &mut VelloPass, scene: &VectorScene) -> f64 {
    let t = Instant::now();
    pass.render_to_intermediate(gpu, scene.inner(), SIZE, Color::TRANSPARENT)
        .expect("o Vello desenha");
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    t.elapsed().as_secs_f64() * 1e3
}

/// Área pintada de laranja, em pixels, somando a cobertura parcial (`r − b` entre o fundo, no canto,
/// e o laranja cheio) — controlo de que o Vello desenhou TUDO: os buffers internos dele têm tamanho
/// fixo, e um transbordo apaga trabalho em silêncio.
fn orange_area(rgba: &[u8]) -> f64 {
    let rb = |p: &[u8]| f64::from(p[0]) - f64::from(p[2]);
    let (bg, full) = (rb(&rgba[0..4]), 200.0 - 40.0);
    rgba.as_chunks::<4>()
        .0
        .iter()
        .map(|p| ((rb(p) - bg) / (full - bg)).clamp(0.0, 1.0))
        .sum()
}

fn min_med(v: &[f64]) -> (f64, f64) {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

#[test]
#[ignore = "régua: corre-se à mão em --release com a placa (ver o cabeçalho)"]
fn measure_gpu_raster_cost() {
    assert_eq!([f64::from(SIZE.0), f64::from(SIZE.1)], [AREA[2], AREA[3]]);
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        println!("sem adaptador: régua da placa não corre");
        return;
    };
    let info = gpu.adapter.get_info();
    let stamps = Stamps::new(&gpu);
    println!(
        "adaptador: {} ({:?}) · TIMESTAMP_QUERY: {}",
        info.name,
        info.backend,
        if stamps.is_some() { "sim" } else { "NÃO" }
    );
    let mut pass = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, SIZE).expect("VelloPass");

    let mut ts = TextSystem::without_system_fonts();
    let scenes: Vec<VectorScene> = SCENES
        .iter()
        .map(|&sc| {
            let set = board_with(sc);
            let mut scene = VectorScene::new();
            one_frame(&set, &mut scene, &mut ts, &mut RenderCache::default());
            scene
        })
        .collect();

    // Aquecimento (fora da régua) + controlo de cobertura.
    for (k, (n, mix)) in SCENES.iter().enumerate() {
        let px = pass
            .render_and_readback(&gpu, scenes[k].inner(), SIZE)
            .expect("readback");
        if *mix != Mix::Rects {
            continue;
        }
        let side = (*n as f64).sqrt().ceil();
        let rect_px = 24.0 * 1000.0 / (side * 30.0);
        println!(
            "N={n}: área laranja {:.0} px · esperado {:.0}",
            orange_area(&px),
            *n as f64 * rect_px * rect_px
        );
    }

    let k_n = SCENES.len();
    let (mut vello, mut cpu, mut wall) = (vec![vec![]; k_n], vec![vec![]; k_n], vec![vec![]; k_n]);
    let mut best = vec![f64::INFINITY; k_n];
    let (mut uncovered, mut fill_min) = (vec![0usize; k_n], f64::INFINITY);
    for round in 0..ROUNDS {
        for k in 0..k_n {
            let i = (k + round) % k_n; // ordem rodada
            let (mut g, mut c, mut w) = (0.0, 0.0, 0.0);
            for _ in 0..FRAMES {
                if let Some(s) = &stamps {
                    let f = s.frame(&gpu, &mut pass, &scenes[i]);
                    if f.fill_ms < f.cpu_ms {
                        uncovered[i] += 1;
                    }
                    fill_min = fill_min.min(f.fill_ms);
                    best[i] = best[i].min(f.vello_ms);
                    g += f.vello_ms;
                    c += f.cpu_ms;
                }
                w += wall_frame(&gpu, &mut pass, &scenes[i]);
            }
            vello[i].push(g / FRAMES as f64);
            cpu[i].push(c / FRAMES as f64);
            wall[i].push(w / FRAMES as f64);
        }
    }

    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", load.trim());
    println!("enchimento mais curto: {fill_min:.3} ms");
    println!(
        "| N | cena | placa mín ms/quadro | placa mediana | placa melhor quadro | Vello CPU mín | Vello CPU mediana | parede mín | parede mediana | descobertos |"
    );
    for (k, (n, mix)) in SCENES.iter().enumerate() {
        let (wm, wd) = min_med(&wall[k]);
        if stamps.is_some() {
            let (gm, gd) = min_med(&vello[k]);
            let (cm, cd) = min_med(&cpu[k]);
            println!(
                "| {n} | {mix:?} | {gm:.3} | {gd:.3} | {:.3} | {cm:.3} | {cd:.3} | {wm:.3} | {wd:.3} | {} |",
                best[k], uncovered[k]
            );
        } else {
            println!("| {n} | {mix:?} | n/d | n/d | n/d | n/d | n/d | {wm:.3} | {wd:.3} | n/d |");
        }
    }
}
