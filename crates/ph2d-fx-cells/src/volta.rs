//! ⭐⭐ **A VOLTA COM MISTURA** (B5, report do dono 11/10: *«o brilho do relevo não participa do
//! Multiply»*). Pela ordem do fio, o `Each Shape` corre ANTES da saída: cada cópia ganha o efeito e
//! SÓ DEPOIS se mistura com as outras. Com as corridas de mistura da placa
//! ([`ph2d_shape_gpu::Corrida`]), as células das formas não voltam em `Normal` à camada: voltam a
//! uma camada PRÓPRIA das formas, corrida a corrida, pelo [`ph2d_shape_gpu::Compositor`] (os
//! cortes e o grupo isolado da placa), e essa camada é colada por cima das sprites no fim — a mesma
//! arrumação da camada sem células (as sprites, e as formas misturadas só entre si por cima).

use std::ops::Range;

use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::{Compositor, Corrida, Mistura, Sessao};

use crate::FORMATO;
use crate::passes::{grupo_dos_quads, render};
use crate::plano::Celula;
use crate::shaders::{COMPOE_WGSL, VOLTA_MULTIPLICA_WGSL};

/// Os pipelines da volta, um por mistura.
pub(crate) struct Pipes {
    normal: wgpu::RenderPipeline,
    soma: wgpu::RenderPipeline,
    tela: wgpu::RenderPipeline,
    /// `None` ⇒ o dispositivo não tem `DUAL_SOURCE_BLENDING` (a placa não lhe dá corridas assim).
    multiplica: Option<wgpu::RenderPipeline>,
}

/// Os pipelines, o compositor e a camada das formas — campos separados para a sessão (que segura
/// o compositor) e os pipelines se emprestarem juntos.
pub(crate) struct Voltas {
    pub(crate) pipes: Pipes,
    pub(crate) compositor: Compositor,
    pub(crate) formas: (wgpu::Texture, wgpu::TextureView),
}

impl Voltas {
    pub(crate) fn new(gpu: &GpuContext, tamanho: (u32, u32)) -> Self {
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("fx_cells volta"),
                source: wgpu::ShaderSource::Wgsl(COMPOE_WGSL.into()),
            });
        let de = |m: Mistura| {
            render(
                gpu,
                &module,
                "vs_volta",
                "fs_volta",
                FORMATO,
                Some(m.estado()),
            )
        };
        let multiplica = ph2d_shape_gpu::multiplica_no_dispositivo(&gpu.device).then(|| {
            let m = gpu
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("fx_cells volta (multiplica)"),
                    source: wgpu::ShaderSource::Wgsl(
                        format!(
                            "enable dual_source_blending;\n{COMPOE_WGSL}{VOLTA_MULTIPLICA_WGSL}"
                        )
                        .into(),
                    ),
                });
            let estado = Some(Mistura::Multiplica.estado());
            render(gpu, &m, "vs_volta", "fs_volta_multiplica", FORMATO, estado)
        });
        Self {
            pipes: Pipes {
                normal: de(Mistura::Normal),
                soma: de(Mistura::Soma),
                tela: de(Mistura::Tela),
                multiplica,
            },
            compositor: Compositor::new(&gpu.device, FORMATO),
            formas: camada(gpu, tamanho),
        }
    }

    /// A camada das formas com `tamanho` (recriada quando a janela muda).
    pub(crate) fn garante(&mut self, gpu: &GpuContext, tamanho: (u32, u32)) {
        if (self.formas.0.width(), self.formas.0.height()) != (tamanho.0.max(1), tamanho.1.max(1)) {
            self.formas = camada(gpu, tamanho);
        }
    }
}

impl Pipes {
    /// As células `formas` (índices na página) das cópias de forma, pela `sessao`: cada célula com a
    /// mistura e o grupo da corrida da SUA cópia (`instancia − n_s`).
    #[expect(clippy::too_many_arguments, reason = "a página, a cópia e o alvo")]
    pub(crate) fn volta(
        &self,
        gpu: &GpuContext,
        sessao: &mut Sessao<'_>,
        celulas: &[Celula],
        formas: Range<u32>,
        n_s: u32,
        corridas: &[Corrida],
        cels: &wgpu::Buffer,
        atlas: &wgpu::TextureView,
        tamanho: (u32, u32),
    ) {
        let da_copia = |copia: u32| {
            corridas
                .iter()
                .find(|c| c.copias.contains(&copia))
                .map_or((Mistura::Normal, 0), |c| (c.mistura, c.grupo))
        };
        let mut das_celulas: Vec<Corrida> = Vec::new();
        for i in formas {
            let (mistura, grupo) = da_copia(celulas[i as usize].instancia - n_s);
            match das_celulas.last_mut() {
                Some(c) if c.mistura == mistura && c.grupo == grupo => c.copias.end = i + 1,
                _ => das_celulas.push(Corrida {
                    copias: i..i + 1,
                    mistura,
                    grupo,
                }),
            }
        }
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut desenha = |pass: &mut wgpu::RenderPass<'static>, c: &Corrida| {
            let pipeline = match c.mistura {
                Mistura::Normal => Some(&self.normal),
                Mistura::Soma => Some(&self.soma),
                Mistura::Tela => Some(&self.tela),
                Mistura::Multiplica => self.multiplica.as_ref(),
            };
            let Some(pipeline) = pipeline else {
                eprintln!(
                    "[fx_cells] Multiplica pedido num dispositivo sem DUAL_SOURCE_BLENDING: a corrida nao volta"
                );
                return;
            };
            let bg = grupo_dos_quads(gpu, pipeline, cels, atlas, tamanho, false);
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..6, c.copias.clone());
        };
        sessao.corridas(&mut enc, &das_celulas, &mut desenha);
        gpu.queue.submit([enc.finish()]);
    }
}

fn camada(gpu: &GpuContext, (w, h): (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fx_cells formas misturadas"),
        size: wgpu::Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMATO,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let v = t.create_view(&wgpu::TextureViewDescriptor::default());
    (t, v)
}
