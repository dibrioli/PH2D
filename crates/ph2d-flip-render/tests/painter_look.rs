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
use ph2d_flip_render::{
    CameraRaw, DEFAULT_TILE, FlipRenderer, ScreenSpace, bin_segments, pack_drawing, walk_pixel,
};

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
// O MOTOR NOVO contra o MESMO oráculo (doc 12 §9, passo 4)
// ---------------------------------------------------------------------------

/// O motor NOVO sobre a MESMA figura, pelo MESMO caminho de dados do produto.
///
/// ⚠️ **Nada aqui é oráculo novo** — o handoff §6 é explícito (*"não escreva oráculo novo antes
/// de usar estes"*). A figura é o `star_path`, a referência é o `painter_deposit`, a exclusão é o
/// `in_the_silhouette_fringe`. O que muda é **quem responde**: `walk_pixel` em vez do
/// `FlipRenderer`.
///
/// ⚠️ **A projeção é a MESMA CÂMERA do render** (`pixel_camera_sized` → `ScreenSpace::from_camera`),
/// e o ponto de amostra é o centro do pixel **projetado por ela** — assim a comparação não depende
/// de eu re-derivar convenção de eixo nenhuma (a câmera é Y-flipada, e é exatamente esse tipo de
/// re-derivação que vira um erro mudo).
fn new_engine_alpha(pts: &[(f32, f32)], r: f32, hardness: f32, w: u32, h: u32) -> Vec<f32> {
    new_engine_alpha_of(&flip_drawing(pts, r, hardness), w, h)
}

/// O mesmo, com o DESENHO como parâmetro — o oráculo do Enio precisa de um desenho de vários
/// traços, e um 2º harness mediria outro produto.
fn new_engine_alpha_of(drawing: &FlipDrawing, w: u32, h: u32) -> Vec<f32> {
    let data = pack_drawing(drawing);
    let screen = ScreenSpace::from_camera(&pixel_camera_sized(w, h));
    let bins = bin_segments(&data, &screen, DEFAULT_TILE);
    let mut out = vec![0.0_f32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let p = screen.point_px([x as f32 + 0.5, y as f32 + 0.5]);
            out[(y * w + x) as usize] = walk_pixel(&bins, &data, &screen, p)[3];
        }
    }
    out
}

/// **SONDA** — o resíduo do motor NOVO por hardness, na coluna a coluna do
/// `measure_the_convex_tip_residual`. É ela que diz se a ficção da reta era mesmo a causa da
/// ponta convexa (+140/255 no motor de hoje).
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_new_engines_star_residual() {
    let (pts, _) = star_path(7.0);
    println!(
        "\n  pontas do traco: {:?} .. {:?}",
        pts[0],
        pts[pts.len() - 1]
    );
    println!("\n=== MOTOR NOVO: residuo por hardness (estrela de um traco, raio 7) ===");
    println!("  hard   falta  n_falta         onde   sobra  n_sobra         onde");
    for hi in 1..=9 {
        let hardness = hi as f32 / 10.0;
        let (pts, r) = star_path(7.0);
        let got = new_engine_alpha(&pts, r, hardness, W, H);
        let dep = painter_deposit(&pts, r, hardness);
        let (mut lo, mut hi_d, mut nlo, mut nhi) = (0i32, 0i32, 0u32, 0u32);
        let (mut lo_at, mut hi_at) = ((0u32, 0u32), (0u32, 0u32));
        for y in 0..H {
            for x in 0..W {
                if in_the_silhouette_fringe(&pts, r, x, y) {
                    continue;
                }
                let i = (y * W + x) as usize;
                let d = (got[i] * 255.0).round() as i32 - (dep[i] * 255.0).round() as i32;
                if d < -16 {
                    nlo += 1;
                }
                if d > 16 {
                    nhi += 1;
                }
                if d < lo {
                    lo = d;
                    lo_at = (x, y);
                }
                if d > hi_d {
                    hi_d = d;
                    hi_at = (x, y);
                }
            }
        }
        println!(
            "  {hardness:.1}   {lo:+5}   {nlo:5}   {lo_at:>10?}   {hi_d:+5}   {nhi:5}   \
             {hi_at:>10?}"
        );
    }

    // A MESMA medição, cega a um disco de raio `r` em torno das PONTAS do traço. Se a falta some
    // aqui, ela é da PONTA (o cap — passo 5), não da lei.
    println!("\n=== o MESMO, cego a um disco de raio r em torno das PONTAS ===");
    println!("  hard   falta  n_falta         onde");
    for hi in 1..=9 {
        let hardness = hi as f32 / 10.0;
        let (pts, r) = star_path(7.0);
        let got = new_engine_alpha(&pts, r, hardness, W, H);
        let dep = painter_deposit(&pts, r, hardness);
        let ends = [pts[0], pts[pts.len() - 1]];
        let (mut lo, mut nlo, mut lo_at) = (0i32, 0u32, (0u32, 0u32));
        for y in 0..H {
            for x in 0..W {
                if in_the_silhouette_fringe(&pts, r, x, y) {
                    continue;
                }
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                if ends
                    .iter()
                    .any(|e| (p.0 - e.0).powi(2) + (p.1 - e.1).powi(2) <= r * r)
                {
                    continue;
                }
                let i = (y * W + x) as usize;
                let d = (got[i] * 255.0).round() as i32 - (dep[i] * 255.0).round() as i32;
                if d < -16 {
                    nlo += 1;
                }
                if d < lo {
                    lo = d;
                    lo_at = (x, y);
                }
            }
        }
        println!("  {hardness:.1}   {lo:+5}   {nlo:5}   {lo_at:>10?}");
    }
}

/// A estrela como **UM** traço e como **CINCO** — a MESMA geometria que o oráculo do Enio
/// (`measure_the_star_one_stroke_against_separate_strokes`) monta, para o motor novo ser medido
/// na figura DELE e não numa parecida.
/// O que a estrela do oráculo do Enio entrega: o desenho de UM traço, o de CINCO, a polilinha
/// da união e os cantos (que são as PONTAS de perna).
type StarPair = (FlipDrawing, FlipDrawing, Vec<(f32, f32)>, Vec<(f32, f32)>);

fn star_one_and_five(r: f32, hardness: f32) -> StarPair {
    let (cx, cy, outer) = (32.0_f32, 32.0, 26.0);
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
    let mut um = leg(corners[0], corners[1]);
    for w in corners.windows(2).skip(1) {
        um.extend(leg(w[0], w[1]).into_iter().skip(1));
    }
    let mut sep = FlipDrawing::new();
    for w in corners.windows(2) {
        sep.strokes.push(
            flip_drawing(&leg(w[0], w[1]), r, hardness)
                .strokes
                .remove(0),
        );
    }
    (flip_drawing(&um, r, hardness), sep, um, corners)
}

