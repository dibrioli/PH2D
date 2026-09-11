//! **A referência que o Enio nomeou.**
//!
//! > *"Diferente do Draw:Filled que faz exatamente como eu estou dizendo."*
//!
//! O Draw:Filled põe `fill` no PRÓPRIO traço: a cor é a triangulação dos pontos da
//! linha, ou seja ela termina **no eixo**, e a metade externa do traço composita sobre o
//! FUNDO. Zero dilatação. E é o desenho aprovado.
//!
//! O balde, na rota do contorno, dilata a cor por `w` (a espessura da linha) — a cor
//! atravessa a metade externa e vai até o raio GEOMÉTRICO. As duas rotas respondem
//! diferente à mesma pergunta, e o usuário já disse qual está certa.
//!
//! Esta sonda mede a distância entre as duas, por fator de dilatação. O oráculo é a
//! **aparência contra a referência declarada**, nunca uma regra que eu invente.
//!
//! `cargo test -p ph2d-flip-render --test probe_bucket_vs_draw_filled -- --ignored --nocapture`

#![allow(clippy::excessive_precision)]

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_fill::{FillMode, FillParams, fill_at, nearest_on_axis};
use ph2d_flip_render::{CameraRaw, FlipRenderer, pack_drawing};

const W: u32 = 320;
const H: u32 = 320;
const PPW: f32 = 100.0;
const CX: f32 = 160.0;
const CY: f32 = 160.0;
const R_AXIS: f32 = 110.0;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("bucket vs draw:filled"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    Some((device, queue))
}

fn world_camera() -> CameraRaw {
    let c = Vec2::new(CX / PPW, CY / PPW);
    let (sx, sy) = (2.0 * PPW / W as f32, -2.0 * PPW / H as f32);
    CameraRaw::new(
        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-c.x * sx, -c.y * sy, 0.0, 1.0],
        ],
        [W as f32, H as f32],
        PPW,
    )
}

fn render(device: &wgpu::Device, queue: &wgpu::Queue, drawing: &FlipDrawing) -> Vec<u8> {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("t"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let mut fr = FlipRenderer::new(device, wgpu::TextureFormat::Rgba8Unorm);
    fr.upload(device, queue, &world_camera(), &pack_drawing(drawing));

    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("d"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let dview = depth.create_view(&wgpu::TextureViewDescriptor::default());

    let bpr = W * 4;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rb"),
        size: u64::from(bpr * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("p"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &dview,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        fr.draw(&mut pass);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let out = slice.get_mapped_range().to_vec();
    buf.unmap();
    out
}

fn axis_points(n: usize, tremor: bool) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            let h = if tremor {
                ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5
            } else {
                0.0
            };
            let rr = (R_AXIS + h * 4.0) / PPW;
            Vec2::new(CX / PPW + rr * a.cos(), CY / PPW + rr * a.sin())
        })
        .collect()
}

const OCRE: Rgba = Rgba([0.78, 0.60, 0.35, 1.0]);
const INK: Rgba = Rgba([0.10, 0.10, 0.12, 1.0]);

fn line_stroke(pts: &[Vec2], width_px: f32, hardness: f32) -> FlipStroke {
    let mut l = FlipStroke::new();
    for p in pts {
        l.push_point(Point {
            pos: *p,
            width: width_px / PPW,
            opacity: 1.0,
            color: INK,
        });
    }
    l.closed = true;
    l.hardness = hardness;
    l
}

/// **A REFERÊNCIA**: o Draw:Filled. Um traço só, com `fill` — a cor vai até o EIXO.
fn draw_filled(pts: &[Vec2], width_px: f32, hardness: f32) -> FlipDrawing {
    let mut l = line_stroke(pts, width_px, hardness);
    l.fill = Some(Fill {
        color: OCRE,
        opacity: 1.0,
    });
    let mut d = FlipDrawing::new();
    d.strokes.push(l);
    d
}

/// O balde pela rota do contorno, com a dilatação **parametrizada**:
/// `width = w·k_w + 2s·k_s`.
fn bucket(pts: &[Vec2], width_px: f32, hardness: f32, k_w: f32, k_s: f32) -> FlipDrawing {
    let n = pts.len();
    let lines = vec![(pts.to_vec(), vec![width_px * 0.5 / PPW; n], true)];
    let res = fill_at(
        &lines,
        Vec2::new(CX / PPW, CY / PPW),
        FillParams {
            precision: 1.6 * PPW,
            gap_reach: 0.0,
            grow: 0,
            trap_px: 0.0,
            mode: FillMode::Paint,
        },
    )
    .expect("preenche");

    // A lei, decomposta nos dois termos, para varrer cada um. O termo `2s` é reproduzido
    // aqui a partir da MESMA porta do produto (`nearest_on_axis`) — sem alisamento, que é
    // sub-pixel neste fixture liso.
    let widths: Vec<f32> = res
        .outer
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let np = res.outer.len();
            let (a, b) = (res.outer[(i + np - 1) % np], res.outer[(i + 1) % np]);
            let t = Vec2::new(b.x - a.x, b.y - a.y);
            let len = (t.x * t.x + t.y * t.y).sqrt().max(1e-9);
            // Normal externa (área positiva neste anel; o produto decide pelo sinal da área).
            let orient = if ph2d_flip_fill::signed_area(&res.outer) >= 0.0 {
                1.0
            } else {
                -1.0
            };
            let nrm = Vec2::new(orient * t.y / len, -orient * t.x / len);
            match nearest_on_axis(&lines, *p) {
                Some((w, q)) => {
                    let s = (q.x - p.x) * nrm.x + (q.y - p.y) * nrm.y;
                    (w * k_w + 2.0 * s * k_s).max(0.0)
                }
                None => 0.0,
            }
        })
        .collect();

    let mut f = FlipStroke::new();
    for (i, p) in res.outer.iter().enumerate() {
        f.push_point(Point {
            pos: *p,
            width: widths[i],
            opacity: 1.0,
            color: OCRE,
        });
    }
    f.closed = true;
    f.hide_stroke = true;
    f.holes = res.holes.clone();
    f.fill = Some(Fill {
        color: OCRE,
        opacity: 1.0,
    });

    let mut d = FlipDrawing::new();
    d.strokes.push(f);
    d.strokes.push(line_stroke(pts, width_px, hardness));
    d
}

