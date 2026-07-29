//! **A SONDA QUE COMPARA O FLIP COM O PAINTER, NA MESMA FIGURA.**
//!
//! O Enio (2026-07-28, 4ª rodada, com foto anotada): *"Tudo que quero é que tenha o aspecto do
//! traço do nosso próprio módulo painter digital"* — setas vermelhas sobre **cunhas escuras**
//! mordendo a tinta nas QUINAS de um rabisco em estrela que cruza a si mesmo.
//!
//! ⚠️ **Toda sonda anterior comparou o Flip com a UNIÃO** (`expected_union_alpha`), e a união é
//! exatamente a coisa sob suspeita. Esta compara com o **DEPÓSITO DO PAINTER**: dabs a cada
//! `spacing × diâmetro` de arco, compostos por `over`, com o `Falloff` REAL do
//! `ph2d_painter_brush`. Um oráculo que fala a lei que o Enio aponta na tela.
//!
//! Roda com:
//! ```text
//! cargo test -p ph2d-flip-render --release --test painter_look -- --ignored --nocapture
//! ```

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_render::{CameraRaw, FlipRenderer, pack_drawing};

const W: u32 = 64;
const H: u32 = 64;

/// O `spacing` default do Painter (`spec_default.rs:29`), em FRAÇÃO DE DIÂMETRO.
const PAINTER_SPACING: f32 = 0.10;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ph2d-flip painter-look device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");
    Some((device, queue))
}

fn pixel_camera_sized(w: u32, h: u32) -> CameraRaw {
    let sx = 2.0 / w as f32;
    let sy = -2.0 / h as f32;
    CameraRaw::new(
        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ],
        [w as f32, h as f32],
        1.0,
    )
}

/// Rasteriza no MESMO harness do `gpu_render.rs` — copiado verbatim porque um 2o caminho
/// de render mediria outro produto.
fn render(device: &wgpu::Device, queue: &wgpu::Queue, drawing: &FlipDrawing) -> Vec<u8> {
    render_sized(device, queue, drawing, W, H)
}

/// ⚠️ **UM caminho de render, com o tamanho como PARÂMETRO.** A sonda de imagem precisa de escala
/// real (768²) e os gates medem em 64²; um 2º harness mediria outro produto.
fn render_sized(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    drawing: &FlipDrawing,
    w: u32,
    h: u32,
) -> Vec<u8> {
    let camera = pixel_camera_sized(w, h);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-flip test target"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut fr = FlipRenderer::new(device, format);
    fr.upload(device, queue, &camera, &pack_drawing(drawing));
    fr.ensure_depth(device, (w, h));

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-flip test pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: fr.depth_view().map(|v| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: v,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        fr.draw(&mut pass);
    }
    queue.submit([encoder.finish()]);
    readback(device, queue, &texture, w, h)
}

fn readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    w: u32,
    h: u32,
) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-flip readback"),
        size: (padded as u64) * (h as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();

    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded as usize) * (h as usize));
    for row in 0..h as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

fn alpha_at(px: &[u8], x: u32, y: u32) -> u8 {
    px[((y * W + x) * 4 + 3) as usize]
}

// ---------------------------------------------------------------------------
// A FIGURA — a estrela de UM traço, que é a foto do Enio: quinas afiadas E
// auto-cruzamento, no mesmo desenho.
// ---------------------------------------------------------------------------

/// A estrela de 5 pontas desenhada **sem levantar a caneta** (o gesto clássico) — quina de 36° em
/// cada ponta e cinco auto-cruzamentos no miolo. É a topologia exata da foto.
fn star_path(r: f32) -> (Vec<(f32, f32)>, f32) {
    let (cx, cy, outer) = (32.0_f32, 32.0_f32, 26.0_f32);
    let mut corners = Vec::new();
    for k in 0..5 {
        // Passo de 2/5 de volta = a estrela de um traço só.
        let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
        corners.push((cx + outer * a.cos(), cy + outer * a.sin()));
    }
    corners.push(corners[0]);
    // Densificado: a fronteira de passagem é ESPACIAL, e só é honesta numa polilinha reamostrada
    // (dois segmentos enormes têm a distância medida pelos extremos).
    let mut pts = vec![corners[0]];
    for w in corners.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let n = (len / 3.0).ceil().max(1.0) as usize;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            pts.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    (pts, r)
}

fn flip_drawing(pts: &[(f32, f32)], r: f32, hardness: f32) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    let mut st = FlipStroke::new();
    for &(x, y) in pts {
        st.push_point(Point {
            pos: Vec2::new(x, y),
            width: r * 2.0,
            opacity: 1.0,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
        });
    }
    st.hardness = hardness;
    d.strokes.push(st);
    d
}

/// A lei de perfil do Painter (`BrushSpec::falloff_weight` + `Falloff::Smooth`), chamada na
/// função REAL — este arquivo não guarda cópia dela.
fn painter_weight(dn: f32, hardness: f32) -> f32 {
    let h = hardness.clamp(0.0, 1.0);
    if h >= 1.0 {
        return f32::from(dn < 1.0);
    }
    let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
    ph2d_painter_brush::Falloff::Smooth.weight(remapped)
}

/// **O DEPÓSITO DO PAINTER** — dabs a cada `spacing × diâmetro` de arco, compostos por `over`.
/// Não é a união: cada dab compõe com o que já está lá, e é isso que o Enio vê na tela do Painter.
fn painter_deposit(pts: &[(f32, f32)], r: f32, hardness: f32) -> Vec<f32> {
    painter_deposit_sized(pts, r, hardness, W, H)
}

/// O mesmo, com o tamanho do alvo como parâmetro (a sonda de imagem usa 768²).
fn painter_deposit_sized(
    pts: &[(f32, f32)],
    r: f32,
    hardness: f32,
    w_img: u32,
    h_img: u32,
) -> Vec<f32> {
    let pitch = (PAINTER_SPACING * 2.0 * r).max(0.25);
    // Reamostra o caminho no passo do Painter.
    let mut dabs: Vec<(f32, f32)> = vec![pts[0]];
    let mut carry = 0.0_f32;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if seg <= 1e-6 {
            continue;
        }
        let mut t = pitch - carry;
        while t <= seg {
            let f = t / seg;
            dabs.push((a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f));
            t += pitch;
        }
        carry = (carry + seg) % pitch;
    }
    // ⚠️ **AMOSTRA NO CENTRO DO PIXEL, e isto e escolha MEDIDA.** Superamostrar 4x4 foi tentado
    // e REVERTIDO: da uma verdade que **nenhum dos dois produtos calcula** — o Painter tambem
    // avalia a queda no centro do texel ao carimbar. Com 4x4, em hardness 0,8 a faixa de queda
    // (`dn in [0,8, 1]`) mede 1,4 px e a media de area discorda da amostra pontual por
    // **-67/255**, penalizando o Flip por algo que o Painter faz IGUAL. O oraculo tem de
    // amostrar como o produto amostra.
    let mut cov = vec![0.0_f32; (w_img * h_img) as usize];
    for &(dx, dy) in &dabs {
        let x0 = ((dx - r).floor().max(0.0)) as u32;
        let x1 = ((dx + r).ceil().min(w_img as f32 - 1.0)) as u32;
        let y0 = ((dy - r).floor().max(0.0)) as u32;
        let y1 = ((dy + r).ceil().min(h_img as f32 - 1.0)) as u32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let dn = ((px - dx).powi(2) + (py - dy).powi(2)).sqrt() / r;
                if dn >= 1.0 {
                    continue;
                }
                let w = painter_weight(dn, hardness);
                let c = &mut cov[(y * w_img + x) as usize];
                *c = 1.0 - (1.0 - *c) * (1.0 - w);
            }
        }
    }
    cov
}

