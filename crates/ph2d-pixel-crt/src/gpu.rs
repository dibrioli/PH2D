//! O passe WGSL da CRT (`crt.wgsl`) — a vista. A lei está no doc da crate; aqui só se monta.
//!
//! **A porta** (onda seguinte): quem desenha a prévia ou a janela do jogo cria UMA vez
//! [`CrtGpu::new`] com o formato do alvo dele e, por quadro, chama [`CrtGpu::codifica`] com a
//! textura do quadro já composto (RGBA8, tamanho do documento) e o alvo do tamanho
//! `w·escala × h·escala`. Com [`crate::CrtParams::default`] não precisa de o chamar: a saída seria
//! a ampliação nearest que ele já faz.

use crate::{CrtParams, LADO_MAXIMO};
use ph2d_gpu::GpuContext;

/// Bytes do uniforme (`struct Params` do WGSL: 4 × u32/i32 + 4 × f32).
const UNIFORME: u64 = 32;

/// O pipeline da CRT para um formato de alvo.
pub struct CrtGpu {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniforme: wgpu::Buffer,
}

fn bytes_dos_params(w: u32, h: u32, escala: u32, p: &CrtParams) -> [u8; 32] {
    let mut b = [0u8; 32];
    b[0..4].copy_from_slice(&w.to_le_bytes());
    b[4..8].copy_from_slice(&h.to_le_bytes());
    b[8..12].copy_from_slice(&escala.to_le_bytes());
    b[12..16].copy_from_slice(&p.k_q12().to_le_bytes());
    b[16..20].copy_from_slice(&p.scanline.to_le_bytes());
    b[20..24].copy_from_slice(&p.mascara.to_le_bytes());
    b[24..28].copy_from_slice(&p.brilho.to_le_bytes());
    b[28..32].copy_from_slice(&p.bloom.to_le_bytes());
    b
}

impl CrtGpu {
    /// Monta o pipeline. `formato` é o do alvo — um formato NÃO-sRGB (`Rgba8Unorm`/`Bgra8Unorm`)
    /// dá os níveis da lei byte a byte; num alvo `*Srgb` a placa codifica por cima.
    #[must_use]
    pub fn new(device: &wgpu::Device, formato: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-pixel-crt shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("crt.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-pixel-crt bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(UNIFORME),
                    },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-pixel-crt layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-pixel-crt pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: formato,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let uniforme = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-pixel-crt params"),
            size: UNIFORME,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            layout,
            uniforme,
        }
    }

    /// Grava o passe: `origem` (`w × h`, RGBA8 não-sRGB) ⇒ `alvo` (`w·escala × h·escala`).
    #[allow(clippy::too_many_arguments)]
    pub fn codifica(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        origem: &wgpu::TextureView,
        alvo: &wgpu::TextureView,
        (w, h, escala): (u32, u32, u32),
        p: &CrtParams,
        timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        queue.write_buffer(&self.uniforme, 0, &bytes_dos_params(w, h, escala, p));
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-pixel-crt bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(origem),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.uniforme.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-pixel-crt pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: alvo,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: timestamps,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// Texturas de uma corrida na placa (origem carregada + alvo legível).
pub struct Corrida {
    pub origem: wgpu::Texture,
    pub alvo: wgpu::Texture,
    pub dims: (u32, u32, u32),
}

/// Carrega `rgba` numa textura de origem e cria o alvo `Rgba8Unorm` da saída.
///
/// # Panics
/// Se a saída passa [`LADO_MAXIMO`] ou `rgba` não tem `w·h·4` bytes.
#[must_use]
pub fn prepara(gpu: &GpuContext, rgba: &[u8], w: u32, h: u32, escala: u32) -> Corrida {
    assert_eq!(rgba.len(), (w * h * 4) as usize);
    assert!(w * escala < LADO_MAXIMO && h * escala < LADO_MAXIMO);
    let tex = |label, width, height, usage| {
        gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage,
            view_formats: &[],
        })
    };
    let origem = tex(
        "ph2d-pixel-crt origem",
        w,
        h,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &origem,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    let alvo = tex(
        "ph2d-pixel-crt alvo",
        w * escala,
        h * escala,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    Corrida {
        origem,
        alvo,
        dims: (w, h, escala),
    }
}

/// Lê o alvo de volta, RGBA8 compacto.
///
/// # Errors
/// Falha do `poll`/`map_async` da placa.
pub fn le_alvo(gpu: &GpuContext, c: &Corrida) -> Result<Vec<u8>, String> {
    let (ow, oh) = (c.dims.0 * c.dims.2, c.dims.1 * c.dims.2);
    let linha = ow * 4;
    let al = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let passo = linha.div_ceil(al) * al;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-pixel-crt leitura"),
        size: u64::from(passo) * u64::from(oh),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &c.alvo,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(passo),
                rows_per_image: Some(oh),
            },
        },
        wgpu::Extent3d {
            width: ow,
            height: oh,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| format!("poll: {e}"))?;
    rx.recv()
        .map_err(|e| format!("canal: {e}"))?
        .map_err(|e| format!("map_async: {e}"))?;
    let m = slice.get_mapped_range().map_err(|e| format!("mapa: {e}"))?;
    let mut out = Vec::with_capacity((linha * oh) as usize);
    for r in 0..oh as usize {
        let i = r * passo as usize;
        out.extend_from_slice(&m[i..i + linha as usize]);
    }
    drop(m);
    staging.unmap();
    Ok(out)
}

/// A CRT inteira na placa (carrega, um passe, lê) — o par do [`crate::crt_cpu`] nos gates.
///
/// # Errors
/// Falha da leitura ([`le_alvo`]).
pub fn crt_na_placa(
    gpu: &GpuContext,
    rgba: &[u8],
    w: u32,
    h: u32,
    escala: u32,
    p: &CrtParams,
) -> Result<Vec<u8>, String> {
    let crt = CrtGpu::new(&gpu.device, wgpu::TextureFormat::Rgba8Unorm);
    let c = prepara(gpu, rgba, w, h, escala);
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let (o, a) = (
        c.origem
            .create_view(&wgpu::TextureViewDescriptor::default()),
        c.alvo.create_view(&wgpu::TextureViewDescriptor::default()),
    );
    crt.codifica(&gpu.device, &gpu.queue, &mut enc, &o, &a, c.dims, p, None);
    gpu.queue.submit([enc.finish()]);
    le_alvo(gpu, &c)
}