/// **SONDA** — o oráculo do Enio contra o motor NOVO, headless.
///
/// ⚠️ **Aqui a lei aditiva faz uma afirmação ALGÉBRICA, não uma aproximação:** um traço que passa
/// duas vezes soma `τ` e devolve `1 − exp(−2τ₁)`; dois traços compõem por `over` e devolvem
/// `1 − (1 − a)²` com `a = 1 − exp(−τ₁)`. **São a mesma expressão.** O cruzamento de um traço
/// consigo mesmo tem de ficar IDÊNTICO a traços separados — e é exatamente essa igualdade que a
/// união chapada do motor de hoje não pode produzir (medida em `-63/255`).
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_new_engine_on_the_star_one_stroke_against_separate_strokes() {
    println!("\n=== MOTOR NOVO: a ESTRELA como UM traco vs CINCO ===");
    let r = 7.0_f32;
    for hardness in [0.4_f32, 0.7, 1.0] {
        let (d_um, d_sep, _, _) = star_one_and_five(r, hardness);
        let a1 = new_engine_alpha_of(&d_um, W, H);
        let a5 = new_engine_alpha_of(&d_sep, W, H);
        let (mut falta, mut ondef) = (0i32, (0u32, 0u32));
        let (mut sobra, mut ondes) = (0i32, (0u32, 0u32));
        let (mut nf, mut ns) = (0u32, 0u32);
        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let d = (a1[i] * 255.0).round() as i32 - (a5[i] * 255.0).round() as i32;
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

// ————————————————————— os GATES do motor novo (passo 4) —————————————————————

/// 🔴🔴 **O ORÁCULO DO ENIO, E ELE MEDE ZERO.**
///
/// A estrela desenhada **sem levantar a caneta** tem de pintar exatamente o que cinco traços
/// separados pintam. No motor de hoje ela não pinta: a união chapada não escurece no cruzamento,
/// e o oráculo mede **−64/255 em 154 pixels** (`measure_the_star_one_stroke_against_separate_strokes`,
/// h=0,4 — o CONTROLE deste gate).
///
/// No motor novo isto não é uma aproximação boa, é uma **IDENTIDADE**:
///
/// ```text
///   um traço, duas passagens:  α = 1 − exp(−(τ₁+τ₂))
///   dois traços, `over`:       α = 1 − (1−a₁)(1−a₂) = 1 − exp(−τ₁)·exp(−τ₂)
/// ```
///
/// ⚠️ E ela é mais forte do que parece: a integral **não sabe onde um traço termina** — partir o
/// caminho em cinco pedaços é partir o domínio de uma integral, e isso não muda a integral. É por
/// isso que o número é 0 e não "pequeno", e é por isso que **não existe primitivo de JUNÇÃO** a
/// construir para este caso (o cap da PONTA é outra coisa, e tem gate próprio abaixo).
#[test]
fn the_new_engine_makes_a_self_crossing_stroke_equal_separate_strokes() {
    let r = 7.0_f32;
    for hardness in [0.4_f32, 0.7, 1.0] {
        let (d_um, d_sep, um, corners) = star_one_and_five(r, hardness);
        let a1 = new_engine_alpha_of(&d_um, W, H);
        let a5 = new_engine_alpha_of(&d_sep, W, H);
        let (mut pior, mut onde, mut n_miolo) = (0i32, (0u32, 0u32), 0u32);
        let (mut pior_franja, mut n_franja) = (0i32, 0u32);
        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let d = (a1[i] * 255.0).round() as i32 - (a5[i] * 255.0).round() as i32;
                // ⚠️ **As PONTAS DE PERNA saem, e a referência é quem manda.** A §9.5 do doc 12
                // concluiu da mutação D que *"qualquer cap tem de ser invariante à partição"* —
                // e isso está **ERRADO**, refutado pelo próprio oráculo: o depósito do Painter
                // abre cada caminho com um dab em `pts[0]`, então CINCO pernas carimbam cinco
                // dabs de ponta que UM caminho não carimba. Medido no depósito REAL: **−59/255 em
                // 178 px** (dureza 0,4), **−102 em 123** (0,7), **−255 em 17** (1,0) — e sempre
                // NOS CANTOS. A identidade que a §9.1 mediu em ZERO era um artefato de o motor
                // ainda **não ter cap nenhum**; ela vale onde o caminho é o MESMO, que é o que o
                // oráculo do Enio de fato pergunta: o CRUZAMENTO.
                let pc0 = (x as f32 + 0.5, y as f32 + 0.5);
                if corners
                    .iter()
                    .any(|c| (pc0.0 - c.0).powi(2) + (pc0.1 - c.1).powi(2) <= (r + 1.5).powi(2))
                {
                    continue;
                }
                // ⚠️ **Toda silhueta sai, não só a da UNIÃO — e "toda" custou duas rodadas.**
                // CINCO traços têm silhuetas INTERNAS que o traço único não tem: cada perna
                // carrega a sua ao longo do FLANCO inteiro, e onde esse flanco está enterrado
                // dentro da perna vizinha os dois modelos discordam por conflação. Excluir só a
                // franja da união deixou **−3/255 em 8 px**; excluir também um disco nas PONTAS
                // de perna (os cantos) deixou **−3 em 4 px**, porque o ofensor medido — (20, 18)
                // — está a 7,07 px do eixo da perna 4→0, ou seja EM CIMA da silhueta dela, e a
                // 14 px do canto mais próximo. A regra que fecha é a geométrica: **a menos de
                // 1,5 px da silhueta de QUALQUER perna**.
                let pc = (x as f32 + 0.5, y as f32 + 0.5);
                let na_silhueta_de_alguma_perna = corners.windows(2).any(|w| {
                    let (a, b) = (w[0], w[1]);
                    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
                    let l2 = vx * vx + vy * vy;
                    let t = (((pc.0 - a.0) * vx + (pc.1 - a.1) * vy) / l2).clamp(0.0, 1.0);
                    let (dx, dy) = (pc.0 - (a.0 + vx * t), pc.1 - (a.1 + vy * t));
                    ((dx * dx + dy * dy).sqrt() - r).abs() < 1.5
                });
                if na_silhueta_de_alguma_perna || in_the_silhouette_fringe(&um, r, x, y) {
                    if d != 0 {
                        n_franja += 1;
                    }
                    if d.abs() > pior_franja.abs() {
                        pior_franja = d;
                    }
                    continue;
                }
                if d != 0 {
                    n_miolo += 1;
                }
                if d.abs() > pior.abs() {
                    pior = d;
                    onde = (x, y);
                }
            }
        }
        // A LEI: no MIOLO a igualdade é exata, ao byte.
        assert!(
            n_miolo == 0,
            "h={hardness:.1}: um traco que cruza a si mesmo TEM de ser identico a tracos \
             separados -- {n_miolo} px diferem no miolo, pior {pior:+} em {onde:?} (o motor de \
             hoje mede -64 em 154 px, e e' esse o defeito)"
        );
        // ⚠️ **A FRANJA fica FORA, e não é conveniência.** UM e CINCO são cenas DIFERENTES ali:
        // uma silhueta contra cinco, e `over` de dois alfas com AA **não** é o AA da união — é o
        // artefato de conflação, e quem está CERTO é o traço único. Medido: ±1/255 em ≤4 px com
        // pincel macio (onde a queda já fez a borda) e **−51 em 42 px em `hardness = 1`**, onde o
        // AA É a borda inteira. Comparar ali mede convenção de composição, não a lei — a mesma
        // razão pela qual todo oráculo deste arquivo exclui a franja. Quem julga a borda é o
        // `the_new_engines_edge_is_the_area_the_silhouette_covers`, com oráculo de ÁREA.
        let _ = (pior_franja, n_franja);
    }
}

