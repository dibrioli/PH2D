//! T1.7 — validação GPU end-to-end do SEAM real de composição por-camada:
//! `stage_layer` (render + resolve) → `inject_slice_from_texture` → o
//! `LayerCompositor` 22-modos DE VERDADE → `blit` no alvo 16F. Espelha o que o
//! `render_loop::flip_pass` faz no shell (o mecanismo, não o contexto).
//!
//! Prova o critério do W1: "2 camadas com blend Multiply/opacity compõem certo".
//! Duas camadas cinzas se sobrepõem; a de cima é Multiply. Na sobreposição o
//! resultado é o PRODUTO (mais escuro que qualquer camada só) — a assinatura
//! inequívoca do Multiply, medida no alvo HDR real.
//!
//! `#[ignore]` — precisa de adapter GPU (roda com `--ignored`; skip gracioso sem).

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba, StrokeTip};
use ph2d_flip_render::{
    CameraRaw, DEFAULT_TILE, FlipCompose, FlipRenderer, ScreenSpace, bin_segments, pack_drawing,
    walk_pixel,
};
use ph2d_gpu::GpuContext;
use ph2d_painter_effects::BlendMode;
use ph2d_render::layer_compositor::{
    LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};

const W: u32 = 64;
const H: u32 = 64;
/// O formato HDR do `game_rt` (o alvo real do blit).
const GAME_RT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// `GpuContext` real (mesmo device do app), ou `None` sem adapter (CI).
fn gpu() -> Option<GpuContext> {
    GpuContext::new(wgpu::Instance::default(), None).ok()
}

/// Câmera px 1:1 (mundo `[0,W]×[0,H]`, y para baixo) — igual `gpu_render.rs`.
fn pixel_camera() -> CameraRaw {
    let sx = 2.0 / W as f32;
    let sy = -2.0 / H as f32;
    let world_to_clip = [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ];
    CameraRaw::new(world_to_clip, [W as f32, H as f32], 1.0)
}

/// Um quadrado fechado PREENCHIDO (fill + contorno na mesma cor → interior
/// uniforme), de `min` a `max` em mundo.
fn filled_square(min: Vec2, max: Vec2, color: Rgba) -> FlipDrawing {
    let mut s = FlipStroke::new();
    for p in [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)] {
        s.push_point(Point {
            pos: p,
            width: 0.5,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill {
        color,
        opacity: 1.0,
    });
    let mut d = FlipDrawing::default();
    d.strokes.push(s);
    d
}

/// Provider dummy (o seam usa `inject`; o dummy só passa o filtro de tamanho).
struct Dummy<'a> {
    px: &'a [u8],
}
impl LayerPixelProvider for Dummy<'_> {
    fn layer_pixels(&self, _k: u64) -> Option<LayerPixels<'_>> {
        Some(LayerPixels {
            version: 0,
            rgba8: self.px,
            dirty: None,
        })
    }
}

/// IEEE half → f32 (readback do alvo `Rgba16Float`).
fn half_to_f32(h: u16) -> f32 {
    let sign = (h >> 15) & 1;
    let exp = (h >> 10) & 0x1f;
    let mant = h & 0x3ff;
    let val = if exp == 0 {
        (mant as f32) * 2f32.powi(-24)
    } else if exp == 0x1f {
        if mant == 0 { f32::INFINITY } else { f32::NAN }
    } else {
        (1.0 + (mant as f32) / 1024.0) * 2f32.powi(exp as i32 - 15)
    };
    if sign == 1 { -val } else { val }
}

/// Lê o alvo 16F de volta como `W*H*4` floats lineares.
fn readback(gpu: &GpuContext, tex: &wgpu::Texture) -> Vec<f32> {
    let bytes_per_row = W * 8; // 4 halves = 8 bytes; 64*8 = 512 (256-alinhado)
    let size = (bytes_per_row * H) as u64;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flip e2e readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let data = buf.slice(..).get_mapped_range();
    let halves: &[u16] = bytemuck::cast_slice(&data[..]);
    halves.iter().map(|&h| half_to_f32(h)).collect()
}

/// Alvo 16F limpo (transparente) — o `game_rt` antes do Flip.
fn cleared_target(gpu: &GpuContext) -> (wgpu::Texture, wgpu::TextureView) {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("flip e2e target"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: GAME_RT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("flip e2e clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    gpu.queue.submit([enc.finish()]);
    (tex, view)
}

#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn two_layers_multiply_composites_like_painter() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando o e2e de composição");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let mut comp = LayerCompositor::new(&gpu);

    // Fundo cinza 0.6 em [8,40]; topo cinza 0.5 em [24,56], Multiply. A
    // sobreposição ≈ [24,40]. Cores LINEARES (o compositor decodifica p/ linear
    // e multiplica lá): 0.6 × 0.5 = 0.30.
    let bottom = pack_drawing(&filled_square(
        Vec2::new(8.0, 8.0),
        Vec2::new(40.0, 40.0),
        Rgba::new(0.6, 0.6, 0.6, 1.0),
    ));
    let top = pack_drawing(&filled_square(
        Vec2::new(24.0, 24.0),
        Vec2::new(56.0, 56.0),
        Rgba::new(0.5, 0.5, 0.5, 1.0),
    ));
    let cam = pixel_camera();
    let ops = vec![
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 1,
            blend_mode: BlendMode::Normal.to_u8(),
            opacity: 1.0,
        },
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 2,
            blend_mode: BlendMode::Multiply.to_u8(),
            opacity: 1.0,
        },
    ];
    let dummy = vec![0u8; (W * H * 4) as usize];

    for (key, data) in [(1u64, &bottom), (2u64, &top)] {
        let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, data, (W, H));
        comp.inject_slice_from_texture(&gpu, &ops, key, slice, W, H, (0, 0, W, H), 0)
            .expect("inject");
    }
    comp.composite(&gpu, &ops, &Dummy { px: &dummy }, W, H, Region::full(W, H))
        .expect("composite");

    let (target, view) = cleared_target(&gpu);
    let out = comp.output_texture().expect("saída do compositor");
    fc.blit(&gpu.device, &gpu.queue, out, &view);

    let px = readback(&gpu, &target);
    let at = |x: u32, y: u32| {
        let i = ((y * W + x) * 4) as usize;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    };
    let bottom_only = at(14, 14); // dentro só do fundo
    let top_only = at(50, 50); // dentro só do topo
    let overlap = at(32, 32); // dentro dos dois

    // As três regiões batem os valores LINEARES esperados (tolerância p/ o
    // round-trip 8-bit sRGB do compositor).
    assert!(
        (bottom_only[0] - 0.6).abs() < 0.03,
        "fundo-só R={} (esperado ~0.6); rgba={bottom_only:?}",
        bottom_only[0]
    );
    assert!(
        (top_only[0] - 0.5).abs() < 0.03,
        "topo-só R={} (esperado ~0.5); rgba={top_only:?}",
        top_only[0]
    );
    assert!(
        (overlap[0] - 0.30).abs() < 0.03,
        "sobreposição R={} (esperado ~0.30 = 0.6×0.5 Multiply); rgba={overlap:?}",
        overlap[0]
    );
    // Assinatura do Multiply: a sobreposição é ESTRITAMENTE mais escura que
    // qualquer camada isolada.
    assert!(
        overlap[0] < bottom_only[0] - 0.1 && overlap[0] < top_only[0] - 0.1,
        "Multiply deve escurecer: ov={} bo={} to={}",
        overlap[0],
        bottom_only[0],
        top_only[0]
    );
    // Alpha coberto ≈ 1 nas três (as camadas são opacas).
    assert!(
        overlap[3] > 0.9,
        "cobertura alpha na sobreposição = {}",
        overlap[3]
    );
}

