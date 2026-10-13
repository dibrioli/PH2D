//! ⭐ **OS BYTES CODIFICADOS DE QUALQUER TEXTURA** (P4 do Motion, doc 132 §4.1) — a porta de leitura
//! irmã do [`crate::SpriteRenderer::texture_view_and_dims`]: o que uma loja guarda (o átlas, uma
//! individual, uma ASSADA) em RGBA de 8 bits no espaço do desenhista, para quem desenha a imagem na
//! CPU (a cena vectorial do Motion, `motion_leaf_images`).
//!
//! Uma textura assada é COMPRIMIDA (KTX2: BC7, ASTC…): não se copia para um buffer como bytes de
//! pixel. Lê-se como a placa a lê — `textureLoad` (a descompressão e a descodificação sRGB são da
//! placa) e a conta [`CODIFICA_WGSL`] de volta ao byte —, num passe de ecrã para um `Rgba8Unorm`.
//! ⇒ a rota da CPU e a placa (o passe das imagens do Motion, que faz a MESMA leitura e a MESMA conta)
//! veem o mesmo texel.

use ph2d_gpu::GpuContext;

/// A conta que leva o valor LIDO de uma textura (linear, se o formato é `…Srgb`) ao byte codificado
/// — partilhada com o passe das imagens do Motion: duas redacções divergiriam no 1.º arredondamento.
pub const CODIFICA_WGSL: &str = "
fn codifica(c: vec3<f32>) -> vec3<f32> {
    let baixo = c * 12.92;
    let alto = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(alto, baixo, c <= vec3<f32>(0.0031308));
}
";

const LER_WGSL: &str = "
@group(0) @binding(0) var fonte: texture_2d<f32>;
@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}
@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let l = textureLoad(fonte, vec2<i32>(p.xy), 0);
    return vec4<f32>(codifica(l.rgb), l.a);
}
";

/// **Lê `vista` (`w × h`, o nível 0) em RGBA de 8 bits CODIFICADO**, linha a linha sem folga — pela
/// placa, à espera dela (uma leitura de ferramenta, nunca do quadro).
fn le_rgba8_codificado(gpu: &GpuContext, vista: &wgpu::TextureView, w: u32, h: u32) -> Vec<u8> {
    let device = &gpu.device;
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ph2d-render leitura rgba8"),
        source: wgpu::ShaderSource::Wgsl(format!("{CODIFICA_WGSL}{LER_WGSL}").into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ph2d-render leitura rgba8"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
    });
    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ph2d-render leitura rgba8"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ph2d-render leitura rgba8"),
        layout: Some(&pl),
        vertex: wgpu::VertexState {
            module: &modulo,
            entry_point: Some("vs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &modulo,
            entry_point: Some("fs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    let (w, h) = (w.max(1), h.max(1));
    let alvo = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render leitura rgba8"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let alvo_vista = alvo.create_view(&wgpu::TextureViewDescriptor::default());
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-render leitura rgba8"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(vista),
        }],
    });
    let linha =
        (w * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let leitura = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-render leitura rgba8"),
        size: u64::from(linha * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-render leitura rgba8"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &alvo_vista,
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
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        alvo.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &leitura,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(linha),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(Some(enc.finish()));
    leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let bytes = leitura.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let ini = (y * linha) as usize;
        out.extend_from_slice(&bytes[ini..ini + (w * 4) as usize]);
    }
    drop(bytes);
    leitura.unmap();
    out
}

impl crate::SpriteRenderer {
    /// ⭐ **Os bytes RGBA8 codificados por trás de um `texture_id`** — as três lojas: o átlas e as
    /// individuais pela cópia de sempre (já são bytes de 8 bits; a individual de 16 bits converte), a
    /// ASSADA pela leitura da placa (`le_rgba8_codificado`). `None` para um id que nenhuma conhece.
    #[must_use]
    pub fn le_rgba8(&self, gpu: &GpuContext, texture_id: u32) -> Option<(u32, u32, Vec<u8>)> {
        if texture_id == crate::RenderInstance::ATLAS_TEXTURE_ID {
            return Some(self.atlas().readback_mip(gpu, 0));
        }
        if crate::RenderInstance::is_cooked_texture_id(texture_id) {
            let (vista, w, h) = self.texture_view_and_dims(texture_id)?;
            return Some((w, h, le_rgba8_codificado(gpu, vista, w, h)));
        }
        self.individual().readback_rgba8(gpu, texture_id).ok()
    }
}