fn shade(v: u8) -> char {
    match v {
        0..=8 => ' ',
        9..=48 => '.',
        49..=96 => ':',
        97..=144 => '+',
        145..=192 => 'o',
        193..=232 => 'O',
        _ => '#',
    }
}

fn ascii(get: impl Fn(u32, u32) -> u8) -> String {
    let mut s = String::new();
    for y in 0..H {
        for x in 0..W {
            s.push(shade(get(x, y)));
        }
        s.push('\n');
    }
    s
}

/// **A SONDA.** Renderiza a estrela no FLIP e computa o DEPÓSITO DO PAINTER na mesma figura,
/// imprime as duas em ASCII lado a lado (uma sobre a outra) e o pior desvio.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_flip_against_the_painters_deposit() {
    let Some((device, queue)) = device() else {
        println!("sem adapter");
        return;
    };
    let hardness = 0.4_f32;
    let (pts, r) = star_path(7.0);
    println!("\n=== A ESTRELA DE UM TRACO, hardness {hardness} raio {r} ===");
    println!("pontos {}", pts.len());

    let px = render(&device, &queue, &flip_drawing(&pts, r, hardness));
    let dep = painter_deposit(&pts, r, hardness);

    println!("\n--- FLIP (o que a foto mostra) ---");
    print!("{}", ascii(|x, y| alpha_at(&px, x, y)));
    println!("\n--- PAINTER (deposito de dabs, over) ---");
    print!(
        "{}",
        ascii(|x, y| (dep[(y * W + x) as usize] * 255.0).round() as u8)
    );

    let (mut lo, mut lo_at, mut hi, mut hi_at) = (0i32, (0u32, 0u32), 0i32, (0u32, 0u32));
    let mut bad = 0u32;
    for y in 0..H {
        for x in 0..W {
            let a = i32::from(alpha_at(&px, x, y));
            let b = (dep[(y * W + x) as usize] * 255.0).round() as i32;
            let d = a - b;
            if d.abs() > 16 {
                bad += 1;
            }
            if d < lo {
                lo = d;
                lo_at = (x, y);
            }
            if d > hi {
                hi = d;
                hi_at = (x, y);
            }
        }
    }
    let (mut nlo, mut nhi) = (0u32, 0u32);
    for y in 0..H {
        for x in 0..W {
            let d =
                i32::from(alpha_at(&px, x, y)) - (dep[(y * W + x) as usize] * 255.0).round() as i32;
            if d < -16 {
                nlo += 1;
            }
            if d > 16 {
                nhi += 1;
            }
        }
    }
    println!(
        "\nFLIP - PAINTER:  falta {lo:+} em {lo_at:?}  |  sobra {hi:+} em {hi_at:?}  |  {bad} px fora de 16"
    );
    println!(
        "  reparticao: {nlo} px FALTAM tinta (a cunha escura da foto)  |  {nhi} px SOBRAM (o tip convexo)"
    );

    // O CONTROLE: um traco RETO, onde o perfil de deposito e exato por construcao.
    let straight: Vec<(f32, f32)> = (0..=24).map(|k| (8.0 + k as f32 * 2.0, 32.0)).collect();
    let spx = render(&device, &queue, &flip_drawing(&straight, r, hardness));
    let sdep = painter_deposit(&straight, r, hardness);
    let mut sworst = 0i32;
    for x in 16..48 {
        for y in 20..44 {
            let d = i32::from(alpha_at(&spx, x, y))
                - (sdep[(y * W + x) as usize] * 255.0).round() as i32;
            if d.abs() > sworst.abs() {
                sworst = d;
            }
        }
    }
    println!("  CONTROLE (traco reto, miolo): pior desvio {sworst:+}/255");
}

/// **O PERFIL QUE O DEPOSITO DEIXA**, na seccao de um traco RETO: o produto `over` sobre a
/// fileira de dabs a `pitch` de arco. `phase` desloca a grade (o dab mais proximo cai em
/// `phase*pitch` do pe da perpendicular) — e a dependencia de fase E a ondulacao que o Painter
/// de fato tem.
fn deposit_profile(dn: f32, hardness: f32, phase: f32) -> f32 {
    let step = PAINTER_SPACING * 2.0; // pitch / raio
    let mut keep = 1.0_f32;
    for k in -12..=12 {
        let along = (k as f32 + phase) * step;
        let d = (dn * dn + along * along).sqrt();
        if d >= 1.0 {
            continue;
        }
        keep *= 1.0 - painter_weight(d, hardness);
    }
    1.0 - keep
}

/// A distancia normalizada a polilinha (a UNIAO que o Flip computa).
fn path_dn(pts: &[(f32, f32)], r: f32, p: (f32, f32)) -> f32 {
    let mut best = f32::INFINITY;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (vx, vy) = (b.0 - a.0, b.1 - a.1);
        let len2 = vx * vx + vy * vy;
        let t = if len2 <= 1e-9 {
            0.0
        } else {
            (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / len2).clamp(0.0, 1.0)
        };
        let (qx, qy) = (a.0 + vx * t, a.1 + vy * t);
        let d = ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt();
        best = best.min(d);
    }
    best / r
}

