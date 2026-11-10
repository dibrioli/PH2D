//! **A CRT na janela do jogo** (P13 §11: «na prévia e no jogo, nunca no ficheiro»).
//!
//! A lei é por TEXEL, por isso o quadro passa à resolução lógica antes dela: o compositor desenha
//! num alvo do tamanho da janela, a placa reduz (nearest) a [`Logica`], corre a CRT ×`escala` e põe o
//! resultado ao centro do ecrã, com o resto a preto. Desligada (`None`), o compositor desenha no
//! ecrã como sempre e nada disto corre — [`passes`] não se mexe.
//!
//! Os níveis atravessam crus: o alvo do compositor (o formato do ecrã, `*Srgb`) lê-se por uma vista
//! não-sRGB, e a última cópia desfaz a curva para a codificação da placa devolver o mesmo byte.

use crate::{CrtParams, LADO_MAXIMO, gpu::CrtGpu};
use ph2d_gpu::GpuContext;

/// A resolução lógica do jogo e a escala inteira que cabe na janela.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Logica {
    pub w: u32,
    pub h: u32,
    pub escala: u32,
}

impl Logica {
    /// Texels visíveis = altura da câmara (m) × `pixels_per_meter` do projecto — a grade em que a
    /// arte foi importada; a largura segue a proporção da janela; `escala = ⌊janela/lógica⌋ ≥ 1`.
    /// Nunca passa a janela (um texel menor que um píxel do ecrã não existe). `None` = janela vazia
    /// ou números não finitos.
    #[must_use]
    pub fn da_camara((jw, jh): (u32, u32), altura_mundo: f32, ppm: f32) -> Option<Self> {
        let texels = altura_mundo * ppm;
        if jw == 0 || jh == 0 || !texels.is_finite() || texels <= 0.0 {
            return None;
        }
        let teto = jh.min(LADO_MAXIMO - 1);
        // Tectos acima: `texels` ∈ (0, 2¹⁴) ⇒ cabe em `u32`.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let h = (texels.round() as u32).clamp(1, teto);
        let w = (u64::from(h) * u64::from(jw) * 2 + u64::from(jh)) / (2 * u64::from(jh));
        let w = u32::try_from(w)
            .unwrap_or(jw)
            .clamp(1, jw.min(LADO_MAXIMO - 1));
        let escala = (jw / w).min(jh / h).max(1);
        Some(Self { w, h, escala })
    }

    /// O canto da saída `w·escala × h·escala` centrada na janela.
    #[must_use]
    pub fn canto(&self, (jw, jh): (u32, u32)) -> (u32, u32) {
        (
            jw.saturating_sub(self.w * self.escala) / 2,
            jh.saturating_sub(self.h * self.escala) / 2,
        )
    }
}

/// O pedido de um quadro do jogo: a lógica e os params ligados.
pub type Pedido = (Logica, CrtParams);

/// O que a janela do jogo pede neste quadro: `None` fora do palco, com a vista desligada ou sem
/// lógica (ver [`Logica::da_camara`]).
#[must_use]
pub fn da_vista(no_palco: bool, janela: (u32, u32), altura_mundo: f32, ppm: f32) -> Option<Pedido> {
    let params = crate::vista::params_ligados().filter(|_| no_palco)?;
    Some((Logica::da_camara(janela, altura_mundo, ppm)?, params))
}

thread_local! {
    static PASSES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static PALCO: std::cell::RefCell<Option<Palco>> = const { std::cell::RefCell::new(None) };
}

/// Quantos passes da CRT esta thread já correu na janela do jogo (o contador dos gates).
#[must_use]
pub fn passes() -> u64 {
    PASSES.with(std::cell::Cell::get)
}