#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn top_layer_opacity_fades_toward_backdrop() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando o e2e de opacity");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let mut comp = LayerCompositor::new(&gpu);

    // Fundo preto opaco cobrindo tudo; topo branco Normal @ opacity 0.5 sobre ele
    // → cinza ~0.5 linear na sobreposição (metade do caminho fundo→topo).
    let bottom = pack_drawing(&filled_square(
        Vec2::new(4.0, 4.0),
        Vec2::new(60.0, 60.0),
        Rgba::new(0.0, 0.0, 0.0, 1.0),
    ));
    let top = pack_drawing(&filled_square(
        Vec2::new(4.0, 4.0),
        Vec2::new(60.0, 60.0),
        Rgba::new(1.0, 1.0, 1.0, 1.0),
    ));
    let cam = pixel_camera();
    let ops = vec![
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 1,
            blend_mode: BlendMode::Normal.to_u8(),
            opacity: 1.0,
        },
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 2,
            blend_mode: BlendMode::Normal.to_u8(),
            opacity: 0.5,
        },
    ];
    let dummy = vec![0u8; (W * H * 4) as usize];
    for (key, data) in [(1u64, &bottom), (2u64, &top)] {
        let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, data, (W, H));
        comp.inject_slice_from_texture(&gpu, &ops, key, slice, W, H, (0, 0, W, H), 0)
            .expect("inject");
    }
    comp.composite(&gpu, &ops, &Dummy { px: &dummy }, W, H, Region::full(W, H))
        .expect("composite");
    let (target, view) = cleared_target(&gpu);
    let out = comp.output_texture().expect("saída do compositor");
    fc.blit(&gpu.device, &gpu.queue, out, &view);

    let px = readback(&gpu, &target);
    let i = ((32 * W + 32) * 4) as usize;
    let r = px[i];
    // Half-opacity branco sobre preto = ~0.5 linear (a prova do opacity por-camada).
    assert!(
        (r - 0.5).abs() < 0.05,
        "branco @opacity 0.5 sobre preto = {r} (esperado ~0.5)"
    );
}

/// Lê a FATIA straight (`Rgba8Unorm`) que o `stage_layer` devolve — o que o `inject` consome.
fn readback_slice(gpu: &GpuContext, tex: &wgpu::Texture) -> Vec<u8> {
    let bytes_per_row = W * 4; // 64*4 = 256, já alinhado
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flip slice readback"),
        size: u64::from(bytes_per_row * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    buf.slice(..).get_mapped_range().to_vec()
}

/// 🔴 **O GATE DA FIAÇÃO** — com o motor novo armado, a fatia que o `stage_layer` entrega ao
/// compositor é a que o **percurso** desenha, não a do rasterizador.
///
/// ⚠️ **O oráculo é a COBERTURA (o alfa), e é de propósito.** O Pass B des-premultiplica o RGB
/// (aritmética própria, que não é o assunto aqui) e **deixa o alfa em paz** — então o alfa da
/// fatia é a única grandeza que atravessa a costura inteira sem ser transformada, e é exatamente
/// a resposta à pergunta do doc 12: *dado um traço, quais pixels ele acende?*
///
/// ⚠️ **A barra é DERIVADA, não escolhida:** o número atravessa `f32` → meia precisão (o `hdr`,
/// 2⁻¹¹ ≈ 4,9e-4 em magnitude 1) → 8 bits (a fatia, 1/255 ≈ 3,9e-3), e o segundo domina ⇒
/// **1,5/255**. Nada mais apertado é afirmável sobre um `u8`.
///
/// ⚠️ E o gate **compara contra o irmão de CPU**, nunca contra o rasterizador: os dois motores
/// discordam por PROJETO (é a razão de a linha existir), então exigir que a fatia case com o
/// raster seria um gate que só pode passar se o trabalho estiver desfeito.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_staged_slice_comes_from_the_new_engine_when_it_is_armed() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando a fiação do motor novo");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    // Um traço ABERTO e macio: a borda tem rampa (onde os dois motores mais divergem) e o
    // cruzamento em L exercita o run-scan por ladrilho.
    let mut st = FlipStroke::new();
    for &(x, y) in &[(10.0, 20.0), (30.0, 20.0), (30.0, 50.0)] {
        st.push_point(Point {
            pos: Vec2::new(x, y),
            width: 9.0,
            opacity: 1.0,
            color: Rgba::new(0.1, 0.1, 0.1, 1.0),
        });
    }
    st.hardness = 0.4;
    let mut d = FlipDrawing::default();
    d.strokes.push(st);
    let data = pack_drawing(&d);

    assert!(!fc.walk_engine_armed(), "o default é o motor que shipa");
    fc.set_walk_engine(&gpu.device, true);
    assert!(fc.walk_engine_armed(), "armado");
    let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
    let px = readback_slice(&gpu, slice);

    let sc = ScreenSpace::from_camera(&cam);
    let bins = bin_segments(&data, &sc, DEFAULT_TILE);
    let (mut pior, mut onde, mut n_tinta) = (0.0_f32, (0, 0), 0u32);
    for y in 0..H {
        for x in 0..W {
            let cpu = walk_pixel(&bins, &data, &sc, [x as f32 + 0.5, y as f32 + 0.5]);
            let got = f32::from(px[((y * W + x) * 4 + 3) as usize]) / 255.0;
            let d = (got - cpu[3]).abs();
            if d > pior {
                pior = d;
                onde = (x, y);
            }
            if cpu[3] > 0.0 {
                n_tinta += 1;
            }
        }
    }
    // A premissa do gate: o traço PINTA. Sem isto, `pior = 0` é verde sobre uma tela vazia.
    assert!(
        n_tinta > 500,
        "a fixture nao contem o fenomeno: so {n_tinta} px de tinta"
    );
    let bar = 1.5 / 255.0;
    assert!(
        pior <= bar,
        "a fatia do produto nao e o percurso: pior |Δ| no alfa {:.4} ({:.2}/255) em {onde:?}",
        pior,
        pior * 255.0
    );
    println!(
        "\n  fiacao OK -- pior |Δ| no alfa {:.2}/255 em {onde:?} ({n_tinta} px de tinta)",
        pior * 255.0
    );
}

