//! **Diagnóstico do encaixe fill↔linha** (smoke do Enio, 2026-07-13): *"o fill não se
//! ajusta à linha de contorno"*.
//!
//! A geometria diz que a borda do fill cai a 0,3 px do EIXO da linha (medido com os
//! números do produto). Se ainda assim a tela mostra a cor descolada, quem mente é o
//! RENDER — e o único oráculo que resolve isso é o **pixel**
//! ([[feedback_render_and_look_when_a_green_gate_is_contradicted]]).
//!
//! Este arquivo rasteriza a cena do produto (traço trêmulo grosso + o fill que o solver
//! devolve), lê os pixels e (a) **grava um PNG** para olhar e (b) afirma a propriedade
//! que o Enio cobrou: **entre a cor e a linha não pode haver fundo**.
//!
//! `cargo test -p ph2d-flip-render --test gpu_fill_fit -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_fill::{
    FILL_TUCK_FRACTION, FillMode, FillParams, contour_widths, contour_widths_with_margin, fill_at,
};
use ph2d_flip_render::{CameraRaw, FlipRenderer, pack_drawing};

/// **A dilatação vem do PRODUTO, não daqui.**
///
/// Até 2026-07-18 este arquivo tinha uma cópia da constante e uma cópia da fórmula
/// (`width_px + 2.0 * tuck`). Os números batiam por acordo tácito — e quando a
/// dilatação do produto ficou 100× grande demais (BUGS #20), **os oito gates deste
/// arquivo continuaram verdes**, incluindo o que se chama *"a cor nunca transborda para
/// fora da linha"*. O defeito foi achado por um humano olhando a tela.
///
/// Um oráculo que reconstrói o que deveria verificar só afirma que a própria aritmética
/// dele é consistente. Agora a lei mora em `ph2d_flip_fill::dilate` e este arquivo a
/// **pergunta**.
///
/// ⚠️ **O mundo desta fixture É o pixel** (câmera de pixel, `pixel_camera_zoom`), então
/// a conversão px→mundo é 1. No produto ela é `SIZE_PX_PER_WORLD = 100`.
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
        label: Some("flip fill-fit"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    Some((device, queue))
}

/// Mundo = pixels do alvo (1:1 em `z = 1`), y para baixo, com **zoom** `z` em torno do
/// centro do alvo — a varredura que os bugs #11/#14/#16 exigiram três vezes (um gate que
/// mede num zoom só não vê o defeito).
///
/// O 3º campo do `CameraRaw` fica em **1.0** em qualquer zoom: é o que o app real faz
/// (`flip_pass::camera_raw`) e é o coração do brush ABSOLUTO — as POSIÇÕES acompanham o
/// zoom, a ESPESSURA não. Aproximar a câmera portanto AFINA a linha em relação ao
/// desenho, que é exatamente o regime onde o contorno vetorizado descolava.
///
/// Em `z = 1.0` a matriz é byte-idêntica à anterior (o centro do zoom é o centro do
/// alvo), então os gates que já existem não se mexem.
fn pixel_camera_zoom(z: f32) -> CameraRaw {
    let (sx, sy) = (2.0 * z / W as f32, -2.0 * z / H as f32);
    CameraRaw::new(
        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-z, z, 0.0, 1.0],
        ],
        [W as f32, H as f32],
        1.0,
    )
}

fn render(device: &wgpu::Device, queue: &wgpu::Queue, drawing: &FlipDrawing) -> Vec<u8> {
    render_zoom(device, queue, drawing, 1.0)
}

/// **A câmera do PRODUTO**: mundo em unidades de documento, `ppw` pixels por unidade —
/// e o `px_per_world` do `CameraRaw` acompanha, que é o que o `flip_pass::camera_raw`
/// faz de verdade (`thickness_px = width · 0.5 · px_per_world` no shader).
///
/// A `pixel_camera_zoom` **congela** esse campo em 1.0 de propósito (ela zoomba as
/// posições e deixa a espessura em px absolutos); esta aqui é o outro regime, e é o do
/// app. Sem ela, TODA fixture deste arquivo vive em `px_per_world = 1` — o único ponto
/// da reta onde um erro de UNIDADE na dilatação é invisível.
fn world_camera(ppw: f32) -> CameraRaw {
    // Em `ppw = 100` a arte (que vive em [0; 3,2] de mundo) preenche o alvo exatamente,
    // então centrar nela é a identidade — os gates existentes não se mexem.
    world_camera_at(ppw, Vec2::new(1.6, 1.6))
}

/// A câmera do produto, **centrada** em `c` (unidades de mundo) — é o que o artista faz
/// ao aproximar: ele centraliza no que está olhando.
fn world_camera_at(ppw: f32, c: Vec2) -> CameraRaw {
    let (sx, sy) = (2.0 * ppw / W as f32, -2.0 * ppw / H as f32);
    CameraRaw::new(
        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-c.x * sx, -c.y * sy, 0.0, 1.0],
        ],
        [W as f32, H as f32],
        ppw,
    )
}

fn render_with(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    drawing: &FlipDrawing,
    cam: &CameraRaw,
) -> Vec<u8> {
    render_inner(device, queue, drawing, cam)
}

fn render_zoom(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    drawing: &FlipDrawing,
    zoom: f32,
) -> Vec<u8> {
    render_inner(device, queue, drawing, &pixel_camera_zoom(zoom))
}

fn render_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    drawing: &FlipDrawing,
    cam: &CameraRaw,
) -> Vec<u8> {
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
    fr.upload(device, queue, cam, &pack_drawing(drawing));

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
                    // Fundo cinza-escuro, como o canvas do app.
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

/// A cena do smoke: um círculo desenhado À MÃO (trêmulo) com linha grossa, preenchido
/// pelo balde com os parâmetros DEFAULT do produto.
fn scene_t(width_px: f32, hardness: f32, tuck: f32) -> FlipDrawing {
    let (cx, cy, r) = (160.0f32, 160.0, 110.0);
    let n = 200;
    let pts: Vec<Vec2> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let (c, s) = unit_circle(t);
            // tremor de mão: ±2 px, determinístico
            let h = ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
            let rr = r + h * 4.0;
            Vec2::new(cx + rr * c, cy + rr * s)
        })
        .collect();

    // O solver, com os números do produto (o mundo aqui É px de tela, então
    // `px_to_world = 1` e a precision default 1.6 entra direto).
    let lines = vec![(pts.clone(), vec![width_px * 0.5; n], true)];
    let res = fill_at(
        &lines,
        Vec2::new(cx, cy),
        FillParams {
            precision: 1.6,
            gap_reach: 0.0,
            grow: 0,
            trap_px: 0.0,
            mode: FillMode::Paint,
        },
    )
    .expect("preenche");

    let mut d = FlipDrawing::new();
    // 1) o preenchimento (atrás), 2) o line-art (na frente) — a ordem do produto.
    let ocre = Rgba::new(0.78, 0.6, 0.35, 1.0);
    // **A dilatação sai da lei do produto**, com a margem que a varredura está testando.
    let widths = contour_widths_with_margin(&lines, &res.outer, tuck);
    let mut f = FlipStroke::new();
    for (i, p) in res.outer.iter().enumerate() {
        f.push_point(Point {
            pos: *p,
            width: widths[i],
            opacity: 1.0,
            color: ocre,
        });
    }
    f.closed = true;
    f.hide_stroke = true;
    f.holes = res.holes;
    f.fill = Some(Fill {
        color: ocre,
        opacity: 1.0,
    });
    d.strokes.push(f);

    let mut line = FlipStroke::new();
    for p in &pts {
        line.push_point(Point {
            pos: *p,
            width: width_px,
            opacity: 1.0,
            color: Rgba::new(0.95, 0.95, 0.96, 1.0),
        });
    }
    line.closed = true;
    line.hardness = hardness;
    d.strokes.push(line);
    d
}