/// A fração do pixel de fato coberta pela **união dura** dos discos, por super-amostragem.
///
/// ⚠️ **Super-amostrar aqui é legítimo, e o §6 do handoff proíbe outra coisa.** A proibição é
/// contra super-amostrar o oráculo do **depósito do Painter** — ali o Painter também avalia a
/// queda no centro do texel, então uma média de área mede uma verdade que **nenhum dos dois**
/// computa. A pergunta deste gate é outra: *que fração do pixel a silhueta cobre?* — e área **é**
/// a definição disso, não uma aproximação melhor dela.
fn union_area_coverage(pts: &[(f32, f32)], r: f32, w: u32, h: u32) -> Vec<f32> {
    const SS: u32 = 16;
    let mut out = vec![0.0_f32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut hits = 0u32;
            for sy in 0..SS {
                for sx in 0..SS {
                    let p = (
                        x as f32 + (sx as f32 + 0.5) / SS as f32,
                        y as f32 + (sy as f32 + 0.5) / SS as f32,
                    );
                    if path_dn(pts, r, p) <= 1.0 {
                        hits += 1;
                    }
                }
            }
            out[(y * w + x) as usize] = f32::from(hits as u16) / (SS * SS) as f32;
        }
    }
    out
}

/// 🔴 **A BORDA É A ÁREA QUE A SILHUETA COBRE** — o ponto cego que os oráculos criaram.
///
/// Todos os gates contra o depósito do Painter **excluem a franja** (`in_the_silhouette_fringe`),
/// e com razão: o depósito não tem AA nenhum. O preço disso é que o motor novo **nunca tinha sido
/// comparado na borda**. Este gate fecha o buraco, e o faz onde ele é MÁXIMO: em `hardness = 1`,
/// onde o perfil é chapado e o `edge` é a borda inteira — e `hardness = 1` é o default e o
/// CONTROLE de todos os smokes (§8 do handoff).
///
/// A geometria é escolhida para não ser o caso fácil e para isolar UMA pergunta: um traço em
/// **30°** (uma borda alinhada aos eixos é exata em qualquer filtro-caixa e esconderia o erro),
/// com as **duas pontas FORA da tela**.
///
/// ⚠️ **As pontas ficam fora de propósito, e a 1ª versão deste gate provou por quê:** com elas
/// dentro, o oráculo de união inclui o **cap redondo** que o motor ainda não tem, e o gate mediu
/// **−156/255 em (55, 33)** — logo além do fim do traço. Isso é o item do passo 5
/// (`the_new_engines_deficit_is_the_endpoint_and_the_corner_...`), não o AA; um fixture que mistura
/// os dois reporta o cap com o nome da borda.
#[test]
fn the_new_engines_edge_is_the_area_the_silhouette_covers() {
    let r = 7.0_f32;
    let ang = 30.0_f32.to_radians();
    let pts: Vec<(f32, f32)> = (-20..=60)
        .map(|k| {
            let t = k as f32 * 2.0;
            (-20.0 + t * ang.cos(), -20.0 + t * ang.sin())
        })
        .collect();
    let got = new_engine_alpha(&pts, r, 1.0, W, H);
    let area = union_area_coverage(&pts, r, W, H);
    let (mut pior, mut onde, mut n) = (0i32, (0u32, 0u32), 0u32);
    for y in 0..H {
        for x in 0..W {
            let i = (y * W + x) as usize;
            // Só a BORDA: no miolo e no papel nu os dois são 1 e 0 por construção, e incluí-los
            // diluiria a média até um gate que não pode falhar.
            if area[i] <= 0.001 || area[i] >= 0.999 {
                continue;
            }
            let d = (got[i] * 255.0).round() as i32 - (area[i] * 255.0).round() as i32;
            if d.abs() > 24 {
                n += 1;
            }
            if d.abs() > pior.abs() {
                pior = d;
                onde = (x, y);
            }
        }
    }
    assert!(
        pior.abs() <= 16 && n == 0,
        "a borda tem de ser a AREA que a silhueta cobre: pior {pior:+} em {onde:?}, {n} px \
         fora de 24"
    );
}

