//! `BandBlit` — cola **uma faixa** de desenho sobre o acumulador do mundo (ADR-0154 Fase 2).
//!
//! Ver [`crate::world_rt::WorldRt`] para por que o alvo é de formato cru e a mistura é de hardware.
//! Este passe é deliberadamente pequeno: um triângulo de tela cheia, uma textura, dois
//! interruptores de convenção.
//!
//! ⛔ **Ele não decide ordem nenhuma.** Quem sabe a ordem é o [`crate::draw_bands`]; aqui só se
//! executa uma colagem.

use ph2d_gpu::GpuContext;

use crate::luz_rampa::neutra_1x1;

/// De onde vem a faixa — e a convenção de cor de cada uma.
///
/// ⚠️ **É um enum e não dois `bool`s soltos**, porque as duas convenções andam sempre juntas e a
/// combinação errada não falha alto: ela dá uma borda escura ou uma cor lavada.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BandSource {
    /// A saída do tonemap: vista `Bgra8UnormSrgb` (a amostragem descodifica) e **pré-multiplicada**.
    Sprites,
    /// O intermediário do Vello: `Rgba8Unorm` (sem descodificação) e com alfa **directa**.
    Vector,
    /// ⭐⭐⭐ A camada do passe de FORMAS instanciado (doc 121 do Motion): meio-float com as cores
    /// no espaço do desenhista (as MESMAS que o Vello recebe, sem descodificação) e
    /// **pré-multiplicada**, porque é a mistura por hardware que a escreve.
    ///
    /// ⚠️ Apendado: a posição é o índice do uniforme, e reordenar trocaria as convenções em silêncio.
    Formas,
    /// ⭐ A GRADE do editor por trás dos objectos: a convenção do Vello, mas é CHROME — nunca recebe
    /// a luz 2D (o documento vetorial usa [`Self::Vector`], que recebe). Apendado.
    Grade,
}

impl BandSource {
    /// As quatro fontes, na ordem do discriminante (o índice dos uniformes).
    pub const TODAS: [BandSource; 4] = [
        BandSource::Sprites,
        BandSource::Vector,
        BandSource::Formas,
        BandSource::Grade,
    ];

    /// `(decode_srgb, premultiplied)` — o par que o shader consome.
    #[must_use]
    pub fn flags(self) -> (u32, u32) {
        match self {
            BandSource::Sprites => (1, 1),
            BandSource::Vector => (0, 0),
            BandSource::Formas => (0, 1),
            BandSource::Grade => (0, 0),
        }
    }
}

impl BandSource {
    /// A fonte é conteúdo do MUNDO que recebe a luz 2D quando acesa? Os sprites já a receberam no
    /// passe deles; a grade é chrome.
    #[must_use]
    pub fn recebe_luz(self) -> bool {
        matches!(self, BandSource::Vector | BandSource::Formas)
    }
}

/// ⭐ **Como a faixa POUSA no alvo** (doc 129 §3.5, a mistura do `Output` com a camada filtrada do
/// Motion): a mistura de HARDWARE sobre os bytes crus — o espaço do desenhista, o mesmo em que o
/// Vello mistura. A lei é a do Vello ([`Self::lei`]): `Soma` = Porter-Duff `plus`
/// (`Compose::Plus`), `Multiplica`/`Tela` = `Mix` W3C com `SrcOver`, alfa do fundo incluída.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Pousa {
    /// O `over` pré-multiplicado de sempre.
    #[default]
    Normal,
    Soma,
    Multiplica,
    Tela,
}

impl Pousa {
    pub const ALL: [Pousa; 4] = [Pousa::Normal, Pousa::Soma, Pousa::Multiplica, Pousa::Tela];

