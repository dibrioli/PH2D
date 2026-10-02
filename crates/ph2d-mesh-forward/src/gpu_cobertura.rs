//! ⭐⭐ **A SOMBRA QUE POUSA NO CHÃO** — larga, mole e sem grão, ao preço de duas leituras.
//!
//! A caixa de luz do estúdio tem `25°` de raio: a penumbra de um objeto a `0,7` do chão tem `~0,33`
//! de raio, `~500` texels do mapa de `2048`. ⛔ **Medido (02/10, gate `a_sombra_pousa_no_chao`):** a
//! busca do PCSS presa a `48` texels não achava bloqueador nenhum fora do miolo e o chão saía
//! INTACTO; espalhar `16` amostras por `500` texels daria grão.
//!
//! ⭐ A saída é a das *contact shadows* dos visualizadores de produto: os objetos vistos de CIMA
//! numa imagem de cobertura (`r` = há objeto, `g` = a profundidade dele vezes a cobertura), reduzida
//! em níveis de `2×2`. O chão lê o nível cujo borrão tem o tamanho FÍSICO da penumbra ali: duro
//! onde a peça encosta, mole longe dela.
//!
//! ⚠️ Cada nível é desenhado numa textura de passagem e COPIADO para o nível dele — ler e escrever
//! a mesma textura no mesmo passe (mesmo noutro nível) é o que o GLES do WebGL2 não garante.

use crate::gpu_alvo::PROFUNDIDADE;

/// O lado da cobertura. ⚠️ Basta pouco: ela só alimenta a sombra MOLE (a dura é do mapa de
/// [`crate::SOMBRA_LADO`]), e o nível mais fino já é mais fino que a penumbra mais curta que o
/// chão recebe.
pub const COBERTURA_LADO: u32 = 512;

pub(crate) const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(crate) struct Cobertura {
    pub textura: wgpu::Texture,
    /// Todos os níveis, para o chão amostrar com `lod`.
    pub vista: wgpu::TextureView,
    profundidade: wgpu::TextureView,
    nivel0: wgpu::TextureView,
    /// `(nível k como fonte, passagem do nível k+1, a vista dela)`.
    degraus: Vec<(wgpu::BindGroup, wgpu::Texture, wgpu::TextureView)>,
    reduz: wgpu::RenderPipeline,
    pub desenha: wgpu::RenderPipeline,
}

fn niveis() -> u32 {
    COBERTURA_LADO.ilog2() + 1
}

impl Cobertura {
    pub(crate) fn nova(
        device: &wgpu::Device,
        modulo: &wgpu::ShaderModule,
        pl_sombra: &wgpu::PipelineLayout,
        vertice: &[wgpu::VertexBufferLayout<'_>],
    ) -> Self {
        let textura = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-mesh-forward cobertura"),
            size: wgpu::Extent3d {
                width: COBERTURA_LADO,
                height: COBERTURA_LADO,
                depth_or_array_layers: 1,
            },
            mip_level_count: niveis(),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMATO,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let vista = textura.create_view(&wgpu::TextureViewDescriptor::default());
        let mip = |k: u32| {
            textura.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: k,
                mip_level_count: Some(1),
                ..Default::default()
            })
        };
        let nivel0 = mip(0);
        let profundidade = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-mesh-forward cobertura profundidade"),
                size: wgpu::Extent3d {
                    width: COBERTURA_LADO,
                    height: COBERTURA_LADO,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: PROFUNDIDADE,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default());

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward reduz"),
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
        let modulo_reduz = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh-forward reduz"),
            source: wgpu::ShaderSource::Wgsl(include_str!("reduz.wgsl").into()),
        });
        let alvo = [Some(wgpu::ColorTargetState {
            format: FORMATO,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let prim = wgpu::PrimitiveState::default();
        let reduz = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh-forward reduz"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ph2d-mesh-forward reduz"),
                    bind_group_layouts: &[Some(&bgl)],
                    immediate_size: 0,
                }),
            ),
            vertex: wgpu::VertexState {
                module: &modulo_reduz,
                entry_point: Some("vs_reduz"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &modulo_reduz,
                entry_point: Some("fs_reduz"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &alvo,
            }),
            primitive: prim,
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let desenha = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh-forward cobertura"),
            layout: Some(pl_sombra),
            vertex: wgpu::VertexState {
                module: modulo,
                entry_point: Some("vs_cobertura"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: vertice,
            },
            fragment: Some(wgpu::FragmentState {
                module: modulo,
                entry_point: Some("fs_cobertura"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &alvo,
            }),
            primitive: prim,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: PROFUNDIDADE,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let degraus = (0..niveis() - 1)
            .map(|k| {
                let fonte = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("ph2d-mesh-forward reduz fonte"),
                    layout: &bgl,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&mip(k)),
                    }],
                });
                let lado = (COBERTURA_LADO >> (k + 1)).max(1);
                let passagem = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("ph2d-mesh-forward reduz passagem"),
                    size: wgpu::Extent3d {
                        width: lado,
                        height: lado,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: FORMATO,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let v = passagem.create_view(&wgpu::TextureViewDescriptor::default());
                (fonte, passagem, v)
            })
            .collect();
        Self {
            textura,
            vista,
            profundidade,
            nivel0,
            degraus,
            reduz,
            desenha,
        }
    }

    /// ⭐ Grava a cobertura (os objetos vistos de cima) e reduz os níveis. `desenha_objetos` desenha
    /// cada objeto com o grupo `1` já posto; o grupo `0` (o quadro) é posto aqui.
    pub(crate) fn grava(
        &self,
        enc: &mut wgpu::CommandEncoder,
        g0_sombra: &wgpu::BindGroup,
        desenha_objetos: impl Fn(&mut wgpu::RenderPass<'_>),
    ) {
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward cobertura"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.nivel0,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.profundidade,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.desenha);
            pass.set_bind_group(0, g0_sombra, &[]);
            desenha_objetos(&mut pass);
        }
        for (k, (fonte, passagem, vista)) in self.degraus.iter().enumerate() {
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ph2d-mesh-forward reduz"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: vista,
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
                pass.set_pipeline(&self.reduz);
                pass.set_bind_group(0, fonte, &[]);
                pass.draw(0..3, 0..1);
            }
            let lado = passagem.width();
            enc.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: passagem,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &self.textura,
                    mip_level: k as u32 + 1,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: lado,
                    height: lado,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}