/// 🔴🔴 **O CONTROLE INEGOCIÁVEL DO §8: `hardness = 1.0` byte-idêntico.**
///
/// `DEFAULT_HARDNESS = 1.0` é o default do produto e **todo o acervo já desenhado passa por ele**
/// — o handoff §8 exige que o traço duro fique byte-idêntico *ou venha a medição que justifique a
/// diferença*. Este gate compara o motor novo com o que SHIPA, no pior lugar possível: a estrela
/// de um traço, que **cruza a si mesma cinco vezes** e tem cinco quinas de 36°.
///
/// ⚠️ **Aqui a franja NÃO é excluída.** Nos oráculos contra o Painter ela sai porque o depósito
/// não tem AA nenhum; aqui os DOIS lados têm AA, e a borda é justamente metade do que "idêntico"
/// significa. É o §11 que tornou esta comparação possível.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn the_new_engine_leaves_the_hard_default_where_the_shipping_engine_put_it() {
    let Some((device, queue)) = device() else {
        return;
    };
    for (reto, nome, pts) in [
        (
            true,
            "reto",
            (0..=24)
                .map(|k| (8.0 + k as f32 * 2.0, 32.0))
                .collect::<Vec<_>>(),
        ),
        (false, "estrela (5 cruzamentos)", star_path(7.0).0),
    ] {
        let r = 7.0_f32;
        let px = render(&device, &queue, &flip_drawing(&pts, r, 1.0));
        let got = new_engine_alpha(&pts, r, 1.0, W, H);
        let ends = [pts[0], pts[pts.len() - 1]];
        let (mut pior, mut onde, mut n) = (0i32, (0u32, 0u32), 0u32);
        let (mut pior_cap, mut n_cap) = (0i32, 0u32);
        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let d = (got[i] * 255.0).round() as i32 - i32::from(alpha_at(&px, x, y));
                let pc = (x as f32 + 0.5, y as f32 + 0.5);
                // ⚠️ O CAP fica separado, não perdoado: o que shipa desenha ponta REDONDA e o
                // motor novo ainda não desenha nenhuma, então incluí-lo aqui reportaria o item do
                // passo 5 com o nome da lei.
                if ends
                    .iter()
                    .any(|e| (pc.0 - e.0).powi(2) + (pc.1 - e.1).powi(2) <= (r + 1.5).powi(2))
                {
                    if d.abs() > 8 {
                        n_cap += 1;
                    }
                    if d.abs() > pior_cap.abs() {
                        pior_cap = d;
                    }
                    continue;
                }
                if d.abs() > 8 {
                    n += 1;
                }
                if d.abs() > pior.abs() {
                    pior = d;
                    onde = (x, y);
                }
            }
        }
        println!("  {nome:24} corpo {pior:+5} ({n:3} px)   CAP {pior_cap:+5} ({n_cap:3} px)");
        // Num traço RETO o corpo é BYTE-IDÊNTICO — é ali que "não mexer no acervo" se verifica.
        if reto {
            assert!(
                pior == 0 && n == 0,
                "reto: em hardness 1 o corpo tem de ser byte-identico ao que shipa -- pior \
                 {pior:+} em {onde:?}, {n} px fora de 8 (o cap mede {pior_cap:+} em {n_cap} px e \
                 e' o passo 5)"
            );
            continue;
        }
        // ⚠️ **Numa figura que CRUZA, eles divergem — e o §8 do handoff prevê exatamente isto:**
        // *"byte-idêntico, **ou** trazer a medição que justifique a diferença"*. A medição é o
        // ÁRBITRO: a área que a união dura de fato cobre. Onde os dois discordam por mais de
        // 8/255, o motor novo tem de estar MAIS PERTO dela — senão a diferença não é melhoria,
        // é regressão vestida de melhoria.
        let area = union_area_coverage(&pts, r, W, H);
        let (mut novo, mut shipa, mut som_n, mut som_s) = (0u32, 0u32, 0i64, 0i64);
        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let a = (got[i] * 255.0).round() as i32;
                let sh = i32::from(alpha_at(&px, x, y));
                let pc = (x as f32 + 0.5, y as f32 + 0.5);
                let no_cap = ends
                    .iter()
                    .any(|e| (pc.0 - e.0).powi(2) + (pc.1 - e.1).powi(2) <= (r + 1.5).powi(2));
                if (a - sh).abs() <= 8 || no_cap {
                    continue;
                }
                let t = (area[i] * 255.0).round() as i32;
                som_n += i64::from((a - t).abs());
                som_s += i64::from((sh - t).abs());
                if (a - t).abs() < (sh - t).abs() {
                    novo += 1;
                } else {
                    shipa += 1;
                }
            }
        }
        let (mn, ms) = (
            som_n as f64 / f64::from((novo + shipa).max(1)),
            som_s as f64 / f64::from((novo + shipa).max(1)),
        );
        println!("  {nome:24} arbitro: NOVO ganha {novo}, SHIPA {shipa} | erro {mn:.1} vs {ms:.1}");
        assert!(
            novo >= shipa * 8 && mn * 2.0 < ms,
            "{nome}: onde os dois discordam o motor novo tem de estar mais perto da AREA -- \
             NOVO ganha {novo}, SHIPA {shipa}; erro medio NOVO {mn:.1} vs SHIPA {ms:.1}"
        );
    }
}

/// 🔴 **A PONTA CONVEXA: +140/255 → +14.**
///
/// O `the_flip_paints_what_the_painters_digital_brush_deposits` (o gate do motor de hoje) precisa
/// admitir **+140/255** de tinta A MAIS no vértice de 36°, e o doc-comment dele nomeia a causa
/// como *"o perfil de traço os superestima"*. A causa REAL é a ficção da reta: o
/// `hardness_mask` soma a densidade ao longo de uma fileira **infinita**, e numa ponta o caminho
/// de verdade **acaba** — a ficção continua depositando onde não há mais traço.
///
/// Integrando sobre o caminho que existe, o excedente colapsa. Medido
/// (`measure_the_new_engines_star_residual`, sobra por hardness 0,1..0,9):
///
/// ```text
///   +5  +6  +7  +8  +9  +11  +14  +13  +0      ← e `n_sobra` = 0 em TODAS
/// ```
#[test]
fn the_new_engine_has_no_convex_tip_overshoot() {
    for hi in 1..=9 {
        let hardness = hi as f32 / 10.0;
        let (pts, r) = star_path(7.0);
        let got = new_engine_alpha(&pts, r, hardness, W, H);
        let dep = painter_deposit(&pts, r, hardness);
        let (mut sobra, mut onde, mut n) = (0i32, (0u32, 0u32), 0u32);
        for y in 0..H {
            for x in 0..W {
                if in_the_silhouette_fringe(&pts, r, x, y) {
                    continue;
                }
                let i = (y * W + x) as usize;
                let d = (got[i] * 255.0).round() as i32 - (dep[i] * 255.0).round() as i32;
                if d > 16 {
                    n += 1;
                }
                if d > sobra {
                    sobra = d;
                    onde = (x, y);
                }
            }
        }
        assert!(
            sobra <= 16 && n == 0,
            "h={hardness:.1}: sobra {sobra} em {onde:?} ({n} px > 16) -- o motor de hoje mede \
             +140 aqui, e a ficcao da reta era a causa"
        );
    }
}

