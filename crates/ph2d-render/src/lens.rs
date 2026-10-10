//! `LensPass` — **a LENTE da câmera na placa**, sobre o acumulador do mundo (doc 129 §3.2).
//!
//! A mesma lei de [`ph2d_lens::aplica_cpu`] em WGSL (`shaders/lens_lib.wgsl` + `lens.wgsl`),
//! gateada contra ela em pixels (`tests/it/lens_gpu.rs`). O que é por QUADRO (a matriz do balanço,
//! a geometria da vinheta, a semente do grão, as escalas da aberração) calcula-se aqui, na CPU,
//! pelas MESMAS funções da `ph2d-lens`, e viaja como uniform: a placa só faz a conta por pixel.
//!
//! ⭐ **Lente neutra ⇒ o passe não corre** e o quadro é byte-idêntico ([`LensUniform::de`] devolve
//! `None`).

use ph2d_gpu::GpuContext;
use ph2d_lens::{Cubo, Lente, aberracao, grade, grao, vinheta};

/// As leis por pixel em WGSL — o texto que a sonda do hash dos testes compõe com o seu `main`.
pub const LENS_LIB_WGSL: &str = concat!(
    ph2d_ease::wgsl_lib!(),
    ph2d_color::wgsl_perceptual!(),
    include_str!("shaders/lens_lib.wgsl"),
);

/// O shader inteiro do passe.
pub const LENS_WGSL: &str = concat!(
    ph2d_ease::wgsl_lib!(),
    ph2d_color::wgsl_perceptual!(),
    include_str!("shaders/lens_lib.wgsl"),
    include_str!("shaders/lens.wgsl"),
);

/// O uniform `LensU` do `lens.wgsl`, campo a campo.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LensUniform {
    pub rect: [f32; 4],
    pub liga: [u32; 4],
    pub modo: [u32; 4],
    pub ab: [f32; 4],
    pub ab_d: [f32; 4],
    pub vin_geo: [f32; 4],
    pub vin_par: [f32; 4],
    pub vin_alvo: [f32; 4],
    pub grade: [f32; 4],
    pub bal: [[f32; 4]; 3],
    pub lut_min: [f32; 4],
    pub lut_max: [f32; 4],
    pub grao: [f32; 4],
}

impl LensUniform {
    /// O uniform do quadro para o sub-rect `rect = [x, y, w, h]`, ou `None` se a lente é neutra
    /// (o passe não corre). `lut` é a LUT carregada, se houver.
    #[must_use]
    pub fn de(l: &Lente, lut: Option<&Cubo>, tique: u32, rect: [u32; 4]) -> Option<Self> {
        if l.neutra(lut.is_some()) || rect[2] == 0 || rect[3] == 0 {
            return None;
        }
        let quadro = [rect[2] as f32, rect[3] as f32];
        let centro = [quadro[0] * 0.5, quadro[1] * 0.5];
        let meia_diag = 0.5 * quadro[0].hypot(quadro[1]);
        let a = &l.aberracao;
        let d = if a.linear {
            aberracao::deslocamento(meia_diag, a)
        } else {
            [0.0; 2]
        };
        let geo = vinheta::geometria(&l.vinheta, quadro);
        let alvo = if l.vinheta.amount >= 0.0 {
            l.vinheta.cor
        } else {
            [1.0; 3]
        };
        let bal = grade::matriz_balanco(l.grade.temperatura, l.grade.matiz);
        let com_lut = lut.filter(|_| l.grade.lut_mix > 0.0);
        let (lut_min, lut_max) = com_lut.map_or(([0.0; 4], [0.0; 4]), |c| {
            let [a0, a1, a2] = c.dominio_min;
            let [b0, b1, b2] = c.dominio_max;
            ([a0, a1, a2, c.n as f32], [b0, b1, b2, 0.0])
        });
        let liga = |neutra: bool| u32::from(!neutra);
        Some(Self {
            rect: [
                rect[0] as f32,
                rect[1] as f32,
                rect[2] as f32,
                rect[3] as f32,
            ],
            liga: [
                liga(a.neutra()),
                liga(l.vinheta.neutra()),
                liga(l.grade.neutra(lut.is_some())),
                liga(l.grao.neutra()),
            ],
            modo: [
                u32::from(a.linear),
                u32::from(com_lut.is_some()),
                u32::from(l.grao.colorido),
                grao::semente(&l.grao, tique),
            ],
            ab: [centro[0], centro[1], 1.0 + a.amount, 1.0 - a.amount],
            ab_d: [d[0], d[1], 0.0, 0.0],
            vin_geo: [geo.centro[0], geo.centro[1], geo.semi[0], geo.semi[1]],
            vin_par: [
                geo.expoente,
                geo.sigma,
                l.vinheta.amount.abs().min(1.0),
                0.0,
            ],
            vin_alvo: [alvo[0], alvo[1], alvo[2], 0.0],
            grade: [
                l.grade.exposure.exp2(),
                l.grade.contraste.exp2(),
                1.0 + l.grade.saturacao,
                l.grade.lut_mix.min(1.0),
            ],
            bal: bal.map(|r| [r[0], r[1], r[2], 0.0]),
            lut_min,
            lut_max,
            grao: [
                l.grao.amount * grao::SIGMA_POR_AMOUNT,
                l.grao.size,
                l.grao.response,
                0.0,
            ],
        })
    }
}

