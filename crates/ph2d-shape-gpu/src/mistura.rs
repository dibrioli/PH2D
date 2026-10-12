//! ⭐⭐ **A MISTURA POR CÓPIA** (B5, família B do Motion) — Soma, Tela e Multiplica no passe de
//! formas, com a lei W3C que o Vello compõe (`blend_mix_compose` do `vello_shaders`), em estados
//! fixos da placa. Pré-multiplicado, `s` a cópia e `b` o que está por baixo:
//!
//! | mistura | cor | alfa |
//! |---|---|---|
//! | Normal | `s + b·(1−αs)` | `αs + αb·(1−αs)` |
//! | Soma (`Compose::Plus`) | `s + b` | `min(αs + αb, 1)` |
//! | Tela | `s + b·(1−s)` | `αs + αb·(1−αs)` |
//! | Multiplica | `s·(1−αb) + b·(1−αs+s)` | `αs + αb·(1−αs)` |
//!
//! O Multiplica pede a 2.ª saída do fragmento como factor (`DUAL_SOURCE_BLENDING`, pedida pelo
//! `ph2d-gpu` onde o adaptador a tem); sem ela, [`ShapePass::sabe`] diz que não, e quem decide a
//! rota deixa essas cópias no Vello.
//!
//! ⭐ **O CORTE a `1`, onde o Vello o faz.** O Vello guarda cada camada em 8 bits: o fundo de uma
//! camada nova (cada cópia com mistura é uma) sai preso a `[0, 1]`, e o alfa do `Plus` é cortado na
//! composição — mas a COR do `Plus` passa solta a quem pinta em `Normal` a seguir, até ao próximo
//! empilhar ou à saída. A placa, em meio-float, não corta nada sozinha; os cortes são passes de ecrã
//! inteiro com mistura `Min` contra o branco: a cor e o alfa antes de uma corrida com mistura, antes de
//! colar um grupo e no fim, se alguma Soma correu; e o ALFA do rascunho de um grupo antes de o colar.
//!
//! ⭐ **O GRUPO isolado** (`Copies`): as cópias misturam-se só entre si e o grupo pousa em `Normal`.
//! Como primeiro conteúdo de um alvo limpo isso é desenhar directo; depois de algo, as corridas do
//! grupo vão a um RASCUNHO limpo, colado por cima do alvo.

use std::ops::Range;

use ph2d_gpu::GpuContext;

use super::{Copias, SHADER, ShapePass, ShapeView};
use crate::contorno::Variantes;

/// A lei de mistura de uma corrida de cópias.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Mistura {
    #[default]
    Normal,
    Soma,
    Tela,
    Multiplica,
}

impl Mistura {
    /// O estado fixo da placa — a tabela do cabeçalho.
    fn estado(self) -> wgpu::BlendState {
        use wgpu::{BlendComponent as C, BlendFactor as F, BlendOperation::Add};
        let c = |src_factor, dst_factor| C {
            src_factor,
            dst_factor,
            operation: Add,
        };
        let sobre = c(F::One, F::OneMinusSrcAlpha);
        match self {
            Self::Normal => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            Self::Soma => wgpu::BlendState {
                color: c(F::One, F::One),
                alpha: c(F::One, F::One),
            },
            Self::Tela => wgpu::BlendState {
                color: c(F::One, F::OneMinusSrc),
                alpha: sobre,
            },
            Self::Multiplica => wgpu::BlendState {
                color: c(F::OneMinusDstAlpha, F::Src1),
                alpha: sobre,
            },
        }
    }
}

/// **O dispositivo desenha o Multiplica exacto?** — a porta única (o passe e a decisão da rota
/// perguntam-na): a 2.ª saída do fragmento (`DUAL_SOURCE_BLENDING`).
#[must_use]
pub fn multiplica_no_dispositivo(device: &wgpu::Device) -> bool {
    device
        .features()
        .contains(wgpu::Features::DUAL_SOURCE_BLENDING)
}

/// Uma CORRIDA: as cópias `copias` (índices no buffer) desenhadas com `mistura`. `grupo != 0` ⇒ as
/// corridas vizinhas com o mesmo `grupo` formam um grupo ISOLADO (`Copies`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Corrida {
    pub copias: Range<u32>,
    pub mistura: Mistura,
    pub grupo: u32,
}

