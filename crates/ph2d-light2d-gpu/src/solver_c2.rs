//! **A montagem das cascatas (C2)** — a pipeline e um grupo por nível (o par de saída alterna desde
//! o topo). Módulo-filho do [`super`] (W5c: saiu do `solver.rs` pelo tecto de 700 LOC).

use super::*;

/// O que a C2 pega do solver ao nascer.
pub(super) struct Entradas<'a> {
    pub device: crate::Placa<'a>,
    pub cfg: &'a Config,
    pub lista: &'a PassList,
    pub vistas: &'a [wgpu::TextureView],
    pub modulo: &'a wgpu::ShaderModule,
    pub lampadas_layout: &'a wgpu::BindGroupLayout,
    pub uni_cascatas: &'a wgpu::Buffer,
    pub sementes_finais: usize,
    pub vizinhos: &'a wgpu::Sampler,
}

/// A pipeline das cascatas e o grupo de cada nível.
pub(super) fn cascatas(e: &Entradas<'_>) -> (wgpu::ComputePipeline, Vec<wgpu::BindGroup>) {
    let formato = e.lista.texturas[CASCATAS[0]].formato;
    let rc_layout = e
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c2"),
            entries: &[
                entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                entrada_textura(1, FLOAT),
                entrada_textura(2, wgpu::TextureSampleType::Uint),
                entrada_textura(3, FLOAT),
                entrada_escrita(4, formato),
                entrada_textura(5, FLOAT),
                entrada_textura(6, FLOAT),
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });
    let rc_pl = e
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d c2"),
            bind_group_layouts: &[Some(&rc_layout), Some(e.lampadas_layout)],
            immediate_size: 0,
        });
    let constantes = [
        ("FUSAO", e.cfg.fusao.codigo()),
        ("PASSO", f64::from(e.cfg.passo_perto)),
        (
            "MASCARA_C0",
            f64::from(u8::from(e.cfg.preenchimento.is_some())),
        ),
        ("MASCARA_DA_FUSAO", e.cfg.mascara_da_fusao.codigo()),
        (
            "FRONTEIRA_NA_FACE",
            f64::from(u8::from(e.cfg.fronteira_na_face)),
        ),
    ];
    let rc = e
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("luz2d c2 cascata"),
            layout: Some(&rc_pl),
            module: e.modulo,
            entry_point: Some("cascata"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &constantes,
                zero_initialize_workgroup_memory: true,
            },
            cache: None,
        });
    let n = e.cfg.n_cascatas();
    let rc_grupos = (0..n)
        .map(|i| {
            let saida = CASCATAS[((n - 1 - i) % 2) as usize];
            let superior = CASCATAS[((n - i) % 2) as usize];
            e.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c2"),
                layout: &rc_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: fatia_uniforme(
                            e.uni_cascatas,
                            u64::from(i),
                            size_of::<UniformeCascata>() as u64,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&e.vistas[EMISSAO]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&e.vistas[e.sementes_finais]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&e.vistas[superior]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&e.vistas[saida]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(&e.vistas[COBERTURA]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&e.vistas[TAU_DENTRO]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(e.vizinhos),
                    },
                ],
            })
        })
        .collect();
    (rc, rc_grupos)
}