/// Delta contra a referência: pior canal e quantos pixels diferem de mais de 8/255.
fn diff(a: &[u8], b: &[u8]) -> (u8, usize) {
    let mut worst = 0u8;
    let mut n = 0usize;
    for i in (0..a.len()).step_by(4) {
        let mut d = 0u8;
        for c in 0..3 {
            d = d.max(a[i + c].abs_diff(b[i + c]));
        }
        worst = worst.max(d);
        if d > 8 {
            n += 1;
        }
    }
    (worst, n)
}

#[test]
#[ignore = "sonda de diagnóstico; roda com --ignored"]
fn probe_bucket_against_draw_filled() {
    let Some((dev, q)) = device() else {
        eprintln!("sem GPU — pulando");
        return;
    };

    println!(
        "\n{:>6} {:>9} {:>7} {:>28} {:>10} {:>10}",
        "w(px)", "hardness", "tremor", "lei da dilatação", "pior Δ", "px≠"
    );
    println!("{}", "-".repeat(80));

    for &tremor in &[false, true] {
        for &width_px in &[8.0f32, 16.0, 32.0] {
            for &hardness in &[1.0f32, 0.8, 0.5] {
                let pts = axis_points(200, tremor);
                let reference = render(&dev, &q, &draw_filled(&pts, width_px, hardness));
                for (name, k_w, k_s) in [
                    ("w + 2s  (HOJE)", 1.0f32, 1.0f32),
                    ("w  (sem compensação)", 1.0, 0.0),
                    ("2s  (só a compensação)", 0.0, 1.0),
                    ("zero", 0.0, 0.0),
                ] {
                    let got = render(&dev, &q, &bucket(&pts, width_px, hardness, k_w, k_s));
                    let (worst, n) = diff(&reference, &got);
                    println!(
                        "{width_px:>6.0} {hardness:>9.2} {:>7} {name:>28} {worst:>10} {n:>10}",
                        if tremor { "sim" } else { "não" }
                    );
                }
            }
        }
    }
    println!(
        "\nA referência é o Draw:Filled (cor até o EIXO, zero dilatação) — o desenho que \
         o Enio aprovou.\nQuanto MENOR o delta, mais a rota do contorno se parece com ela.\n"
    );
}
