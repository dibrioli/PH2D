//! ⭐⭐ **O COMPOSITOR DAS CORRIDAS** (B5) — a parte da mistura por cópia que não depende de QUEM
//! desenha: a ordem das corridas, o grupo `Copies` isolado num RASCUNHO, e os cortes a `1` onde o
//! Vello os faz (ver o cabeçalho de [`crate::Mistura`]). Dois desenhadores: o passe de formas
//! ([`crate::ShapePass::draw_em_corridas`]) e a volta das células do `Each Shape` (`ph2d-fx-cells`),
//! que devolve cada cópia JÁ filtrada e a mistura ali — o efeito entra na mistura, na ordem do fio.
//!
//! Uma [`Sessao`] aceita as corridas em VÁRIAS chamadas (as células vêm página a página): um grupo
//! aberto continua aberto na chamada seguinte se ela começar pelo mesmo grupo.

use ph2d_gpu::GpuContext;

use crate::{Corrida, Mistura};

/// Um passe de ecrã inteiro do `corte.wgsl`.
fn de_tela(
    device: &wgpu::Device,
    pl: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    fs: &str,
    format: wgpu::TextureFormat,
    blend: wgpu::BlendState,
    write_mask: wgpu::ColorWrites,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ph2d-shape-gpu tela"),
        layout: Some(pl),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs_tela"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(fs),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(blend),
                write_mask,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// Os cortes, a colagem e o rascunho, para alvos de um formato.
pub struct Compositor {
    corte_rgba: wgpu::RenderPipeline,
    corte_alfa: wgpu::RenderPipeline,
    colagem: wgpu::RenderPipeline,
    colagem_bgl: wgpu::BindGroupLayout,
    formato: wgpu::TextureFormat,
    /// O rascunho dos grupos isolados, do tamanho do último alvo: a vista e o grupo da colagem.
    rascunho: Option<((u32, u32), wgpu::TextureView, wgpu::BindGroup)>,
}

impl Compositor {
    #[must_use]
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let tela = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-shape-gpu (corte)"),
            source: wgpu::ShaderSource::Wgsl(include_str!("corte.wgsl").into()),
        });
        let vazio = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-shape-gpu corte"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let min = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Min,
        };
        let minimo = wgpu::BlendState {
            color: min,
            alpha: min,
        };
        let corte = |mask| de_tela(device, &vazio, &tela, "fs_um", format, minimo, mask);
        let colagem_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-shape-gpu colagem"),
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
        let colagem_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-shape-gpu colagem"),
            bind_group_layouts: &[Some(&colagem_bgl)],
            immediate_size: 0,
        });
        Self {
            corte_rgba: corte(wgpu::ColorWrites::ALL),
            corte_alfa: corte(wgpu::ColorWrites::ALPHA),
            colagem: de_tela(
                device,
                &colagem_pl,
                &tela,
                "fs_grupo",
                format,
                wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                wgpu::ColorWrites::ALL,
            ),
            colagem_bgl,
            formato: format,
            rascunho: None,
        }
    }

    /// Uma sessão sobre `alvo` (`tamanho` px), que começa com `load`.
    pub fn sessao<'a>(
        &'a mut self,
        gpu: &'a GpuContext,
        alvo: &'a wgpu::TextureView,
        tamanho: (u32, u32),
        load: wgpu::LoadOp<wgpu::Color>,
    ) -> Sessao<'a> {
        Sessao {
            comp: self,
            gpu,
            alvo,
            tamanho,
            estado: Estado {
                load,
                desenhado: matches!(load, wgpu::LoadOp::Load),
                solto: false,
            },
            grupo: None,
            direto: 0,
        }
    }

    /// Cola `fonte` (pré-multiplicada, do formato deste compositor) por cima de `alvo`.
    pub fn cola(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        fonte: &wgpu::TextureView,
        alvo: &wgpu::TextureView,
    ) {
        let bg = self.grupo_de(gpu, fonte);
        let mut estado = Estado {
            load: wgpu::LoadOp::Load,
            desenhado: true,
            solto: false,
        };
        let mut pass = abre(encoder, alvo, &mut estado);
        pass.set_pipeline(&self.colagem);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..3, 0..1);
    }

    fn grupo_de(&self, gpu: &GpuContext, vista: &wgpu::TextureView) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-shape-gpu colagem"),
            layout: &self.colagem_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(vista),
            }],
        })
    }

    /// O rascunho de `tamanho` (recriado quando o alvo muda de tamanho).
    fn rascunho(
        &mut self,
        gpu: &GpuContext,
        tamanho: (u32, u32),
    ) -> (wgpu::TextureView, wgpu::BindGroup) {
        if self.rascunho.as_ref().is_none_or(|(t, ..)| *t != tamanho) {
            let textura = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-shape-gpu rascunho do grupo"),
                size: wgpu::Extent3d {
                    width: tamanho.0.max(1),
                    height: tamanho.1.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.formato,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let vista = textura.create_view(&wgpu::TextureViewDescriptor::default());
            let bg = self.grupo_de(gpu, &vista);
            self.rascunho = Some((tamanho, vista, bg));
        }
        let (_, v, b) = self.rascunho.as_ref().expect("acabou de ser garantido");
        (v.clone(), b.clone())
    }

    /// O corte a `1`: só o alfa, ou a cor e o alfa.
    fn corta(&self, pass: &mut wgpu::RenderPass<'_>, so_alfa: bool) {
        pass.set_pipeline(if so_alfa {
            &self.corte_alfa
        } else {
            &self.corte_rgba
        });
        pass.draw(0..3, 0..1);
    }
}