    /// **A lei**, sobre cores PRÉ-MULTIPLICADAS em `[0, 1]`: `b` o alvo, `s` a faixa.
    #[must_use]
    pub fn lei(self, b: [f32; 4], s: [f32; 4]) -> [f32; 4] {
        let (ab, sa) = (b[3], s[3]);
        let alfa = sa + ab - sa * ab;
        let canal = |i: usize| match self {
            Pousa::Normal => s[i] + b[i] * (1.0 - sa),
            Pousa::Soma => (s[i] + b[i]).min(1.0),
            Pousa::Tela => s[i] + b[i] - s[i] * b[i],
            Pousa::Multiplica => s[i] * (1.0 - ab) + b[i] * (1.0 - sa) + s[i] * b[i],
        };
        let a = if self == Pousa::Soma {
            (sa + ab).min(1.0)
        } else {
            alfa
        };
        [canal(0), canal(1), canal(2), a]
    }

    /// Os passes de hardware, por ordem (índices de [`PASSES`]). ⚠️ O `Multiplica` não cabe numa
    /// equação de mistura só (o termo `s·(1 − αb + b)` não é um factor): o 1.º passe escreve
    /// `s·b + b·(1 − αs)` sem tocar no alfa, o 2.º soma `s·(1 − αb)` lendo o alfa ORIGINAL do fundo.
    fn passes(self) -> &'static [usize] {
        match self {
            Pousa::Normal => &[0],
            Pousa::Soma => &[1],
            Pousa::Tela => &[2],
            Pousa::Multiplica => &[3, 4],
        }
    }
}

const fn componente(src: wgpu::BlendFactor, dst: wgpu::BlendFactor) -> wgpu::BlendComponent {
    wgpu::BlendComponent {
        src_factor: src,
        dst_factor: dst,
        operation: wgpu::BlendOperation::Add,
    }
}

/// As equações de mistura de cada passe de [`Pousa::passes`] — `(cor, alfa)`.
const PASSES: [wgpu::BlendState; 5] = {
    use wgpu::BlendFactor as F;
    let over = componente(F::One, F::OneMinusSrcAlpha);
    [
        wgpu::BlendState {
            color: over,
            alpha: over,
        },
        wgpu::BlendState {
            color: componente(F::One, F::One),
            alpha: componente(F::One, F::One),
        },
        wgpu::BlendState {
            color: componente(F::One, F::OneMinusSrc),
            alpha: over,
        },
        wgpu::BlendState {
            color: componente(F::Dst, F::OneMinusSrcAlpha),
            alpha: componente(F::Zero, F::One),
        },
        wgpu::BlendState {
            color: componente(F::OneMinusDstAlpha, F::One),
            alpha: componente(F::OneMinusDstAlpha, F::One),
        },
    ]
};

/// O módulo da colagem: a luz recebida e a conta do tonemap À FRENTE (as mesmas do sprite).
pub const BAND_BLIT_WGSL: &str = concat!(
    ph2d_ease::wgsl_lib!(),
    ph2d_color::wgsl_perceptual!(),
    include_str!("shaders/luz_rampa.wgsl"),
    include_str!("shaders/receber.wgsl"),
    include_str!("shaders/tonemap_conta.wgsl"),
    include_str!("shaders/band_blit.wgsl")
);

pub struct BandBlit {
    /// Um pipeline por equação de [`PASSES`]; só o `over` nasce com o passe, os outros no 1.º
    /// pedido (a colagem do mundo nunca os pede).
    pipelines: [std::sync::OnceLock<wgpu::RenderPipeline>; PASSES.len()],
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    target_format: wgpu::TextureFormat,
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: [wgpu::Buffer; 4],
    /// Os mesmos com `com_normal = 1` (P14 «L»: a banda normal ligada).
    uniforms_normal: [wgpu::Buffer; 4],
    /// P13 L: com `com_normal = com_rampa = 1` (a banda de rampa e a LUT ligadas).
    uniforms_rampa: [wgpu::Buffer; 4],
    /// A banda de rampa (alfa 0) e a LUT 1×1 das colagens sem elas.
    rampa_neutra: [wgpu::TextureView; 2],
    /// A banda normal 1×1 de alfa 0 («sem normal») das colagens sem ela.
    normal_neutra: wgpu::TextureView,
    /// ⭐ O `@group(3)` da luz 2D (+ o LUT do tonemap nos bindings 3–4).
    luz_bgl: wgpu::BindGroupLayout,
    luz: crate::luz_recebida::GrupoDeLuz,
}