/// 🔴 **O PISO** — com o motor novo armado, um quadrado PREENCHIDO continua preenchido.
///
/// ⚠️ Este gate existe porque o percurso responde por TRAÇOS e nada mais: o fill vem do pipeline
/// de triângulos que sempre o desenhou, e o kernel o compõe por BAIXO. Sem esta afirmação a
/// costura pode nascer quebrada em silêncio — e o sintoma (*"o motor novo comeu os fills"*) leria
/// como um defeito do kernel, que é justamente o que ele não é.
///
/// O oráculo é o INTERIOR (longe de qualquer borda, onde nenhum dos dois motores tem opinião):
/// preenchido é `alpha ≈ 255`, e um piso ausente é `0`. A folga de 2/255 é o round-trip de 8 bits.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_new_engine_keeps_the_fill_under_the_stroke() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando o piso de fills");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    let data = pack_drawing(&filled_square(
        Vec2::new(12.0, 12.0),
        Vec2::new(52.0, 52.0),
        Rgba::new(0.6, 0.6, 0.6, 1.0),
    ));

    fc.set_walk_engine(&gpu.device, true);
    let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
    let px = readback_slice(&gpu, slice);
    let alpha = |x: u32, y: u32| px[((y * W + x) * 4 + 3) as usize];

    // O miolo (bem dentro), e um ponto fora para o gate não passar por "tudo opaco".
    for &(x, y) in &[(32u32, 32u32), (20, 44), (44, 20)] {
        assert!(
            alpha(x, y) >= 253,
            "o fill sumiu sob o motor novo: alpha em ({x},{y}) = {}",
            alpha(x, y)
        );
    }
    assert_eq!(alpha(2, 2), 0, "fora da forma tem de continuar vazio");
    println!(
        "\n  piso OK -- miolo {} / fora {}",
        alpha(32, 32),
        alpha(2, 2)
    );
}

/// 🔴 **O GATE QUE FALTAVA** — os dois motores põem a tinta no MESMO lugar do framebuffer.
///
/// ⚠️ **Nenhum gate de paridade pode afirmar isto, e o smoke do Enio provou.** O percurso da CPU
/// (`walk_pixel`, o oráculo) e o do device leem o MESMO `ScreenSpace::point_px`, então um erro de
/// convenção ali move os DOIS lados igual e a comparação segue verde — a cegueira door-contra-door
/// que o fold da luz do Painter já documentou, aqui num sinal que o olho vê na hora. Os 23 gates
/// do `painter_look` também passavam: todos comparam FORMA, ou comparam o percurso contra um
/// oráculo que atravessa a mesma porta.
///
/// **O único oráculo possível é o RASTERIZADOR:** ele passa pelo pipeline gráfico, e é o pipeline
/// gráfico que define o que "linha 0 de uma textura" significa. Foi ele que nomeou o defeito —
/// raster nas linhas 3..8, percurso em 55..60, com `55 = 64−1−8`: espelho vertical exato, colunas
/// idênticas (o sintoma que o Enio reportou como *"canvas invertido, o pincel não pinta no lugar
/// certo"* — UMA causa, três sintomas).
///
/// ⚠️ **A fixture é ASSIMÉTRICA de propósito** (perto do topo, à esquerda): um traço centrado é
/// invariante ao espelho, e o gate passaria sobre o bug. E `hardness = 1.0` porque este gate fala
/// de POSIÇÃO — na borda macia os dois motores divergem por projeto, e essa divergência é assunto
/// dos gates de forma, não deste.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn both_engines_put_the_ink_in_the_same_place() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando a posição da tinta");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    let mut st = FlipStroke::new();
    for &(x, y) in &[(8.0, 6.0), (28.0, 6.0)] {
        st.push_point(Point {
            pos: Vec2::new(x, y),
            width: 6.0,
            opacity: 1.0,
            color: Rgba::new(0.1, 0.1, 0.1, 1.0),
        });
    }
    st.hardness = 1.0;
    let mut d = FlipDrawing::default();
    d.strokes.push(st);
    let data = pack_drawing(&d);

    // `(linha0, linha1, coluna0, coluna1, centroide_y, n)` da tinta de cada motor.
    let mut medida = [[0.0_f32; 5]; 2];
    for (i, armado) in [false, true].into_iter().enumerate() {
        fc.set_walk_engine(&gpu.device, armado);
        let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
        let px = readback_slice(&gpu, slice);
        let (mut r0, mut r1, mut c0, mut c1) = (u32::MAX, 0u32, u32::MAX, 0u32);
        let (mut soma_y, mut n) = (0.0_f64, 0u32);
        for y in 0..H {
            for x in 0..W {
                if px[((y * W + x) * 4 + 3) as usize] > 32 {
                    r0 = r0.min(y);
                    r1 = r1.max(y);
                    c0 = c0.min(x);
                    c1 = c1.max(x);
                    soma_y += f64::from(y);
                    n += 1;
                }
            }
        }
        assert!(
            n > 100,
            "{} nao pintou o bastante ({n} px) — a fixture nao contem o fenomeno",
            if armado { "o percurso" } else { "o raster" }
        );
        medida[i] = [
            r0 as f32,
            r1 as f32,
            c0 as f32,
            c1 as f32,
            (soma_y / f64::from(n)) as f32,
        ];
        println!(
            "  {} linhas {r0}..{r1}  colunas {c0}..{c1}  centro_y {:.2}  ({n} px)",
            if armado { "PERCURSO" } else { "RASTER  " },
            soma_y / f64::from(n)
        );
    }
    let [raster, percurso] = medida;
    // A caixa: 2 px de folga (a borda de união dura ainda difere em sub-pixel entre os motores).
    for (k, nome) in ["linha0", "linha1", "coluna0", "coluna1"]
        .iter()
        .enumerate()
    {
        assert!(
            (percurso[k] - raster[k]).abs() <= 2.0,
            "os motores discordam de LUGAR em {nome}: raster {} vs percurso {} \
             (espelho vertical? {})",
            raster[k],
            percurso[k],
            if (percurso[0] - (H as f32 - 1.0 - raster[1])).abs() <= 2.0 {
                "SIM — o Y do point_px"
            } else {
                "nao"
            }
        );
    }
    // O centroide pega um espelho mesmo se a caixa por acaso ficar simétrica.
    assert!(
        (percurso[4] - raster[4]).abs() <= 1.5,
        "o centro da tinta discorda: raster {:.2} vs percurso {:.2}",
        raster[4],
        percurso[4]
    );
}

