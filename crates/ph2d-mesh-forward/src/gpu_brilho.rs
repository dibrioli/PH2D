//! ⭐⭐⭐ **O BRILHO na placa do celular** — a cadeia do `ph2d_bloom::halo` em passes de DESENHO.
//!
//! O brilho do Render traçado (`ph2d-field-gpu/src/brilho.rs`) é compute + armazenamento, que o
//! WebGL2 não tem; o do Motion (`ph2d-render/src/shaders/bloom.wgsl`) desenha, mas amostra pelo
//! *sampler* da placa e compõe ANTES do olhar. Daqui sai a MESMA lei do modelador — a
//! `ph2d_bloom::wgsl`, com a bilinear escrita à mão — e a composição dele, depois do olhar.
//!
//! ```text
//! cena-linear (MRT, w×h) ─desce+corte─▶ n₀ ─desce─▶ n₁ … n_{k-1}
//!                                      acc_{k-2} = tenda(n_{k-1}) + n_{k-2} … acc₀
//! ecrã: bytes(olhar) ⊕ olhar(cor(tenda(acc₀ → w×h)))
//! ```
//!
//! ⚠️ Os níveis são `Rgba16Float` (o formato do alvo da cena, e o de todo brilho de celular): a
//! paridade com a CPU (`f32`) mede-se em BYTES no fim, não no meio.

/// O formato da cena-linear e dos níveis.
pub(crate) const LINEAR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// Os bytes do `Passe` do WGSL: `4 u32 + 8 f32`.
const PASSE: usize = 48;
/// Os bytes do `Ecra` do WGSL: `4 u32 + 12 f32`.
pub(crate) const ECRA: u64 = 64;

/// Os pipelines do brilho — compilados com o desenhista, só onde a placa desenha `Rgba16Float`.
pub(crate) struct Brilho {
    pub objeto: wgpu::RenderPipeline,
    pub chao: wgpu::RenderPipeline,
    pub fundo: wgpu::RenderPipeline,
    desce: wgpu::RenderPipeline,
    sobe: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
}

/// Os lados dos níveis — o `lw / 2` repetido do `ph2d_bloom::halo`.
pub(crate) fn lados(w: u32, h: u32, n: usize) -> Vec<(u32, u32)> {
    let mut v = Vec::with_capacity(n);
    let (mut a, mut b) = (w, h);
    for _ in 0..n {
        a /= 2;
        b /= 2;
        v.push((a, b));
    }
    v
}

fn empilha(dados: &mut Vec<u8>, u: [u32; 4], f: &[f32]) {
    let at = dados.len();
    for x in u {
        dados.extend_from_slice(&x.to_le_bytes());
    }
    for x in f {
        dados.extend_from_slice(&x.to_le_bytes());
    }
    dados.resize(at + crate::gpu::SLOT as usize, 0);
}

/// ⭐ **Os uniformes dos passes**, um por fatia de `256` bytes: os `n` que descem e os `n − 1` que
/// sobem (o degrau `k` na fatia `n + k`).
pub(crate) fn passes(b: &ph2d_bloom::Bloom, (w, h): (u32, u32)) -> Vec<u8> {
    let n = ph2d_bloom::levels_that_fit(w as usize, h as usize);
    let l = lados(w, h, n);
    let p = &b.params;
    #[allow(clippy::cast_precision_loss)]
    let base = p.upsample_basis(w as f32 / h.max(1) as f32);
    let mut dados = Vec::with_capacity((2 * n) * crate::gpu::SLOT as usize);
    for k in 0..n {
        let (sw, sh) = if k == 0 { (w, h) } else { l[k - 1] };
        let primeiro = if k == 0 { 1.0 } else { 0.0 };
        empilha(
            &mut dados,
            [sw, sh, l[k].0, l[k].1],
            &[
                base[0],
                base[1],
                base[2],
                base[3],
                p.threshold,
                p.knee,
                p.clamp_limit(),
                primeiro,
            ],
        );
    }
    for k in 0..n.saturating_sub(1) {
        empilha(
            &mut dados,
            [l[k + 1].0, l[k + 1].1, l[k].0, l[k].1],
            &[base[0], base[1], base[2], base[3], 0.0, 0.0, 0.0, 0.0],
        );
    }
    dados
}