/// ⚠️ **O QUE SOBRA DEPOIS DO CAP — e a PONTA não é mais parte disso.**
///
/// Este gate nasceu afirmando o contrário: que o déficit contra o depósito do Painter era
/// dominado pela **PONTA** (−36 no total contra −27 cego a ela), com a terceira asserção servindo
/// de **tripwire** — *"no dia em que o passo 5 fechar o cap, ela fica vermelha pedindo os números
/// novos"*. O cap fechou (o meio dab de fronteira, `tau::end_dab`), a tripwire disparou, e o gate
/// se **inverte**: agora ele pina que a ponta NÃO contribui mais.
///
/// | | pior | n |
/// |---|---|---|
/// | tudo | **−27** | ≤3 px |
/// | cego a um disco `r` nas PONTAS | **−27** | ≤3 px — **o MESMO** |
///
/// O que sobra é a **QUINA CONVEXA**, e só ela: o Painter compõe **dabs discretos** a
/// `0,1·diâmetro`, a integral é o limite denso dessa mesma composição, e numa quina de 36° a
/// discretização dele deposita um pouco mais. ⚠️ **NÃO é a quadratura, e isto foi MEDIDO:**
/// subindo `SUB` de 4 para 16 o número anda ≤1/255 (−20→−19 · −24→−24 · −27→−26); **abaixo** de 4
/// ele piora (SUB=2 leva a −30 em 11 px), e é por isso que 4 é o joelho. Casar isto exigiria
/// reproduzir a discreteza do Painter, que é **outro motor** (o candidato C1, o buffer de dabs).
#[test]
fn the_new_engines_only_deficit_is_the_convex_corner_and_this_is_its_number() {
    let mut pior_tudo = 0i32;
    let mut pior_cego = 0i32;
    let (mut n_tudo, mut n_cego) = (0u32, 0u32);
    for hi in 1..=9 {
        let hardness = hi as f32 / 10.0;
        let (pts, r) = star_path(7.0);
        let got = new_engine_alpha(&pts, r, hardness, W, H);
        let dep = painter_deposit(&pts, r, hardness);
        let ends = [pts[0], pts[pts.len() - 1]];
        for y in 0..H {
            for x in 0..W {
                if in_the_silhouette_fringe(&pts, r, x, y) {
                    continue;
                }
                let i = (y * W + x) as usize;
                let d = (got[i] * 255.0).round() as i32 - (dep[i] * 255.0).round() as i32;
                pior_tudo = pior_tudo.min(d);
                if d < -16 {
                    n_tudo += 1;
                }
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                if !ends
                    .iter()
                    .any(|e| (p.0 - e.0).powi(2) + (p.1 - e.1).powi(2) <= r * r)
                {
                    pior_cego = pior_cego.min(d);
                    if d < -16 {
                        n_cego += 1;
                    }
                }
            }
        }
    }
    assert!(
        pior_tudo >= -30 && n_tudo <= 8,
        "o deficit da quina convexa passou do medido (-27 em <=3 px): {pior_tudo} em {n_tudo} px"
    );
    // ⚠️ **A INVERSÃO:** antes o gate exigia que a ponta DOMINASSE; agora exige que ela seja
    // INDISTINGUÍVEL. Esconder as pontas não pode mudar nada — é isso que "o cap fechou"
    // significa em número, e é o que fica vermelho se alguém desfizer o `end_dab`.
    assert!(
        pior_tudo == pior_cego && n_tudo == n_cego,
        "esconder as PONTAS nao pode mais mudar o deficit (o cap fechou): tudo {pior_tudo}/{n_tudo} \
         contra cego {pior_cego}/{n_cego}"
    );
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

/// **SONDA** — uma janela dos dois motores lado a lado, para um desvio nomeado parar de ser teoria.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_two_engines_side_by_side_in_a_window() {
    let Some((device, queue)) = device() else {
        return;
    };
    let (pts, r) = star_path(7.0);
    let px = render(&device, &queue, &flip_drawing(&pts, r, 1.0));
    let got = new_engine_alpha(&pts, r, 1.0, W, H);
    // ⚠️ O ARBITRO e' a AREA da uniao dura -- comparar os dois motores entre si so' diz que eles
    // discordam, nunca QUEM esta' errado.
    let area = union_area_coverage(&pts, r, W, H);
    println!("\n=== janela em torno de (11,57), hardness 1 ===");
    println!(
        "      SHIPA (o que shipa)         |   NOVO                        |   AREA (arbitro)"
    );
    for y in 52..62 {
        let (mut a, mut b, mut c) = (String::new(), String::new(), String::new());
        for x in 8..18 {
            a.push_str(&format!("{:4}", alpha_at(&px, x, y)));
            b.push_str(&format!(
                "{:4}",
                (got[(y * W + x) as usize] * 255.0).round() as i32
            ));
            c.push_str(&format!(
                "{:4}",
                (area[(y * W + x) as usize] * 255.0).round() as i32
            ));
        }
        println!("  y={y:2} |{a}  |{b}  |{c}");
    }
}

/// **SONDA** — o ladrilho de um pixel tem o segmento que o cobre?
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_whether_the_tile_holds_the_segment_that_covers_the_pixel() {
    let (pts, r) = star_path(7.0);
    let data = pack_drawing(&flip_drawing(&pts, r, 1.0));
    let sc = ScreenSpace::from_camera(&pixel_camera_sized(W, H));
    let bins = bin_segments(&data, &sc, DEFAULT_TILE);
    println!("\n=== o ladrilho tem o segmento que cobre? ===");
    println!("  pixel(mundo)   tile   n_segs   min(d-r) NA LISTA   min(d-r) EM TUDO");
    for (x, y) in [(11u32, 57u32), (12, 58), (12, 57), (11, 54)] {
        let p = sc.point_px([x as f32 + 0.5, y as f32 + 0.5]);
        let ti = bins.tile_of_pixel(p[0], p[1]).expect("tile");
        let lista = bins.segs_of(ti);
        let sd_de = |segs: &[ph2d_flip_render::BinSeg]| -> f32 {
            let mut best = f32::MAX;
            for s in segs {
                let (pa, pb) = (data.points[s.a as usize], data.points[s.b as usize]);
                let (sa, sb) = (sc.point_px(pa.pos), sc.point_px(pb.pos));
                let (vx, vy) = (sb[0] - sa[0], sb[1] - sa[1]);
                let l2 = vx * vx + vy * vy;
                let t = if l2 <= 1e-12 {
                    0.0
                } else {
                    (((p[0] - sa[0]) * vx + (p[1] - sa[1]) * vy) / l2).clamp(0.0, 1.0)
                };
                let (dx, dy) = (p[0] - (sa[0] + vx * t), p[1] - (sa[1] + vy * t));
                let rr = sc.radius_px(pa.width) * (1.0 - t) + sc.radius_px(pb.width) * t;
                best = best.min((dx * dx + dy * dy).sqrt() - rr);
            }
            best
        };
        let todos: Vec<ph2d_flip_render::BinSeg> = bins.segs.clone();
        println!(
            "  ({x:2},{y:2})        {ti:3}   {:6}   {:17.3}   {:17.3}",
            lista.len(),
            sd_de(lista),
            sd_de(&todos)
        );
    }
}