impl BandBlit {
    pub fn new(gpu: &GpuContext, target_format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-render band blit bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let [l0, l1, l2, l5, l6] = crate::luz_recebida::entradas_do_layout();
        let luz_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-render band blit luz bgl"),
            entries: &[
                l0,
                l1,
                l2,
                l5,
                l6,
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-render band blit layout"),
            bind_group_layouts: &[Some(&bgl), None, None, Some(&luz_bgl)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-render band blit shader"),
            source: wgpu::ShaderSource::Wgsl(BAND_BLIT_WGSL.into()),
        });
        // ⚠️ **`One / OneMinusSrcAlpha` — o `over` pré-multiplicado.** O shader entrega sempre
        // pré-multiplicado, e o alvo é de formato CRU: ⇒ a mistura acontece sobre os bytes
        // codificados, que É o espaço do desenhista.
        let pipelines = std::array::from_fn(|_| std::sync::OnceLock::new());
        let _ = pipelines[0].set(pipeline(device, &layout, &shader, target_format, PASSES[0]));
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render band blit sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        // Um uniforme por fonte, construído uma vez: os quatro valores são constantes.
        let uniforms = BandSource::TODAS.map(|s| uniform_for(device, s, false, false));
        let uniforms_normal = BandSource::TODAS.map(|s| uniform_for(device, s, true, false));
        let uniforms_rampa = BandSource::TODAS.map(|s| uniform_for(device, s, true, true));
        let rampa_neutra = [
            wgpu::TextureFormat::Rgba8Unorm,
            crate::luz_rampa::FORMATO_DA_LUT,
        ]
        .map(|format| neutra_1x1(device, format));
        let normal_neutra = neutra_1x1(device, wgpu::TextureFormat::Rgba8Unorm);
        // O grupo NEUTRO precisa de um LUT ligado (o layout o pede) — nunca amostrado: `ligada = 0`.
        let lut_neutro = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render band blit lut neutro"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let lut_vista = lut_neutro.create_view(&wgpu::TextureViewDescriptor::default());
        let luz = crate::luz_recebida::GrupoDeLuz::novo(
            &gpu.device,
            &luz_bgl,
            &lut_entradas(&lut_vista, &sampler),
        );
        Self {
            pipelines,
            shader,
            layout,
            target_format,
            bgl,
            sampler,
            uniforms,
            uniforms_normal,
            uniforms_rampa,
            rampa_neutra,
            normal_neutra,
            luz_bgl,
            luz,
        }
    }

    /// ⭐ **Acende (`Some`) ou apaga (`None`) a luz 2D** para as colagens que se seguem: as fontes do
    /// MUNDO ([`BandSource::recebe_luz`]) passam pela conta do sprite. `tela_para_mundo` = o afim
    /// píxel do alvo → mundo, o INVERSO do que o Vello usou para desenhar o documento; `tonemap` dá o
    /// LUT (o mesmo do passe de sprite). `None` = a colagem de sempre, byte a byte.
    pub fn receber_luz(
        &mut self,
        gpu: &GpuContext,
        luz: Option<(
            &crate::LuzRecebida<'_>,
            [[f64; 3]; 2],
            &crate::tonemap::Tonemap,
        )>,
    ) {
        let Some((l, tela_para_mundo, tonemap)) = luz else {
            self.luz.definir(gpu, &self.luz_bgl, None, &[]);
            return;
        };
        let (lut, lut_amostra) = tonemap.lut();
        let extra = lut_entradas(lut, lut_amostra);
        let uniforme = l.uniforme(Some(tela_para_mundo));
        self.luz.definir_com_momento(
            gpu,
            &self.luz_bgl,
            Some((l.fluencia, l.momento, uniforme)),
            &extra,
        );
    }

