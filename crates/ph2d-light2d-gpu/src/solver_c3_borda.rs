//! ⭐ W5d — a borda exata da captura que a C3 lê (o `c3_borda` do `c3_borda.wgsl`) e, com
//! `Config::borda_cantos` (W6f), o passe que espalha as cunhas dos vértices (`c3_cantos`). Módulo-filho
//! de `solver_c3.rs` (o tecto de 700 LOC).

use super::*;

/// A assinatura do `pipeline_em` da C3 (as constantes comuns por cima das próprias).
pub(super) type PipelineEm<'a> = dyn Fn(&str, &[Option<&wgpu::BindGroupLayout>], &str, &[(&str, f64)]) -> wgpu::ComputePipeline
    + 'a;

pub(super) struct Borda {
    pipeline: wgpu::ComputePipeline,
    grupo: wgpu::BindGroup,
    tamanho: [u32; 2],
    /// ⭐ W6f: o `c3_cantos` e o grupo 3 dele.
    cantos: Option<(wgpu::ComputePipeline, wgpu::BindGroup)>,
}

impl Borda {
    /// Lê a cobertura crua `cob`, escreve a da C3 em `b` (grupo 3, bindings 12–15) e, com `cantos`
    /// (`[as cunhas dos vértices, as espalhadas]`), espalha-as (bindings 16–17).
    pub(super) fn nova(
        e: &Entradas<'_>,
        b: usize,
        cob: usize,
        (cantos, vazia): (Option<[usize; 2]>, Option<usize>),
        pipeline_em: &PipelineEm<'_>,
    ) -> Self {
        let device = e.placa;
        let g3 = layout(
            device,
            "luz2d c3 borda",
            &[
                entrada_textura(12, FLOAT),
                entrada_escrita(13, wgpu::TextureFormat::Rgba16Float),
                // ⭐ W6d: a contínua lê a 3×3 por quatro `textureGather`.
                wgpu::BindGroupLayoutEntry {
                    binding: 14,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                entrada_escrita(15, wgpu::TextureFormat::Rgba16Float),
            ],
        );
        let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c3 borda"),
            layout: &g3,
            entries: &[
                vista(12, &e.vistas[cob]),
                vista(13, &e.vistas[b]),
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: wgpu::BindingResource::Sampler(e.vizinhos),
                },
                vista(
                    15,
                    &e.vistas[cantos.map_or_else(|| vazia.expect("a vazia"), |c| c[0])],
                ),
            ],
        });
        let cantos = cantos.map(|[v, v2]| {
            let g = layout(
                device,
                "luz2d c3 cantos",
                &[
                    entrada_textura(16, FLOAT),
                    entrada_escrita(17, wgpu::TextureFormat::Rgba16Float),
                ],
            );
            let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 cantos"),
                layout: &g,
                entries: &[vista(16, &e.vistas[v]), vista(17, &e.vistas[v2])],
            });
            (
                pipeline_em(
                    "luz2d c3 cantos",
                    &[None, None, None, Some(&g)],
                    "c3_cantos",
                    &[],
                ),
                grupo,
            )
        });
        Self {
            pipeline: pipeline_em(
                "luz2d c3 borda",
                &[None, None, None, Some(&g3)],
                "c3_borda",
                &[],
            ),
            grupo,
            tamanho: e.lista.texturas[b].tamanho,
            cantos,
        }
    }

    /// Os despachos, antes de tudo o que lê a borda.
    pub(super) fn gravar(&self, p: &mut wgpu::ComputePass<'_>) {
        let [w, h] = self.tamanho;
        p.set_pipeline(&self.pipeline);
        p.set_bind_group(3, &self.grupo, &[]);
        p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        if let Some((pipe, g)) = &self.cantos {
            p.set_pipeline(pipe);
            p.set_bind_group(3, g, &[]);
            p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
    }
}
