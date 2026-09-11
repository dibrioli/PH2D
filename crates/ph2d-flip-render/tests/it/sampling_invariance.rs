//! **A TINTA É FATO DO CAMINHO, NUNCA DE QUÃO FINO O MOTOR AMOSTROU O CAMINHO.**
//!
//! É a mesma lei que esta linha já pinou quatro vezes no relevo do Painter (a cápsula do
//! depósito · a mordida do arado · o aro · o gate de proteção), agora no rasterizador do Flip.
//!
//! **O defeito que este arquivo existe para pegar** (2026-07-28, medido): o alcance de influência
//! de um segmento é `≈ 3 × raio`, e a lista de vizinhos que o fragment recebe era capeada por
//! **CONTAGEM** (`MAX_RIBBON_EXTRAS`). Contagem necessária = `alcance / passo`, então quando a
//! polilinha fica mais densa que `3·r / 16 = 0,1875·r` a lista **trunca**, o pixel volta ao
//! first-wins do Grease Pencil e a cauda macia de um quad é pintada sobre o NÚCLEO do vizinho.
//!
//! Medido no penhasco, contra o depósito REAL do Painter, numa estrela de um traço:
//!
//! | passo / raio | falta de tinta | px fora de 16 |
//! |---|---|---|
//! | 0,80 | −4 | 0 |
//! | 0,40 | −4 | 0 |
//! | 0,20 | −4 | 0 |
//! | **0,10** | **−184** | 19 |
//! | **0,05** | **−255** (a tinta SOME) | 102 |
//!
//! ⚠️ **E o produto atravessa a cerca com a mão LENTA** (`flip_draw_tests::
//! the_real_pipeline_step_in_radii`): o RDP tem tolerância `0,05 × espessura = 0,1·r` e a
//! reamostragem só ACRESCENTA pontos — um arco de mão a 400 amostras entrega passo mínimo
//! **0,137·r** e a 1200 amostras **0,108·r**, com **125 de 251** segmentos abaixo da cerca.
//! Não é um caso patológico: é desenhar devagar.
//!
//! ⚠️ **Por que só com traço ÚNICO** (o oráculo do Enio: *"o problema só aparece se o cruzamento
//! é feito com traço único; se cruzo vários traços diferentes o traço fica melhor"*): dois traços
//! distintos têm depth diferente e compõem por `over` — o parceiro do cruzamento **não precisa
//! estar na lista de extras**. A lista só existe para o traço que volta sobre si mesmo.
//!
//! Roda com:
//! ```text
//! cargo test -p ph2d-flip-render --release --test sampling_invariance -- --ignored --nocapture
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
        label: Some("ph2d-flip sampling-invariance device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");
    Some((device, queue))
}

fn pixel_camera() -> CameraRaw {
    let sx = 2.0 / W as f32;
    let sy = -2.0 / H as f32;
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
    let camera = pixel_camera();
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-flip sampling target"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
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
    fr.ensure_depth(device, (W, H));

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-flip sampling pass"),
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

    let unpadded = W * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-flip readback"),
        size: u64::from(padded) * u64::from(H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    rx.recv().expect("map").expect("map ok");
    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded as usize) * (H as usize));
    for row in 0..H as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
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

/// A lei de perfil do Painter, chamada na função REAL — este arquivo não guarda cópia dela.
fn painter_weight(dn: f32, hardness: f32) -> f32 {
    let h = hardness.clamp(0.0, 1.0);
    if h >= 1.0 {
        return f32::from(dn < 1.0);
    }
    let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
    ph2d_painter_brush::Falloff::Smooth.weight(remapped)
}

/// **O DEPÓSITO DO PAINTER** — dabs a cada `spacing × diâmetro` de arco, compostos por `over`.
/// Amostrado no CENTRO do pixel, como o Painter faz (supersamplear mediria uma verdade que
/// nenhum dos dois produtos computa).
fn painter_deposit(pts: &[(f32, f32)], r: f32, hardness: f32) -> Vec<f32> {
    let step = PAINTER_SPACING * 2.0 * r;
    let mut dabs: Vec<(f32, f32)> = Vec::new();
    let mut carry = 0.0_f32;
    dabs.push(pts[0]);
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if len <= 1e-6 {
            continue;
        }
        let mut t = step - carry;
        while t <= len {
            let f = t / len;
            dabs.push((a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f));
            t += step;
        }
        carry = (carry + len) % step;
    }
    let mut out = vec![0.0_f32; (W * H) as usize];
    for y in 0..H {
        for x in 0..W {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut keep = 1.0_f32;
            for &(dx, dy) in &dabs {
                let d = ((px - dx).powi(2) + (py - dy).powi(2)).sqrt() / r;
                if d < 1.0 {
                    keep *= 1.0 - painter_weight(d, hardness);
                }
            }
            out[(y * W + x) as usize] = 1.0 - keep;
        }
    }
    out
}