/// `SIZE_PX_PER_WORLD` do produto — quantos px de tela vale UMA unidade de documento na
/// escala de referência do Size. (Espelhado e não importado porque `ph2d-tool-flip` não é
/// dependência desta crate; o gate abaixo é justamente quem o mantém honesto: se o número
/// divergir, as duas descrições da MESMA arte param de render a mesma imagem.)
const PRODUCT_PPW: f32 = 100.0;

/// **A MESMA arte, descrita na escala do PRODUTO** — geometria em unidades de documento
/// (~1,1 de raio, não 110 px) e câmera a 100 px por unidade.
///
/// Existe porque toda outra fixture deste arquivo vive num mundo que JÁ é pixel
/// (`px_per_world = 1`), e esse é **exatamente o ponto da reta onde um erro de unidade na
/// dilatação some**: `2·0,5/1` e `2·0,5` são o mesmo número. O BUGS #20 (a margem em px
/// somada a uma largura em unidades de mundo, 100× grande demais) passou por aqui sem
/// mover um pixel — não porque os gates estivessem mal escritos, mas porque a fixture não
/// continha o fenômeno.
fn scene_world(width_px: f32, hardness: f32) -> FlipDrawing {
    scene_world_at(width_px, hardness, PRODUCT_PPW)
}

/// A mesma cena com o **zoom da câmera** variável — o produto recomputa a resolução do
/// buffer a partir dele (`precision = style.precision / doc_per_px`), então os dois têm
/// de andar juntos ou a fixture descreve um regime que o app não tem.
fn scene_world_at(width_px: f32, hardness: f32, ppw: f32) -> FlipDrawing {
    scene_world_full(width_px, hardness, ppw, true)
}

/// `tremor = false` desenha o círculo **LISO**.
///
/// ⚠️ Existe para o `sweep_zoom`, e a razão é uma armadilha que a 1ª versão dele caiu: o
/// tremor de mão vive em unidades de MUNDO (±0,04), então **na tela ele cresce com o
/// zoom** — a ±1 px em 25 px/unidade e a ±5,6 px em 140. Uma sonda em círculo perfeito
/// passa a cortar a própria linha, e o "transbordo" sobe sozinho: a tabela mostraria a
/// tendência que se quer medir mesmo que a margem estivesse perfeita. Sem tremor, a sonda
/// e o eixo coincidem em qualquer zoom, e o que sobra é o erro de vetorização — que é o
/// que a margem existe para compensar.
fn scene_world_full(width_px: f32, hardness: f32, ppw: f32, tremor: bool) -> FlipDrawing {
    // A ARTE é a mesma em qualquer zoom (ela vive em unidades de documento); o que muda
    // é a câmera. A geometria continua ancorada na escala de referência do Size.
    let k = PRODUCT_PPW;
    let _ = ppw;
    let (cx, cy, r) = (160.0f32 / k, 160.0 / k, 110.0 / k);
    let n = 200;
    let pts: Vec<Vec2> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let (c, s) = unit_circle(t);
            let h = if tremor {
                ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5
            } else {
                0.0
            };
            let rr = r + h * 4.0 / k;
            Vec2::new(cx + rr * c, cy + rr * s)
        })
        .collect();

    let lines = vec![(pts.clone(), vec![width_px * 0.5 / k; n], true)];
    let res = fill_at(
        &lines,
        Vec2::new(cx, cy),
        FillParams {
            // O produto: `style.precision / doc_per_px`, e `doc_per_px = 1/ppw` — logo a
            // resolução do buffer ACOMPANHA o zoom da câmera.
            precision: 1.6 * ppw,
            gap_reach: 0.0,
            grow: 0,
            trap_px: 0.0,
            mode: FillMode::Paint,
        },
    )
    .expect("preenche na escala do produto");

    let mut d = FlipDrawing::new();
    let ocre = Rgba::new(0.78, 0.6, 0.35, 1.0);
    // A lei do produto, com a conversão do PRODUTO — é este argumento que o BUGS #20
    // ignorava.
    let widths = contour_widths(&lines, &res.outer);
    let mut f = FlipStroke::new();
    for (i, p) in res.outer.iter().enumerate() {
        f.push_point(Point {
            pos: *p,
            width: widths[i],
            opacity: 1.0,
            color: ocre,
        });
    }
    f.closed = true;
    f.hide_stroke = true;
    f.holes = res.holes;
    f.fill = Some(Fill {
        color: ocre,
        opacity: 1.0,
    });
    d.strokes.push(f);

    let mut line = FlipStroke::new();
    for p in &pts {
        line.push_point(Point {
            pos: *p,
            width: width_px / k,
            opacity: 1.0,
            color: Rgba::new(0.95, 0.95, 0.96, 1.0),
        });
    }
    line.closed = true;
    line.hardness = hardness;
    d.strokes.push(line);
    d
}

fn scene_h(width_px: f32, hardness: f32) -> FlipDrawing {
    scene_t(width_px, hardness, FILL_TUCK_FRACTION)
}

fn scene(width_px: f32) -> FlipDrawing {
    scene_t(width_px, 1.0, FILL_TUCK_FRACTION)
}

/// Círculo sem transcendentais (HR-5 vale no teste também — e é o mesmo helper do solver).
fn unit_circle(t: f32) -> (f32, f32) {
    let q = (t * 4.0).floor() as i32 % 4;
    let u = (t * 4.0).fract();
    let d = 1.0 + u * u;
    let (x, y) = ((1.0 - u * u) / d, 2.0 * u / d);
    match q {
        0 => (x, y),
        1 => (-y, x),
        2 => (-x, -y),
        _ => (y, -x),
    }
}

