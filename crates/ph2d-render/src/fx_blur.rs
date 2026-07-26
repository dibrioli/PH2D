//! **`FxBlurPass`** — o borrão de GPU do FX raster vetorial (plano 24), 100% residente.
//!
//! Recebe a forma isolada já rasterizada numa textura (premultiplicada — é o que o Vello escreve)
//! e produz a imagem final BORRADA numa textura de saída (alfa RETO — o que o override do Vello
//! espera), **sem readback e sem blur na CPU**. É o que torna o FX viável em RUNTIME de jogo: a
//! forma pode animar todo frame que o custo é um render + dois passes separáveis na placa, nunca
//! um roundtrip GPU→CPU→GPU.
//!
//! Molde: [`crate::impasto_light::ImpastoLightPass`] (passe bespoke textura→textura). Gaussiana
//! **separável** (horizontal, depois vertical) por ping-pong, com o *finalize* fundido no passe
//! vertical:
//! - **Blur** (`kind = 0`): des-premultiplica o borrado → alfa reto, `alpha *= opacity`.
//! - **Glow / Drop Shadow** (`kind = 1/2`): descarta a cor da forma e pinta a **cor do efeito**
//!   com o alfa borrado da silhueta (`tint.rgb`, `alpha = borrado.a · tint.a · opacity`). O
//!   deslocamento da sombra é do COMPOSITE (o retângulo de destino), não deste passe.
//!
//! A saída é `Rgba8Unorm` com `STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC` — pronta para o
//! `Renderer::register_texture` do Vello (que a copia GPU→GPU para o atlas).

use ph2d_gpu::GpuContext;

/// Meia-largura MÁXIMA do kernel (o laço do shader é limitado por ela). `96` cobre `sigma ≈ 32`
/// px com suporte de 3σ — um borrão bem forte na tela. Acima disso o kernel satura no cap (o
/// borrão para de crescer), o que é um limite de CUSTO honesto do passe, não um teto de produto.
const MAX_HALF: u32 = 96;

const FX_BLUR_WGSL: &str = r#"
struct Globals {
    dims: vec2<u32>,
    half: u32,
    kind: u32,
    tint: vec4<f32>,
    inv_two_sigma2: f32,
    opacity: f32,
    pad0: f32,
    pad1: f32,
};
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> g: Globals;

fn gauss(i: f32) -> f32 { return exp(-i * i * g.inv_two_sigma2); }

// Horizontal: entrada premultiplicada -> temp premultiplicada.
@compute @workgroup_size(8, 8, 1)
fn cs_blur_h(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    let h = i32(g.half);
    let w = i32(g.dims.x);
    for (var k = -h; k <= h; k = k + 1) {
        let sx = clamp(i32(id.x) + k, 0, w - 1);
        let wt = gauss(f32(k));
        acc = acc + textureLoad(src, vec2<i32>(sx, i32(id.y)), 0) * wt;
        wsum = wsum + wt;
    }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), acc / wsum);
}

// Vertical + finalize: temp premultiplicada -> saída RETA (tint/opacity aplicados aqui).
@compute @workgroup_size(8, 8, 1)
fn cs_finalize_v(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    let h = i32(g.half);
    let hh = i32(g.dims.y);
    for (var k = -h; k <= h; k = k + 1) {
        let sy = clamp(i32(id.y) + k, 0, hh - 1);
        let wt = gauss(f32(k));
        acc = acc + textureLoad(src, vec2<i32>(i32(id.x), sy), 0) * wt;
        wsum = wsum + wt;
    }
    let premul = acc / wsum;
    var outc: vec4<f32>;
    if (g.kind == 0u) {
        // Blur: des-premultiplica (a saída do override do Vello é RETA).
        var rgb = vec3<f32>(0.0);
        if (premul.a > 0.0001) { rgb = premul.rgb / premul.a; }
        outc = vec4<f32>(rgb, premul.a * g.opacity);
    } else {
        // Glow / Drop Shadow: a cor é do efeito; o alfa é a silhueta borrada.
        outc = vec4<f32>(g.tint.rgb, premul.a * g.tint.a * g.opacity);
    }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), clamp(outc, vec4<f32>(0.0), vec4<f32>(1.0)));
}
"#;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    dims: [u32; 2],
    half: u32,
    kind: u32,
    tint: [f32; 4],
    inv_two_sigma2: f32,
    opacity: f32,
    pad0: f32,
    pad1: f32,
}