/// **A SONDA DA CURA** — e se o perfil do Flip FOSSE o perfil que o deposito do Painter deixa?
/// Compara, na MESMA estrela: (a) a lei de hoje sobre a uniao, (b) o perfil de deposito sobre a
/// uniao, (c) o deposito do Painter (o alvo).
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_what_the_deposit_profile_would_paint() {
    let hardness = 0.4_f32;
    let (pts, r) = star_path(7.0);
    println!("\n=== O PERFIL DO DEPOSITO (pitch {PAINTER_SPACING} x diametro) ===");
    println!("  dn    lei-de-hoje   deposito(fase 0)  deposito(fase .5)");
    for i in 0..=20 {
        let dn = i as f32 / 20.0;
        println!(
            "  {dn:.2}    {:.4}        {:.4}            {:.4}",
            painter_weight(dn, hardness),
            deposit_profile(dn, hardness, 0.0),
            deposit_profile(dn, hardness, 0.5)
        );
    }

    let target = painter_deposit(&pts, r, hardness);
    println!("\n--- MODELO: uniao com o perfil de DEPOSITO ---");
    print!(
        "{}",
        ascii(|x, y| {
            let dn = path_dn(&pts, r, (x as f32 + 0.5, y as f32 + 0.5));
            (deposit_profile(dn, hardness, 0.0) * 255.0).round() as u8
        })
    );
    for (name, prof) in [
        ("lei de hoje  ", 0u8),
        ("deposito f=0 ", 1),
        ("deposito f=.5", 2),
    ] {
        let (mut lo, mut hi, mut bad) = (0i32, 0i32, 0u32);
        let (mut lo_at, mut hi_at) = ((0u32, 0u32), (0u32, 0u32));
        for y in 0..H {
            for x in 0..W {
                let dn = path_dn(&pts, r, (x as f32 + 0.5, y as f32 + 0.5));
                let m = match prof {
                    0 => painter_weight(dn, hardness),
                    1 => deposit_profile(dn, hardness, 0.0),
                    _ => deposit_profile(dn, hardness, 0.5),
                };
                let d = (m * 255.0).round() as i32
                    - (target[(y * W + x) as usize] * 255.0).round() as i32;
                if d.abs() > 16 {
                    bad += 1;
                }
                if d < lo {
                    lo = d;
                    lo_at = (x, y);
                }
                if d > hi {
                    hi = d;
                    hi_at = (x, y);
                }
            }
        }
        println!(
            "  {name}: falta {lo:+4} em {lo_at:?}  sobra {hi:+4} em {hi_at:?}  {bad} px fora de 16"
        );
    }
}

/// As distancias por SEGMENTO, na ordem do arco.
fn seg_dns(pts: &[(f32, f32)], r: f32, p: (f32, f32)) -> Vec<f32> {
    pts.windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let (vx, vy) = (b.0 - a.0, b.1 - a.1);
            let len2 = vx * vx + vy * vy;
            let t = if len2 <= 1e-9 {
                0.0
            } else {
                (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / len2).clamp(0.0, 1.0)
            };
            let (qx, qy) = (a.0 + vx * t, a.1 + vy * t);
            ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt() / r
        })
        .collect()
}

/// **UMA PASSAGEM = UM MINIMO LOCAL do perfil de distancia ao longo do arco.** Numa quina
/// concava o perfil desce, SOBE no vertice e desce de novo: DUAS aproximacoes. Numa curva
/// suave, uma so.
fn passage_minima(dns: &[f32], hyst: f32) -> Vec<f32> {
    let mut out = Vec::new();
    let mut run = f32::INFINITY;
    for &d in dns {
        if d > run + hyst {
            if run < 1.0 {
                out.push(run);
            }
            run = d;
        } else {
            run = run.min(d);
        }
    }
    if run < 1.0 {
        out.push(run);
    }
    out
}

/// **A SONDA DA COMPOSICAO POR PASSAGEM.** Sob o modelo de deposito a composicao e EXATA: o
/// produto sobre todos os dabs FATORA por passagem. Mede se compor as quinas fecha o resto.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_whether_composing_the_corner_closes_it() {
    let hardness = 0.4_f32;
    let (pts, r) = star_path(7.0);
    let target = painter_deposit(&pts, r, hardness);
    println!("\n=== UNIAO vs COMPOSICAO POR PASSAGEM (perfil de deposito) ===");
    for hyst in [f32::INFINITY, 0.0, 0.02, 0.05, 0.10] {
        let (mut lo, mut hi, mut bad) = (0i32, 0i32, 0u32);
        let (mut lo_at, mut hi_at) = ((0u32, 0u32), (0u32, 0u32));
        for y in 0..H {
            for x in 0..W {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let dns = seg_dns(&pts, r, p);
                let m = if hyst.is_infinite() {
                    let d = dns.iter().cloned().fold(f32::INFINITY, f32::min);
                    deposit_profile(d, hardness, 0.0)
                } else {
                    let mut keep = 1.0_f32;
                    for d in passage_minima(&dns, hyst) {
                        keep *= 1.0 - deposit_profile(d, hardness, 0.0);
                    }
                    1.0 - keep
                };
                let d = (m * 255.0).round() as i32
                    - (target[(y * W + x) as usize] * 255.0).round() as i32;
                if d.abs() > 16 {
                    bad += 1;
                }
                if d < lo {
                    lo = d;
                    lo_at = (x, y);
                }
                if d > hi {
                    hi = d;
                    hi_at = (x, y);
                }
            }
        }
        let name = if hyst.is_infinite() {
            "UNIAO       ".to_string()
        } else {
            format!("compoe h={hyst:.2}")
        };
        println!(
            "  {name}: falta {lo:+4} em {lo_at:?}  sobra {hi:+4} em {hi_at:?}  {bad} px fora de 16"
        );
    }
}

/// **SONDA** — a coluna do padrao vs a do airbrush, para o discriminante do gate ser MEDIDO.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_airbrush_column() {
    let Some((device, queue)) = device() else {
        return;
    };
    let hardness = 0.5_f32;
    let pts: Vec<(f32, f32)> = (0..=8).map(|k| (4.0 + k as f32 * 7.0, 32.0)).collect();
    let std_d = flip_drawing(&pts, 10.0, hardness);
    let mut air_d = flip_drawing(&pts, 10.0, hardness);
    air_d.strokes[0].airbrush = true;
    let std = render(&device, &queue, &std_d);
    let air = render(&device, &queue, &air_d);
    println!("\n=== COLUNA x=32, y=32..43 (raio 10, hardness {hardness}) ===");
    print!("  padrao : ");
    for y in 32..=43 {
        print!("{:4}", alpha_at(&std, 32, y));
    }
    print!("\n  airbrush:");
    for y in 32..=43 {
        print!("{:4}", alpha_at(&air, 32, y));
    }
    println!("\n  dn     : ");
    for y in 32..=43 {
        print!("{:6.2}", (y - 32) as f32 / 10.0);
    }
    println!();
}

// ---------------------------------------------------------------------------
// GATE — a entrega da wave, na figura da foto
// ---------------------------------------------------------------------------

