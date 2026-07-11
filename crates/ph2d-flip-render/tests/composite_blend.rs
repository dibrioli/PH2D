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
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_render::{CameraRaw, FlipCompose, FlipRenderer, pack_drawing};
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
    for p in [
        min,
        Vec2::new(max.x, min.y),
        max,
        Vec2::new(min.x, max.y),
    ] {
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
    let bytes_per_row = W * 8; // 4 halfs = 8 bytes; 64*8 = 512 (256-alinhado)
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
    let halfs: &[u16] = bytemuck::cast_slice(&data[..]);
    halfs.iter().map(|&h| half_to_f32(h)).collect()
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
            key: 1,
            blend_mode: BlendMode::Normal.to_u8(),
            opacity: 1.0,
        },
        LayerOp::Layer {
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
    assert!(overlap[3] > 0.9, "cobertura alpha na sobreposição = {}", overlap[3]);
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
            key: 1,
            blend_mode: BlendMode::Normal.to_u8(),
            opacity: 1.0,
        },
        LayerOp::Layer {
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
