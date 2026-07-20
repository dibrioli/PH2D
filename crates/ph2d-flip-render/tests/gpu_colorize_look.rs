//! **Sonda VISUAL do zíper do Colorize** (6º smoke, 2026-07-20: *"ainda não perfeito"* —
//! dentes finos e REGULARES alternando as duas cores em cima do divisor; e, na cena com a
//! cor vazando pelo vão, mordidas em zigue-zague na própria linha).
//!
//! A sonda de GEOMETRIA (`probe_the_zipper_on_the_divider`) não reproduz o defeito contra o
//! eixo CRU — mas a borda cravada É o eixo cru: se o zíper existe, ele é contra o que se
//! VÊ (a linha renderizada / a outra cor). O oráculo aqui é o PIXEL: a MESMA materialização
//! do shell (colorize → contour_widths → fill atrás, linha na frente), rasterizada com o
//! renderer real, numa arte de MÃO (pontos densos na taxa do ponteiro + ruído) — a arte
//! sintética de 41 pontos limpos não contém o fenômeno.
//!
//! `cargo test -p ph2d-flip-render --test gpu_colorize_look -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_colorize::{Scribble, colorize};
use ph2d_flip_fill::contour_widths;
use ph2d_flip_render::{CameraRaw, FlipRenderer, pack_drawing};

const W: u32 = 640;
const H: u32 = 480;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("colorize look"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    Some((device, queue))
}

/// Câmera de pixel (mundo = px do alvo, y para baixo), como a `pixel_camera_zoom` do
/// `gpu_fill_fit` — `px_per_world = 1`, e a precision default 1,6 do produto entra direto.
fn pixel_camera() -> CameraRaw {
    let (sx, sy) = (2.0 / W as f32, -2.0 / H as f32);
    CameraRaw::new(
        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ],
        [W as f32, H as f32],
        1.0,
    )
}

fn render(device: &wgpu::Device, queue: &wgpu::Queue, drawing: &FlipDrawing) -> Vec<u8> {
    let cam = pixel_camera();
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
    fr.upload(device, queue, &cam, &pack_drawing(drawing));

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
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.18,
                        g: 0.18,
                        b: 0.19,
                        a: 1.0,
                    }),
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

/// Um traço de MÃO: amostrado na taxa do ponteiro (~2,5 px) com ruído determinístico de
/// ±1,3 px — o que um arrasto real produz (quantização de tela + micro-jitter).
fn hand_stroke(pts: &[Vec2], seed: usize) -> Vec<Vec2> {
    let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let (step, jitter) = (2.5f32, 2.6f32);
    let mut out = Vec::new();
    let mut k = 0usize;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let d = Vec2::new(b.x - a.x, b.y - a.y);
        let len = (d.x * d.x + d.y * d.y).sqrt();
        let n = (len / step).ceil().max(1.0) as usize;
        for s in 0..n {
            let t = s as f32 / n as f32;
            let p = Vec2::new(a.x + d.x * t, a.y + d.y * t);
            out.push(Vec2::new(
                p.x + h(k + seed) * jitter,
                p.y + h(k + seed + 91) * jitter,
            ));
            k += 1;
        }
    }
    out.push(*pts.last().expect("polyline"));
    out
}

/// A arte das fotos do 6º smoke: caixa com divisor fora-do-centro + vão no meio, tudo
/// desenhado "à mão" (denso + ruidoso), linha de `width_px` com pincel macio.
fn art(width_px: f32) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    let mut strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = Vec::new();
    for (a, b, s) in [
        (Vec2::new(40.0, 60.0), Vec2::new(600.0, 60.0), 0usize),
        (Vec2::new(600.0, 60.0), Vec2::new(600.0, 420.0), 977),
        (Vec2::new(600.0, 420.0), Vec2::new(40.0, 420.0), 1954),
        (Vec2::new(40.0, 420.0), Vec2::new(40.0, 60.0), 2931),
        // divisor fora-do-centro em x=400, vão y∈[210, 270]
        (Vec2::new(400.0, 60.0), Vec2::new(400.0, 210.0), 3908),
        (Vec2::new(400.0, 270.0), Vec2::new(400.0, 420.0), 4885),
    ] {
        let pts = hand_stroke(&[a, b], s);
        let n = pts.len();
        strokes.push((pts, vec![width_px * 0.5; n], false));
    }
    strokes
}