    /// Cola `src` sobre `target`, com a convenção de `source`.
    ///
    /// ⚠️ `LoadOp::Load` — o alvo é um ACUMULADOR. Um `Clear` aqui apagaria as faixas anteriores, e
    /// o sintoma seria «só a última coisa desenhada aparece».
    pub fn blit(
        &self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        src: &wgpu::TextureView,
        source: BandSource,
    ) {
        self.blit_rect(gpu, target, src, source, None);
    }

    /// O [`Self::blit`] com `src` INTEIRO esticado sobre o retângulo `[x, y, w, h]` do alvo (o resto
    /// do alvo fica intacto) — a camada filtrada do Motion, que só corre na área da cena.
    /// `None` = o alvo inteiro.
    pub fn blit_rect(
        &self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        src: &wgpu::TextureView,
        source: BandSource,
        rect: Option<[u32; 4]>,
    ) {
        self.blit_pousa(gpu, target, src, source, rect, Pousa::Normal);
    }

    /// O [`Self::blit_rect`] com a faixa a pousar por `pousa` ([`Pousa::lei`]).
    pub fn blit_pousa(
        &self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        src: &wgpu::TextureView,
        source: BandSource,
        rect: Option<[u32; 4]>,
        pousa: Pousa,
    ) {
        self.colar_pousa(gpu, target, src, None, None, source, rect, pousa);
    }

    /// ⭐ P14 «L» — o [`Self::blit_rect`] com a BANDA NORMAL `normal` (do tamanho da banda; rgb =
    /// `n·½+½`, Y para cima; alfa 0 = sem normal): a luz recebida usa `(F + n_xy·M)/2π` por píxel.
    pub fn blit_com_normal(
        &self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        src: &wgpu::TextureView,
        normal: &wgpu::TextureView,
        source: BandSource,
        rect: Option<[u32; 4]>,
    ) {
        self.colar(gpu, target, src, Some(normal), None, source, rect);
    }

    /// ⭐ P13 L — o [`Self::blit_com_normal`] com a BANDA DE RAMPA `rampa` (r = rampa, g = degrau,
    /// a = 1 só no texel em rampa) e a `lut` (`luz_rampa::FORMATO_DA_LUT`, linha = rampa): o texel
    /// em rampa troca de degrau pela luz (`luz_receber_rampa`); o resto, a lei de sempre.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_com_rampa(
        &self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        src: &wgpu::TextureView,
        normal: Option<&wgpu::TextureView>,
        rampa: (&wgpu::TextureView, &wgpu::TextureView),
        source: BandSource,
        rect: Option<[u32; 4]>,
    ) {
        self.colar(gpu, target, src, normal, Some(rampa), source, rect);
    }

    #[allow(clippy::too_many_arguments)]
    fn colar(
        &self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        src: &wgpu::TextureView,
        normal: Option<&wgpu::TextureView>,
        rampa: Option<(&wgpu::TextureView, &wgpu::TextureView)>,
        source: BandSource,
        rect: Option<[u32; 4]>,
    ) {
        self.colar_pousa(gpu, target, src, normal, rampa, source, rect, Pousa::Normal);
    }

    #[allow(clippy::too_many_arguments)]
    fn colar_pousa(
        &self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        src: &wgpu::TextureView,
        normal: Option<&wgpu::TextureView>,
        rampa: Option<(&wgpu::TextureView, &wgpu::TextureView)>,
        source: BandSource,
        rect: Option<[u32; 4]>,
        pousa: Pousa,
    ) {
        let flags = match (normal, rampa) {
            (_, Some(_)) => &self.uniforms_rampa[source as usize],
            (Some(_), None) => &self.uniforms_normal[source as usize],
            (None, None) => &self.uniforms[source as usize],
        };
        let [rampa_neutra, lut_neutra] = &self.rampa_neutra;
        let (banda_rampa, lut_rampa) = rampa.unwrap_or((rampa_neutra, lut_neutra));
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render band blit bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: flags.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        normal.unwrap_or(&self.normal_neutra),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(banda_rampa),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(lut_rampa),
                },
            ],
        });
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render band blit encoder"),
            });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render band blit pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: ph2d_gpu::pass_profiler::render_writes("render.band_blit")
                    .as_ref()
                    .map(ph2d_gpu::pass_profiler::PassTimestamps::render),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some([x, y, w, h]) = rect {
                #[allow(clippy::cast_precision_loss)]
                pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
                pass.set_scissor_rect(x, y, w, h);
            }
            pass.set_bind_group(0, &bg, &[]);
            pass.set_bind_group(3, self.luz.ligar(source.recebe_luz()), &[]);
            // Os passes de um modo partilham o passe de desenho: a mistura do 2.º lê o que o 1.º
            // escreveu (a ordem das primitivas é garantida).
            for &i in pousa.passes() {
                pass.set_pipeline(self.pipelines[i].get_or_init(|| {
                    pipeline(
                        &gpu.device,
                        &self.layout,
                        &self.shader,
                        self.target_format,
                        PASSES[i],
                    )
                }));
                pass.draw(0..3, 0..1);
            }
        }
        gpu.queue.submit(Some(enc.finish()));
    }
}

fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    blend: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ph2d-render band blit pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    })
}

fn lut_entradas<'a>(
    lut: &'a wgpu::TextureView,
    amostra: &'a wgpu::Sampler,
) -> [wgpu::BindGroupEntry<'a>; 2] {
    [
        wgpu::BindGroupEntry {
            binding: 3,
            resource: wgpu::BindingResource::TextureView(lut),
        },
        wgpu::BindGroupEntry {
            binding: 4,
            resource: wgpu::BindingResource::Sampler(amostra),
        },
    ]
}

fn uniform_for(
    device: &wgpu::Device,
    source: BandSource,
    com_normal: bool,
    com_rampa: bool,
) -> wgpu::Buffer {
    use wgpu::util::DeviceExt as _;
    let (decode, premul) = source.flags();
    let data: [u32; 8] = [
        decode,
        premul,
        u32::from(source.recebe_luz()),
        u32::from(com_normal),
        u32::from(com_rampa),
        0,
        0,
        0,
    ];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ph2d-render band blit flags"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **As duas convenções são OPOSTAS em ambos os eixos**, e trocá-las não falha alto — dá uma
    /// borda escura ou uma cor lavada. Este gate é o que torna a troca observável.
    #[test]
    fn the_two_sources_disagree_on_both_conventions() {
        assert_eq!(BandSource::Sprites.flags(), (1, 1));
        assert_eq!(BandSource::Vector.flags(), (0, 0));
        // ⭐ A terceira: as cores do desenhista (como o Vello) e pré-multiplicada (como a sprite).
        assert_eq!(BandSource::Formas.flags(), (0, 1));
        // ⭐ A grade: a convenção do vetor, mas nunca recebe a luz (é chrome).
        assert_eq!(BandSource::Grade.flags(), (0, 0));
        assert!(BandSource::Vector.recebe_luz() && BandSource::Formas.recebe_luz());
        assert!(!BandSource::Sprites.recebe_luz() && !BandSource::Grade.recebe_luz());
    }

    /// O índice do uniforme é o discriminante — um enum reordenado trocaria os dois em silêncio.
    #[test]
    fn the_uniform_index_follows_the_discriminant() {
        assert_eq!(BandSource::Sprites as usize, 0);
        assert_eq!(BandSource::Vector as usize, 1);
        assert_eq!(BandSource::Formas as usize, 2);
        assert_eq!(BandSource::Grade as usize, 3);
    }
}