/// 🔴 **O TRAÇO DO FLIP PINTA O QUE O PINCEL DIGITAL DO PAINTER DEPOSITA.**
///
/// A figura é a da foto do Enio: a estrela de UM traço (quina de 36° em cada ponta, cinco
/// auto-cruzamentos no miolo), pincel macio. O oráculo é o **depósito de verdade** — dabs a
/// `spacing × diâmetro` de arco compostos por `over`, com o `Falloff` REAL do
/// `ph2d_painter_brush`.
///
/// ⚠️ **O gate afirma as duas metades separadamente, e isso é load-bearing:**
///
/// - **FALTAR tinta é o defeito** (a cunha ESCURA que as setas vermelhas apontam) ⇒ **zero**
///   pixel pode ficar abaixo do depósito por mais de 16/255. Medido antes da cura: **−112**.
/// - **SOBRAR é o resíduo NOMEADO**, e vive só no canto CONVEXO: ali os dabs do Painter recuam
///   em vez de correr paralelos, e o perfil de traço os superestima (medido: +140/255 no vértice
///   de 36°, e ele EVAPORA conforme a hardness sobe: +13 em 0,8, **zero** em 0,9). A ponta do
///   Flip fica mais cheia — a direção OPOSTA à queixa, e discutivelmente a
///   mais correta (a junção redonda é a forma ideal; o afinamento do Painter é artefato da
///   discretização DELE). Um limite frouxo aqui seria gate que não pode falhar, então o teto é
///   afirmado no NÚMERO medido em vez de deixado livre.
///
/// E o CONTROLE: num traço RETO o perfil é exato por construção ⇒ ±4/255. Sem ele, "casar o
/// depósito" poderia ser satisfeito por qualquer curva mais cheia que a antiga.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn the_flip_paints_what_the_painters_digital_brush_deposits() {
    let Some((device, queue)) = device() else {
        return;
    };
    for hardness in [0.2_f32, 0.4, 0.7] {
        let (pts, r) = star_path(7.0);
        let px = render(&device, &queue, &flip_drawing(&pts, r, hardness));
        let dep = painter_deposit(&pts, r, hardness);
        let (mut falta, mut falta_at, mut sobra) = (0i32, (0u32, 0u32), 0i32);
        let mut n_falta = 0u32;
        for y in 0..H {
            for x in 0..W {
                if in_the_silhouette_fringe(&pts, r, x, y) {
                    continue;
                }
                let d = i32::from(alpha_at(&px, x, y))
                    - (dep[(y * W + x) as usize] * 255.0).round() as i32;
                if d < -16 {
                    n_falta += 1;
                }
                if d < falta {
                    falta = d;
                    falta_at = (x, y);
                }
                sobra = sobra.max(d);
            }
        }
        assert!(
            n_falta == 0 && falta > -16,
            "hardness {hardness}: {n_falta} px com MENOS tinta que o Painter (pior {falta} em \
             {falta_at:?}) -- e a cunha escura da foto"
        );
        assert!(
            sobra <= 140,
            "hardness {hardness}: sobra {sobra} passou do residuo NOMEADO do canto convexo (140)"
        );
    }

    // CONTROLE: no corpo reto o perfil de traço É o depósito, por construção.
    let straight: Vec<(f32, f32)> = (0..=24).map(|k| (8.0 + k as f32 * 2.0, 32.0)).collect();
    let spx = render(&device, &queue, &flip_drawing(&straight, 7.0, 0.4));
    let sdep = painter_deposit(&straight, 7.0, 0.4);
    let mut worst = 0i32;
    for x in 16..48 {
        for y in 20..44 {
            let d = i32::from(alpha_at(&spx, x, y))
                - (sdep[(y * W + x) as usize] * 255.0).round() as i32;
            if d.abs() > worst.abs() {
                worst = d;
            }
        }
    }
    assert!(
        worst.abs() <= 4,
        "no corpo RETO o Flip tem de SER o deposito do Painter: pior desvio {worst}/255"
    );
}

/// A FRANJA da silhueta: o Flip fecha a borda com AA (`edge`), o deposito do Painter nao tem
/// termo de AA nenhum (ele carimba num buffer). Comparar ali mede CONVENCAO DE BORDA, nao a lei
/// — entao a comparacao pula uma faixa de ~1,5 px em torno de `dn = 1`.
fn in_the_silhouette_fringe(pts: &[(f32, f32)], r: f32, x: u32, y: u32) -> bool {
    let dn = path_dn(pts, r, (x as f32 + 0.5, y as f32 + 0.5));
    (dn - 1.0).abs() < 1.5 / r
}

/// **SONDA** — o residuo do canto CONVEXO por hardness (para o gate afirmar o numero MEDIDO).
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_convex_tip_residual() {
    let Some((device, queue)) = device() else {
        return;
    };
    println!("\n=== RESIDUO por hardness (estrela de um traco, raio 7) ===");
    println!("  hard   falta  n_falta   sobra  n_sobra");
    for hi in 1..=9 {
        let hardness = hi as f32 / 10.0;
        let (pts, r) = star_path(7.0);
        let px = render(&device, &queue, &flip_drawing(&pts, r, hardness));
        let dep = painter_deposit(&pts, r, hardness);
        let (mut lo, mut hi_d, mut nlo, mut nhi) = (0i32, 0i32, 0u32, 0u32);
        for y in 0..H {
            for x in 0..W {
                if in_the_silhouette_fringe(&pts, r, x, y) {
                    continue;
                }
                let d = i32::from(alpha_at(&px, x, y))
                    - (dep[(y * W + x) as usize] * 255.0).round() as i32;
                if d < -16 {
                    nlo += 1;
                }
                if d > 16 {
                    nhi += 1;
                }
                lo = lo.min(d);
                hi_d = hi_d.max(d);
            }
        }
        println!("  {hardness:.1}   {lo:+5}   {nlo:5}   {hi_d:+5}   {nhi:5}");
    }
}

// ---------------------------------------------------------------------------
// A SONDA DE IMAGEM — para o Enio e eu olharmos os MESMOS pixels
// ---------------------------------------------------------------------------