/// O pipeline de desenho das cópias com o fragmento `fs` e a mistura `m` — a porta única dos cinco.
pub(super) fn pipeline_de(
    device: &wgpu::Device,
    pl: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    fs: &str,
    m: Mistura,
    format: wgpu::TextureFormat,
    opcoes: wgpu::PipelineCompilationOptions<'_>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ph2d-shape-gpu"),
        layout: Some(pl),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs_main"),
            compilation_options: opcoes.clone(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(fs),
            compilation_options: opcoes,
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(m.estado()),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

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

/// Os pipelines das corridas com mistura e o rascunho dos grupos.
pub(super) struct Misturas {
    soma: Variantes<wgpu::RenderPipeline>,
    tela: Variantes<wgpu::RenderPipeline>,
    /// `None` ⇒ o dispositivo não tem `DUAL_SOURCE_BLENDING`.
    multiplica: Option<Variantes<wgpu::RenderPipeline>>,
    corte_rgba: wgpu::RenderPipeline,
    corte_alfa: wgpu::RenderPipeline,
    colagem: wgpu::RenderPipeline,
    colagem_bgl: wgpu::BindGroupLayout,
    formato: wgpu::TextureFormat,
    /// O rascunho dos grupos isolados, do tamanho do último alvo: a vista e o grupo da colagem.
    rascunho: Option<((u32, u32), wgpu::TextureView, wgpu::BindGroup)>,
}

impl Misturas {
    pub(super) fn new(
        device: &wgpu::Device,
        pl: &wgpu::PipelineLayout,
        module: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> Self {
        let de = |module: &wgpu::ShaderModule, fs: &'static str, m: Mistura| {
            Variantes::cria(&[], |opcoes| {
                pipeline_de(device, pl, module, fs, m, format, opcoes)
            })
        };
        let multiplica = multiplica_no_dispositivo(device).then(|| {
            let m = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-shape-gpu (multiplica)"),
                source: wgpu::ShaderSource::Wgsl(
                    format!(
                        "enable dual_source_blending;\n{SHADER}{}",
                        include_str!("multiplica.wgsl")
                    )
                    .into(),
                ),
            });
            de(&m, "fs_multiplica", Mistura::Multiplica)
        });
        let tela = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-shape-gpu (corte)"),
            source: wgpu::ShaderSource::Wgsl(include_str!("corte.wgsl").into()),
        });
        let vazio = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-shape-gpu corte"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let minimo = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Min,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Min,
            },
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
            soma: de(module, "fs_mistura", Mistura::Soma),
            tela: de(module, "fs_mistura", Mistura::Tela),
            multiplica,
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
            let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-shape-gpu colagem"),
                layout: &self.colagem_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&vista),
                }],
            });
            self.rascunho = Some((tamanho, vista, bg));
        }
        let (_, v, b) = self.rascunho.as_ref().expect("acabou de ser garantido");
        (v.clone(), b.clone())
    }
}

/// Um alvo a ser pintado por corridas: o `load` do próximo passe, se já tem conteúdo, e se a COR
/// pode estar acima de `1` (uma Soma correu e ninguém cortou).
struct Alvo<'a> {
    vista: &'a wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    desenhado: bool,
    solto: bool,
}

/// O que todas as corridas de um desenho partilham.
struct Ligacoes<'a> {
    bg: &'a wgpu::BindGroup,
    leitura: &'a wgpu::BindGroup,
    /// Há tracejado carregado: as corridas desenham com a variante COMPLETA (a mesma imagem que a
    /// enxuta; a escolha por quadro da placa é de UM desenho indirecto, não de intervalos).
    completa: bool,
}

impl ShapePass {
    /// **O dispositivo desenha esta mistura?** — `false` só para o Multiplica sem
    /// `DUAL_SOURCE_BLENDING`.
    #[must_use]
    pub fn sabe(&self, m: Mistura) -> bool {
        m != Mistura::Multiplica || self.misturas.multiplica.is_some()
    }