/// A LUT na placa: `n³` texels `Rgba32Float` (os valores EXACTOS da CPU; `textureLoad`, sem
/// filtragem).
struct LutGpu {
    _tex: wgpu::Texture,
    view: wgpu::TextureView,
}

/// A cópia do sub-rect que o passe lê.
struct Copia {
    tex: wgpu::Texture,
    view: wgpu::TextureView,
    tamanho: (u32, u32),
}

pub struct LensPass {
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    format: wgpu::TextureFormat,
    copia: Option<Copia>,
    lut: Option<(Cubo, LutGpu)>,
    /// Ligada quando não há LUT: o layout pede SEMPRE uma textura 3D.
    lut_vazia: LutGpu,
}

impl LensPass {
    /// `format` é o do alvo — o do acumulador ([`crate::WorldRt::FORMAT`]).
    pub fn new(gpu: &GpuContext, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let texture = |binding, view_dimension| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension,
                multisampled: false,
            },
            count: None,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-render lens bgl"),
            entries: &[
                texture(0, wgpu::TextureViewDimension::D2),
                texture(1, wgpu::TextureViewDimension::D3),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-render lens layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-render lens shader"),
            source: wgpu::ShaderSource::Wgsl(LENS_WGSL.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-render lens pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                // Sem mistura: o texel da cena é SUBSTITUÍDO pelo da lente.
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render lens uniform"),
            size: std::mem::size_of::<LensUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let lut_vazia = sobe_lut(gpu, 1, &[[0.0; 3]]);
        Self {
            pipeline,
            bgl,
            uniform,
            format,
            copia: None,
            lut: None,
            lut_vazia,
        }
    }

    /// Troca a LUT (`None` tira-a). Sobe a textura só quando o cubo muda.
    pub fn carrega_lut(&mut self, gpu: &GpuContext, cubo: Option<&Cubo>) {
        match cubo {
            None => self.lut = None,
            Some(c) if self.lut.as_ref().is_some_and(|(atual, _)| atual == c) => {}
            Some(c) => self.lut = Some((c.clone(), sobe_lut(gpu, c.n as u32, &c.dados))),
        }
    }

    /// **A lente sobre `rect = [x, y, w, h]` do `alvo`** (formato do `new`, usos
    /// `RENDER_ATTACHMENT | COPY_SRC`). Devolve `false` sem tocar em nada quando a lente é neutra.
    pub fn aplica(
        &mut self,
        gpu: &GpuContext,
        alvo: &wgpu::Texture,
        alvo_view: &wgpu::TextureView,
        rect: [u32; 4],
        lente: &Lente,
        tique: u32,
    ) -> bool {
        assert_eq!(
            alvo.format(),
            self.format,
            "o alvo da lente mudou de formato"
        );
        let rect = corta(rect, (alvo.width(), alvo.height()));
        let lut = self.lut.as_ref().map(|(c, _)| c);
        let Some(u) = LensUniform::de(lente, lut, tique, rect) else {
            return false;
        };
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        let tamanho = (rect[2], rect[3]);
        if self.copia.as_ref().is_none_or(|c| c.tamanho != tamanho) {
            self.copia = Some(cria_copia(gpu, self.format, tamanho));
        }
        let copia = self.copia.as_ref().expect("criada acima");
        let lut_view = self
            .lut
            .as_ref()
            .map_or(&self.lut_vazia.view, |(_, g)| &g.view);
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render lens bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&copia.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(lut_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform.as_entire_binding(),
                },
            ],
        });
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render lens encoder"),
            });
        enc.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: alvo,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: rect[0],
                    y: rect[1],
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &copia.tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: rect[2],
                height: rect[3],
                depth_or_array_layers: 1,
            },
        );
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render lens pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: alvo_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: ph2d_gpu::pass_profiler::render_writes("render.lens")
                    .as_ref()
                    .map(ph2d_gpu::pass_profiler::PassTimestamps::render),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_viewport(
                rect[0] as f32,
                rect[1] as f32,
                rect[2] as f32,
                rect[3] as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(rect[0], rect[1], rect[2], rect[3]);
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit(Some(enc.finish()));
        true
    }
}

/// O sub-rect cortado ao alvo.
#[must_use]
pub fn corta(rect: [u32; 4], alvo: (u32, u32)) -> [u32; 4] {
    let x = rect[0].min(alvo.0);
    let y = rect[1].min(alvo.1);
    [x, y, rect[2].min(alvo.0 - x), rect[3].min(alvo.1 - y)]
}

fn cria_copia(gpu: &GpuContext, format: wgpu::TextureFormat, tamanho: (u32, u32)) -> Copia {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render lens copia"),
        size: wgpu::Extent3d {
            width: tamanho.0,
            height: tamanho.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    Copia { tex, view, tamanho }
}

fn sobe_lut(gpu: &GpuContext, n: u32, dados: &[[f32; 3]]) -> LutGpu {
    let tamanho = wgpu::Extent3d {
        width: n,
        height: n,
        depth_or_array_layers: n,
    };
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render lens lut"),
        size: tamanho,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    // A ordem da spec (o vermelho a variar mais depressa) É a ordem x·y·z da textura.
    let texels: Vec<[f32; 4]> = dados.iter().map(|c| [c[0], c[1], c[2], 1.0]).collect();
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&texels),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(n * 16),
            rows_per_image: Some(n),
        },
        tamanho,
    );
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    LutGpu { _tex: tex, view }
}

#[cfg(test)]
#[path = "lens_tests.rs"]
mod tests;