/// A materialização do shell (espelho de `flip_colorize_apply`/`fill_stroke`): fills
/// ATRÁS (com a dilatação da lei do produto), linhas NA FRENTE.
fn materialize(
    lines: &[(Vec<Vec2>, Vec<f32>, bool)],
    scribbles: &[Scribble],
    palette: &[Rgba],
    width_px: f32,
) -> FlipDrawing {
    let regions = colorize(lines, scribbles, 1.6, 0.0);
    eprintln!("  colorize: {} regioes", regions.len());
    let mut d = FlipDrawing::new();
    for r in &regions {
        let color = palette[r.label as usize];
        let widths = contour_widths(lines, &r.fill.outer);
        let mut f = FlipStroke::new();
        for (i, p) in r.fill.outer.iter().enumerate() {
            f.push_point(Point {
                pos: *p,
                width: widths.get(i).copied().unwrap_or(0.0),
                opacity: 1.0,
                color,
            });
        }
        f.closed = true;
        f.hide_stroke = true;
        f.holes = r.fill.holes.clone();
        f.fill = Some(Fill {
            color,
            opacity: 1.0,
        });
        d.strokes.push(f);
    }
    for (pts, _, _) in lines {
        let mut line = FlipStroke::new();
        for p in pts {
            line.push_point(Point {
                pos: *p,
                width: width_px,
                opacity: 1.0,
                color: Rgba::new(0.92, 0.92, 0.95, 1.0),
            });
        }
        line.hardness = 0.35;
        d.strokes.push(line);
    }
    d
}