/// **Entre a cor e a linha não pode haver FUNDO.** Varre raios do centro para fora e,
/// em cada um, classifica os pixels: cor / linha / fundo. A sequência tem de ser
/// `cor… linha… fundo` — um `fundo` ENTRE cor e linha é o vão que o Enio viu.
#[test]
#[ignore = "precisa de GPU"]
fn the_fill_meets_the_line_with_no_gap() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    for width_px in [6.0f32, 16.0, 32.0] {
        let px = render(&device, &queue, &scene(width_px));
        let at = |x: i32, y: i32| -> (u8, u8, u8) {
            let i = ((y as u32 * W + x as u32) * 4) as usize;
            (px[i], px[i + 1], px[i + 2])
        };
        // Classificação por cor (o fundo é cinza-escuro, a linha branca, o fill ocre).
        let kind = |c: (u8, u8, u8)| -> char {
            let (r, g, b) = (c.0 as i32, c.1 as i32, c.2 as i32);
            if r > 200 && g > 200 && b > 200 {
                'L' // linha
            } else if r > 120 && g > 90 && b < 140 && r > b + 40 {
                'C' // cor (ocre)
            } else if r < 70 && g < 70 && b < 80 {
                'F' // fundo
            } else {
                '?' // mistura de borda (AA)
            }
        };

        let mut worst_gap = 0usize;
        let mut worst_ray = String::new();
        for k in 0..64 {
            let t = k as f32 / 64.0;
            let (c, s) = unit_circle(t);
            let mut ray = String::new();
            for step in 60..150 {
                let x = (160.0 + c * step as f32).round() as i32;
                let y = (160.0 + s * step as f32).round() as i32;
                if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                    break;
                }
                ray.push(kind(at(x, y)));
            }
            // O vão: um bloco de FUNDO depois da última COR e antes da primeira LINHA.
            let (Some(last_c), Some(first_l)) = (ray.rfind('C'), ray.find('L')) else {
                continue;
            };
            if first_l > last_c {
                let between = &ray[last_c + 1..first_l];
                let gap = between.chars().filter(|c| *c == 'F').count();
                if gap > worst_gap {
                    worst_gap = gap;
                    worst_ray = ray.clone();
                }
            }
        }
        // A LARGURA da faixa branca visível (os 'L' entre a cor e o fundo), por raio:
        // se ela vale ~w, a linha está cobrindo o que deveria; a variação dela é o que
        // o olho lê como "o fill não acompanha a linha".
        let mut widths: Vec<usize> = Vec::new();
        for k in 0..64 {
            let t = k as f32 / 64.0;
            let (c, s) = unit_circle(t);
            let mut ray = String::new();
            for step in 60..150 {
                let x = (160.0 + c * step as f32).round() as i32;
                let y = (160.0 + s * step as f32).round() as i32;
                if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                    break;
                }
                ray.push(kind(at(x, y)));
            }
            if let (Some(lc), Some(ll)) = (ray.rfind('C'), ray.rfind('L')) {
                widths.push(ll.saturating_sub(lc));
            }
        }
        widths.sort_unstable();
        println!(
            "linha {width_px}px: vao = {worst_gap} px | faixa BRANCA visivel: min {} med {} max {} px",
            widths.first().copied().unwrap_or(0),
            widths[widths.len() / 2],
            widths.last().copied().unwrap_or(0),
        );
        if worst_gap > 0 {
            println!("  raio: {worst_ray}");
        }
        assert!(
            worst_gap <= 1,
            "linha de {width_px}px: {worst_gap} px de FUNDO entre a cor e a linha \
             (a cor descolou do contorno)\n  raio: {worst_ray}"
        );
    }
}

/// Grava a cena num PNG para OLHAR (não afirma nada — é a régua visual).
/// `cargo test -p ph2d-flip-render --test gpu_fill_fit look -- --ignored --nocapture`
#[test]
#[ignore = "diagnostico visual: grava PNG"]
fn look() {
    let Some((device, queue)) = device() else {
        return;
    };
    for (width_px, hardness) in [(6.0f32, 1.0f32), (16.0, 1.0), (16.0, 0.4), (32.0, 0.4)] {
        let px = render(&device, &queue, &scene_h(width_px, hardness));
        let path = format!(
            "/tmp/flip_fill_fit_{}_h{}.png",
            width_px as i32,
            (hardness * 10.0) as i32
        );
        image::RgbaImage::from_raw(W, H, px)
            .expect("buffer")
            .save(&path)
            .expect("png");
        println!("{path}");
    }
}

/// **O gate do encaixe: sob a linha MACIA não pode aparecer o fundo.**
///
/// É o defeito que o smoke pegou e que só o pixel mostra. Com `hardness < 1` a linha é
/// semi-transparente nas bordas: se a cor parar no eixo, a metade EXTERNA da linha
/// mistura com o fundo escuro e o contorno ganha um halo sujo — a arte não fecha. Com a
/// dilatação (o contorno do fill vestindo a espessura da linha), a linha macia mistura
/// com a COR, e o encaixe é o do Grease Pencil.
///
/// Mutação que sangra: zere a `width` dos pontos do contorno do fill (o 1º corte) e a
/// contagem de pixels escuros dentro da silhueta explode.
#[test]
#[ignore = "precisa de GPU"]
fn a_soft_line_never_shows_the_background_through_the_fill_edge() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    for width_px in [8.0f32, 16.0, 32.0] {
        let px = render(&device, &queue, &scene_h(width_px, 0.35));
        let dark = |x: i32, y: i32| -> bool {
            let i = ((y as u32 * W + x as u32) * 4) as usize;
            // Cinza do fundo (r≈g≈b, escuro) — a cor e a linha são bem mais claras.
            let (r, g, b) = (px[i] as i32, px[i + 1] as i32, px[i + 2] as i32);
            r < 90 && (r - b).abs() < 12 && (r - g).abs() < 12
        };
        // Varre o ANEL da linha (do eixo até a silhueta externa): ali, com a cor por
        // baixo, nenhum pixel pode ser fundo.
        let mut leaks = 0;
        for k in 0..256 {
            let t = k as f32 / 256.0;
            let (c, s) = unit_circle(t);
            // o eixo está em r≈110 (±2 de tremor); a silhueta externa em 110 + w/2.
            let from = 110.0 + 3.0;
            let to = 110.0 + width_px * 0.5 - 2.0;
            let mut step = from;
            while step <= to {
                let x = (160.0 + c * step).round() as i32;
                let y = (160.0 + s * step).round() as i32;
                if (0..W as i32).contains(&x) && (0..H as i32).contains(&y) && dark(x, y) {
                    leaks += 1;
                }
                step += 1.0;
            }
        }
        println!("linha macia {width_px}px: {leaks} pixels de FUNDO sob a linha");
        assert_eq!(
            leaks, 0,
            "linha macia de {width_px}px: o fundo aparece atraves da linha em {leaks} pixels \
             — a cor nao entrou por baixo dela (o fill nao se ajusta ao contorno)"
        );
    }
}