/// **SONDA** — onde os dois motores discordam, QUEM está mais perto da área?
///
/// Comparar dois motores entre si só diz que eles discordam. O árbitro é a **área da união dura**,
/// que em `hardness = 1` é o que a tinta de fato cobre.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_who_is_closer_to_the_truth_where_the_engines_disagree() {
    let Some((device, queue)) = device() else {
        return;
    };
    let (pts, r) = star_path(7.0);
    let px = render(&device, &queue, &flip_drawing(&pts, r, 1.0));
    let got = new_engine_alpha(&pts, r, 1.0, W, H);
    let area = union_area_coverage(&pts, r, W, H);
    let ends = [pts[0], pts[pts.len() - 1]];
    let (mut novo_ganha, mut shipa_ganha, mut empate) = (0u32, 0u32, 0u32);
    let (mut som_novo, mut som_shipa) = (0i64, 0i64);
    let (mut cap_px, mut corpo_px) = (0u32, 0u32);
    let (mut cap_novo, mut cap_shipa, mut cap_ganha) = (0i64, 0i64, 0u32);
    for y in 0..H {
        for x in 0..W {
            let i = (y * W + x) as usize;
            let a = (got[i] * 255.0).round() as i32;
            let s = i32::from(alpha_at(&px, x, y));
            if (a - s).abs() <= 8 {
                continue;
            }
            let pc = (x as f32 + 0.5, y as f32 + 0.5);
            if ends
                .iter()
                .any(|e| (pc.0 - e.0).powi(2) + (pc.1 - e.1).powi(2) <= (r + 1.5).powi(2))
            {
                cap_px += 1;
                let t = (area[i] * 255.0).round() as i32;
                cap_novo += i64::from((a - t).abs());
                cap_shipa += i64::from((s - t).abs());
                if (a - t).abs() < (s - t).abs() {
                    cap_ganha += 1;
                }
                continue;
            }
            corpo_px += 1;
            let t = (area[i] * 255.0).round() as i32;
            let (en, es) = ((a - t).abs(), (s - t).abs());
            som_novo += i64::from(en);
            som_shipa += i64::from(es);
            match en.cmp(&es) {
                std::cmp::Ordering::Less => novo_ganha += 1,
                std::cmp::Ordering::Greater => shipa_ganha += 1,
                std::cmp::Ordering::Equal => empate += 1,
            }
        }
    }
    println!("\n=== ONDE OS DOIS DISCORDAM (>8/255), quem esta' mais perto da AREA? ===");
    println!("  pixels no corpo: {corpo_px}   (no cap, excluidos: {cap_px})");
    println!(
        "  NOVO mais perto: {novo_ganha}   SHIPA mais perto: {shipa_ganha}   empate: {empate}"
    );
    println!(
        "  erro MEDIO contra a area -- NOVO {:.1}/255   SHIPA {:.1}/255",
        som_novo as f64 / f64::from(corpo_px.max(1)),
        som_shipa as f64 / f64::from(corpo_px.max(1))
    );
    println!(
        "  NO CAP ({cap_px} px): NOVO ganha {cap_ganha} | erro NOVO {:.1}/255  SHIPA {:.1}/255",
        cap_novo as f64 / f64::from(cap_px.max(1)),
        cap_shipa as f64 / f64::from(cap_px.max(1))
    );
}

/// **SONDA** — o que EXATAMENTE falta na ponta, por dureza e contra o árbitro certo.
///
/// Em `hardness = 1` o árbitro é a ÁREA (o que a união dura cobre). Em dureza macia é o DEPÓSITO
/// do Painter, que carimba um dab no primeiro ponto — e é ali que a integral, que acaba com o
/// caminho, pode ficar devendo.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_what_the_cap_actually_owes() {
    let r = 7.0_f32;
    let pts: Vec<(f32, f32)> = (0..=20).map(|k| (20.0 + k as f32 * 1.5, 32.0)).collect();
    println!("\n=== O QUE A PONTA DEVE (traco reto, r={r}) ===");
    println!("  ⚠️ as DUAS pontas, separadas: o Painter carimba um dab no PRIMEIRO ponto e o");
    println!("     percurso dele acaba ANTES do ultimo -- as duas nao sao simetricas.");
    println!("  hardness   arbitro          INICIO(pior/n/med)      FIM(pior/n/med)");
    for hi in [10_u32, 4, 7, 2] {
        let hardness = hi as f32 / 10.0;
        let got = new_engine_alpha(&pts, r, hardness, W, H);
        let (nome, want) = if hi == 10 {
            ("AREA          ", union_area_coverage(&pts, r, W, H))
        } else {
            ("deposito Pintr", painter_deposit(&pts, r, hardness))
        };
        let mut linha = String::new();
        for (ini, ponta) in [(true, pts[0]), (false, pts[pts.len() - 1])] {
            let (mut pior, mut n, mut som, mut cnt) = (0i32, 0u32, 0i64, 0u32);
            for y in 0..H {
                for x in 0..W {
                    let pc = (x as f32 + 0.5, y as f32 + 0.5);
                    // Só o disco da PONTA, e só a metade que fica ALÉM dela (a região de cap).
                    let d2 = (pc.0 - ponta.0).powi(2) + (pc.1 - ponta.1).powi(2);
                    let alem = if ini { pc.0 < ponta.0 } else { pc.0 > ponta.0 };
                    if d2 > (r + 1.0).powi(2) || !alem {
                        continue;
                    }
                    let i = (y * W + x) as usize;
                    let d = (got[i] * 255.0).round() as i32 - (want[i] * 255.0).round() as i32;
                    if d.abs() > 16 {
                        n += 1;
                    }
                    if d.abs() > pior.abs() {
                        pior = d;
                    }
                    som += i64::from(d.abs());
                    cnt += 1;
                }
            }
            linha.push_str(&format!(
                "  {pior:+5} {n:3} {:5.1}  ",
                som as f64 / f64::from(cnt.max(1))
            ));
        }
        println!("  {hardness:.1}        {nome} {linha}");
    }
}