/// 🔴 **O AIRBRUSH CHEGOU AO PERCURSO** — e o oráculo é o rasterizador, que sabe a resposta
/// FECHADA numa reta (a corda pelo tubo é a projeção de Abel da esfera).
///
/// Duas metades, e a segunda é o que impede um no-op de passar:
/// 1. numa reta os dois motores traçam o MESMO perfil (barra 5/255 — o AA da borda macia mais a
///    quadratura, onde eles divergem por projeto);
/// 2. o airbrush **não é** o perfil padrão — em `dn = 0,9` o padrão vale ~12 e o airbrush ~190.
///    Sem esta metade, um `d_tau_of` que ignorasse a flag passaria na primeira.
///
/// ⚠️ **Numa reta os dois coincidem; na CURVA o percurso é mais correto** — a corda fechada do
/// rasterizador só vale para uma reta infinita, e o percurso integra a densidade ao longo do
/// caminho de verdade. É por isso que a fixture é reta: é o único lugar onde o raster é oráculo.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_airbrush_reaches_the_walk_and_matches_the_closed_form() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando o airbrush");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    // Traço RETO no meio, raio 10 (largura 20), hardness 0,5 — as linhas y = 32+3·dn.
    let perfil = |gpu: &GpuContext,
                  fr: &mut FlipRenderer,
                  fc: &mut FlipCompose,
                  airbrush: bool,
                  armado: bool| {
        let mut st = FlipStroke::new();
        for &x in &[4.0_f32, 60.0] {
            st.push_point(Point {
                pos: Vec2::new(x, 32.0),
                width: 20.0,
                opacity: 1.0,
                color: Rgba::new(0.0, 0.0, 0.0, 1.0),
            });
        }
        st.hardness = 0.5;
        st.airbrush = airbrush;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        fc.set_walk_engine(&gpu.device, armado);
        let slice = fc.stage_layer(&gpu.device, &gpu.queue, fr, &cam, &data, (W, H));
        let px = readback_slice(gpu, slice);
        [32u32, 35, 37, 39, 41].map(|y| px[((y * W + 32) * 4 + 3) as usize])
    };

    let raster_air = perfil(&gpu, &mut fr, &mut fc, true, false);
    let walk_air = perfil(&gpu, &mut fr, &mut fc, true, true);
    let walk_pad = perfil(&gpu, &mut fr, &mut fc, false, true);
    println!(
        "\n  raster airbrush {raster_air:?}\n  walk   airbrush {walk_air:?}\n  walk   padrao   {walk_pad:?}"
    );

    // (1) o perfil casa com a forma fechada.
    for (i, (&r, &w)) in raster_air.iter().zip(walk_air.iter()).enumerate() {
        let d = (i32::from(r) - i32::from(w)).abs();
        assert!(
            d <= 5,
            "o airbrush do percurso divergiu da forma fechada em dn={:.1}: raster {r} vs percurso {w}",
            i as f32 * 0.3
        );
    }
    // (2) e ele NÃO é o perfil padrão — a borda é o discriminante.
    assert!(
        i32::from(walk_air[4]) - i32::from(walk_pad[4]) > 100,
        "o airbrush nao chegou ao percurso: borda airbrush {} vs padrao {} \
         (a flag esta sendo ignorada?)",
        walk_air[4],
        walk_pad[4]
    );
}

/// **SONDA** — o perfil do airbrush, atravessado, nos dois motores.
#[test]
#[ignore = "sonda; roda com --ignored"]
fn measure_the_airbrush_profile_in_both_engines() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    // Traço RETO horizontal no meio, raio 10 (largura 20), hardness 0,5.
    for airbrush in [false, true] {
        let mut st = FlipStroke::new();
        for &x in &[4.0_f32, 60.0] {
            st.push_point(Point {
                pos: Vec2::new(x, 32.0),
                width: 20.0,
                opacity: 1.0,
                color: Rgba::new(0.0, 0.0, 0.0, 1.0),
            });
        }
        st.hardness = 0.5;
        st.airbrush = airbrush;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        println!("\n  airbrush={airbrush}  (dn: 0 = eixo, 1 = borda em y=42)");
        for armado in [false, true] {
            fc.set_walk_engine(&gpu.device, armado);
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            let px = readback_slice(&gpu, slice);
            let a = |y: u32| px[((y * W + 32) * 4 + 3) as usize];
            println!(
                "    {}  dn 0.0={:3}  0.3={:3}  0.5={:3}  0.7={:3}  0.9={:3}",
                if armado { "PERCURSO" } else { "RASTER  " },
                a(32),
                a(35),
                a(37),
                a(39),
                a(41)
            );
        }
    }
}