/// **A varredura que escolhe a margem** (`FILL_TUCK_PX`): dois defeitos OPOSTOS, e o
/// valor certo é o que zera um sem acordar o outro.
///
/// - margem de menos → sobra um fio de linha SEM cor por baixo (com pincel macio, o
///   fundo aparece: o "não se ajusta" do smoke);
/// - margem demais → a cor TRANSBORDA a linha (uma orla colorida por fora do desenho —
///   exatamente o defeito que matou o `grow = +2` default no BUGS #11).
#[test]
#[ignore = "diagnostico: escolhe a constante"]
fn sweep_tuck() {
    let Some((device, queue)) = device() else {
        return;
    };
    println!("tuck |  linha | fundo sob a linha | transbordo alem dela");
    for tuck in [0.0f32, 0.03, 0.06, 0.10, 0.15, 0.25] {
        for width_px in [8.0f32, 16.0, 32.0] {
            let px = render(&device, &queue, &scene_t(width_px, 0.35, tuck));
            let at = |x: i32, y: i32| -> (i32, i32, i32) {
                let i = ((y as u32 * W + x as u32) * 4) as usize;
                (px[i] as i32, px[i + 1] as i32, px[i + 2] as i32)
            };
            let is_bg =
                |c: (i32, i32, i32)| c.0 < 90 && (c.0 - c.2).abs() < 12 && (c.0 - c.1).abs() < 12;
            // ocre puro (sem branco de linha por cima): r bem > b, e claro
            let is_colour = |c: (i32, i32, i32)| c.0 > 120 && c.0 > c.2 + 40;

            let (mut bg_under, mut spill) = (0, 0);
            for k in 0..256 {
                let t = k as f32 / 256.0;
                let (c, s) = unit_circle(t);
                // ANEL da linha: do eixo (110) à silhueta (110 + w/2).
                let mut step = 113.0;
                while step <= 110.0 + width_px * 0.5 - 2.0 {
                    let (x, y) = (
                        (160.0 + c * step).round() as i32,
                        (160.0 + s * step).round() as i32,
                    );
                    if is_bg(at(x, y)) {
                        bg_under += 1;
                    }
                    step += 1.0;
                }
                // FORA da silhueta: ali não pode haver cor.
                let mut step = 110.0 + width_px * 0.5 + 2.0;
                while step <= 110.0 + width_px * 0.5 + 8.0 {
                    let (x, y) = (
                        (160.0 + c * step).round() as i32,
                        (160.0 + s * step).round() as i32,
                    );
                    if is_colour(at(x, y)) {
                        spill += 1;
                    }
                    step += 1.0;
                }
            }
            println!("{tuck:>4} | {width_px:>5}px | {bg_under:>17} | {spill:>20}");
        }
    }
}

/// **E o defeito OPOSTO: a cor não pode transbordar a linha.**
///
/// A dilatação que faz a cor encaixar por baixo do line-art é a mesma que, exagerada,
/// a empurra para FORA dele — a orla colorida que matou o `grow = +2` default
/// (BUGS #11). Este gate guarda esse lado; o `sweep_tuck` mostra a curva inteira.
#[test]
#[ignore = "precisa de GPU"]
fn the_colour_never_spills_outside_the_line() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    for width_px in [8.0f32, 16.0, 32.0] {
        let px = render(&device, &queue, &scene_h(width_px, 1.0));
        let is_colour = |x: i32, y: i32| -> bool {
            let i = ((y as u32 * W + x as u32) * 4) as usize;
            let (r, b) = (px[i] as i32, px[i + 2] as i32);
            r > 120 && r > b + 40
        };
        let mut spill = 0;
        let mut samples = 0;
        for k in 0..256 {
            let t = k as f32 / 256.0;
            let (c, s) = unit_circle(t);
            // De 2 px além da silhueta para fora: ali só pode haver fundo.
            let mut step = 110.0 + width_px * 0.5 + 2.0;
            while step <= 110.0 + width_px * 0.5 + 8.0 {
                let (x, y) = (
                    (160.0 + c * step).round() as i32,
                    (160.0 + s * step).round() as i32,
                );
                samples += 1;
                if (0..W as i32).contains(&x) && (0..H as i32).contains(&y) && is_colour(x, y) {
                    spill += 1;
                }
                step += 1.0;
            }
        }
        let pct = 100.0 * spill as f32 / samples as f32;
        println!("linha {width_px}px: transbordo {spill}/{samples} ({pct:.1}%)");
        assert!(
            pct < 2.0,
            "linha de {width_px}px: a cor vazou para FORA do contorno em {pct:.1}% do anel \
             externo — a dilatacao esta grande demais (o defeito do BUGS #11)"
        );
    }
}

/// A cena que o smoke expôs: um polígono de **cantos AGUDOS** (o "bico" da imagem do
/// Enio). É onde a vetorização do contorno mais se afasta da linha — o marching squares
/// + RDP chanfram a quina, e o fill descola.
fn spiky(width_px: f32) -> FlipDrawing {
    // Uma estrela de 5 pontas: cantos agudos para dentro E para fora.
    let (cx, cy) = (160.0f32, 160.0);
    let n = 10;
    let pts: Vec<Vec2> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let (c, s) = unit_circle(t);
            let r = if i % 2 == 0 { 120.0 } else { 55.0 };
            Vec2::new(cx + r * c, cy + r * s)
        })
        .collect();
    let lines = vec![(pts.clone(), vec![width_px * 0.5; n], true)];
    let res = fill_at(
        &lines,
        Vec2::new(cx, cy),
        FillParams {
            precision: 1.6,
            gap_reach: 0.0,
            grow: 0,
            trap_px: 0.0,
            mode: FillMode::Paint,
        },
    )
    .expect("a estrela preenche");

    let mut d = FlipDrawing::new();
    let ocre = Rgba::new(0.78, 0.6, 0.35, 1.0);
    let widths = contour_widths_with_margin(&lines, &res.outer, FILL_TUCK_FRACTION);
    let mut f = FlipStroke::new();
    for (i, p) in res.outer.iter().enumerate() {
        f.push_point(Point {
            pos: *p,
            width: widths[i],
            opacity: 1.0,
            color: ocre,
        });
    }
    f.closed = true;
    f.hide_stroke = true;
    f.holes = res.holes;
    f.fill = Some(Fill {
        color: ocre,
        opacity: 1.0,
    });
    d.strokes.push(f);

    let mut line = FlipStroke::new();
    for p in &pts {
        line.push_point(Point {
            pos: *p,
            width: width_px,
            opacity: 1.0,
            color: Rgba::new(0.95, 0.95, 0.96, 1.0),
        });
    }
    line.closed = true;
    d.strokes.push(line);
    d
}