/// ⭐ **O uniforme da codificação** — com `ha_brilho = 0` o halo não é lido.
pub(crate) fn ecra(
    b: &ph2d_bloom::Bloom,
    (w, h): (u32, u32),
    exposicao: f32,
    vista: u32,
    ha_brilho: bool,
) -> Vec<u8> {
    let n = ph2d_bloom::levels_that_fit(w as usize, h as usize);
    let (aw, ah) = lados(w, h, n).first().copied().unwrap_or((1, 1));
    let p = &b.params;
    #[allow(clippy::cast_precision_loss)]
    let base = p.upsample_basis(w as f32 / h.max(1) as f32);
    let mut dados = Vec::with_capacity(ECRA as usize);
    #[allow(clippy::cast_precision_loss)]
    let f = [
        base[0],
        base[1],
        base[2],
        base[3],
        p.saturation,
        p.intensity,
        exposicao,
        f32::from(u8::from(ha_brilho)),
        p.tint[0],
        p.tint[1],
        p.tint[2],
        vista as f32,
    ];
    for x in [aw, ah, w, h] {
        dados.extend_from_slice(&x.to_le_bytes());
    }
    for x in f {
        dados.extend_from_slice(&x.to_le_bytes());
    }
    dados
}

fn textura(
    device: &wgpu::Device,
    (w, h): (u32, u32),
    amostras: u32,
    usos: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-mesh-forward brilho"),
        size: wgpu::Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: amostras,
        dimension: wgpu::TextureDimension::D2,
        format: LINEAR,
        usage: usos,
        view_formats: &[],
    })
}

/// ⭐ **A cadeia de um tamanho**: a cena-linear (MSAA e resolvida), os níveis que descem, os que
/// sobem, e os grupos de ligação de cada passe — nascem quando o tamanho muda, nunca por quadro.
pub(crate) struct Cadeia {
    pub linear_msaa: wgpu::TextureView,
    pub linear: wgpu::Texture,
    pub linear_vista: wgpu::TextureView,
    desce: Vec<wgpu::TextureView>,
    sobe: Vec<wgpu::TextureView>,
    grupos_desce: Vec<wgpu::BindGroup>,
    grupos_sobe: Vec<wgpu::BindGroup>,
    pub uniforme: wgpu::Buffer,
    pub ecra_bind: wgpu::BindGroup,
}

