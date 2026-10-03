//! ⭐⭐⭐ **O CÉU QUE O CHÃO VÊ** — uma vez por quadro, numa textura do enquadramento da cobertura
//! (`ceu_chao.wgsl`): a fracção do céu, ponderada pelo cosseno, que as peças tapam em cada ponto do
//! chão, medida contra o Cycles (gate `tests_chao::o_ceu_do_chao_e_o_do_cycles`).
//!
//! ⚠️ Não depende da câmara: girar a vista não a muda, e o custo é o do lado desta textura, não o do
//! ecrã. O chão lê-a com uma leitura filtrada.
//!
//! ⭐ **Só se refaz quando a resposta muda**: a chave é TUDO o que os dois passes leem (o uniforme, os
//! objetos com a pose e a geração das malhas). Uma chave igual dá a mesma textura ao bit — não é
//! acumular entre quadros, é não repetir a mesma conta (gate `o_ceu_do_chao_segue_as_pecas`).

use crate::gpu_cobertura::COBERTURA_LADO;

/// O lado da textura do céu do chão.
pub const CEU_CHAO_LADO: u32 = 512;

const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Os floats do uniforme (`P` do WGSL).
const UNIFORME: usize = 12;

/// Quanto custa e quão fino é o céu do chão (ver o gate e o instrumento do custo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parametros {
    /// Fatias de azimute por texel (o entrelaçado 4×4 dá `16×` mais no borrão).
    pub fatias: u32,
    /// A razão entre dois passos de uma fatia.
    pub razao: f32,
    /// A pegada lateral de uma amostra, em fracções da distância.
    pub pegada: f32,
    /// A fracção da borda do quadro onde o escurecimento cai a zero.
    pub borda: f32,
}

impl Default for Parametros {
    fn default() -> Self {
        Self {
            fatias: 4,
            razao: 1.25,
            pegada: 0.03,
            borda: 0.15,
        }
    }
}