/// Escreve um BMP 24-bit (sem compressão, linhas de baixo para cima, padding de 4 bytes).
///
/// ⚠️ **À mão, de propósito:** um encoder de PNG entraria como dependência nova num
/// `Cargo.toml` só para uma sonda de diagnóstico. BMP abre em qualquer visualizador do
/// Linux e custa zero superfície.
fn write_bmp(path: &std::path::Path, w: u32, h: u32, rgb: &[u8]) {
    let row = (w * 3).next_multiple_of(4) as usize;
    let pixels = row * h as usize;
    let mut out = Vec::with_capacity(54 + pixels);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&((54 + pixels) as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    for _ in 0..6 {
        out.extend_from_slice(&0u32.to_le_bytes());
    }
    for y in (0..h).rev() {
        let start = out.len();
        for x in 0..w {
            let i = ((y * w + x) * 3) as usize;
            out.push(rgb[i + 2]); // BGR
            out.push(rgb[i + 1]);
            out.push(rgb[i]);
        }
        out.resize(start + row, 0);
    }
    std::fs::write(path, out).expect("escrever bmp");
}

/// Tinta branca sobre o cinza do canvas — a mesma leitura da foto do Enio.
fn over_dark(alpha: f32) -> [u8; 3] {
    const BG: f32 = 56.0;
    const INK: f32 = 245.0;
    let v = (BG + (INK - BG) * alpha.clamp(0.0, 1.0)).round() as u8;
    [v, v, v]
}

/// 🖼️ **AS IMAGENS.** O que o Flip pinta e o que o pincel digital do Painter deposita, na MESMA
/// figura e na MESMA escala — para pararmos de trafegar foto-de-tela contra fixture.
///
/// ⚠️ **A figura é a do report:** a estrela de UM traço (quina afiada em cada ponta, cinco
/// auto-cruzamentos), pincel macio e GROSSO. Dois traços cruzados nunca tiveram o defeito.
#[test]
#[ignore = "sonda de imagem; roda com --ignored"]
fn render_the_two_side_by_side() {
    let Some((device, queue)) = device() else {
        println!("sem adapter -- nada a escrever");
        return;
    };
    let dir = std::path::Path::new("/home/enio/flip_vs_painter");
    std::fs::create_dir_all(dir).expect("criar diretorio");
    const S: u32 = 768;
    let r = 80.0_f32;

    // ⚠️ O 3º par usa polilinha DENSA de propósito: é o modo de falha do orçamento de vizinhos,
    // e a pergunta que ele responde é *"o defeito da sua foto é ESTE?"*.
    for (nome, hardness, passo) in [
        ("macio_h0.4", 0.4_f32, 0.8_f32),
        ("medio_h0.7", 0.7, 0.8),
        ("DENSO_h0.4_orcamento_estourado", 0.4, 0.08),
        // ⚠️ A geometria EXATA da cena de smoke: 6 pontos CRUS, sem reamostragem nenhuma
        // (`one_stroke_star` empurra os cantos direto no `FlipStroke`, sem passar pelo
        // `stroke_from_samples` do produto). Segmentos ENORMES contra o alcance.
        ("SMOKE_6_pontos_crus_h0.4", 0.4, 1e9),
        // ⚠️ **O RABISCO COMPACTO** — a figura do Enio. Braços de ~2,5·r em vez de 3,8·r: o traço
        // INTEIRO cabe dentro do alcance de si mesmo, então a caminhada da fita nunca "sai" e o
        // critério ESPACIAL de passagem declara o cruzamento como a MESMA passagem ⇒ união ⇒
        // vinco. Os fixtures largos não continham o fenômeno.
        ("COMPACTO_h0.4_a_figura_do_report", 0.4, 0.8),
        ("A_DO_REPORT_h0.4", 0.4, 0.8),
    ] {
        // A estrela, na escala da imagem.
        let outer = if nome.starts_with("COMPACTO") {
            2.5 * r
        } else {
            300.0_f32
        };
        let (cx, cy) = (S as f32 * 0.5, S as f32 * 0.5);
        // A figura: estrela de um traço, ou o "A" da 2ª foto do Enio (um traço só que cruza a si
        // mesmo em ÂNGULO RASO, com os braços bem separados — é no OMBRO parcialmente coberto
        // que o vinco da união aparece, e as estrelas saturadas o escondiam).
        let corners: Vec<(f32, f32)> = if nome.starts_with("A_DO_REPORT") {
            vec![
                (140.0, 700.0),
                (384.0, 150.0),
                (628.0, 700.0),
                (250.0, 470.0),
                (700.0, 430.0),
            ]
        } else {
            let mut c = Vec::new();
            for k in 0..5 {
                let a =
                    -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
                c.push((cx + outer * a.cos(), cy + outer * a.sin()));
            }
            c.push(c[0]);
            c
        };
        // A DENSIDADE é a do PRODUTO (`RESAMPLE_STEP_FRACTION = 0.4` × largura = `0.8·r`).
        let step = passo * r;
        let mut pts = vec![corners[0]];
        for w in corners.windows(2) {
            let (a, b) = (w[0], w[1]);
            let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (len / step).ceil().max(1.0) as usize;
            for k in 1..=n {
                let t = k as f32 / n as f32;
                pts.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }

        let px = render_sized(&device, &queue, &flip_drawing(&pts, r, hardness), S, S);
        let dep = painter_deposit_sized(&pts, r, hardness, S, S);

        let mut flip_rgb = vec![0u8; (S * S * 3) as usize];
        let mut paint_rgb = vec![0u8; (S * S * 3) as usize];
        for y in 0..S {
            for x in 0..S {
                let i = ((y * S + x) * 3) as usize;
                let a_flip = f32::from(px[((y * S + x) * 4 + 3) as usize]) / 255.0;
                flip_rgb[i..i + 3].copy_from_slice(&over_dark(a_flip));
                paint_rgb[i..i + 3].copy_from_slice(&over_dark(dep[(y * S + x) as usize]));
            }
        }
        write_bmp(&dir.join(format!("1_FLIP_{nome}.bmp")), S, S, &flip_rgb);
        write_bmp(&dir.join(format!("2_PAINTER_{nome}.bmp")), S, S, &paint_rgb);

        // E o lado-a-lado, para uma janela so.
        let mut both = vec![0u8; (S * 2 * S * 3) as usize];
        for y in 0..S {
            for x in 0..S {
                let src = ((y * S + x) * 3) as usize;
                let l = ((y * S * 2 + x) * 3) as usize;
                let rr = ((y * S * 2 + S + x) * 3) as usize;
                both[l..l + 3].copy_from_slice(&flip_rgb[src..src + 3]);
                both[rr..rr + 3].copy_from_slice(&paint_rgb[src..src + 3]);
            }
            // divisória
            let d = ((y * S * 2 + S) * 3) as usize;
            both[d..d + 3].copy_from_slice(&[200, 60, 60]);
        }
        write_bmp(
            &dir.join(format!("3_LADO_A_LADO_{nome}.bmp")),
            S * 2,
            S,
            &both,
        );
    }
    println!("\nimagens escritas em {}", dir.display());
    for e in std::fs::read_dir(dir).expect("ler dir") {
        println!("  {}", e.expect("entrada").path().display());
    }
}

/// **SONDA** — ONDE está o penhasco: o defeito aparece quando a polilinha fica mais densa que o
/// orçamento de vizinhos (`MAX_RIBBON_EXTRAS`) consegue cobrir.
///
/// O produto reamostra a `0,8·r` (`RESAMPLE_STEP_FRACTION = 0.4` da largura), mas o **RDP** roda
/// ANTES com tolerância `0,1·r` e a reamostragem **só acrescenta** pontos — nunca remove. Numa
/// curva de raio `R` o RDP guarda cordas de `≈ √(0,8·R·r)`, que fica ABAIXO do passo quando
/// `R < 0,8·r`: **um rabisco mais fechado que o próprio pincel**.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_where_the_neighbour_budget_breaks() {
    let Some((device, queue)) = device() else {
        return;
    };
    let hardness = 0.4_f32;
    let r = 7.0_f32;
    println!("\n=== O PENHASCO: falta de tinta por DENSIDADE da polilinha ===");
    println!("  passo/raio   segmentos   falta   n_falta");
    for frac in [0.8_f32, 0.4, 0.2, 0.1, 0.05] {
        let (cx, cy, outer) = (32.0_f32, 32.0_f32, 26.0_f32);
        let mut corners = Vec::new();
        for k in 0..5 {
            let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
            corners.push((cx + outer * a.cos(), cy + outer * a.sin()));
        }
        corners.push(corners[0]);
        let step = frac * r;
        let mut pts = vec![corners[0]];
        for w in corners.windows(2) {
            let (a, b) = (w[0], w[1]);
            let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (len / step).ceil().max(1.0) as usize;
            for k in 1..=n {
                let t = k as f32 / n as f32;
                pts.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        let px = render(&device, &queue, &flip_drawing(&pts, r, hardness));
        let dep = painter_deposit(&pts, r, hardness);
        let (mut lo, mut nlo) = (0i32, 0u32);
        for y in 0..H {
            for x in 0..W {
                if in_the_silhouette_fringe(&pts, r, x, y) {
                    continue;
                }
                let d = i32::from(alpha_at(&px, x, y))
                    - (dep[(y * W + x) as usize] * 255.0).round() as i32;
                if d < -16 {
                    nlo += 1;
                }
                lo = lo.min(d);
            }
        }
        println!("  {frac:8.2}   {:9}   {lo:+5}   {nlo:5}", pts.len() - 1);
    }
}

/// 🔴 **O ORÁCULO DO ENIO** (2026-07-28): *"o problema só aparece se o cruzamento é feito com
/// traço ÚNICO (sem mouse up). Se cruzo vários traços diferentes (após mouse up) esse aspecto 3d
/// não aparece e o traço fica melhor"*.
///
/// ⚠️ **O oráculo dele não é o Painter — é o PRÓPRIO FLIP com traços separados.** Dois traços
/// distintos têm depth diferente ⇒ o mais novo compõe por `over` ⇒ no ombro onde as duas caudas
/// se encontram o resultado é `1 − (1−a)(1−b)`. Um traço só cai na UNIÃO (`max`), que no MESMO
/// ponto dá `max(a,b)` — mais TRANSPARENTE, e o fundo aparecendo por baixo lê como uma DOBRA 3D.
///
/// Esta sonda mede a diferença nas duas figuras: a QUINA (dois braços do mesmo traço) e o
/// CRUZAMENTO (o traço voltando sobre si).
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_one_stroke_against_two_strokes() {
    let Some((device, queue)) = device() else {
        return;
    };
    println!("\n=== UM TRACO vs DOIS TRACOS (o oraculo do Enio) ===");
    let r = 7.0_f32;
    for (nome, a, b, c) in [
        (
            "QUINA 60 graus  ",
            (8.0_f32, 12.0_f32),
            (32.0, 40.0),
            (56.0, 12.0),
        ),
        ("QUINA 30 graus  ", (6.0, 20.0), (32.0, 40.0), (58.0, 20.0)),
        ("CRUZAMENTO raso ", (6.0, 24.0), (40.0, 34.0), (10.0, 44.0)),
    ] {
        for hardness in [0.4_f32, 0.7] {
            // Densifica cada perna no passo do produto.
            let leg = |p: (f32, f32), q: (f32, f32)| -> Vec<(f32, f32)> {
                let len = ((q.0 - p.0).powi(2) + (q.1 - p.1).powi(2)).sqrt();
                let n = (len / (0.8 * r)).ceil().max(1.0) as usize;
                (0..=n)
                    .map(|k| {
                        let t = k as f32 / n as f32;
                        (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t)
                    })
                    .collect()
            };
            // UM traço: a → b → c.
            let mut um = leg(a, b);
            um.extend(leg(b, c).into_iter().skip(1));
            let d_um = flip_drawing(&um, r, hardness);
            // DOIS traços: a → b e b → c, separados (depth distinto).
            let mut d_dois = flip_drawing(&leg(a, b), r, hardness);
            d_dois
                .strokes
                .push(flip_drawing(&leg(b, c), r, hardness).strokes.remove(0));

            let px1 = render(&device, &queue, &d_um);
            let px2 = render(&device, &queue, &d_dois);
            let (mut pior, mut onde, mut n) = (0i32, (0u32, 0u32), 0u32);
            for y in 0..H {
                for x in 0..W {
                    let d = i32::from(alpha_at(&px1, x, y)) - i32::from(alpha_at(&px2, x, y));
                    if d < -8 {
                        n += 1;
                    }
                    if d < pior {
                        pior = d;
                        onde = (x, y);
                    }
                }
            }
            println!(
                "  {nome} h={hardness:.1}: UM traco tem ate {pior:+4} MENOS tinta que DOIS \
                 (em {onde:?}), {n} px abaixo de -8"
            );
        }
    }
}

/// 🔴🔴 **O ORÁCULO DO ENIO, NA FIGURA DELE, NOS DOIS SENTIDOS.**
///
/// ⚠️ A sonda irmã acima só conta `d < -8` — *um traço tem MENOS tinta*. Mas o que o Enio
/// aponta na foto é uma **cunha ESCURA**, e escuro é tinta a MAIS. Uma sonda que só olha para
/// um lado do sinal não pode ver o defeito reportado; esta olha para os dois, na ESTRELA (a
/// figura da foto: cinco quinas de 36° e cinco auto-cruzamentos) e escreve as duas imagens.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_star_one_stroke_against_separate_strokes() {
    let Some((device, queue)) = device() else {
        return;
    };
    println!("\n=== A ESTRELA: UM TRACO vs CINCO TRACOS, NOS DOIS SENTIDOS ===");
    let r = 7.0_f32;
    // Os cinco cantos da estrela (passo de 2/5 de volta), fechando no primeiro.
    let (cx, cy, outer) = (32.0_f32, 32.0_f32, 26.0_f32);
    let mut corners: Vec<(f32, f32)> = (0..5)
        .map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
            (cx + outer * a.cos(), cy + outer * a.sin())
        })
        .collect();
    corners.push(corners[0]);
    let leg = |p: (f32, f32), q: (f32, f32)| -> Vec<(f32, f32)> {
        let len = ((q.0 - p.0).powi(2) + (q.1 - p.1).powi(2)).sqrt();
        let n = (len / (0.8 * r)).ceil().max(1.0) as usize;
        (0..=n)
            .map(|k| {
                let t = k as f32 / n as f32;
                (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t)
            })
            .collect()
    };
    for hardness in [0.4_f32, 0.7, 1.0] {
        // UM traço: percorre os cinco cantos sem levantar.
        let mut um = leg(corners[0], corners[1]);
        for w in corners.windows(2).skip(1) {
            um.extend(leg(w[0], w[1]).into_iter().skip(1));
        }
        let d_um = flip_drawing(&um, r, hardness);
        // CINCO traços: uma perna cada, depth distinto ⇒ compõem por `over`.
        let mut d_sep = FlipDrawing::new();
        for w in corners.windows(2) {
            d_sep.strokes.push(
                flip_drawing(&leg(w[0], w[1]), r, hardness)
                    .strokes
                    .remove(0),
            );
        }
        let px1 = render(&device, &queue, &d_um);
        let px2 = render(&device, &queue, &d_sep);
        let (mut falta, mut ondef) = (0i32, (0u32, 0u32));
        let (mut sobra, mut ondes) = (0i32, (0u32, 0u32));
        let (mut nf, mut ns) = (0u32, 0u32);
        for y in 0..H {
            for x in 0..W {
                let d = i32::from(alpha_at(&px1, x, y)) - i32::from(alpha_at(&px2, x, y));
                if d < -8 {
                    nf += 1;
                }
                if d > 8 {
                    ns += 1;
                }
                if d < falta {
                    falta = d;
                    ondef = (x, y);
                }
                if d > sobra {
                    sobra = d;
                    ondes = (x, y);
                }
            }
        }
        println!(
            "  h={hardness:.1}: FALTA {falta:+4} em {ondef:?} ({nf} px < -8)   \
             SOBRA {sobra:+4} em {ondes:?} ({ns} px > +8)"
        );
    }
}