/// Grava as duas cenas das fotos num PNG para OLHAR (não afirma nada — é a régua visual).
#[test]
#[ignore = "diagnostico visual: grava PNG em /tmp"]
fn look_at_the_zipper() {
    let Some((device, queue)) = device() else {
        eprintln!("sem GPU — pulando");
        return;
    };
    let width_px = 7.0;
    let lines = art(width_px);
    let seg_ = |a: Vec2, b: Vec2, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let red = Rgba::new(0.86, 0.27, 0.27, 1.0);
    let blue = Rgba::new(0.27, 0.47, 0.86, 1.0);

    // Cena 1 (foto 1): vermelho à esquerda, azul à direita.
    let two = vec![
        Scribble {
            label: 0,
            points: seg_(Vec2::new(200.0, 150.0), Vec2::new(200.0, 330.0), 8),
            width: 12.0,
        },
        Scribble {
            label: 1,
            points: seg_(Vec2::new(500.0, 150.0), Vec2::new(500.0, 330.0), 8),
            width: 12.0,
        },
    ];
    // A divergência MÚTUA das duas bordas na banda do divisor (x∈[385,415], y∈[70,200]):
    // amostra densa da borda azul → distância à polilinha da borda vermelha. Se as duas
    // fossem a MESMA curva, isto seria ~0; o zíper é esta função alternando.
    let regions = colorize(&lines, &two, 1.6, 0.0);
    let rings_of = |label: u16| -> Vec<Vec<Vec2>> {
        regions
            .iter()
            .filter(|r| r.label == label)
            .flat_map(|r| std::iter::once(r.fill.outer.clone()).chain(r.fill.holes.clone()))
            .collect()
    };
    let in_band = |p: Vec2| (385.0..415.0).contains(&p.x) && (70.0..200.0).contains(&p.y);
    let band_pts = |rings: &[Vec<Vec2>]| -> Vec<Vec2> {
        let mut out = Vec::new();
        for ring in rings {
            let n = ring.len();
            for i in 0..n {
                let (a, b) = (ring[i], ring[(i + 1) % n]);
                if !in_band(a) && !in_band(b) {
                    continue;
                }
                let d = Vec2::new(b.x - a.x, b.y - a.y);
                let len = (d.x * d.x + d.y * d.y).sqrt();
                let steps = (len / 0.5).ceil().max(1.0) as usize;
                for s in 0..steps {
                    let t = s as f32 / steps as f32;
                    let p = Vec2::new(a.x + d.x * t, a.y + d.y * t);
                    if in_band(p) {
                        out.push(p);
                    }
                }
            }
        }
        out
    };
    let red_rings = rings_of(0);
    let blue_pts = band_pts(&rings_of(1));
    let dist_to = |p: Vec2, rings: &[Vec<Vec2>]| -> f32 {
        let mut best = f32::MAX;
        for ring in rings {
            let n = ring.len();
            for i in 0..n {
                let (a, b) = (ring[i], ring[(i + 1) % n]);
                let ab = Vec2::new(b.x - a.x, b.y - a.y);
                let l2 = ab.x * ab.x + ab.y * ab.y;
                let t = if l2 <= 0.0 {
                    0.0
                } else {
                    (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
                };
                let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
                best = best.min((dx * dx + dy * dy).sqrt());
            }
        }
        best
    };
    let devs: Vec<(Vec2, f32)> = blue_pts
        .iter()
        .map(|&p| (p, dist_to(p, &red_rings)))
        .collect();
    let stats = |label: &str, f: &dyn Fn(Vec2) -> bool| {
        let sel: Vec<f32> = devs.iter().filter(|(p, _)| f(*p)).map(|(_, d)| *d).collect();
        let worst = sel.iter().fold(0.0f32, |m, &d| m.max(d));
        let mean = sel.iter().sum::<f32>() / sel.len().max(1) as f32;
        eprintln!(
            "  divergencia mutua [{label}]: {} amostras, media {mean:.3}px, pior {worst:.3}px",
            sel.len()
        );
    };
    stats("banda toda", &|_| true);
    stats("colado na linha x∈[397,403] y∈[80,190]", &|p: Vec2| {
        (397.0..403.0).contains(&p.x) && (80.0..190.0).contains(&p.y)
    });
    // As 8 piores amostras, para localizar.
    let mut sorted = devs.clone();
    sorted.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (p, d) in sorted.iter().take(8) {
        eprintln!("    pior: ({:.1}, {:.1}) dev {d:.2}px", p.x, p.y);
    }
    // A cadeia CRUA do anel azul em y∈[100,140] vs os vértices do EIXO no mesmo trecho.
    let mut blue_v: Vec<Vec2> = rings_of(1)
        .iter()
        .flatten()
        .copied()
        .filter(|p| (390.0..412.0).contains(&p.x) && (100.0..140.0).contains(&p.y))
        .collect();
    blue_v.sort_by(|a, b| a.y.total_cmp(&b.y));
    eprintln!("  anel AZUL y∈[100,140]: {:?}", blue_v.iter().map(|p| (p.y, p.x)).collect::<Vec<_>>());
    let mut red_v: Vec<Vec2> = rings_of(0)
        .iter()
        .flatten()
        .copied()
        .filter(|p| (390.0..412.0).contains(&p.x) && (100.0..140.0).contains(&p.y))
        .collect();
    red_v.sort_by(|a, b| a.y.total_cmp(&b.y));
    eprintln!("  anel VERM y∈[100,140]: {:?}", red_v.iter().map(|p| (p.y, p.x)).collect::<Vec<_>>());
    let axis_v: Vec<(f32, f32)> = lines[4]
        .0
        .iter()
        .filter(|p| (100.0..140.0).contains(&p.y))
        .map(|p| (p.y, p.x))
        .collect();
    eprintln!("  eixo (traço 4) y∈[100,140]: {axis_v:?}");
    // A zona do laço escuro: o anel azul em torno da ponta do toco (y∈[190,225]).
    let mut tipzone: Vec<(f32, f32)> = Vec::new();
    for ring in rings_of(1) {
        let n = ring.len();
        for i in 0..n {
            let p = ring[i];
            if (392.0..410.0).contains(&p.x) && (188.0..228.0).contains(&p.y) {
                tipzone.push((p.y, p.x));
            }
        }
    }
    eprintln!("  anel AZUL na ponta (ordem do anel): {tipzone:?}");

    let d = materialize(&lines, &two, &[red, blue], width_px);
    let px = render(&device, &queue, &d);
    image::save_buffer(
        "/tmp/colorize_zipper_two_sides.png",
        &px,
        W,
        H,
        image::ColorType::Rgba8,
    )
    .expect("png");
    eprintln!("  -> /tmp/colorize_zipper_two_sides.png");

    // Cena 2 (foto 2): SÓ vermelho — vaza pelo vão, a fenda abraça os tocos do divisor.
    let one = vec![Scribble {
        label: 0,
        points: seg_(Vec2::new(200.0, 150.0), Vec2::new(200.0, 330.0), 8),
        width: 12.0,
    }];
    let d = materialize(&lines, &one, &[red], width_px);
    let px = render(&device, &queue, &d);
    image::save_buffer(
        "/tmp/colorize_zipper_flood.png",
        &px,
        W,
        H,
        image::ColorType::Rgba8,
    )
    .expect("png");
    eprintln!("  -> /tmp/colorize_zipper_flood.png");
}
