//! ⏱ **A RÉGUA do nível de detalhe** (W5) — o mesmo quadro desenhado com cada variante de
//! [`Detail`] no MESMO processo, intercaladas em rodadas de ordem rodada, vale o MÍNIMO (mediana ao
//! lado): o encode na CPU (o `paint`) e a parede da placa (o `VelloPass` do produto).
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it measure_detail -- --ignored --nocapture
//! ```

use super::measure_encode_cost::{AREA, FRAMES, ROUNDS, notes, one_frame, routes_of};
use super::measure_gpu_raster_cost::{SIZE, min_med, wall_frame};
use ph2d_board_model::{BoardOp, BoardSet, Camera, Element, Frame, Rgba, Shape, ShapeType, Style};
use ph2d_board_render::{Detail, RenderCache};
use ph2d_board_route::RouteCache;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::time::Instant;

/// Lado de um frame da cena de frames: 10×10 formas de 24 com passo 30, e margem.
const FRAME_SIDE: f64 = 320.0;
const FRAME_STEP: f64 = 360.0;

/// `nf` frames numa grelha, cada um com 100 formas da mistura da W1 (contorno, palavra), a vista
/// a enquadrar tudo.
fn framed(nf: usize) -> BoardSet {
    let mut set = BoardSet::default();
    let id = set.create(format!("{nf}"));
    let b = set.get_mut(id).unwrap();
    let side = (nf as f64).sqrt().ceil();
    let ink = Rgba([30, 30, 30, 255]);
    for k in 0..nf {
        let (fr, fc) = ((k as f64 / side).floor(), k as f64 % side);
        let (fx, fy) = (fc * FRAME_STEP, fr * FRAME_STEP);
        let frame = Frame {
            name: format!("frame {k}"),
            style: Style::new(Some(Rgba([255, 255, 255, 255])), None, ink),
        };
        let f = Element::new_frame(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            frame,
            [fx, fy, FRAME_SIDE, FRAME_SIDE],
        );
        let fid = f.id;
        BoardOp::Put(f).apply(&mut b.doc);
        for i in 0..100 {
            let (r, c) = ((i / 10) as f64, (i % 10) as f64);
            let mut style = Style::new(Some(Rgba([200, 120, 40, 255])), Some(ink), ink);
            style.stroke_width = 1.0;
            style.font_size = 6.0;
            let shape = Shape {
                kind: [ShapeType::Rectangle, ShapeType::Ellipse, ShapeType::Diamond][i % 3],
                style,
                text: format!("ideia {i}").into(),
            };
            let bx = [fx + 10.0 + c * 30.0, fy + 10.0 + r * 30.0, 24.0, 24.0];
            let mut el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), shape, bx);
            el.frame = Some(fid);
            BoardOp::Put(el).apply(&mut b.doc);
        }
    }
    let w = side * FRAME_STEP;
    b.camera = Camera {
        center_x: w / 2.0,
        center_y: w / 2.0,
        zoom: 1000.0 / w,
    };
    set
}

fn detail(frame_px: f64, notes: bool) -> Detail {
    Detail {
        frame_title_only_px: frame_px,
        note_blocks: notes,
    }
}

#[test]
#[ignore = "régua: corre-se à mão em --release com a placa (ver o cabeçalho)"]
fn measure_detail_variants() {
    let boards: Vec<(&str, BoardSet)> = vec![
        ("10 mil notas de longe", notes(10_000, false)),
        ("100 mil notas de longe", notes(100_000, false)),
        ("100 frames × 100 formas", framed(100)),
        ("1 000 frames × 100 formas", framed(1_000)),
    ];
    let variants: Vec<(usize, &str, Detail)> = vec![
        (0, "notas com sombra", detail(0.0, false)),
        (0, "notas em bloco", detail(0.0, true)),
        (1, "notas com sombra", detail(0.0, false)),
        (1, "notas em bloco", detail(0.0, true)),
        (2, "frame: sempre o conteúdo", detail(0.0, true)),
        (2, "frame: só título < 48 px", detail(48.0, true)),
        (2, "frame: só título < 96 px", detail(96.0, true)),
        (2, "frame: só título < 192 px", detail(192.0, true)),
        (3, "frame: sempre o conteúdo", detail(0.0, true)),
        (3, "frame: só título < 48 px", detail(48.0, true)),
        (3, "frame: só título < 96 px", detail(96.0, true)),
        (3, "frame: só título < 192 px", detail(192.0, true)),
    ];
    let routes: Vec<RouteCache> = boards.iter().map(|(_, b)| routes_of(b)).collect();
    let gpu = GpuContext::new(GpuContext::default_instance(), None).ok();
    let mut pass = gpu
        .as_ref()
        .map(|g| VelloPass::new(g, wgpu::TextureFormat::Rgba8Unorm, SIZE).expect("VelloPass"));
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    let mut caches: Vec<RenderCache> = variants
        .iter()
        .map(|(b, _, d)| {
            let mut c = RenderCache::default();
            c.detail = *d;
            one_frame(&boards[*b].1, &mut scene, &mut ts, &mut c, &routes[*b]);
            c
        })
        .collect();
    let n = variants.len();
    let (mut cpu, mut wall) = (vec![vec![]; n], vec![vec![]; n]);
    for round in 0..ROUNDS {
        for k in 0..n {
            let i = (k + round) % n;
            let b = variants[i].0;
            let t = Instant::now();
            for _ in 0..FRAMES {
                one_frame(
                    &boards[b].1,
                    &mut scene,
                    &mut ts,
                    &mut caches[i],
                    &routes[b],
                );
            }
            cpu[i].push(t.elapsed().as_secs_f64() * 1e3 / FRAMES as f64);
            if let (Some(g), Some(p)) = (gpu.as_ref(), pass.as_mut()) {
                let w: f64 = (0..FRAMES).map(|_| wall_frame(g, p, &scene)).sum();
                wall[i].push(w / FRAMES as f64);
            }
        }
    }
    let _ = AREA;
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", load.trim());
    println!("| quadro | variante | CPU mín ms | CPU mediana | placa (parede) mín | mediana |");
    for (k, (b, name, _)) in variants.iter().enumerate() {
        let (cm, cd) = min_med(&cpu[k]);
        let (wm, wd) = if wall[k].is_empty() {
            (f64::NAN, f64::NAN)
        } else {
            min_med(&wall[k])
        };
        println!(
            "| {} | {name} | {cm:.3} | {cd:.3} | {wm:.3} | {wd:.3} |",
            boards[*b].0
        );
    }
}