/// **A forma fechada pinta A SI MESMA — e aí não há vértices para dessincronizar.**
///
/// A resposta ao smoke do Enio (*"nem todo vertex da linha está conectado ao vertex de
/// fill"*): quando a região é o interior de uma forma fechada, o balde não vetoriza nada
/// — ele liga o `fill` no **próprio traço**, e a cor passa a ser a triangulação dos
/// pontos DELE (o material `stroke + fill` do GP, como o Suzanne é desenhado). Um
/// conjunto de vértices só.
///
/// Aqui a régua é o pixel, na cena que mais dói: uma ESTRELA (quinas agudas para dentro
/// e para fora), onde o contorno vetorizado chanfrava os bicos.
#[test]
#[ignore = "precisa de GPU"]
fn a_self_filled_shape_has_no_desync_at_all() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    for width_px in [8.0f32, 20.0] {
        // A estrela, com o fill NO PRÓPRIO TRAÇO (o que o balde agora produz).
        let (cx, cy) = (160.0f32, 160.0);
        let n = 10;
        let pts: Vec<Vec2> = (0..n)
            .map(|i| {
                let (c, s) = unit_circle(i as f32 / n as f32);
                let r = if i % 2 == 0 { 120.0 } else { 55.0 };
                Vec2::new(cx + r * c, cy + r * s)
            })
            .collect();
        let ocre = Rgba::new(0.78, 0.6, 0.35, 1.0);
        let mut d = FlipDrawing::new();
        let mut shape = FlipStroke::new();
        for p in &pts {
            shape.push_point(Point {
                pos: *p,
                width: width_px,
                opacity: 1.0,
                color: Rgba::new(0.95, 0.95, 0.96, 1.0),
            });
        }
        shape.closed = true;
        shape.fill = Some(Fill {
            color: ocre,
            opacity: 1.0,
        });
        d.strokes.push(shape);
        let px = render(&device, &queue, &d);

        // A cor não pode aparecer FORA da arte (o polígono + a meia-espessura da linha).
        let inside = |p: Vec2| -> bool {
            let mut c = false;
            for i in 0..n {
                let (a, b) = (pts[i], pts[(i + 1) % n]);
                if (a.y > p.y) != (b.y > p.y) {
                    let t = (p.y - a.y) / (b.y - a.y);
                    if p.x < a.x + t * (b.x - a.x) {
                        c = !c;
                    }
                }
            }
            c
        };
        let mut spill = 0;
        for y in 0..H {
            for x in 0..W {
                let i = ((y * W + x) * 4) as usize;
                let (r, b) = (px[i] as i32, px[i + 2] as i32);
                if !(r > 120 && r > b + 40) {
                    continue; // não é a cor
                }
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                if inside(p) {
                    continue;
                }
                // Fora do polígono, a cor só pode estar sob a linha (a triangulação vai
                // até o EIXO; a linha cobre meia-espessura para cada lado).
                let mut dist = f32::MAX;
                for i in 0..n {
                    let (a, b) = (pts[i], pts[(i + 1) % n]);
                    let ab = b - a;
                    let t = (((p - a).x * ab.x + (p - a).y * ab.y)
                        / (ab.x * ab.x + ab.y * ab.y).max(1e-9))
                    .clamp(0.0, 1.0);
                    let c = a + ab * t;
                    dist = dist.min(((p - c).x.powi(2) + (p - c).y.powi(2)).sqrt());
                }
                if dist > width_px * 0.5 + 1.5 {
                    spill += 1;
                }
            }
        }
        println!("estrela AUTO-preenchida, linha {width_px}px: {spill} px de cor fora da arte");
        assert_eq!(
            spill, 0,
            "a forma auto-preenchida vazou {spill} px — os vertices deveriam ser OS MESMOS"
        );
    }
}

/// A estrela como a MÃO a desenha: um traço **ABERTO** (`closed = false`), cuja ponta
/// volta ao começo sem fechar o bit. É o que o `flip_draw::build_stroke` produz fora do
/// modo `Shape: Filled` — ou seja, **todo traço que o Enio desenha**.
///
/// **A estrela ENCOLHE em mundo por `1/zoom`** (e a câmera aproxima na mesma razão), o
/// que a deixa do MESMO tamanho na tela em todo zoom. Não é um truque para o gate passar
/// — é o único jeito de a varredura de zoom significar alguma coisa:
///
/// - o regime que quebrava (BUGS #14/#16) é a RAZÃO entre o erro da geometria (assado em
///   unidades de DOCUMENTO) e a espessura da linha (absoluta, px de TELA). Aproximar a
///   câmera multiplica o erro por `z` e deixa a linha do mesmo tamanho: um desvio que
///   estava escondido sob a linha aparece. Encolher o mundo por `1/z` reproduz **essa
///   mesma razão** — que é o que o gate tem de medir;
/// - e a **costura fica em quadro**. Com a estrela parada e a câmera aproximando 5×, a
///   tela vira um campo de cor liso (a câmera entra DENTRO da forma) e as asserções não
///   olham fronteira nenhuma — verde vacuoso. Foi o 1º corte deste gate, e ele passava
///   com a costura fora da tela.
fn hand_drawn_star(width_px: f32, zoom: f32) -> (FlipDrawing, Vec<Vec2>) {
    let (cx, cy) = (160.0f32, 160.0);
    let n = 10;
    let mut pts: Vec<Vec2> = (0..n)
        .map(|i| {
            let (c, s) = unit_circle(i as f32 / n as f32);
            let r = if i % 2 == 0 { 120.0 } else { 55.0 } / zoom;
            Vec2::new(cx + r * c, cy + r * s)
        })
        .collect();
    // A mão volta ao começo — perto, mas NÃO no mesmo ponto, e sem fechar o traço.
    pts.push(Vec2::new(pts[0].x + 0.8 / zoom, pts[0].y + 0.6 / zoom));

    let ocre = Rgba::new(0.78, 0.6, 0.35, 1.0);
    let mut shape = FlipStroke::new();
    for p in &pts {
        shape.push_point(Point {
            pos: *p,
            width: width_px,
            opacity: 1.0,
            color: Rgba::new(0.95, 0.95, 0.96, 1.0),
        });
    }
    assert!(!shape.closed, "a forma da MAO e aberta — e esse e o ponto");
    shape.fill = Some(Fill {
        color: ocre,
        opacity: 1.0,
    });
    let mut d = FlipDrawing::new();
    d.strokes.push(shape);
    (d, pts)
}

