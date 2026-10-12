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
use crate::compositor::Compositor;
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
    #[must_use]
    pub fn estado(self) -> wgpu::BlendState {
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

/// Os pipelines das corridas com mistura e o compositor delas.
pub(super) struct Misturas {
    soma: Variantes<wgpu::RenderPipeline>,
    tela: Variantes<wgpu::RenderPipeline>,
    /// `None` ⇒ o dispositivo não tem `DUAL_SOURCE_BLENDING`.
    multiplica: Option<Variantes<wgpu::RenderPipeline>>,
    compositor: Compositor,
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
        Self {
            soma: de(module, "fs_mistura", Mistura::Soma),
            tela: de(module, "fs_mistura", Mistura::Tela),
            multiplica,
            compositor: Compositor::new(device, format),
        }
    }
}

impl ShapePass {
    /// **O dispositivo desenha esta mistura?** — `false` só para o Multiplica sem
    /// `DUAL_SOURCE_BLENDING`.
    #[must_use]
    pub fn sabe(&self, m: Mistura) -> bool {
        m != Mistura::Multiplica || self.misturas.multiplica.is_some()
    }

    /// ⭐⭐ **As cópias por CORRIDAS de mistura** (B5), pelo [`Compositor`]. Sem corridas, ou com uma
    /// só `Normal` sem grupo, é o [`Self::draw`] de sempre, byte a byte. Com tracejado carregado as
    /// corridas usam a variante COMPLETA (a mesma imagem que a enxuta; a escolha por quadro da placa
    /// é de UM desenho indirecto, não de intervalos). ⚠️ O `encoder` tem de ser submetido antes do
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
        let completa = self.tracejado;
        let (normal, m) = (&self.pipeline, &mut self.misturas);
        let (soma, tela, multiplica) = (&m.soma, &m.tela, m.multiplica.as_ref());
        let mut desenha = |pass: &mut wgpu::RenderPass<'static>, c: &Corrida| {
            let variantes = match c.mistura {
                Mistura::Normal => Some(normal),
                Mistura::Soma => Some(soma),
                Mistura::Tela => Some(tela),
                Mistura::Multiplica => multiplica,
            };
            let Some(variantes) = variantes else {
                eprintln!(
                    "[formas] Multiplica pedido num dispositivo sem DUAL_SOURCE_BLENDING: a corrida nao desenha"
                );
                return;
            };
            pass.set_pipeline(variantes.de(completa));
            pass.set_bind_group(0, &bg, &[]);
            pass.set_bind_group(1, &leitura, &[]);
            pass.draw(0..6, c.copias.clone());
        };
        let mut sessao = m.compositor.sessao(gpu, target, tamanho, load);
        sessao.corridas(encoder, corridas, &mut desenha);
        sessao.fecha(encoder);
    }
}
