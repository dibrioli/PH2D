//! ⭐ **O mundo fora da grade na placa** (W5b, F4) — os buffers dos objetos distantes
//! ([`crate::distantes`]), a atlas das máscaras e o passe que as desenha; o grupo 1 da C3 (o das
//! lâmpadas) liga-os. Nascem com o mínimo que a [`PassList`] declara e crescem como a geometria
//! (potências de dois, nunca encolhem; contados em [`Solver::bytes_da_geometria_residente`]).
//! Módulo-filho do [`super`].

use super::*;
use crate::captura_sprite::Amostragem;
use crate::distantes::{Distante, Mascara, MundoDistante};

/// Os rótulos dos três buffers (objetos · arestas · índices) — a [`PassList`] declara-os.
pub(crate) const NOMES: [&str; 3] = [
    "luz2d/buf/distantes",
    "luz2d/buf/arestas",
    "luz2d/buf/indices",
];
/// O mínimo de cada um: um elemento (`Distante`, `vec4<f32>`, `u32` com o alinhamento de 16).
pub(crate) const MINIMOS: [u64; 3] = [size_of::<Distante>() as u64, 16, 16];
/// A atlas das máscaras — a [`PassList`] declara a de 1 × 1.
pub(crate) const NOME_DA_ATLAS: &str = "luz2d/tex/mascaras";
pub(crate) const FORMATO_DA_ATLAS: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
pub(crate) const USOS_DA_ATLAS: wgpu::TextureUsages =
    wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::TEXTURE_BINDING);

/// Uma máscara como o `c3_mascara.wgsl` a lê (32 bytes).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MascaraNaPlaca {
    janela: [f32; 4],
    uv: [f32; 4],
}

/// O passe das máscaras — nasce no primeiro quadro que tem uma.
struct Pintor {
    pipeline: wgpu::RenderPipeline,
    layout_lista: wgpu::BindGroupLayout,
    layout_textura: wgpu::BindGroupLayout,
    amostra: wgpu::Sampler,
    amostra_exata: wgpu::Sampler,
    uni: wgpu::Buffer,
    lista: Option<(wgpu::Buffer, u64)>,
}

impl Pintor {
    fn novo(device: crate::Placa<'_>) -> Self {
        let entrada = |binding, ty, vis| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vis,
            ty,
            count: None,
        };
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let buffer = |ty| wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let layout_lista = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d mascaras"),
            entries: &[
                entrada(0, buffer(wgpu::BufferBindingType::Uniform), vf),
                entrada(
                    1,
                    buffer(wgpu::BufferBindingType::Storage { read_only: true }),
                    vf,
                ),
            ],
        });
        let layout_textura = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d mascaras textura"),
            entries: &[
                entrada(
                    0,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    wgpu::ShaderStages::FRAGMENT,
                ),
                entrada(
                    1,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    wgpu::ShaderStages::FRAGMENT,
                ),
            ],
        });
        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("luz2d c3_mascara.wgsl"),
            source: wgpu::ShaderSource::Wgsl(shaders::C3_MASCARA.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d mascaras"),
            bind_group_layouts: &[Some(&layout_lista), Some(&layout_textura)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("luz2d mascaras"),
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
                    format: FORMATO_DA_ATLAS,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        // A mesma amostragem da C0 dos sprites: bilinear numa textura de sprite, o texel exato no
        // desenho do Vello (o píxel dele já é a cobertura).
        let amostra = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("luz2d mascaras amostra"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let amostra_exata = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("luz2d mascaras amostra exata"),
            ..Default::default()
        });
        let uni = criar_buffer(
            &device,
            "luz2d/buf/mascaras_atlas",
            16,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        Self {
            pipeline,
            layout_lista,
            layout_textura,
            amostra,
            amostra_exata,
            uni,
            lista: None,
        }
    }

    /// Desenha `mascaras` na atlas `alvo` de lado `lado`.
    fn desenhar(
        &mut self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        (alvo, lado_da_atlas, lado): (&wgpu::TextureView, u32, u32),
        mascaras: &[Mascara],
        (texturas, tempos): (&[(&wgpu::TextureView, Amostragem)], Option<&Tempos>),
    ) {
        let device = &gpu.device;
        // Agrupadas por textura (um grupo por lote).
        let mut ordem: Vec<&Mascara> = mascaras.iter().collect();
        ordem.sort_by_key(|m| m.textura);
        let placa: Vec<MascaraNaPlaca> = ordem
            .iter()
            .map(|m| MascaraNaPlaca {
                janela: [m.janela[0] as f32, m.janela[1] as f32, lado as f32, 0.0],
                uv: m.uv,
            })
            .collect();
        let bytes = (placa.len() * size_of::<MascaraNaPlaca>()) as u64;
        if self.lista.as_ref().is_none_or(|(_, cap)| *cap < bytes) {
            let cap = bytes.next_power_of_two();
            let b = criar_buffer(
                device,
                "luz2d/buf/mascaras",
                cap,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            );
            self.lista = Some((b, cap));
        }
        let (lista, _) = self.lista.as_ref().expect("criada acima");
        gpu.queue
            .write_buffer(lista, 0, bytemuck::cast_slice(&placa));
        let n = lado_da_atlas as f32;
        gpu.queue
            .write_buffer(&self.uni, 0, bytemuck::bytes_of(&[n, n, 0.0, 0.0]));
        let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d mascaras"),
            layout: &self.layout_lista,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uni.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: lista.as_entire_binding(),
                },
            ],
        });
        let mut lotes: Vec<(wgpu::BindGroup, std::ops::Range<u32>)> = Vec::new();
        let mut i = 0;
        while i < ordem.len() {
            let t = ordem[i].textura;
            let fim = i + ordem[i..].iter().take_while(|m| m.textura == t).count();
            if let Some((vista, amostragem)) = texturas.get(t) {
                let amostra = match amostragem {
                    Amostragem::Bilinear => &self.amostra,
                    Amostragem::Exata => &self.amostra_exata,
                };
                let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("luz2d mascaras textura"),
                    layout: &self.layout_textura,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(vista),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(amostra),
                        },
                    ],
                });
                lotes.push((g, i as u32..fim as u32));
            }
            i = fim;
        }
        let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("luz2d C3 mascaras"),
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
            timestamp_writes: tempos.map(|t| t.escrita_render(tempos::C3_MASCARAS)),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        p.set_pipeline(&self.pipeline);
        p.set_bind_group(0, &grupo, &[]);
        for (g, r) in &lotes {
            p.set_bind_group(1, g, &[]);
            p.draw(0..4, r.clone());
        }
    }
}