/// As FRONTEIRAS dos blocos acesos ao longo de uma linha da fatia — `alpha > 128` em runs.
/// Comparar fronteiras (e não alfa pixel a pixel) é o que torna o rasterizador um oráculo de
/// POSIÇÃO utilizável: os dois motores divergem na rampa da borda **por projeto**, e nada disso
/// move onde uma conta começa.
fn runs_along(px: &[u8], y: u32) -> Vec<(u32, u32)> {
    let mut v = Vec::new();
    let mut open: Option<u32> = None;
    for x in 0..W {
        let on = px[((y * W + x) * 4 + 3) as usize] > 128;
        match (on, open) {
            (true, None) => open = Some(x),
            (false, Some(s)) => {
                v.push((s, x - 1));
                open = None;
            }
            _ => {}
        }
    }
    if let Some(s) = open {
        v.push((s, W - 1));
    }
    v
}

/// 🔴 **O PINCEL PONTILHADO CHEGOU AO PERCURSO** — e o oráculo é o rasterizador, que é quem sabe
/// ONDE cada conta fica (o `arc_len` que ele lê é o mesmo, mas a lei dele é outra).
///
/// Três metades, e a segunda é a que mata um no-op:
/// 1. o percurso desenha **CONTAS**: `n` blocos separados por vãos, `n ≥ 3`;
/// 2. as contas estão **NO MESMO LUGAR** que as do raster (fronteira a fronteira, folga 1 px);
/// 3. o MESMO traço em `Continuous` é **UM** bloco — sem isto, um `TipShape::of` que devolvesse
///    sempre `Continuous` passaria em (1) e (2) sobre uma linha cheia comparada com ela mesma.
///
/// ⚠️ **`dot_spacing` 2,0 é o default do produto** (vão de um diâmetro) e a `hardness` é 1,0: este
/// gate fala de POSIÇÃO, e na borda macia os dois motores divergem por projeto — isso é assunto dos
/// gates de forma. A pitch é `dot_spacing × ref_width` = 2 × 8 = 16 px de arco, e o traço mede 48,
/// então as contas caem em `x = 8, 24, 40, 56` — a última **exatamente no fim do traço**, que é o
/// caso que a convenção meio-aberta do `bead_range` perderia sem a exceção da ponta.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_dotted_tip_reaches_the_walk_and_the_beads_land_where_the_raster_puts_them() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando o tip pontilhado");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    let fileira = |gpu: &GpuContext,
                   fr: &mut FlipRenderer,
                   fc: &mut FlipCompose,
                   tip: StrokeTip,
                   armado: bool| {
        let mut st = FlipStroke::new();
        for &x in &[8.0_f32, 56.0] {
            st.push_point(Point {
                pos: Vec2::new(x, 32.0),
                width: 8.0,
                opacity: 1.0,
                color: Rgba::new(0.0, 0.0, 0.0, 1.0),
            });
        }
        st.hardness = 1.0;
        st.tip = tip;
        st.dot_spacing = 2.0;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        fc.set_walk_engine(&gpu.device, armado);
        let slice = fc.stage_layer(&gpu.device, &gpu.queue, fr, &cam, &data, (W, H));
        runs_along(&readback_slice(gpu, slice), 32)
    };

    let raster = fileira(&gpu, &mut fr, &mut fc, StrokeTip::Dots, false);
    let walk = fileira(&gpu, &mut fr, &mut fc, StrokeTip::Dots, true);
    let cheia = fileira(&gpu, &mut fr, &mut fc, StrokeTip::Continuous, true);
    println!("\n  raster contas {raster:?}\n  walk   contas {walk:?}\n  walk   cheia  {cheia:?}");

    // (1) são CONTAS.
    assert!(
        walk.len() >= 3,
        "o percurso nao desenhou contas: {} bloco(s) em {walk:?}",
        walk.len()
    );
    // (3) e o mesmo traço contínuo é UM bloco — o discriminante.
    assert_eq!(
        cheia.len(),
        1,
        "a fixture nao contem o fenomeno: a linha CHEIA tambem saiu em pedacos ({cheia:?})"
    );
    // (2) e elas estão onde o raster as põe.
    assert_eq!(
        walk.len(),
        raster.len(),
        "contagem de contas divergiu: raster {raster:?} vs percurso {walk:?}"
    );
    for (i, (r, w)) in raster.iter().zip(walk.iter()).enumerate() {
        let (d0, d1) = (
            (i64::from(r.0) - i64::from(w.0)).abs(),
            (i64::from(r.1) - i64::from(w.1)).abs(),
        );
        assert!(
            d0 <= 1 && d1 <= 1,
            "a conta {i} nao esta onde o raster a poe: raster {r:?} vs percurso {w:?}"
        );
    }
}