/// A franja da SILHUETA — onde o Flip tem AA analítico e o oráculo do depósito não tem nenhum.
/// Comparar ali mede a diferença de AA, não a lei da tinta.
fn in_the_fringe(pts: &[(f32, f32)], r: f32, x: u32, y: u32) -> bool {
    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
    let mut best = f32::MAX;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = (b.0 - a.0, b.1 - a.1);
        let l2 = ab.0 * ab.0 + ab.1 * ab.1;
        let t = if l2 <= 0.0 {
            0.0
        } else {
            (((px - a.0) * ab.0 + (py - a.1) * ab.1) / l2).clamp(0.0, 1.0)
        };
        let (dx, dy) = (px - (a.0 + t * ab.0), py - (a.1 + t * ab.1));
        best = best.min((dx * dx + dy * dy).sqrt());
    }
    (best - r).abs() < 2.0
}

/// A estrela de UM traço, densificada no passo pedido. Quina afiada em cada ponta e cinco
/// auto-cruzamentos — a topologia que obriga a lista de vizinhos a trabalhar.
fn star(step: f32) -> Vec<(f32, f32)> {
    let (cx, cy, outer) = (32.0_f32, 32.0_f32, 26.0_f32);
    let mut corners: Vec<(f32, f32)> = (0..5)
        .map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
            (cx + outer * a.cos(), cy + outer * a.sin())
        })
        .collect();
    corners.push(corners[0]);
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
    pts
}

/// 🔴 **O GATE.** A MESMA figura, amostrada de `0,80·r` até `0,04·r`, tem de pintar a MESMA
/// tinta — porque a densidade da polilinha é uma escolha do motor, não do artista.
///
/// O oráculo é o **depósito real do Painter** (não uma cópia da regra do produto), e a barra é a
/// mesma em toda densidade: quem afrouxar a barra na ponta densa está medindo o bug.
///
/// ⚠️ **A franja da silhueta é excluída** (`in_the_fringe`): ali o Flip tem AA analítico e o
/// oráculo não tem nenhum, então a diferença mede AA e não a lei da tinta.
#[test]
#[ignore = "precisa de adapter; roda com --ignored"]
fn the_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled() {
    let Some((device, queue)) = device() else {
        println!("sem adapter -- skip");
        return;
    };
    let r = 7.0_f32;
    // A barra: o desvio que o produto já tinha na densidade SÃ (medido −4 de 255 em 0,80·r),
    // com folga. Um truncamento da lista mede −184 e −255, três ORDENS acima disto.
    const BAR: i32 = -24;
    let mut pior_global = 0i32;
    for hardness in [0.4_f32, 0.7] {
        for frac in [0.80_f32, 0.40, 0.20, 0.10, 0.05, 0.04] {
            let pts = star(frac * r);
            let px = render(&device, &queue, &flip_drawing(&pts, r, hardness));
            let dep = painter_deposit(&pts, r, hardness);
            let mut falta = 0i32;
            for y in 0..H {
                for x in 0..W {
                    if in_the_fringe(&pts, r, x, y) {
                        continue;
                    }
                    let d = i32::from(px[((y * W + x) * 4 + 3) as usize])
                        - (dep[(y * W + x) as usize] * 255.0).round() as i32;
                    falta = falta.min(d);
                }
            }
            println!(
                "  h={hardness:.1} passo={frac:.2}xr  ({:4} segmentos)  falta {falta:+5}",
                pts.len() - 1
            );
            pior_global = pior_global.min(falta);
            assert!(
                falta >= BAR,
                "densidade {frac:.2}xr (hardness {hardness:.1}): falta {falta} de tinta contra o \
                 deposito do Painter (barra {BAR}). A lista de vizinhos truncou: o pixel voltou \
                 ao first-wins e a cauda macia de um quad foi pintada sobre o NUCLEO do vizinho."
            );
        }
    }
    println!("\n  pior desvio em TODA densidade: {pior_global:+}");
}