/// 🖼️ **AS DUAS IMAGENS DO ORÁCULO** — a MESMA estrela como UM traço e como CINCO, na escala em
/// que o Enio olha. É a única forma de decidir se o `-63` medido é o que a foto mostra.
#[test]
#[ignore = "sonda de imagem; roda com --ignored"]
fn render_one_stroke_against_separate_strokes() {
    let Some((device, queue)) = device() else {
        return;
    };
    let dir = std::path::Path::new("/home/enio/flip_um_vs_varios");
    std::fs::create_dir_all(dir).expect("criar diretorio");
    const S: u32 = 768;
    let r = 40.0_f32;
    let (cx, cy, outer) = (S as f32 * 0.5, S as f32 * 0.5, 300.0_f32);
    let mut corners: Vec<(f32, f32)> = (0..5)
        .map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
            (cx + outer * a.cos(), cy + outer * a.sin())
        })
        .collect();
    corners.push(corners[0]);
    let leg = |p: (f32, f32), q: (f32, f32)| -> Vec<(f32, f32)> {
        let len = ((q.0 - p.0).powi(2) + (q.1 - p.1).powi(2)).sqrt();
        let n = (len / (0.8 * r)).ceil().max(1.0) as usize;
        (0..=n)
            .map(|k| {
                let t = k as f32 / n as f32;
                (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t)
            })
            .collect()
    };
    for hardness in [0.4_f32, 0.7] {
        let mut um = leg(corners[0], corners[1]);
        for w in corners.windows(2).skip(1) {
            um.extend(leg(w[0], w[1]).into_iter().skip(1));
        }
        let mut d_sep = FlipDrawing::new();
        for w in corners.windows(2) {
            d_sep.strokes.push(
                flip_drawing(&leg(w[0], w[1]), r, hardness)
                    .strokes
                    .remove(0),
            );
        }
        let a = render_sized(&device, &queue, &flip_drawing(&um, r, hardness), S, S);
        let b = render_sized(&device, &queue, &d_sep, S, S);
        let mut both = vec![0u8; (S * 2 * S * 3) as usize];
        for y in 0..S {
            for x in 0..S {
                let l = ((y * S * 2 + x) * 3) as usize;
                let rr = ((y * S * 2 + S + x) * 3) as usize;
                let i = ((y * S + x) * 4 + 3) as usize;
                both[l..l + 3].copy_from_slice(&over_dark(f32::from(a[i]) / 255.0));
                both[rr..rr + 3].copy_from_slice(&over_dark(f32::from(b[i]) / 255.0));
            }
            let d = ((y * S * 2 + S) * 3) as usize;
            both[d..d + 3].copy_from_slice(&[200, 60, 60]);
        }
        write_bmp(
            &dir.join(format!("UM_esquerda__VARIOS_direita_h{hardness:.1}.bmp")),
            S * 2,
            S,
            &both,
        );
    }
    println!("imagens em {}", dir.display());
}