/// 🔴 **O FADE SUB-PIXEL CHEGOU AO PERCURSO** — e aqui o rasterizador é oráculo **exato**, não
/// aproximado: o fade é um port do `gpencil_frag.glsl:534`, uma multiplicação na cobertura, sem
/// nenhuma quadratura envolvida.
///
/// Duas metades, e a segunda é a que mata um no-op:
/// 1. numa varredura de larguras SUB-PIXEL os dois motores dão o MESMO alfa (medido: **idêntico ao
///    nível de byte** em 0,15 / 0,3 / 0,5 / 0,8 px);
/// 2. o traço fino **não é** o traço de 1 px — o alfa CRESCE monotonicamente com a largura. Sem esta
///    metade, um `cover` que ignorasse o fade passaria na primeira se o raster também o ignorasse, e
///    um fade cravado em zero passaria por não pintar nada em lugar nenhum.
///
/// ⚠️ **A `hardness` é 1,0 de propósito:** ali `f = F_MAX` e `1 − exp(−τ)` já está **saturado**, o
/// que torna este o único regime onde escalar o `τ` em vez da cobertura seria indistinguível de não
/// fazer nada — é exatamente a confusão que a fixture tem de conter para o gate valer algo.
///
/// ⚠️ **O valor a 1 px é 166, não 255, e isso é geometria correta:** o piso põe o raio em 0,65 px, o
/// centro do pixel fica a 0,5 do eixo, então `edge = 0,5 − (0,5 − 0,65) = 0,65` ⇒ 166/255. Os dois
/// motores concordam nele, que é o que importa.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_sub_pixel_fade_reaches_the_walk_and_matches_the_raster() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando o fade sub-pixel");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    let mut picos = Vec::new();
    for w in [0.15_f32, 0.3, 0.5, 0.8] {
        let mut st = FlipStroke::new();
        for &x in &[8.0_f32, 56.0] {
            st.push_point(Point {
                pos: Vec2::new(x, 32.0),
                width: w,
                opacity: 1.0,
                color: Rgba::new(0.0, 0.0, 0.0, 1.0),
            });
        }
        st.hardness = 1.0;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        let mut pico = [0u8; 2];
        for (i, armado) in [false, true].into_iter().enumerate() {
            fc.set_walk_engine(&gpu.device, armado);
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            let px = readback_slice(&gpu, slice);
            for y in 0..H {
                for x in 16..48 {
                    pico[i] = pico[i].max(px[((y * W + x) * 4 + 3) as usize]);
                }
            }
        }
        println!(
            "  largura {w:.2} px:  raster {:>3}  percurso {:>3}",
            pico[0], pico[1]
        );
        assert_eq!(
            pico[0], pico[1],
            "o percurso nao desbota como o raster em {w:.2} px"
        );
        assert!(
            pico[1] > 0,
            "a fixture nao contem o fenomeno: nada pintou em {w:.2} px"
        );
        picos.push(pico[1]);
    }
    // A metade que mata o no-op: o alfa CRESCE com a largura. Um fade constante (1 ou 0) o achata.
    for par in picos.windows(2) {
        assert!(
            par[1] > par[0],
            "o alfa nao cresce com a largura ({:?}) — o fade esta constante",
            picos
        );
    }
}

/// 📏 **SONDA — o pico de alfa de um traço FINO nos dois motores.** Roda com
/// `cargo test -p ph2d-flip-render --release --test composite_blend measure_the_thin_line -- --ignored --nocapture`.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_thin_line_in_both_engines() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando a sonda");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    println!("  largura   raster  percurso");
    for w in [0.15_f32, 0.3, 0.5, 0.8, 1.0, 1.3, 2.0, 6.0] {
        let mut st = FlipStroke::new();
        for &x in &[8.0_f32, 56.0] {
            st.push_point(Point {
                pos: Vec2::new(x, 32.0),
                width: w,
                opacity: 1.0,
                color: Rgba::new(0.0, 0.0, 0.0, 1.0),
            });
        }
        st.hardness = 1.0;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        let mut pico = [0.0_f32; 2];
        for (i, armado) in [false, true].into_iter().enumerate() {
            fc.set_walk_engine(&gpu.device, armado);
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            let px = readback_slice(&gpu, slice);
            let mut m = 0u8;
            for y in 0..H {
                for x in 16..48 {
                    m = m.max(px[((y * W + x) * 4 + 3) as usize]);
                }
            }
            pico[i] = f32::from(m);
        }
        println!(
            "  {w:>7.2}   {:>6.0}  {:>8.0}   (fade esperado {:.3})",
            pico[0],
            pico[1],
            ph2d_flip_render::sub_pixel_fade(w)
        );
    }
}

/// 📏 **SONDA — A CONTA SUB-PIXEL, a única divergência que esta wave DEIXA ABERTA.**
///
/// Os dois motores discordam sobre o raio de uma conta abaixo de um pixel, e a discordância é
/// ESTRUTURAL, não um bug de port: o `flip.wgsl` usa `in.thickness * 0.5` — a espessura **CRUA** —
/// como raio da conta, então uma conta sub-pixel encolhe **e** desbota; o percurso usa o raio
/// **CLAMPADO** (`bead_at` lê `radius_px`) e desbota uma vez só.
///
/// **Medido** (`dot_spacing` 4,0, dureza 1, pico de alfa e contagem de pixels acesos):
///
/// ```text
///   largura   raster        percurso
///     0,40    21 (  0 px)   57 ( 76 px)
///     0,80    99 ( 40 px)  146 ( 52 px)
///     1,30   149 ( 40 px)  163 ( 32 px)
///     3,00   243 ( 48 px)  255 ( 60 px)
/// ```
///
/// ⚠️ **A 0,40 px o RASTER apaga a fileira inteira** (nenhum pixel acima de 32/255) e o percurso
/// desenha um pontilhado fraco. **Não foi "corrigido" para o raster de propósito**, e a razão é a
/// regra que o próprio `flip.wgsl` escreve ao lado do `MIN_WIDTH_PX`: *sem o clamp a fita não cobre
/// o centro de nenhum pixel e a linha PISCA*. Desaparecer ao dar zoom out é exatamente o modo de
/// falha que o par clamp+fade existe para remover — então o percurso aplica a MESMA regra às contas
/// (forma no piso, cobertura desbotada) em vez de carregar uma segunda regra para dentro de si.
///
/// **Fica ABERTO como decisão de produto:** o smoke decide se um pontilhado muito fino deve sumir
/// (raster) ou virar uma fileira fraca (percurso). Acima de 1,3 px os dois convergem, que é onde
/// todo pincel pontilhado do produto vive hoje.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_sub_pixel_bead_in_both_engines() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando a sonda da conta sub-pixel");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    println!("  largura  raster  percurso");
    for w in [0.4_f32, 0.8, 1.3, 3.0] {
        let mut st = FlipStroke::new();
        for &x in &[8.0_f32, 56.0] {
            st.push_point(Point {
                pos: Vec2::new(x, 32.0),
                width: w,
                opacity: 1.0,
                color: Rgba::new(0.0, 0.0, 0.0, 1.0),
            });
        }
        st.hardness = 1.0;
        st.tip = StrokeTip::Dots;
        st.dot_spacing = 4.0;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        let mut pico = [0u8; 2];
        let mut n = [0u32; 2];
        for (i, armado) in [false, true].into_iter().enumerate() {
            fc.set_walk_engine(&gpu.device, armado);
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            let px = readback_slice(&gpu, slice);
            for y in 0..H {
                for x in 0..W {
                    let a = px[((y * W + x) * 4 + 3) as usize];
                    pico[i] = pico[i].max(a);
                    if a > 32 {
                        n[i] += 1;
                    }
                }
            }
        }
        println!(
            "  {w:>7.2}  {:>3} ({:>3}px)  {:>3} ({:>3}px)",
            pico[0], n[0], pico[1], n[1]
        );
    }
}