/// O estado de um alvo: o `load` do próximo passe, se já tem conteúdo, e se a COR pode estar acima
/// de `1` (uma Soma correu e ninguém cortou).
struct Estado {
    load: wgpu::LoadOp<wgpu::Color>,
    desenhado: bool,
    solto: bool,
}

/// As corridas de um desenho sobre um alvo, em uma ou mais chamadas — fechada por [`Sessao::fecha`].
pub struct Sessao<'a> {
    comp: &'a mut Compositor,
    gpu: &'a GpuContext,
    alvo: &'a wgpu::TextureView,
    tamanho: (u32, u32),
    estado: Estado,
    /// O grupo aberto no RASCUNHO: o id, a vista, o grupo da colagem e o estado.
    grupo: Option<(u32, wgpu::TextureView, wgpu::BindGroup, Estado)>,
    /// O grupo desenhado DIRECTO no alvo (1.º conteúdo de um alvo limpo: isolar não muda nada).
    direto: u32,
}

impl Sessao<'_> {
    /// Mais `corridas`, pela ordem; `desenha` pinta UMA corrida no passe dado (o pipeline da
    /// mistura dela e as ligações de quem desenha).
    ///
    /// ⚠️ Num alvo, o alfa acima de `1` depois de uma Soma não precisa de corte próprio: o `Normal`
    /// que vier compõe a cor só com o alfa da FONTE, e `min(αs + x·(1−αs), 1) = αs + min(x, 1)·(1−αs)`;
    /// toda mistura seguinte, a colagem e o fim cortam tudo antes. O RASCUNHO é fonte da colagem e
    /// sai com o alfa cortado.
    pub fn corridas(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        corridas: &[Corrida],
        desenha: &mut dyn FnMut(&mut wgpu::RenderPass<'static>, &Corrida),
    ) {
        for c in corridas {
            if self.grupo.as_ref().is_some_and(|g| g.0 != c.grupo) {
                self.fecha_o_grupo(encoder);
            }
            if self.direto != c.grupo {
                self.direto = 0;
            }
            if c.grupo != 0 && self.grupo.is_none() && self.direto == 0 {
                if self.estado.desenhado {
                    let (vista, colar) = self.comp.rascunho(self.gpu, self.tamanho);
                    let limpo = Estado {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        desenhado: false,
                        solto: false,
                    };
                    self.grupo = Some((c.grupo, vista, colar, limpo));
                } else {
                    self.direto = c.grupo;
                }
            }
            let (vista, estado) = match self.grupo.as_mut() {
                Some((_, v, _, e)) => (&*v, e),
                None => (self.alvo, &mut self.estado),
            };
            let mut pass = abre(encoder, vista, estado);
            if c.mistura != Mistura::Normal && std::mem::take(&mut estado.solto) {
                self.comp.corta(&mut pass, false);
            }
            desenha(&mut pass, c);
            estado.solto |= c.mistura == Mistura::Soma;
        }
    }

    /// Cola o grupo aberto por cima do alvo (com o alfa dele cortado, e o alvo cortado antes).
    fn fecha_o_grupo(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let Some((_, vista, colar, mut estado)) = self.grupo.take() else {
            return;
        };
        if estado.solto {
            let mut pass = abre(encoder, &vista, &mut estado);
            self.comp.corta(&mut pass, true);
        }
        let mut pass = abre(encoder, self.alvo, &mut self.estado);
        if std::mem::take(&mut self.estado.solto) {
            self.comp.corta(&mut pass, false);
        }
        pass.set_pipeline(&self.comp.colagem);
        pass.set_bind_group(0, &colar, &[]);
        pass.draw(0..3, 0..1);
        self.estado.solto = estado.solto;
    }

    /// Fecha o grupo aberto e corta o alvo, se uma Soma o deixou solto.
    pub fn fecha(mut self, encoder: &mut wgpu::CommandEncoder) {
        self.fecha_o_grupo(encoder);
        if self.estado.solto {
            let mut pass = abre(encoder, self.alvo, &mut self.estado);
            self.comp.corta(&mut pass, false);
        }
    }
}

/// Abre um passe sobre `vista` com o `load` do estado; o seguinte carrega o que este deixou.
fn abre(
    encoder: &mut wgpu::CommandEncoder,
    vista: &wgpu::TextureView,
    estado: &mut Estado,
) -> wgpu::RenderPass<'static> {
    let load = std::mem::replace(&mut estado.load, wgpu::LoadOp::Load);
    estado.desenhado = true;
    encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-shape-gpu corridas"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: vista,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        })
        .forget_lifetime()
}