/// 🔬 **A QUINA E O CRUZAMENTO DE PERTO** — um traço vs dois, MUITO ampliados e macios, que é
/// onde o "aspecto 3D" do report tem de estar visível. Escreve também o MAPA DE DIFERENÇA.
#[test]
#[ignore = "sonda de imagem; roda com --ignored"]
fn render_the_corner_and_the_crossing_up_close() {
    let Some((device, queue)) = device() else {
        return;
    };
    let dir = std::path::Path::new("/home/enio/flip_quina");
    std::fs::create_dir_all(dir).expect("criar diretorio");
    const S: u32 = 512;
    let r = 55.0_f32;
    let dense = |p: (f32, f32), q: (f32, f32)| -> Vec<(f32, f32)> {
        let len = ((q.0 - p.0).powi(2) + (q.1 - p.1).powi(2)).sqrt();
        let n = (len / (0.8 * r)).ceil().max(1.0) as usize;
        (0..=n)
            .map(|k| {
                let t = k as f32 / n as f32;
                (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t)
            })
            .collect()
    };
    for (nome, a, b, c, hardness) in [
        (
            "quina30_h0.4",
            (30.0_f32, 60.0_f32),
            (470.0_f32, 256.0_f32),
            (30.0_f32, 452.0_f32),
            0.4_f32,
        ),
        (
            "quina30_h0.7",
            (30.0, 60.0),
            (470.0, 256.0),
            (30.0, 452.0),
            0.7,
        ),
        (
            "raso15_h0.4",
            (30.0, 150.0),
            (470.0, 256.0),
            (30.0, 362.0),
            0.4,
        ),
    ] {
        let mut um = dense(a, b);
        um.extend(dense(b, c).into_iter().skip(1));
        let mut dois = FlipDrawing::new();
        dois.strokes
            .push(flip_drawing(&dense(a, b), r, hardness).strokes.remove(0));
        dois.strokes
            .push(flip_drawing(&dense(b, c), r, hardness).strokes.remove(0));
        let pa = render_sized(&device, &queue, &flip_drawing(&um, r, hardness), S, S);
        let pb = render_sized(&device, &queue, &dois, S, S);
        let mut both = vec![0u8; (S * 3 * S * 3) as usize];
        for y in 0..S {
            for x in 0..S {
                let i = ((y * S + x) * 4 + 3) as usize;
                let (av, bv) = (f32::from(pa[i]) / 255.0, f32::from(pb[i]) / 255.0);
                let l = ((y * S * 3 + x) * 3) as usize;
                let m = ((y * S * 3 + S + x) * 3) as usize;
                let rr = ((y * S * 3 + 2 * S + x) * 3) as usize;
                both[l..l + 3].copy_from_slice(&over_dark(av));
                both[m..m + 3].copy_from_slice(&over_dark(bv));
                // Mapa: vermelho = UM tem menos, verde = UM tem mais, ganho 4x.
                let d = (av - bv) * 4.0;
                let up = (d.clamp(0.0, 1.0) * 255.0) as u8;
                let dn = ((-d).clamp(0.0, 1.0) * 255.0) as u8;
                both[rr..rr + 3].copy_from_slice(&[dn, up, 40]);
            }
            for k in [S, 2 * S] {
                let d = ((y * S * 3 + k) * 3) as usize;
                both[d..d + 3].copy_from_slice(&[200, 60, 60]);
            }
        }
        write_bmp(
            &dir.join(format!("UM__DOIS__DIFF_{nome}.bmp")),
            S * 3,
            S,
            &both,
        );
    }
    println!("imagens em {}", dir.display());
}