/// 🟩 **A forma desenhada À MÃO pinta a si mesma — em QUALQUER zoom.**
///
/// O gate que faltava (smoke do Enio 2026-07-13: *"nem todo vertex da linha está conectado
/// ao vertex de fill… áreas de dessincronização e gaps"*). O `a_self_filled_shape_*` acima
/// provava a forma **fechada** — e no produto **nada é fechado**: a caneta só liga o bit no
/// modo `Shape: Filled`. Um traço aberto com `fill` era descartado pelo `pack` (`s.closed &&
/// …`) e pelo `filled_shape_target` (o mesmo filtro), então o balde caía SEMPRE no contorno
/// vetorizado — cujo erro é assado em unidades de DOCUMENTO enquanto a linha é absoluta em
/// px de TELA, e por isso **o zoom o amplia**.
///
/// Duas asserções, e as duas são necessárias:
///
/// 1. **o interior não tem FUNDO** — é ela que mata o falso-zero: um gate que só medisse
///    "a cor não vaza" ficaria VERDE com o fill invisível (não há cor para vazar);
/// 2. **a cor não escapa da arte** — o defeito oposto (o `grow` que matou o BUGS #11).
///
/// Mutação que sangra (as duas, rodadas):
/// - devolva `s.closed &&` à condição do fill em `pack.rs` → o interior vira fundo (1);
/// - dilate o polígono do fill (um `grow` no traço) → a cor sai da linha (2).
#[test]
#[ignore = "precisa de GPU"]
fn a_hand_drawn_open_shape_paints_itself_at_any_zoom() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    // O fundo do alvo (`render`): cinza-escuro, canal a canal (Rgba8Unorm, sem sRGB).
    let is_bg = |r: i32, b: i32| r < 160 && (r - b).abs() <= 12;
    let is_colour = |r: i32, b: i32| r > 120 && r > b + 40;

    for width_px in [8.0f32, 20.0] {
        for zoom in [1.0f32, 2.5, 5.0] {
            let (d, pts) = hand_drawn_star(width_px, zoom);
            let px = render_zoom(&device, &queue, &d, zoom);

            // A geometria na TELA: as posições acompanham o zoom, a espessura NÃO
            // (brush absoluto) — é o regime em que o contorno vetorizado descolava.
            let scr: Vec<Vec2> = pts
                .iter()
                .map(|p| Vec2::new((p.x - 160.0) * zoom + 160.0, (p.y - 160.0) * zoom + 160.0))
                .collect();
            let n = scr.len();
            let inside = |p: Vec2| -> bool {
                let mut c = false;
                for i in 0..n {
                    let (a, b) = (scr[i], scr[(i + 1) % n]);
                    if (a.y > p.y) != (b.y > p.y) {
                        let t = (p.y - a.y) / (b.y - a.y);
                        if p.x < a.x + t * (b.x - a.x) {
                            c = !c;
                        }
                    }
                }
                c
            };
            let dist_to_ring = |p: Vec2| -> f32 {
                let mut dist = f32::MAX;
                for i in 0..n {
                    let (a, b) = (scr[i], scr[(i + 1) % n]);
                    let ab = b - a;
                    let t = (((p - a).x * ab.x + (p - a).y * ab.y)
                        / (ab.x * ab.x + ab.y * ab.y).max(1e-9))
                    .clamp(0.0, 1.0);
                    let c = a + ab * t;
                    dist = dist.min(((p - c).x.powi(2) + (p - c).y.powi(2)).sqrt());
                }
                dist
            };

            let (mut bg_inside, mut spill, mut interior) = (0, 0, 0);
            for y in 0..H {
                for x in 0..W {
                    let i = ((y * W + x) * 4) as usize;
                    let (r, b) = (px[i] as i32, px[i + 2] as i32);
                    let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    // Só o que está DENTRO do alvo (num zoom alto a estrela sai da tela).
                    if inside(p) {
                        interior += 1;
                        if is_bg(r, b) {
                            bg_inside += 1;
                        }
                    } else if is_colour(r, b) && dist_to_ring(p) > width_px * 0.5 + 1.5 {
                        spill += 1;
                    }
                }
            }
            println!(
                "mao ABERTA, linha {width_px}px @ zoom {zoom}x: {interior} px de interior, \
                 {bg_inside} de FUNDO dentro, {spill} de cor fora da arte"
            );
            // A costura TEM de estar em quadro: a estrela ocupa ~19k px de interior na
            // tela em qualquer zoom (ela encolhe em mundo na mesma razão). Se um dia
            // alguém "simplificar" a cena e a câmera entrar dentro da forma, o interior
            // vira a tela toda (102400) e as duas asserções abaixo ficam vácuas.
            assert!(
                (15_000..30_000).contains(&interior),
                "a costura saiu de quadro ({interior} px de interior): o gate ficaria \
                 verde sem olhar fronteira nenhuma"
            );
            assert_eq!(
                bg_inside, 0,
                "o interior da forma da MAO tem {bg_inside} px de FUNDO — o preenchimento \
                 dela nao foi renderizado (o `pack` exige `closed`?)"
            );
            assert_eq!(
                spill, 0,
                "a cor escapou da arte em {spill} px no zoom {zoom}x"
            );
        }
    }

    // E a régua visual: o PNG para OLHAR (o pixel é o oráculo). A MESMA cena da asserção
    // — em `1x` e `5x` a estrela tem o mesmo tamanho na tela e a linha a mesma espessura;
    // o que muda é que em `5x` a geometria vive num mundo 5× menor, então um desvio de
    // documento apareceria 5× maior. As duas imagens serem indistinguíveis É o resultado.
    for zoom in [1.0f32, 5.0] {
        let (d, _) = hand_drawn_star(8.0, zoom);
        let px = render_zoom(&device, &queue, &d, zoom);
        let path = format!("/tmp/flip_hand_drawn_z{}.png", zoom as i32);
        image::RgbaImage::from_raw(W, H, px)
            .expect("buffer")
            .save(&path)
            .expect("png");
        println!("{path}");
    }
}