impl Cadeia {
    pub(crate) fn nova(
        device: &wgpu::Device,
        brilho: &Brilho,
        (w, h): (u32, u32),
        resolvida: &wgpu::TextureView,
        ecra_bgl: &wgpu::BindGroupLayout,
        ecra_ub: &wgpu::Buffer,
    ) -> Self {
        let n = ph2d_bloom::levels_that_fit(w as usize, h as usize);
        let l = lados(w, h, n);
        let v = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let ra = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let tb = wgpu::TextureUsages::TEXTURE_BINDING;
        let linear_msaa = v(&textura(device, (w, h), crate::MSAA, ra));
        let linear = textura(device, (w, h), 1, ra | tb | wgpu::TextureUsages::COPY_DST);
        let linear_vista = v(&linear);
        let desce: Vec<_> = l
            .iter()
            .map(|s| v(&textura(device, *s, 1, ra | tb)))
            .collect();
        let sobe: Vec<_> = l
            .iter()
            .take(n.saturating_sub(1))
            .map(|s| v(&textura(device, *s, 1, ra | tb)))
            .collect();
        let uniforme = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward brilho passes"),
            size: (2 * n).max(1) as u64 * crate::gpu::SLOT,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let grupo = |origem: &wgpu::TextureView, nivel: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-mesh-forward brilho passe"),
                layout: &brilho.bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(origem),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(nivel),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &uniforme,
                            offset: 0,
                            size: std::num::NonZeroU64::new(PASSE as u64),
                        }),
                    },
                ],
            })
        };
        let grupos_desce = (0..n)
            .map(|k| {
                let o = if k == 0 { &linear_vista } else { &desce[k - 1] };
                grupo(o, o)
            })
            .collect();
        let grupos_sobe = (0..n.saturating_sub(1))
            .map(|k| {
                let o = if k + 2 == n {
                    &desce[n - 1]
                } else {
                    &sobe[k + 1]
                };
                grupo(o, &desce[k])
            })
            .collect();
        let acc0 = sobe.first().or(desce.first()).unwrap_or(resolvida);
        let ecra_bind = crate::gpu_alvo::ecra_bind(device, ecra_bgl, resolvida, acc0, ecra_ub);
        Self {
            linear_msaa,
            linear,
            linear_vista,
            desce,
            sobe,
            grupos_desce,
            grupos_sobe,
            uniforme,
            ecra_bind,
        }
    }

    /// Grava os passes da cadeia: desce `n` vezes (o corte no primeiro), sobe `n − 1`.
    pub(crate) fn grava(&self, enc: &mut wgpu::CommandEncoder, brilho: &Brilho) {
        let passe = |enc: &mut wgpu::CommandEncoder,
                     alvo: &wgpu::TextureView,
                     pipeline: &wgpu::RenderPipeline,
                     grupo: &wgpu::BindGroup,
                     fatia: usize| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward brilho"),
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
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, grupo, &[(fatia as u64 * crate::gpu::SLOT) as u32]);
            pass.draw(0..3, 0..1);
        };
        let n = self.desce.len();
        for k in 0..n {
            passe(enc, &self.desce[k], &brilho.desce, &self.grupos_desce[k], k);
        }
        for k in (0..n.saturating_sub(1)).rev() {
            passe(
                enc,
                &self.sobe[k],
                &brilho.sobe,
                &self.grupos_sobe[k],
                n + k,
            );
        }
    }
}

impl Brilho {
    /// Compila os QUATRO pipelines do brilho (objeto e chão com a cena-linear, descer, subir).
    pub(crate) fn novo(
        device: &wgpu::Device,
        cor: wgpu::TextureFormat,
        objeto: &wgpu::RenderPipelineDescriptor<'_>,
        chao: &wgpu::RenderPipelineDescriptor<'_>,
        fundo: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> Self {
        let dois = |blend, mascara| {
            [
                Some(wgpu::ColorTargetState {
                    format: cor,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: LINEAR,
                    blend: None,
                    write_mask: mascara,
                }),
            ]
        };
        let com_cena = |d: &wgpu::RenderPipelineDescriptor<'_>,
                        entrada,
                        alvos: &[Option<wgpu::ColorTargetState>]| {
            let f = d
                .fragment
                .as_ref()
                .expect("o objeto e o chão têm fragmento");
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-mesh-forward com cena-linear"),
                fragment: Some(wgpu::FragmentState {
                    module: f.module,
                    entry_point: Some(entrada),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: alvos,
                }),
                ..d.clone()
            })
        };
        let obj = com_cena(
            objeto,
            "fs_objeto_brilho",
            &dois(None, wgpu::ColorWrites::ALL),
        );
        let ch = com_cena(
            chao,
            "fs_chao_brilho",
            &dois(
                Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                wgpu::ColorWrites::empty(),
            ),
        );
        let fu = com_cena(
            fundo,
            "fs_fundo_brilho",
            &dois(None, wgpu::ColorWrites::ALL),
        );
        let tex = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward brilho"),
            entries: &[
                tex(0),
                tex(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-mesh-forward brilho"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let cadeia = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh-forward brilho"),
            source: wgpu::ShaderSource::Wgsl(crate::fonte::brilho().into()),
        });
        let degrau = |entrada| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-mesh-forward brilho degrau"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &cadeia,
                    entry_point: Some("vs_cheio"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &cadeia,
                    entry_point: Some(entrada),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: LINEAR,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            objeto: obj,
            chao: ch,
            fundo: fu,
            desce: degrau("fs_desce"),
            sobe: degrau("fs_sobe"),
            bgl,
        }
    }
}