// ————————————————————— o MOTOR NOVO na MESMA lei (passo 4) —————————————————————

/// O motor novo sobre a MESMA figura, pelo MESMO caminho de dados e pela MESMA câmera.
fn new_engine_alpha(pts: &[(f32, f32)], r: f32, hardness: f32) -> Vec<f32> {
    let data = pack_drawing(&flip_drawing(pts, r, hardness));
    let screen = ScreenSpace::from_camera(&pixel_camera());
    let bins = bin_segments(&data, &screen, DEFAULT_TILE);
    let mut out = vec![0.0_f32; (W * H) as usize];
    for y in 0..H {
        for x in 0..W {
            let p = screen.point_px([x as f32 + 0.5, y as f32 + 0.5]);
            out[(y * W + x) as usize] = walk_pixel(&bins, &data, &screen, p)[3];
        }
    }
    out
}

/// 🔴 **A MESMA LEI, NO MOTOR NOVO — e aqui ela é ESTRUTURAL, não afinada.**
///
/// No motor de hoje a invariância é uma propriedade que teve de ser CONQUISTADA (o cabeçalho
/// deste arquivo conta como): a lista de vizinhos que o fragment recebe é de tamanho FIXO, então
/// "quantos vizinhos cabem" é uma constante contra a qual a densidade da polilinha corre.
///
/// No motor novo **não existe essa constante**: a lista por ladrilho é limitada por MEMÓRIA, e a
/// tinta é `∫ f ds / pitch` — uma integral sobre o caminho, cujo valor não sabe em quantos pedaços
/// o caminho foi partido. Densificar a polilinha subdivide o domínio da integral, e subdividir o
/// domínio de uma integral não a muda.
///
/// ⚠️ Este gate roda **headless** (o irmão acima precisa de adapter), então ele corre na varredura
/// normal — que é onde uma regressão de invariância tem de aparecer.
#[test]
fn the_new_engine_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled() {
    let r = 7.0_f32;
    // ⚠️ **A barra NÃO é a do irmão de GPU, e a diferença é nomeada.** O motor novo carrega um
    // deslocamento CONSTANTE contra o depósito do Painter — a PONTA do traço (o Painter carimba
    // um dab no primeiro ponto; a integral não tem caminho além do fim) mais a discretização das
    // quinas. Esse deslocamento é assunto do
    // `painter_look::the_new_engines_deficit_is_the_endpoint_and_the_corner_and_these_are_its_numbers`,
    // que o pina em -36; aqui ele é **fundo**, e a barra só existe para um truncamento (que mede
    // -184 e -255) não passar. Quem julga ESTE arquivo é a segunda asserção.
    const BAR: i32 = -40;
    for hardness in [0.4_f32, 0.7] {
        // ⚠️ A tabela INTEIRA primeiro, e só então as asserções: um gate que aborta na 1ª célula
        // reporta um número onde a pergunta é uma CURVA.
        let tabela: Vec<(f32, i32)> = [0.80_f32, 0.40, 0.20, 0.10, 0.05, 0.04]
            .into_iter()
            .map(|frac| {
                let pts = star(frac * r);
                let got = new_engine_alpha(&pts, r, hardness);
                let dep = painter_deposit(&pts, r, hardness);
                let mut falta = 0i32;
                for y in 0..H {
                    for x in 0..W {
                        if in_the_fringe(&pts, r, x, y) {
                            continue;
                        }
                        let i = (y * W + x) as usize;
                        let d = (got[i] * 255.0).round() as i32 - (dep[i] * 255.0).round() as i32;
                        falta = falta.min(d);
                    }
                }
                (frac, falta)
            })
            .collect();
        let pior = tabela.iter().map(|&(_, f)| f).min().unwrap_or(0);
        let melhor = tabela.iter().map(|&(_, f)| f).max().unwrap_or(0);
        assert!(
            pior >= BAR,
            "hardness {hardness:.1}: falta {pior} contra o deposito do Painter \
             (barra {BAR}) -- tabela {tabela:?}"
        );
        // ⚠️ **A barra sozinha admitiria uma DERIVA lenta dentro dela**, e a lei não é sobre o
        // tamanho do desvio: é sobre ele NÃO ANDAR. 20× de subdivisão, o mesmo número.
        assert!(
            melhor - pior <= 2,
            "hardness {hardness:.1}: a tinta ANDOU com a densidade ({pior} a {melhor}) -- \
             tabela {tabela:?}"
        );
    }
}