pub(super) struct Distantes {
    buffers: [wgpu::Buffer; 3],
    caps: [u64; 3],
    atlas: wgpu::Texture,
    vista: wgpu::TextureView,
    lado: u32,
    pintor: Option<Pintor>,
    fabrica: crate::Fabrica,
}

impl Distantes {
    /// Com os mínimos que a [`PassList`] declarou (`buffers`) e a atlas de 1 × 1 dela.
    pub(super) fn novo(
        buffers: [wgpu::Buffer; 3],
        atlas: wgpu::Texture,
        fabrica: crate::Fabrica,
    ) -> Self {
        let vista = atlas.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            buffers,
            caps: MINIMOS,
            atlas,
            vista,
            lado: 1,
            pintor: None,
            fabrica,
        }
    }

    /// As entradas 1–4 do grupo das lâmpadas.
    pub(super) fn entradas(&self) -> [wgpu::BindGroupEntry<'_>; 4] {
        [
            wgpu::BindGroupEntry {
                binding: 1,
                resource: self.buffers[0].as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: self.buffers[1].as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: self.buffers[2].as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&self.vista),
            },
        ]
    }

    /// Sobe `m` e desenha as máscaras dele. `true` se algum recurso foi realocado (o grupo das
    /// lâmpadas tem de ser refeito).
    pub(super) fn subir(
        &mut self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        m: &MundoDistante,
        (texturas, tempos): (&[(&wgpu::TextureView, Amostragem)], Option<&Tempos>),
    ) -> bool {
        let device = &gpu.device;
        let mut mudou = false;
        let dados: [&[u8]; 3] = [
            bytemuck::cast_slice(&m.objetos),
            bytemuck::cast_slice(&m.arestas),
            bytemuck::cast_slice(&m.indices),
        ];
        for k in 0..3 {
            let bytes = dados[k].len() as u64;
            if bytes > self.caps[k] {
                self.caps[k] = bytes.next_power_of_two();
                self.buffers[k] = criar_buffer(
                    device,
                    NOMES[k],
                    self.caps[k],
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                );
                mudou = true;
            }
            if bytes > 0 {
                gpu.queue.write_buffer(&self.buffers[k], 0, dados[k]);
            }
        }
        if m.mascaras.is_empty() {
            // Sem máscaras, um passe vazio escreve os carimbos delas (valem 0).
            if let Some(t) = tempos {
                enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("luz2d C3 mascaras (vazia)"),
                    timestamp_writes: Some(t.escrita_compute(tempos::C3_MASCARAS)),
                });
            }
            return mudou;
        }
        if m.lado_da_atlas > self.lado {
            self.lado = m.lado_da_atlas;
            self.atlas = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(NOME_DA_ATLAS),
                size: wgpu::Extent3d {
                    width: self.lado,
                    height: self.lado,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FORMATO_DA_ATLAS,
                usage: USOS_DA_ATLAS,
                view_formats: &[],
            });
            self.vista = self
                .atlas
                .create_view(&wgpu::TextureViewDescriptor::default());
            mudou = true;
        }
        self.pintor
            .get_or_insert_with(|| {
                Pintor::novo(crate::Placa {
                    device,
                    fabrica: &self.fabrica,
                })
            })
            .desenhar(
                gpu,
                enc,
                (&self.vista, self.lado, m.lado),
                &m.mascaras,
                (texturas, tempos),
            );
        mudou
    }

    /// Os bytes que CRESCERAM além do mínimo que a [`PassList`] declara (esse já está nela), e o
    /// passe das máscaras quando nasceu.
    pub(super) fn bytes_residentes(&self) -> u64 {
        let buffers: u64 = self
            .caps
            .iter()
            .zip(MINIMOS)
            .map(|(&cap, min)| if cap > min { cap } else { 0 })
            .sum();
        let atlas = if self.lado > 1 {
            u64::from(self.lado) * u64::from(self.lado)
        } else {
            0
        };
        let pintor = self
            .pintor
            .as_ref()
            .map_or(0, |p| 16 + p.lista.as_ref().map_or(0, |(_, cap)| *cap));
        buffers + atlas + pintor
    }
}

/// O grupo 1 da C3: as lâmpadas e o mundo fora da grade.
pub(super) fn grupo_das_lampadas(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    lampadas: &wgpu::Buffer,
    distantes: &Distantes,
) -> wgpu::BindGroup {
    let mut entradas = vec![wgpu::BindGroupEntry {
        binding: 0,
        resource: lampadas.as_entire_binding(),
    }];
    entradas.extend(distantes.entradas());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("luz2d c3 lampadas"),
        layout,
        entries: &entradas,
    })
}