/// **SONDA** — o DEPÓSITO DO PAINTER é invariante à partição do caminho?
///
/// A §9.5 do doc 12 concluiu, da mutação D, que *"qualquer primitivo de cap tem de ser invariante
/// à partição"*. A conclusão veio de uma identidade que o motor exibia **por não ter cap nenhum**.
/// Esta sonda pergunta ao ÁRBITRO: o depósito do Painter — a referência — satisfaz a identidade?
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_whether_the_painters_own_deposit_is_partition_invariant() {
    let r = 7.0_f32;
    println!("\n=== O DEPOSITO DO PAINTER: UM caminho vs CINCO (a referencia e' invariante?) ===");
    println!("  hardness   pior   n(>8)   onde");
    for hardness in [0.4_f32, 0.7, 1.0] {
        let (_, _, um, corners) = star_one_and_five(r, hardness);
        let d1 = painter_deposit(&um, r, hardness);
        // CINCO caminhos separados, compostos por `over` — como o oráculo do Enio os desenha.
        let mut d5 = vec![0.0_f32; (W * H) as usize];
        for w in corners.windows(2) {
            let len = ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt();
            let n = (len / (0.8 * r)).ceil().max(1.0) as usize;
            let perna: Vec<(f32, f32)> = (0..=n)
                .map(|k| {
                    let t = k as f32 / n as f32;
                    (
                        w[0].0 + (w[1].0 - w[0].0) * t,
                        w[0].1 + (w[1].1 - w[0].1) * t,
                    )
                })
                .collect();
            let dk = painter_deposit(&perna, r, hardness);
            for (o, v) in d5.iter_mut().zip(&dk) {
                *o = 1.0 - (1.0 - *o) * (1.0 - v);
            }
        }
        let (mut pior, mut onde, mut n) = (0i32, (0u32, 0u32), 0u32);
        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let d = (d1[i] * 255.0).round() as i32 - (d5[i] * 255.0).round() as i32;
                if d.abs() > 8 {
                    n += 1;
                }
                if d.abs() > pior.abs() {
                    pior = d;
                    onde = (x, y);
                }
            }
        }
        println!("  {hardness:.1}       {pior:+5}   {n:5}   {onde:?}");
    }
}

/// 🖼️🖼️ **O QUADRO DO VEREDITO** — os TRÊS motores na figura da queixa, lado a lado.
///
/// `PAINTER (a referência) | FLIP que SHIPA | FLIP NOVO | a diferença NOVO−PAINTER`
///
/// A figura é a estrela de **um traço** desenhada com a **mão LENTA** (o passo mínimo que o
/// pipeline de autoria produz, `0,106·r` — o lado da cerca em que o defeito vive), com dureza
/// baixa: exatamente o gesto que o handoff §7 nomeia como o julgamento final.
///
/// ⚠️ **Isto não substitui o smoke no app** — substitui *discutir sobre números*. O veredito é do
/// Enio, e ele precisa de pixels.
///
/// ```text
/// cargo test -p ph2d-flip-render --release --test painter_look render_the_verdict -- --ignored
/// ```
#[test]
#[ignore = "sonda de imagem; roda com --ignored"]
fn render_the_verdict_three_engines_side_by_side() {
    let Some((device, queue)) = device() else {
        return;
    };
    let dir = std::path::Path::new("/home/enio/flip_veredito");
    std::fs::create_dir_all(dir).expect("criar diretorio");
    const S: u32 = 640;
    // ⚠️ **A PROPORÇÃO é parte da fixture, e a 1ª versão desta sonda a errou.** O defeito foi
    // medido numa estrela de raio 26 com traço `r = 7` — razão **0,27**. Com `r = 26` sobre raio
    // 250 (razão 0,10) as três colunas saem indistinguíveis e o diff sai preto: a imagem diz
    // *"está tudo bem"* sobre um desenho que **não contém o fenômeno**. `r = 67` sobre 250
    // reproduz a razão de onde a cunha escura vive.
    for (nome, r, hardness) in [
        ("h0.2", 67.0_f32, 0.2_f32),
        ("h0.4", 67.0, 0.4),
        ("h0.7", 67.0, 0.7),
        ("h1.0_controle", 67.0, 1.0),
    ] {
        let (cx, cy, outer) = (S as f32 * 0.5, S as f32 * 0.52, 250.0_f32);
        let mut corners: Vec<(f32, f32)> = (0..5)
            .map(|k| {
                let a =
                    -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
                (cx + outer * a.cos(), cy + outer * a.sin())
            })
            .collect();
        corners.push(corners[0]);
        // MÃO LENTA: o passo mínimo que a autoria de fato entrega (doc do `sampling_invariance`).
        let mut pts = vec![corners[0]];
        for w in corners.windows(2) {
            let (a, b) = (w[0], w[1]);
            let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (len / (0.106 * r)).ceil().max(1.0) as usize;
            for k in 1..=n {
                let t = k as f32 / n as f32;
                pts.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        let velho = render_sized(&device, &queue, &flip_drawing(&pts, r, hardness), S, S);
        let novo = new_engine_alpha(&pts, r, hardness, S, S);
        let dep = painter_deposit_sized(&pts, r, hardness, S, S);
        let cols = 4;
        let mut img = vec![0u8; (S * cols * S * 3) as usize];
        for y in 0..S {
            for x in 0..S {
                let i = (y * S + x) as usize;
                let a_velho = f32::from(velho[i * 4 + 3]) / 255.0;
                let a_novo = novo[i];
                let a_dep = dep[i];
                let put = |img: &mut [u8], col: u32, rgb: [u8; 3]| {
                    let o = ((y * S * cols + col * S + x) * 3) as usize;
                    img[o..o + 3].copy_from_slice(&rgb);
                };
                put(&mut img, 0, over_dark(a_dep));
                put(&mut img, 1, over_dark(a_velho));
                put(&mut img, 2, over_dark(a_novo));
                let d = (a_novo - a_dep) * 4.0;
                put(
                    &mut img,
                    3,
                    [
                        ((-d).clamp(0.0, 1.0) * 255.0) as u8,
                        (d.clamp(0.0, 1.0) * 255.0) as u8,
                        30,
                    ],
                );
            }
            for k in 1..cols {
                let o = ((y * S * cols + k * S) * 3) as usize;
                img[o..o + 3].copy_from_slice(&[200, 60, 60]);
            }
        }
        let nome_arq = format!("PAINTER__SHIPA__NOVO__DIFF_{nome}.bmp");
        write_bmp(&dir.join(&nome_arq), S * cols, S, &img);

        // ⚠️ **E o RECORTE da ponta, ampliado** — sem ele o artefato não mostra o defeito. A tela
        // inteira faz as três estrelas parecerem iguais (a cunha é local e a figura é grande), e
        // foi exatamente isso que a 1ª rodada desta sonda produziu: uma imagem dizendo *"está tudo
        // bem"*. A ampliação é NEAREST de propósito — um filtro suave inventaria a borda que a
        // comparação existe para julgar.
        const CW: u32 = 260;
        const CH: u32 = 210;
        const Z: u32 = 3;
        let (ox, oy) = (S / 2 - CW / 2, 6);
        let mut zoom = vec![0u8; (CW * 3 * Z * CH * Z * 3) as usize];
        for zy in 0..CH * Z {
            for col in 0..3u32 {
                for zx in 0..CW * Z {
                    let sx = ox + zx / Z + col * S;
                    let sy = oy + zy / Z;
                    let src = ((sy * S * cols + sx) * 3) as usize;
                    let dst = ((zy * CW * 3 * Z + col * CW * Z + zx) * 3) as usize;
                    zoom[dst..dst + 3].copy_from_slice(&img[src..src + 3]);
                }
            }
            for k in 1..3u32 {
                let d = ((zy * CW * 3 * Z + k * CW * Z) * 3) as usize;
                zoom[d..d + 3].copy_from_slice(&[200, 60, 60]);
            }
        }
        write_bmp(
            &dir.join(format!("PONTA_ampliada_{nome}.bmp")),
            CW * 3 * Z,
            CH * Z,
            &zoom,
        );
        // O número ao lado da imagem: quem está mais perto da referência.
        // ⚠️ **O PICO, não a média.** A queixa do Enio é uma CUNHA — um defeito local sobre uma
        // tela quase toda vazia —, e a média sobre 640² a dilui até parecer que não existe.
        let (mut pv, mut pn) = (0i32, 0i32);
        for y in 0..S {
            for x in 0..S {
                if in_the_silhouette_fringe(&pts, r, x, y) {
                    continue;
                }
                let i = (y * S + x) as usize;
                let t = (dep[i] * 255.0).round() as i32;
                let dv = i32::from(velho[i * 4 + 3]) - t;
                let dn = (novo[i] * 255.0).round() as i32 - t;
                if dv.abs() > pv.abs() {
                    pv = dv;
                }
                if dn.abs() > pn.abs() {
                    pn = dn;
                }
            }
        }
        println!("  {nome_arq}   PICO contra o PAINTER -- SHIPA {pv:+5}/255   NOVO {pn:+5}/255");
    }
    println!("\n  escrito em {}", dir.display());
}

/// 📏 **SONDA — o resíduo da estrela medido no PERCURSO, não no rasterizador.**
///
/// ⚠️ **Ela existe porque o número que motiva o item 4 da fila do padrão-ouro descreve o produto
/// ERRADO.** O `−64 (58 px em h=1,0)` de `measure_the_star_one_stroke_against_separate_strokes`
/// sai do `FlipRenderer`, ou seja do **rasterizador** — que desde `9a4bdd07b` é a *escape hatch*,
/// não o default. Ler dele que *"o resíduo sobrevive à dureza máxima ⇒ é geometria de junta"* é
/// uma conclusão sobre um motor que o artista não usa mais.
///
/// Aqui a MESMA cena roda pelo `walk_pixel` (a referência do percurso, byte-paridade com o device
/// pelo `walk_gpu_parity`), nos dois modos: um traço com quina contra cinco traços que empilham
/// duas tampas em cada ponta.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_star_residual_on_the_walk_not_the_raster() {
    let (pts, r) = star_path(7.0);
    let sc = ph2d_flip_render::ScreenSpace {
        world_to_clip: [
            [2.0 / W as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0 / H as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0],
        ],
        viewport: [W as f32, H as f32],
        px_per_world: 1.0,
    };
    println!("\n=== A ESTRELA no PERCURSO: um traco vs cinco ===");
    for hardness in [0.4_f32, 0.7, 1.0] {
        let um = flip_drawing(&pts, r, hardness);
        // Cinco traços: cada perna da estrela vira um traço próprio, com as duas tampas dela.
        let mut cinco = FlipDrawing::default();
        let passo = (pts.len() - 1) / 5;
        for k in 0..5 {
            let fatia: Vec<(f32, f32)> =
                pts[k * passo..=((k + 1) * passo).min(pts.len() - 1)].to_vec();
            let d = flip_drawing(&fatia, r, hardness);
            cinco.strokes.extend(d.strokes);
        }
        let alpha = |d: &FlipDrawing| {
            let g = ph2d_flip_render::pack_drawing(d);
            let bins = ph2d_flip_render::bin_segments(&g, &sc, ph2d_flip_render::DEFAULT_TILE);
            let mut out = vec![0.0_f32; (W * H) as usize];
            for y in 0..H {
                for x in 0..W {
                    let p = [x as f32 + 0.5, y as f32 + 0.5];
                    out[(y * W + x) as usize] = ph2d_flip_render::walk_pixel(&bins, &g, &sc, p)[3];
                }
            }
            out
        };
        let (a, b) = (alpha(&um), alpha(&cinco));
        let (mut falta, mut falta_em, mut sobra, mut sobra_em, mut n_falta) =
            (0_i32, (0_u32, 0_u32), 0_i32, (0_u32, 0_u32), 0_u32);
        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let d = ((a[i] - b[i]) * 255.0).round() as i32;
                if d < falta {
                    falta = d;
                    falta_em = (x, y);
                }
                if d > sobra {
                    sobra = d;
                    sobra_em = (x, y);
                }
                if d < -8 {
                    n_falta += 1;
                }
            }
        }
        let i = (falta_em.1 * W + falta_em.0) as usize;
        // ⚠️ **As duas metades do mecanismo, no pixel.** Um traço computa a ÁREA EXATA da união
        // (o que a wave da §22.7 estabeleceu, com gate); cinco traços são compostos `over`, que é
        // `a + b − ab` — a união PROBABILÍSTICA, que só coincide com a geométrica quando os dois
        // são disjuntos. Nas pontas da estrela eles se sobrepõem, então divergir é obrigatório.
        println!(
            "  h={hardness}: FALTA {falta:5} em {falta_em:?} ({n_falta} px < -8)   SOBRA {sobra:5} \
             em {sobra_em:?}   |  um={:.4} cinco={:.4}",
            a[i], b[i]
        );
    }
}