/// 🔴 **A TAMPA CHATA CHEGOU AO PERCURSO** — e o rasterizador é oráculo **exato**: onde a tinta
/// acaba é geometria, não quadratura.
///
/// ⚠️ **No raster a tampa chata não é um campo de distância, é a AUSÊNCIA de geometria** — o vertex
/// estende o quad por `r` numa tampa Round e por **zero** numa Flat, e o `capsule_dn` do fragment é
/// sempre o redondo. O percurso **não tem quad**, então a truncagem tem de morar no SDF (interseção
/// com um semi-plano = um `max`). É a mesma imagem por dois mecanismos diferentes, e é por isso que
/// este gate compara CONTRA O RASTER em vez de contra uma fórmula.
///
/// Três metades, e a terceira é a que mata o no-op:
/// 1. com tampa **chata** os dois motores acabam no MESMO pixel (medido `[(16, 47)]`);
/// 2. com tampa **redonda** também (`[(10, 53)]`) — a truncagem não pode vazar para quem não pediu;
/// 3. as duas **DIFEREM** entre si por ~6 px de cada lado (o raio). Sem esta metade, um percurso que
///    ignorasse a flag passaria em (1) e (2) se o raster também a ignorasse.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_flat_cap_reaches_the_walk_and_ends_where_the_raster_ends() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando a tampa chata");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    let mut medido = Vec::new();
    for cap in [
        (ph2d_flip::Cap::Round, ph2d_flip::Cap::Round),
        (ph2d_flip::Cap::Flat, ph2d_flip::Cap::Flat),
    ] {
        let mut st = FlipStroke::new();
        for &x in &[16.0_f32, 48.0] {
            st.push_point(Point {
                pos: Vec2::new(x, 32.0),
                width: 12.0,
                opacity: 1.0,
                color: Rgba::new(0.0, 0.0, 0.0, 1.0),
            });
        }
        st.hardness = 1.0;
        st.cap = cap;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        let mut lados = [Vec::new(), Vec::new()];
        for (i, armado) in [false, true].into_iter().enumerate() {
            fc.set_walk_engine(&gpu.device, armado);
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            lados[i] = runs_along(&readback_slice(&gpu, slice), 32);
        }
        println!("  {cap:?}  raster {:?}  percurso {:?}", lados[0], lados[1]);
        assert_eq!(
            lados[0].len(),
            1,
            "a fixture nao contem o fenomeno: o raster nao pintou uma faixa unica em {cap:?}"
        );
        assert_eq!(
            lados[0], lados[1],
            "os motores discordam de onde a tinta acaba em {cap:?}"
        );
        medido.push(lados[1][0]);
    }
    // A metade que mata o no-op: chata e redonda TÊM de diferir, e por ~um raio de cada lado.
    let (redonda, chata) = (medido[0], medido[1]);
    assert!(
        chata.0 >= redonda.0 + 4 && chata.1 + 4 <= redonda.1,
        "a tampa chata nao truncou nada: chata {chata:?} contra redonda {redonda:?}"
    );
}

/// 📏 **SONDA — onde a tinta ACABA, com tampa redonda e com tampa chata, nos dois motores.**
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_where_the_cap_ends_in_both_engines() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando a sonda da tampa");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    for (nome, cap) in [
        ("ROUND", (ph2d_flip::Cap::Round, ph2d_flip::Cap::Round)),
        ("FLAT ", (ph2d_flip::Cap::Flat, ph2d_flip::Cap::Flat)),
    ] {
        let mut st = FlipStroke::new();
        // ⚠️ O fim em `48.4` cai no MEIO de um pixel (centros em 47,5 e 48,5) — numa fronteira
        // exata a rampa de anti-aliasing é degenerada e a sonda não mede nada.
        for &x in &[16.0_f32, 48.4] {
            st.push_point(Point {
                pos: Vec2::new(x, 32.0),
                width: 12.0,
                opacity: 1.0,
                color: Rgba::new(0.0, 0.0, 0.0, 1.0),
            });
        }
        st.hardness = 1.0;
        st.cap = cap;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        for (i, armado) in [false, true].into_iter().enumerate() {
            fc.set_walk_engine(&gpu.device, armado);
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            let px = readback_slice(&gpu, slice);
            let rampa: Vec<u8> = (44..52)
                .map(|x| px[((32 * W + x) * 4 + 3) as usize])
                .collect();
            println!(
                "  {nome} {}  y=32 {:?}   rampa x=44..51 {rampa:?}",
                if i == 0 { "raster  " } else { "percurso" },
                runs_along(&px, 32)
            );
        }
    }
}