    /// ⭐⭐ **As cópias por CORRIDAS de mistura** (B5). Sem corridas, ou com uma só `Normal` sem
    /// grupo, é o [`Self::draw`] de sempre, byte a byte. ⚠️ O `encoder` tem de ser submetido antes do
    /// próximo desenho (a mesma regra do `draw`).
    #[expect(
        clippy::too_many_arguments,
        reason = "o draw de sempre mais as corridas"
    )]
    pub fn draw_em_corridas(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        view: ShapeView,
        copias: Copias<'_>,
        corridas: &[Corrida],
    ) {
        let simples = match corridas {
            [] => true,
            [c] => c.mistura == Mistura::Normal && c.grupo == 0,
            _ => false,
        };
        if simples || copias.count == 0 || self.carregadas.is_empty() {
            return self.draw(gpu, encoder, target, load, view, copias);
        }
        gpu.queue
            .write_buffer(&self.view_buf, 0, bytemuck::bytes_of(&view));
        let bg = self.liga(gpu, copias.buffer);
        self.contorno
            .calcula(gpu, encoder, &bg, copias.count, self.tracejado);
        self.ultimo = Some((bg.clone(), copias.count));
        let leitura = self.contorno.grupo_de_leitura(gpu);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "o tamanho do alvo, inteiro"
        )]
        let tamanho = (view.alvo[0] as u32, view.alvo[1] as u32);
        let ligacoes = Ligacoes {
            bg: &bg,
            leitura: &leitura,
            completa: self.tracejado,
        };
        let mut alvo = Alvo {
            vista: target,
            load,
            desenhado: matches!(load, wgpu::LoadOp::Load),
            solto: false,
        };
        let mut i = 0;
        while i < corridas.len() {
            let g = corridas[i].grupo;
            let mut j = i + 1;
            while g != 0 && j < corridas.len() && corridas[j].grupo == g {
                j += 1;
            }
            if g != 0 && alvo.desenhado {
                let (vista, colar) = self.misturas.rascunho(gpu, tamanho);
                let mut r = Alvo {
                    vista: &vista,
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    desenhado: false,
                    solto: false,
                };
                self.corridas_em(encoder, &mut r, &corridas[i..j], &ligacoes, false);
                let mut pass = abre(encoder, &mut alvo);
                if std::mem::take(&mut alvo.solto) {
                    self.corta(&mut pass, false);
                }
                pass.set_pipeline(&self.misturas.colagem);
                pass.set_bind_group(0, &colar, &[]);
                pass.draw(0..3, 0..1);
                alvo.solto = r.solto;
            } else {
                self.corridas_em(encoder, &mut alvo, &corridas[i..j], &ligacoes, true);
            }
            i = j;
        }
        if alvo.solto {
            let mut pass = abre(encoder, &mut alvo);
            self.corta(&mut pass, false);
        }
    }

    /// As `corridas` num passe sobre `alvo`, com os cortes do cabeçalho. `cortado_depois` = o alvo leva
    /// o corte da cor e do alfa a seguir (o alvo do desenho: antes de colar um grupo e no fim); senão
    /// é o RASCUNHO de um grupo, que vai ser FONTE da colagem e sai com o alfa cortado.
    ///
    /// ⚠️ Num alvo, o alfa acima de `1` depois de uma Soma não precisa de corte próprio: o `Normal` que
    /// vier compõe a cor só com o alfa da FONTE, e `min(αs + x·(1−αs), 1) = αs + min(x, 1)·(1−αs)`;
    /// toda mistura seguinte, a colagem e o fim cortam tudo antes.
    fn corridas_em(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        alvo: &mut Alvo<'_>,
        corridas: &[Corrida],
        l: &Ligacoes<'_>,
        cortado_depois: bool,
    ) {
        let mut pass = abre(encoder, alvo);
        for c in corridas {
            if c.mistura != Mistura::Normal && std::mem::take(&mut alvo.solto) {
                self.corta(&mut pass, false);
            }
            let variantes = match c.mistura {
                Mistura::Normal => Some(&self.pipeline),
                Mistura::Soma => Some(&self.misturas.soma),
                Mistura::Tela => Some(&self.misturas.tela),
                Mistura::Multiplica => self.misturas.multiplica.as_ref(),
            };
            let Some(variantes) = variantes else {
                eprintln!(
                    "[formas] Multiplica pedido num dispositivo sem DUAL_SOURCE_BLENDING: a corrida nao desenha"
                );
                continue;
            };
            pass.set_pipeline(variantes.de(l.completa));
            pass.set_bind_group(0, l.bg, &[]);
            pass.set_bind_group(1, l.leitura, &[]);
            pass.draw(0..6, c.copias.clone());
            alvo.solto |= c.mistura == Mistura::Soma;
        }
        if !cortado_depois && alvo.solto {
            self.corta(&mut pass, true);
        }
    }

    /// O corte a `1`: só o alfa, ou a cor e o alfa.
    fn corta(&self, pass: &mut wgpu::RenderPass<'_>, so_alfa: bool) {
        pass.set_pipeline(if so_alfa {
            &self.misturas.corte_alfa
        } else {
            &self.misturas.corte_rgba
        });
        pass.draw(0..3, 0..1);
    }
}

/// Abre um passe sobre `alvo` com o `load` dele; o seguinte carrega o que este deixou.
fn abre(encoder: &mut wgpu::CommandEncoder, alvo: &mut Alvo<'_>) -> wgpu::RenderPass<'static> {
    let load = std::mem::replace(&mut alvo.load, wgpu::LoadOp::Load);
    alvo.desenhado = true;
    encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-shape-gpu corridas"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: alvo.vista,
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