/// O fim do quadro do jogo: com `crt = None`, `compoe(alvo)` e mais nada; com
/// `Some((logica, params))`, `compoe` desenha num alvo intermédio e a CRT leva-o ao `alvo`.
/// `formato` = o do `alvo` (o do ecrã).
pub fn compoe(
    gpu: &GpuContext,
    formato: wgpu::TextureFormat,
    alvo: &wgpu::TextureView,
    janela: (u32, u32),
    crt: Option<(Logica, CrtParams)>,
    compoe: impl FnOnce(&wgpu::TextureView),
) {
    let Some((l, params)) = crt else {
        compoe(alvo);
        return;
    };
    PALCO.with(|c| {
        let mut c = c.borrow_mut();
        let p = match c.take() {
            Some(p) if p.chave == (formato, janela, l) => c.insert(p),
            _ => c.insert(Palco::new(&gpu.device, formato, janela, l)),
        };
        compoe(&p.composto);
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-pixel-crt palco"),
            });
        p.reduz.codifica(
            gpu,
            &mut enc,
            &p.cru,
            &p.logico,
            P::copia(janela, (l.w, l.h), false),
        );
        let (lw, lh) = (l.w, l.h);
        p.crt.codifica(
            &gpu.device,
            &gpu.queue,
            &mut enc,
            &p.logico,
            &p.saida,
            (lw, lh, l.escala),
            &params,
            None,
        );
        let tam = (lw * l.escala, lh * l.escala);
        let poe = P {
            src: tam,
            dst: tam,
            off: l.canto(janela),
            decodifica: formato.is_srgb(),
        };
        p.poe.codifica(gpu, &mut enc, &p.saida, alvo, poe);
        gpu.queue.submit([enc.finish()]);
    });
    PASSES.with(|n| n.set(n.get() + 1));
}

/// O uniforme do `posiciona.wgsl`.
#[derive(Copy, Clone)]
struct P {
    src: (u32, u32),
    dst: (u32, u32),
    off: (u32, u32),
    decodifica: bool,
}

impl P {
    fn copia(src: (u32, u32), dst: (u32, u32), decodifica: bool) -> Self {
        Self {
            src,
            dst,
            off: (0, 0),
            decodifica,
        }
    }

    fn bytes(self) -> [u8; 32] {
        let v = [
            self.src.0,
            self.src.1,
            self.dst.0,
            self.dst.1,
            self.off.0,
            self.off.1,
            u32::from(self.decodifica),
            0,
        ];
        let mut b = [0u8; 32];
        for (i, x) in v.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&x.to_le_bytes());
        }
        b
    }
}

/// Os alvos e pipelines do palco, refeitos só quando muda `(formato, janela, lógica)`.
struct Palco {
    chave: (wgpu::TextureFormat, (u32, u32), Logica),
    composto: wgpu::TextureView,
    cru: wgpu::TextureView,
    logico: wgpu::TextureView,
    saida: wgpu::TextureView,
    reduz: Posiciona,
    crt: CrtGpu,
    poe: Posiciona,
}

const RGBA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

impl Palco {
    fn new(
        dev: &wgpu::Device,
        formato: wgpu::TextureFormat,
        janela: (u32, u32),
        l: Logica,
    ) -> Self {
        let cru_fmt = formato.remove_srgb_suffix();
        let uso = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let tex = |label, (width, height): (u32, u32), format, views: &[wgpu::TextureFormat]| {
            dev.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: uso,
                view_formats: views,
            })
        };
        let composto = tex("ph2d-pixel-crt composto", janela, formato, &[cru_fmt]);
        let cru = composto.create_view(&wgpu::TextureViewDescriptor {
            format: Some(cru_fmt),
            ..Default::default()
        });
        let vista = |t: wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            chave: (formato, janela, l),
            composto: vista(composto),
            cru,
            logico: vista(tex("ph2d-pixel-crt lógico", (l.w, l.h), RGBA, &[])),
            saida: vista(tex(
                "ph2d-pixel-crt saída",
                (l.w * l.escala, l.h * l.escala),
                RGBA,
                &[],
            )),
            reduz: Posiciona::new(dev, RGBA),
            crt: CrtGpu::new(dev, RGBA),
            poe: Posiciona::new(dev, formato),
        }
    }
}

/// O passe do `posiciona.wgsl` para um formato de alvo (cada um com o seu uniforme).
struct Posiciona {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniforme: wgpu::Buffer,
}

impl Posiciona {
    fn new(dev: &wgpu::Device, formato: wgpu::TextureFormat) -> Self {
        let shader = dev.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-pixel-crt posiciona"),
            source: wgpu::ShaderSource::Wgsl(include_str!("posiciona.wgsl").into()),
        });
        let layout = dev.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-pixel-crt posiciona bgl"),
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
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                },
            ],
        });
        let pl = dev.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-pixel-crt posiciona layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = dev.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-pixel-crt posiciona pipeline"),
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
        let uniforme = dev.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-pixel-crt posiciona params"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            layout,
            uniforme,
        }
    }

    fn codifica(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        origem: &wgpu::TextureView,
        alvo: &wgpu::TextureView,
        p: P,
    ) {
        gpu.queue.write_buffer(&self.uniforme, 0, &p.bytes());
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-pixel-crt posiciona bg"),
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
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-pixel-crt posiciona pass"),
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
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..3, 0..1);
    }
}