/// 🔴 **O SELF OVERLAP CHEGOU AO PERCURSO** — a última flag da lista, e a única cujo mecanismo no
/// raster é o **DEPTH**.
///
/// Lá o bit troca a profundidade por-traço por uma por-SEGMENTO, então faces sobrepostas de partes
/// diferentes do mesmo traço passam o GREATER estrito e blendam `over` em vez de serem descartadas. O
/// percurso não tem faces nem depth: ele resolve UM depósito por traço, e a cobertura **satura** no
/// cruzamento (que é a regra `OFF` do GP, *"the stroke cannot overlap itself"*).
///
/// ⚠️ **A partição é inevitável, e é ÁLGEBRA:** em opacidade 1 os dois casos coincidem
/// (`1 − Π exp(−τ_p) = 1 − exp(−τ)`, que é o que o percurso já calcula), então a diferença é inteira
/// sobre **como o `opacity` entra** — e isso pede o `τ` de cada passagem separado.
///
/// Quatro asserções, e a terceira e a quarta são as que matam os no-ops:
/// 1. `OFF` os dois motores concordam nos três pontos (o controle: era assim antes desta wave);
/// 2. `ON` também;
/// 3. `ON` **difere de `OFF` no cruzamento** (191 contra 127 — senão a flag seria inerte);
/// 4. `ON` **NÃO difere no braço nem na junção** (127 nos dois — senão a partição estaria cortando
///    onde não há cruzamento, que foi exatamente o defeito da 1ª versão desta wave).
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_self_overlap_reaches_the_walk_and_composes_like_the_raster() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando o self overlap");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    // `[braço, cruzamento, junção]` por (overlap, motor).
    let mut m = [[[0u8; 3]; 2]; 2];
    for (o, overlap) in [false, true].into_iter().enumerate() {
        let data = pack_drawing(&self_crossing_x(overlap));
        for (i, armado) in [false, true].into_iter().enumerate() {
            fc.set_walk_engine(&gpu.device, armado);
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            let px = readback_slice(&gpu, slice);
            let a = |x: u32, y: u32| px[((y * W + x) * 4 + 3) as usize];
            m[o][i] = [a(20, 20), a(32, 32), a(50, 32)];
            println!(
                "  overlap={overlap:<5} {}  braço {:>3}  cruzamento {:>3}  junção {:>3}",
                if i == 0 { "raster  " } else { "percurso" },
                m[o][i][0],
                m[o][i][1],
                m[o][i][2]
            );
        }
    }
    for (o, nome) in ["OFF", "ON"].into_iter().enumerate() {
        assert_eq!(
            m[o][0], m[o][1],
            "os motores discordam com o self overlap {nome}"
        );
    }
    // A flag TEM de mudar o cruzamento — e só ele.
    assert!(
        m[1][1][1] > m[0][1][1] + 32,
        "a flag ficou inerte no cruzamento: ON {} contra OFF {}",
        m[1][1][1],
        m[0][1][1]
    );
    assert_eq!(
        (m[1][1][0], m[1][1][2]),
        (m[0][1][0], m[0][1][2]),
        "a partição cortou onde NAO ha cruzamento: braço/junção mudaram com a flag"
    );
}

/// Um "X" desenhado com UM traço só, denso (o regime real, pós-`resample_smooth`) e a opacidade em
/// 0,5 — sem ela o self overlap é invisível, porque tinta opaca já satura.
fn self_crossing_x(overlap: bool) -> FlipDrawing {
    self_crossing_x_op(overlap, 0.5)
}

/// O mesmo X, com a opacidade escolhida.
fn self_crossing_x_op(overlap: bool, op: f32) -> FlipDrawing {
    let pernas = [(12.0_f32, 12.0), (52.0, 52.0), (52.0, 12.0), (12.0, 52.0)];
    let mut st = FlipStroke::new();
    let push = |st: &mut FlipStroke, x: f32, y: f32| {
        st.push_point(Point {
            pos: Vec2::new(x, y),
            width: 9.0,
            opacity: op,
            color: Rgba::new(0.0, 0.0, 0.0, 1.0),
        });
    };
    for w in pernas.windows(2) {
        for k in 0..24 {
            let t = k as f32 / 24.0;
            push(
                &mut st,
                w[0].0 + (w[1].0 - w[0].0) * t,
                w[0].1 + (w[1].1 - w[0].1) * t,
            );
        }
    }
    push(&mut st, pernas[3].0, pernas[3].1);
    st.hardness = 1.0;
    st.self_overlap = overlap;
    let mut d = FlipDrawing::default();
    d.strokes.push(st);
    d
}

/// 📏 **SONDA — em opacidade 1 a flag muda algo? (nos DOIS motores, e onde)**
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_self_overlap_at_full_opacity() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    for armado in [false, true] {
        fc.set_walk_engine(&gpu.device, armado);
        let mut lados = Vec::new();
        for overlap in [false, true] {
            let data = pack_drawing(&self_crossing_x_op(overlap, 1.0));
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            lados.push(readback_slice(&gpu, slice));
        }
        let (mut pior, mut onde, mut n) = (0i32, (0u32, 0u32), 0u32);
        for y in 0..H {
            for x in 0..W {
                let i = ((y * W + x) * 4 + 3) as usize;
                let d = i32::from(lados[1][i]) - i32::from(lados[0][i]);
                if d.abs() > pior.abs() {
                    pior = d;
                    onde = (x, y);
                }
                if d != 0 {
                    n += 1;
                }
            }
        }
        println!(
            "  {}  opacidade 1: pior Δalfa {pior:+} em {onde:?}, {n} px diferem",
            if armado { "percurso" } else { "raster  " }
        );
    }
}

/// 📏 **SONDA — o SELF OVERLAP: o que o raster faz e o que o percurso faz hoje.**
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_self_overlap_in_both_engines() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando a sonda do self overlap");
        return;
    };
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let cam = pixel_camera();
    // Um "X" de um traço só: desce, sobe cruzando. Denso (o regime real, pós-resample).
    let pernas = [(12.0_f32, 12.0), (52.0, 52.0), (52.0, 12.0), (12.0, 52.0)];
    for overlap in [false, true] {
        let mut st = FlipStroke::new();
        for w in pernas.windows(2) {
            for k in 0..24 {
                let t = k as f32 / 24.0;
                st.push_point(Point {
                    pos: Vec2::new(
                        w[0].0 + (w[1].0 - w[0].0) * t,
                        w[0].1 + (w[1].1 - w[0].1) * t,
                    ),
                    width: 9.0,
                    opacity: 0.5,
                    color: Rgba::new(0.0, 0.0, 0.0, 1.0),
                });
            }
        }
        st.push_point(Point {
            pos: Vec2::new(pernas[3].0, pernas[3].1),
            width: 9.0,
            opacity: 0.5,
            color: Rgba::new(0.0, 0.0, 0.0, 1.0),
        });
        st.hardness = 1.0;
        st.self_overlap = overlap;
        let mut d = FlipDrawing::default();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        for (i, armado) in [false, true].into_iter().enumerate() {
            fc.set_walk_engine(&gpu.device, armado);
            let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
            let px = readback_slice(&gpu, slice);
            let a = |x: u32, y: u32| px[((y * W + x) * 4 + 3) as usize];
            println!(
                "  overlap={overlap:<5} {}  braço {:>3}  cruzamento {:>3}  junção {:>3}",
                if i == 0 { "raster  " } else { "percurso" },
                a(20, 20),
                a(32, 32),
                a(50, 32),
            );
        }
    }
}