/// **O contorno do fill tem de SEGUIR a linha — inclusive nas quinas.**
///
/// Smoke do Enio (2026-07-13): *"nem todo vertex da linha está conectado ao vertex de
/// fill… isso cria áreas de dessincronização e gaps"*. Exato: o contorno vem do RASTER
/// (marching squares + RDP), então os vértices dele não têm nada a ver com os da
/// polilinha — nas quinas ele chanfra (deixa vão), nas retas ele desliza (transborda).
///
/// Aqui a régua é o pixel: numa ESTRELA (cantos agudos para dentro e para fora), conta-se
/// quanto de cor aparece FORA da silhueta da linha. A dilatação não salva este caso: o
/// erro é de FORMA, não de escala.
#[test]
#[ignore = "precisa de GPU"]
fn the_fill_contour_follows_the_line_even_at_sharp_corners() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    for width_px in [8.0f32, 16.0] {
        let px = render(&device, &queue, &spiky(width_px));
        let is_colour = |i: usize| -> bool {
            let (r, b) = (px[i] as i32, px[i + 2] as i32);
            r > 120 && r > b + 40
        };
        // A silhueta da estrela: um ponto está DENTRO da arte se estiver a menos de
        // w/2 de algum segmento da polilinha, ou dentro do polígono. Cor fora disso é
        // vazamento — e é o que a imagem do smoke mostra nas quinas.
        let (cx, cy) = (160.0f32, 160.0);
        let n = 10;
        let poly: Vec<Vec2> = (0..n)
            .map(|i| {
                let (c, s) = unit_circle(i as f32 / n as f32);
                let r = if i % 2 == 0 { 120.0 } else { 55.0 };
                Vec2::new(cx + r * c, cy + r * s)
            })
            .collect();
        let inside = |p: Vec2| -> bool {
            let mut c = false;
            for i in 0..n {
                let (a, b) = (poly[i], poly[(i + 1) % n]);
                if (a.y > p.y) != (b.y > p.y) {
                    let t = (p.y - a.y) / (b.y - a.y);
                    if p.x < a.x + t * (b.x - a.x) {
                        c = !c;
                    }
                }
            }
            c
        };
        let dist_to_line = |p: Vec2| -> f32 {
            let mut best = f32::MAX;
            for i in 0..n {
                let (a, b) = (poly[i], poly[(i + 1) % n]);
                let ab = b - a;
                let t = (((p - a).x * ab.x + (p - a).y * ab.y)
                    / (ab.x * ab.x + ab.y * ab.y).max(1e-9))
                .clamp(0.0, 1.0);
                let c = a + ab * t;
                best = best.min(((p - c).x.powi(2) + (p - c).y.powi(2)).sqrt());
            }
            best
        };

        let mut spill = 0;
        for y in 0..H {
            for x in 0..W {
                let i = ((y * W + x) * 4) as usize;
                if !is_colour(i) {
                    continue;
                }
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                // Cor legítima: dentro do polígono, ou sob a linha (+1 px de AA).
                if inside(p) || dist_to_line(p) <= width_px * 0.5 + 1.5 {
                    continue;
                }
                spill += 1;
            }
        }
        println!("estrela, linha {width_px}px: {spill} px de cor FORA da arte");
        assert!(
            spill <= 20,
            "linha de {width_px}px: {spill} pixels de cor caíram FORA da arte — o contorno \
             do fill nao segue a linha (as quinas dessincronizam)"
        );
    }
}

/// **A MESMA arte, descrita em duas escalas 100× distantes, tem de render A MESMA
/// IMAGEM.**
///
/// Este é o gate que faltava, e a razão pela qual o BUGS #20 sobreviveu a oito oráculos
/// de pixel: todos eles descreviam a arte num mundo que já era pixel, onde a conversão
/// px→mundo é `1` e **nenhum erro de unidade tem como aparecer**. A dilatação ficou 100×
/// grande demais, a cor virou um blob que ignorava o line-art, e a suíte inteira ficou
/// verde — o defeito foi achado por um humano olhando a tela.
///
/// A propriedade é a mais forte disponível: a arte não sabe em que unidade foi escrita.
/// Um círculo de raio 110 a 1 px/unidade e um de raio 1,1 a 100 px/unidade são o MESMO
/// desenho, e o produto tem de pintá-los igual. Qualquer grandeza que atravesse a
/// fronteira px↔mundo sem se converter quebra a igualdade — e quebra em proporção ao
/// fator, que aqui é 100.
///
/// Mutação que ele mata: `margin_world` ignorando o `px_per_world` (o BUGS #20 literal).
/// A margem vira 1,0 unidade de MUNDO = 100 px de dilatação num desenho de 320 px.
#[test]
#[ignore = "GPU"]
fn the_same_art_at_the_products_scale_renders_the_same() {
    let Some((device, queue)) = device() else {
        return;
    };
    for (width_px, hardness) in [(8.0f32, 1.0f32), (16.0, 1.0), (16.0, 0.35), (32.0, 0.35)] {
        let a = render(&device, &queue, &scene_h(width_px, hardness));
        let b = render_with(
            &device,
            &queue,
            &scene_world(width_px, hardness),
            &world_camera(PRODUCT_PPW),
        );
        assert_eq!(a.len(), b.len(), "mesmo alvo, mesmo tamanho");

        // Quantos pixels diferem, e por quanto. As duas cenas percorrem caminhos de f32
        // diferentes (as coordenadas são divididas por 100 e remultiplicadas), então a
        // igualdade EXATA não é a política — mas a diferença tem de ser ruído de
        // arredondamento, não geometria.
        let mut differing = 0usize;
        let mut worst = 0i32;
        for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
            let d = (0..3)
                .map(|c| (pa[c] as i32 - pb[c] as i32).abs())
                .max()
                .unwrap_or(0);
            if d > 0 {
                differing += 1;
            }
            worst = worst.max(d);
        }
        let total = a.len() / 4;
        let pct = 100.0 * differing as f32 / total as f32;
        println!(
            "linha {width_px}px h{hardness}: {differing}/{total} px diferem ({pct:.2}%), \
             pior delta {worst}"
        );

        // ⚠️ **Dois limites, porque são duas perguntas** (a lição do passe de luz na GPU):
        // *quão longe* um pixel foi, e *quantos* foram.
        //
        // Os dois números saem do FOSSO MEDIDO entre correto e quebrado, nunca colados
        // na observação — um limite raspando o valor observado é um limite ajustado para
        // passar, e ele flaka no primeiro driver diferente:
        //
        // |            | pixels diferentes | pior delta |
        // |---|---|---|
        // | correto    | 0,01–0,03 %       | 1 (23 na linha fina e dura: um pixel de AA) |
        // | **BUGS #20** | **43,4 %**      | **153** |
        //
        // A CONTAGEM é a discriminadora de verdade (fosso de ~2000×); a magnitude só
        // acompanha, com folga, porque na borda de uma silhueta anti-aliasada um pixel
        // isolado pode saltar bastante sem que a geometria tenha se mexido.
        // ⚠️ **A magnitude é a metade FRACA, e ficou mais fraca de propósito.** Com a
        // margem em 0,25 a borda da cor pousa exatamente onde o alpha da linha é mais
        // íngreme, então um pixel de AA na fronteira salta mais entre as duas escalas
        // (20 → 82) sem que geometria nenhuma tenha se movido — é o preço de encostar a
        // cor na linha, não um defeito. Quem prova é a CONTAGEM (0,04% contra 43,4%).
        assert!(
            worst <= 120,
            "linha {width_px}px: pior delta {worst} — a mesma arte em duas escalas \
             esta sendo pintada DIFERENTE (grandeza que nao atravessa px<->mundo)"
        );
        assert!(
            pct <= 1.0,
            "linha {width_px}px: {pct:.2}% dos pixels diferem — isso e geometria, \
             nao arredondamento"
        );
    }
}

