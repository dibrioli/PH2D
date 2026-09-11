//! **O gate de presença descreve o produto, ou o meu modelo?**
//!
//! `a_soft_line_never_shows_the_background_through_the_fill_edge` exige que, sob uma
//! linha MACIA, nenhum pixel do anel `[eixo, silhueta]` seja fundo. Era ele que o termo
//! `w` da dilatação satisfazia.
//!
//! A pergunta que decide se ele fica: **o Draw:Filled passa nele?** O Draw:Filled não
//! dilata nada — a cor termina no eixo — e é o desenho que o Enio aprovou. Se ele
//! reprova neste gate, o gate está exigindo algo que a referência aprovada não faz, e
//! quem tem de mudar é o gate.
//!
//! Testar onde o fato pode ser CONTRADITO, e não onde é conveniente.
//!
//! `cargo test -p ph2d-flip-render --test probe_halo_under_soft_line -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_fill::{FillMode, FillParams, contour_widths, fill_at};
use ph2d_flip_render::{CameraRaw, FlipRenderer, pack_drawing};

const W: u32 = 320;
const H: u32 = 320;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("halo probe"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    Some((device, queue))
}

fn camera() -> CameraRaw {
    CameraRaw::new(
        [
            [2.0 / W as f32, 0.0, 0.0, 0.0],
            [0.0, -2.0 / H as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ],
        [W as f32, H as f32],
        1.0,
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
    fr.upload(device, queue, &camera(), &pack_drawing(drawing));
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
                        r: 0.10,
                        g: 0.10,
                        b: 0.11,
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

fn unit_circle(t: f32) -> (f32, f32) {
    let a = t * std::f32::consts::TAU;
    (a.cos(), a.sin())
}

fn points() -> Vec<Vec2> {
    let (cx, cy, r) = (160.0f32, 160.0, 110.0);
    let n = 200;
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let (c, s) = unit_circle(t);
            let h = ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
            let rr = r + h * 4.0;
            Vec2::new(cx + rr * c, cy + rr * s)
        })
        .collect()
}

const OCRE: Rgba = Rgba([0.78, 0.6, 0.35, 1.0]);

fn line_stroke(width_px: f32, hardness: f32) -> FlipStroke {
    let mut l = FlipStroke::new();
    for p in &points() {
        l.push_point(Point {
            pos: *p,
            width: width_px,
            opacity: 1.0,
            color: Rgba([0.95, 0.95, 0.96, 1.0]),
        });
    }
    l.closed = true;
    l.hardness = hardness;
    l
}

/// A referência aprovada: `fill` no PRÓPRIO traço, cor até o eixo, zero dilatação.
fn draw_filled(width_px: f32, hardness: f32) -> FlipDrawing {
    let mut l = line_stroke(width_px, hardness);
    l.fill = Some(Fill {
        color: OCRE,
        opacity: 1.0,
    });
    let mut d = FlipDrawing::new();
    d.strokes.push(l);
    d
}

/// O balde, com a dilatação parametrizada em `k_w` (1.0 = a lei antiga, 0.0 = a nova).
fn bucket(width_px: f32, hardness: f32, k_w: f32) -> FlipDrawing {
    let pts = points();
    let n = pts.len();
    let lines = vec![(pts.clone(), vec![width_px * 0.5; n], true)];
    let res = fill_at(
        &lines,
        Vec2::new(160.0, 160.0),
        FillParams {
            precision: 1.6,
            gap_reach: 0.0,
            grow: 0,
            trap_px: 0.0,
            mode: FillMode::Paint,
        },
    )
    .expect("preenche");
    let base = contour_widths(&lines, &res.outer);
    let mut f = FlipStroke::new();
    for (i, p) in res.outer.iter().enumerate() {
        f.push_point(Point {
            pos: *p,
            // `k_w` ressuscita o termo morto para o A/B — o produto usa só `base`.
            width: base[i] + k_w * width_px,
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
    d.strokes.push(line_stroke(width_px, hardness));
    d
}

/// A MESMA varredura do gate `a_soft_line_never_shows_the_background_through_the_fill_edge`.
fn background_leaks(px: &[u8], width_px: f32) -> usize {
    let dark = |x: i32, y: i32| -> bool {
        let i = ((y as u32 * W + x as u32) * 4) as usize;
        let (r, g, b) = (px[i] as i32, px[i + 1] as i32, px[i + 2] as i32);
        r < 90 && (r - b).abs() < 12 && (r - g).abs() < 12
    };
    let mut leaks = 0;
    for k in 0..256 {
        let t = k as f32 / 256.0;
        let (c, s) = unit_circle(t);
        let mut step = 110.0 + 3.0;
        while step <= 110.0 + width_px * 0.5 - 2.0 {
            let x = (160.0 + c * step).round() as i32;
            let y = (160.0 + s * step).round() as i32;
            if (0..W as i32).contains(&x) && (0..H as i32).contains(&y) && dark(x, y) {
                leaks += 1;
            }
            step += 1.0;
        }
    }
    leaks
}

#[test]
#[ignore = "sonda de diagnóstico; roda com --ignored"]
fn probe_does_draw_filled_pass_the_presence_gate() {
    let Some((device, queue)) = device() else {
        eprintln!("sem GPU — pulando");
        return;
    };
    println!(
        "\n{:>6} {:>9} {:>26} {:>22}",
        "w(px)", "dureza", "rota", "fundo sob a linha"
    );
    println!("{}", "-".repeat(70));
    for &hardness in &[0.35f32, 0.5] {
        for &width_px in &[8.0f32, 16.0, 32.0] {
            let cases: [(&str, FlipDrawing); 3] = [
                (
                    "Draw:Filled (A REFERENCIA)",
                    draw_filled(width_px, hardness),
                ),
                ("balde: lei NOVA (2s)", bucket(width_px, hardness, 0.0)),
                ("balde: lei ANTIGA (w+2s)", bucket(width_px, hardness, 1.0)),
            ];
            for (name, d) in cases {
                let px = render(&device, &queue, &d);
                let leaks = background_leaks(&px, width_px);
                println!("{width_px:>6.0} {hardness:>9.2} {name:>26} {leaks:>22}");
            }
        }
    }
    println!(
        "\nSe o Draw:Filled tambem vaza, o gate exige do balde algo que a REFERENCIA \
         APROVADA nao faz.\n"
    );
}