/// 🔬 **FLIP × PAINTER × DIFERENÇA, com o OMBRO à vista.** As fixtures anteriores usavam pincel
/// GROSSO contra estrela pequena — tudo satura e as duas leis ficam indistinguíveis por
/// construção. Aqui o pincel é FINO em relação à figura, que é o regime onde o ombro macio
/// (a metade do traço que não é núcleo) de fato se vê.
#[test]
#[ignore = "sonda de imagem; roda com --ignored"]
fn render_flip_painter_and_the_difference() {
    let Some((device, queue)) = device() else {
        return;
    };
    let dir = std::path::Path::new("/home/enio/flip_diff");
    std::fs::create_dir_all(dir).expect("criar diretorio");
    const S: u32 = 640;
    for (nome, r, hardness) in [
        ("fino_h0.4", 22.0_f32, 0.4_f32),
        ("fino_h0.7", 22.0, 0.7),
        ("medio_h0.4", 45.0, 0.4),
    ] {
        let (cx, cy, outer) = (S as f32 * 0.5, S as f32 * 0.52, 260.0_f32);
        let mut corners: Vec<(f32, f32)> = (0..5)
            .map(|k| {
                let a =
                    -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
                (cx + outer * a.cos(), cy + outer * a.sin())
            })
            .collect();
        corners.push(corners[0]);
        let mut pts = vec![corners[0]];
        for w in corners.windows(2) {
            let (a, b) = (w[0], w[1]);
            let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (len / (0.8 * r)).ceil().max(1.0) as usize;
            for k in 1..=n {
                let t = k as f32 / n as f32;
                pts.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        let px = render_sized(&device, &queue, &flip_drawing(&pts, r, hardness), S, S);
        let dep = painter_deposit_sized(&pts, r, hardness, S, S);
        let mut trio = vec![0u8; (S * 3 * S * 3) as usize];
        for y in 0..S {
            for x in 0..S {
                let a = f32::from(px[((y * S + x) * 4 + 3) as usize]) / 255.0;
                let b = dep[(y * S + x) as usize];
                let l = ((y * S * 3 + x) * 3) as usize;
                let m = ((y * S * 3 + S + x) * 3) as usize;
                let rr = ((y * S * 3 + 2 * S + x) * 3) as usize;
                trio[l..l + 3].copy_from_slice(&over_dark(a));
                trio[m..m + 3].copy_from_slice(&over_dark(b));
                let d = (a - b) * 4.0;
                trio[rr..rr + 3].copy_from_slice(&[
                    ((-d).clamp(0.0, 1.0) * 255.0) as u8,
                    (d.clamp(0.0, 1.0) * 255.0) as u8,
                    30,
                ]);
            }
            for k in [S, 2 * S] {
                let d = ((y * S * 3 + k) * 3) as usize;
                trio[d..d + 3].copy_from_slice(&[200, 60, 60]);
            }
        }
        write_bmp(
            &dir.join(format!("FLIP__PAINTER__DIFF_{nome}.bmp")),
            S * 3,
            S,
            &trio,
        );
    }
    println!("imagens em {}", dir.display());
}

/// 🖼️ **A MÃO LENTA** — a estrela na densidade que o produto de fato entrega quando o artista
/// desenha devagar (`0,106 × raio`, medido em `flip_hardness_smoke::tests`), contra o depósito
/// real do Painter. É a imagem do defeito e da cura.
///
/// Escreve em `/home/enio/flip_lenta/<PH2D_TAG>.bmp` para o antes/depois caberem lado a lado.
#[test]
#[ignore = "sonda de imagem; roda com --ignored"]
fn render_the_slow_hand_star() {
    let Some((device, queue)) = device() else {
        return;
    };
    let dir = std::path::Path::new("/home/enio/flip_lenta");
    std::fs::create_dir_all(dir).expect("criar diretorio");
    let tag = std::env::var("PH2D_TAG").unwrap_or_else(|_| "atual".into());
    const S: u32 = 640;
    let r = 22.0_f32;
    let (cx, cy, outer) = (S as f32 * 0.5, S as f32 * 0.52, 260.0_f32);
    let mut corners: Vec<(f32, f32)> = (0..5)
        .map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
            (cx + outer * a.cos(), cy + outer * a.sin())
        })
        .collect();
    corners.push(corners[0]);
    let step = 0.106 * r;
    let mut pts = vec![corners[0]];
    for w in corners.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let n = (len / step).ceil().max(1.0) as usize;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            pts.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    let px = render_sized(&device, &queue, &flip_drawing(&pts, r, 0.4), S, S);
    let dep = painter_deposit_sized(&pts, r, 0.4, S, S);
    let mut trio = vec![0u8; (S * 2 * S * 3) as usize];
    for y in 0..S {
        for x in 0..S {
            let a = f32::from(px[((y * S + x) * 4 + 3) as usize]) / 255.0;
            let b = dep[(y * S + x) as usize];
            let l = ((y * S * 2 + x) * 3) as usize;
            let rr = ((y * S * 2 + S + x) * 3) as usize;
            trio[l..l + 3].copy_from_slice(&over_dark(a));
            let d = (a - b) * 4.0;
            trio[rr..rr + 3].copy_from_slice(&[
                ((-d).clamp(0.0, 1.0) * 255.0) as u8,
                (d.clamp(0.0, 1.0) * 255.0) as u8,
                30,
            ]);
        }
        let d = ((y * S * 2 + S) * 3) as usize;
        trio[d..d + 3].copy_from_slice(&[200, 60, 60]);
    }
    write_bmp(&dir.join(format!("{tag}.bmp")), S * 2, S, &trio);
    println!(
        "escrito {}/{tag}.bmp ({} segmentos)",
        dir.display(),
        pts.len() - 1
    );
}