/// A sonda do encaixe na escala do produto: quantas amostras mostram **fundo sob a
/// linha**, quantas mostram **cor além dela**, e quantas caíram **fora do alvo**.
///
/// Porta ÚNICA — o diagnóstico (`sweep_zoom`) e o gate
/// (`the_fill_reaches_under_the_line_at_product_scale`) medem pela MESMA função. Duas
/// sondas separadas divergem, e aí a tabela que se lê e o número que trava o CI param de
/// falar da mesma coisa.
fn probe_fit_at(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    ppw: f32,
    width_px: f32,
) -> (usize, usize, usize) {
    // Círculo LISO: a sonda é um círculo perfeito, então a arte tem de ser um também.
    // Com o tremor (que vive em MUNDO e cresce em tela com o zoom) a sonda passaria a
    // cortar a própria linha, e a tendência medida seria a do artefato.
    let d = scene_world_full(width_px, 0.35, ppw, false);
    let cam = world_camera_at(ppw, Vec2::new(1.6, 1.6));
    let px = render_with(device, queue, &d, &cam);
    let to_screen = |w: Vec2| -> (f32, f32) {
        (
            (w.x - 1.6) * ppw + W as f32 * 0.5,
            (w.y - 1.6) * ppw + H as f32 * 0.5,
        )
    };
    let at = |x: f32, y: f32| -> Option<(u8, u8)> {
        let (xi, yi) = (x.round() as i32, y.round() as i32);
        if xi < 0 || yi < 0 || xi >= W as i32 || yi >= H as i32 {
            return None;
        }
        let i = (yi as usize * W as usize + xi as usize) * 4;
        Some((px[i], px[i + 2]))
    };
    let r_world = 110.0 / 100.0;
    let half_px = width_px * 0.5 * ppw / 100.0;
    let is_bg = |r_: u8, b: u8| r_ < 60 && b < 60;
    let is_colour = |r_: u8, b: u8| r_ > 120 && b < 130;
    let (mut bare, mut spill, mut outside) = (0usize, 0usize, 0usize);
    for k in 0..256 {
        let t = k as f32 / 256.0;
        let (c, s) = unit_circle(t);
        let mut probe = |off_px: f32, want_colour: bool, hit: &mut usize| {
            let rr = r_world + off_px / ppw;
            let (x, y) = to_screen(Vec2::new(1.6 + rr * c, 1.6 + rr * s));
            match at(x, y) {
                None => outside += 1,
                Some((r_, b)) => {
                    let v = if want_colour {
                        is_colour(r_, b)
                    } else {
                        is_bg(r_, b)
                    };
                    if v {
                        *hit += 1;
                    }
                }
            }
        };
        let mut step = -half_px + 1.0;
        while step <= half_px - 1.0 {
            probe(step, false, &mut bare);
            step += 1.0;
        }
        let mut step = half_px + 2.0;
        while step <= half_px + 8.0 {
            probe(step, true, &mut spill);
            step += 1.0;
        }
    }
    (bare, spill, outside)
}

/// **DIAGNÓSTICO: a franja em função do ZOOM** (smoke do Enio, 2026-07-18 — *"o fill está
/// extrapolando um pouquinho para fora da linha"*).
///
/// Duas grandezas se encontram na margem e são constantes em unidades DIFERENTES: a
/// margem é fixa em MUNDO, o erro de vetorização que ela compensa é ~fixo em TELA. A
/// hipótese era transbordo crescendo com o zoom; a tabela **refutou** (transbordo zero em
/// todo zoom) e mostrou o defeito OPOSTO — sub-cobertura ao afastar. Foi ela que motivou
/// a compensação COM SINAL.
///
/// ⚠️ **Amostra só o que está NA TELA.** A 1ª versão lia fora do alvo e contava `(0,0)`
/// como *fundo* — a 400 px/unidade o anel inteiro está fora de um alvo de 320 px, e ela
/// "mostrou" 16 105 px de vazamento que eram a borda da textura.
#[test]
#[ignore = "GPU"]
fn sweep_zoom() {
    let Some((device, queue)) = device() else {
        return;
    };
    println!(" ppw | margem tela | linha | fundo sob a linha | transbordo | fora da tela");
    for ppw in [25.0f32, 50.0, 100.0, 140.0] {
        for width_px in [8.0f32, 16.0] {
            let (bare, spill, outside) = probe_fit_at(&device, &queue, ppw, width_px);
            println!(
                "{ppw:>4} | {:>11.2} | {width_px:>4}px | {bare:>17} | {spill:>10} | {outside:>12}",
                0.005 * ppw
            );
        }
    }
}

/// **A cor alcança POR BAIXO da linha, na escala do produto.**
///
/// O gate que pina o ganho da compensação com sinal — e ele existe porque as mutações
/// revelaram que os outros dez **não conseguiam vê-lo**: tirar a compensação inteira
/// (voltar à margem uniforme) deixava a suíte de pixel inteira verde. Eles afirmam a
/// ausência de defeito GROSSO (`fundo = 0` na fixture trêmula, transbordo < 2%), e uma
/// margem uniforme também satisfaz isso. O que nenhum deles media era **quão bem** a cor
/// entra sob a linha.
///
/// Somado sobre a faixa de zoom (as 6 linhas do `sweep_zoom` que ficam inteiras na tela):
///
/// | lei | amostras de fundo sob a linha |
/// |---|---|
/// | margem FIXA, sem compensação (até 2026-07-18) | 158 |
/// | **compensação + margem em FRAÇÃO (0,03)** | **156** |
/// | sem margem nenhuma | 224 |
///
/// ⚠️ **Este gate NÃO discrimina a mudança de 2026-07-18** (156 contra 158 não é fosso),
/// e ele não deve fingir que sim. O que a mudança conserta é a DISTRIBUIÇÃO da franja
/// entre espessuras de traço, e quem guarda isso é
/// `dilate::the_margin_scales_with_the_line_it_dresses` — a propriedade some quando se
/// soma tudo sobre a figura. O papel deste aqui é o de sempre: barrar uma regressão
/// GROSSA de cobertura (224 = a lei sem margem nenhuma).
///
/// O limite fica em 175, entre 156 e 224.
#[test]
#[ignore = "GPU"]
fn the_fill_reaches_under_the_line_at_product_scale() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut total = 0usize;
    for ppw in [25.0f32, 50.0, 100.0] {
        for width_px in [8.0f32, 16.0] {
            let (bare, _, outside) = probe_fit_at(&device, &queue, ppw, width_px);
            assert_eq!(
                outside, 0,
                "premissa: nesta faixa de zoom o anel inteiro esta na tela"
            );
            total += bare;
        }
    }
    println!("fundo sob a linha, somado na faixa de zoom: {total}");
    assert!(
        total <= 175,
        "{total} amostras de FUNDO sob a linha — a cor nao esta entrando por baixo do          line-art (158 = o que uma margem uniforme, sem compensacao, deixa)"
    );
}