/// A entrada da ligação do céu do chão no grupo `0` do desenhista (filtrável: o chão lê-a com uma
/// leitura bilinear).
pub(crate) fn entrada(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

pub(crate) struct CeuChao {
    cru: wgpu::TextureView,
    /// O céu do chão já borrado — o que o chão lê (`r` pré-multiplicado pela validade `g`).
    pub vista: wgpu::TextureView,
    uniforme: wgpu::Buffer,
    g_ceu: wgpu::BindGroup,
    g_borra: wgpu::BindGroup,
    ceu: wgpu::RenderPipeline,
    borra: wgpu::RenderPipeline,
    pub parametros: Parametros,
    lado: u32,
    /// Sobe a cada malha subida ou esquecida: a mesma pose com outra forma é outra resposta.
    geracao: u64,
    chave: Vec<u32>,
    /// A chave da textura que está feita.
    feita: std::cell::RefCell<Option<Vec<u32>>>,
}

fn textura(device: &wgpu::Device, rotulo: &str, lado: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(rotulo),
            size: wgpu::Extent3d {
                width: lado,
                height: lado,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMATO,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

impl CeuChao {
    /// `lado` = o da textura ([`CEU_CHAO_LADO`] no desenhista; o instrumento do custo varre-o).
    pub(crate) fn novo(device: &wgpu::Device, cobertura: &wgpu::TextureView, lado: u32) -> Self {
        let cru = textura(device, "ph2d-mesh-forward céu do chão cru", lado);
        let vista = textura(device, "ph2d-mesh-forward céu do chão", lado);
        let uniforme = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward céu do chão"),
            size: (UNIFORME * 4) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let liso = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-mesh-forward céu do chão"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let frag = wgpu::ShaderStages::FRAGMENT;
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward céu do chão"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: frag,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: frag,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: frag,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let grupo = |fonte: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-mesh-forward céu do chão"),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniforme.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(fonte),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&liso),
                    },
                ],
            })
        };
        let (g_ceu, g_borra) = (grupo(cobertura), grupo(&cru));
        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh-forward céu do chão"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ceu_chao.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-mesh-forward céu do chão"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let alvo = [Some(wgpu::ColorTargetState {
            format: FORMATO,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let pipeline = |rotulo: &str, fs: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(rotulo),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &modulo,
                    entry_point: Some("vs_ceu"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &modulo,
                    entry_point: Some(fs),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &alvo,
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            ceu: pipeline("ph2d-mesh-forward céu do chão", "fs_ceu"),
            borra: pipeline("ph2d-mesh-forward céu do chão borrão", "fs_borra"),
            cru,
            vista,
            uniforme,
            g_ceu,
            g_borra,
            parametros: Parametros::default(),
            lado,
            geracao: 0,
            chave: Vec::new(),
            feita: std::cell::RefCell::new(None),
        }
    }

    /// Uma malha foi subida ou esquecida.
    pub(crate) fn malha_mudou(&mut self) {
        self.geracao += 1;
    }

    /// Escreve o uniforme do quadro e a chave dele: `ceu` e `ceu_vp` são os da cobertura (o
    /// `Enquadra`), `objetos` os da cena.
    pub(crate) fn prepara(
        &mut self,
        queue: &wgpu::Queue,
        ceu: [f32; 4],
        ceu_vp: &[[f32; 4]; 4],
        chao: f32,
        objetos: &[crate::Instancia],
    ) {
        let p = self.parametros;
        // `PH2D_CEU_CHAO=fatias,razao,pegada` — a varredura do instrumento, só nos testes.
        #[cfg(test)]
        let p = std::env::var("PH2D_CEU_CHAO").map_or(p, |v| {
            let c: Vec<f32> = v.split(',').filter_map(|x| x.parse().ok()).collect();
            Parametros {
                fatias: c[0] as u32,
                razao: c[1],
                pegada: c[2],
                ..p
            }
        });
        let texel = 2.0 * ceu[0] / COBERTURA_LADO as f32;
        // Os passos chegam à diagonal do quadro a partir de um texel.
        let alcance = 2.0 * std::f32::consts::SQRT_2 * ceu[0];
        let passos = ((alcance / texel).ln() / p.razao.ln()).ceil().max(1.0);
        let niveis = COBERTURA_LADO.ilog2() as f32;
        let u: [f32; UNIFORME] = [
            ceu_vp[1][2] * chao + ceu_vp[3][2],
            texel,
            ceu[1],
            niveis,
            p.fatias as f32,
            passos,
            p.razao,
            COBERTURA_LADO as f32,
            p.borda,
            self.lado as f32,
            p.pegada,
            0.0,
        ];
        self.chave.clear();
        self.chave.extend(u.iter().map(|x| x.to_bits()));
        self.chave
            .extend([self.geracao as u32, (self.geracao >> 32) as u32]);
        for o in objetos {
            self.chave.extend([o.malha as u32, (o.malha >> 32) as u32]);
            self.chave
                .extend(o.modelo.iter().flatten().map(|x| x.to_bits()));
        }
        if self.feita.borrow().as_ref() != Some(&self.chave) {
            queue.write_buffer(&self.uniforme, 0, bytemuck::cast_slice(&u));
        }
    }

    /// Os dois passes, só se a chave do quadro não é a da textura feita.
    pub(crate) fn grava(&self, enc: &mut wgpu::CommandEncoder) {
        let mut feita = self.feita.borrow_mut();
        if feita.as_ref() == Some(&self.chave) {
            return;
        }
        self.grava_passes(enc);
        *feita = Some(self.chave.clone());
    }

    /// Os dois passes: o céu entrelaçado e o borrão que o junta.
    pub(crate) fn grava_passes(&self, enc: &mut wgpu::CommandEncoder) {
        for (alvo, grupo, pipe) in [
            (&self.cru, &self.g_ceu, &self.ceu),
            (&self.vista, &self.g_borra, &self.borra),
        ] {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward céu do chão"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: alvo,
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
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, grupo, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