struct Tex {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

/// Uma textura `Rgba8Unorm` com os usos que o passe (e o override do Vello) exigem.
fn make_tex(gpu: &GpuContext, w: u32, h: u32) -> Tex {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render fx_blur tex"),
        size: wgpu::Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex { texture, view }
}

/// Cria uma textura de saída para o FX (o chamador a mantém viva por-forma — o Vello a copia no
/// render, DEPOIS do recook). Mesma configuração da temp interna.
#[must_use]
pub fn make_output_texture(gpu: &GpuContext, w: u32, h: u32) -> wgpu::Texture {
    make_tex(gpu, w, h).texture
}

/// O passe de borrão do FX. Pipeline build-once; a textura temporária de ping-pong é reusada e
/// redimensionada por forma.
pub struct FxBlurPass {
    pipeline_h: wgpu::ComputePipeline,
    pipeline_v: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    globals: wgpu::Buffer,
    temp: Option<(Tex, u32, u32)>,
}

impl FxBlurPass {
    /// Constrói os dois pipelines (H e V+finalize) de um shader. Barato — nenhuma textura até o
    /// 1º [`Self::run`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render fx_blur shader"),
                source: wgpu::ShaderSource::Wgsl(FX_BLUR_WGSL.into()),
            });
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render fx_blur bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render fx_blur layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });
        let make_pipe = |entry: &str| {
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ph2d-render fx_blur pipeline"),
                    layout: Some(&layout),
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                })
        };
        let pipeline_h = make_pipe("cs_blur_h");
        let pipeline_v = make_pipe("cs_finalize_v");
        let globals = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render fx_blur globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline_h,
            pipeline_v,
            bgl,
            globals,
            temp: None,
        }
    }

    /// Borra `src` (premultiplicada, `w×h`) em `dst` (reta, `w×h`, do chamador), aplicando o
    /// tint/opacity do modo. Encoda H → temp → (V+finalize) → dst num só submit.
    ///
    /// `sigma_px` é o desvio do Gaussiano em pixels de TELA; `kind` 0=Blur/1=Glow/2=Drop Shadow;
    /// `tint` a cor reta do efeito; `opacity` em `[0,1]`.
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &mut self,
        gpu: &GpuContext,
        src: &wgpu::Texture,
        dst: &wgpu::Texture,
        w: u32,
        h: u32,
        sigma_px: f32,
        kind: u8,
        tint: [f32; 4],
        opacity: f32,
    ) {
        if w == 0 || h == 0 {
            return;
        }
        self.ensure_temp(gpu, w, h);
        let sigma = sigma_px.max(1e-4);
        let half = ((3.0 * sigma).ceil() as u32).clamp(1, MAX_HALF);
        let g = Globals {
            dims: [w, h],
            half,
            kind: u32::from(kind),
            tint,
            inv_two_sigma2: 1.0 / (2.0 * sigma * sigma),
            opacity,
            pad0: 0.0,
            pad1: 0.0,
        };
        gpu.queue
            .write_buffer(&self.globals, 0, bytemuck::bytes_of(&g));

        let temp = &self.temp.as_ref().expect("just ensured").0;
        let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());
        let dst_view = dst.create_view(&wgpu::TextureViewDescriptor::default());
        let bg_h = self.bind(gpu, &src_view, &temp.view);
        let bg_v = self.bind(gpu, &temp.view, &dst_view);

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render fx_blur encoder"),
            });
        let (gx, gy) = (w.div_ceil(8), h.div_ceil(8));
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-render fx_blur H"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline_h);
            pass.set_bind_group(0, &bg_h, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-render fx_blur V"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline_v);
            pass.set_bind_group(0, &bg_v, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        gpu.queue.submit([encoder.finish()]);
    }

    fn bind(
        &self,
        gpu: &GpuContext,
        src: &wgpu::TextureView,
        dst: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render fx_blur bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(dst),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.globals.as_entire_binding(),
                },
            ],
        })
    }

    fn ensure_temp(&mut self, gpu: &GpuContext, w: u32, h: u32) {
        if matches!(self.temp, Some((_, tw, th)) if tw == w && th == h) {
            return;
        }
        self.temp = Some((make_tex(gpu, w, h), w, h));
    }
}