/// 📏 **SONDA — despeja o campo de alfa do PERCURSO num arquivo, para um A/B entre duas versões
/// do motor.** `PH2D_WALK_DUMP=<arquivo>` diz onde.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn dump_the_walk_alpha_field() {
    let Ok(dest) = std::env::var("PH2D_WALK_DUMP") else {
        println!("sem PH2D_WALK_DUMP");
        return;
    };
    let (pts, r) = star_path(7.0);
    let sc = ph2d_flip_render::ScreenSpace {
        world_to_clip: [
            [2.0 / W as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0 / H as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0],
        ],
        viewport: [W as f32, H as f32],
        px_per_world: 1.0,
    };
    let mut out = String::new();
    for hardness in [0.4_f32, 0.7, 1.0] {
        let g = ph2d_flip_render::pack_drawing(&flip_drawing(&pts, r, hardness));
        let bins = ph2d_flip_render::bin_segments(&g, &sc, ph2d_flip_render::DEFAULT_TILE);
        for y in 0..H {
            for x in 0..W {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                let a = ph2d_flip_render::walk_pixel(&bins, &g, &sc, p)[3];
                out.push_str(&format!("{a:.6}\n"));
            }
        }
    }
    std::fs::write(&dest, out).expect("escreve");
    println!("despejado em {dest}");
}
