//! ⭐ C1 — o campo de distância (JFA): as duas pipelines, os uniformes dos saltos e os grupos.
//! Módulo-filho do [`super`] (o tecto de 700 LOC).

use super::*;

pub(super) struct Jfa {
    pub semente: wgpu::ComputePipeline,
    pub salto: wgpu::ComputePipeline,
    pub grupos: Vec<wgpu::BindGroup>,
}

pub(super) fn montar(
    device: crate::Placa<'_>,
    queue: &wgpu::Queue,
    cfg: &Config,
    saltos: &[u32],
    vistas: &[wgpu::TextureView],
    uni_jfa: &wgpu::Buffer,
) -> Jfa {
    let jfa_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("luz2d c1"),
        entries: &[
            entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
            entrada_textura(1, FLOAT),
            entrada_textura(2, wgpu::TextureSampleType::Uint),
            entrada_escrita(3, wgpu::TextureFormat::R32Uint),
        ],
    });
    let jfa_modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("luz2d c1_jfa.wgsl"),
        source: wgpu::ShaderSource::Wgsl(shaders::C1.into()),
    });
    let jfa_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("luz2d c1"),
        bind_group_layouts: &[Some(&jfa_layout)],
        immediate_size: 0,
    });
    let compute = |rotulo,
                   pl: &wgpu::PipelineLayout,
                   modulo: &wgpu::ShaderModule,
                   entrada,
                   constantes: &[(&str, f64)]| {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(rotulo),
            layout: Some(pl),
            module: modulo,
            entry_point: Some(entrada),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: constantes,
                zero_initialize_workgroup_memory: true,
            },
            cache: None,
        })
    };
    let jfa_semente = compute("luz2d c1 semente", &jfa_pl, &jfa_modulo, "semente", &[]);
    let jfa_salto = compute("luz2d c1 salto", &jfa_pl, &jfa_modulo, "salto", &[]);
    let jfa_tamanho = cfg.grade_jfa();
    let fino = [cfg.grade.largura, cfg.grade.altura];
    let mut uni = vec![0u8; (PASSO_UNIFORME * (saltos.len() as u64 + 1)) as usize];
    for (j, salto) in std::iter::once(0).chain(saltos.iter().copied()).enumerate() {
        let u = UniformeJfa {
            fino,
            tamanho: jfa_tamanho,
            fator: cfg.fator_jfa,
            salto,
            _pad: [0; 2],
        };
        let o = j * PASSO_UNIFORME as usize;
        uni[o..o + size_of::<UniformeJfa>()].copy_from_slice(bytemuck::bytes_of(&u));
    }
    queue.write_buffer(uni_jfa, 0, &uni);
    let jfa_grupo = |j: usize, entrada: usize, saida: usize| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c1"),
            layout: &jfa_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fatia_uniforme(uni_jfa, j as u64, size_of::<UniformeJfa>() as u64),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&vistas[COBERTURA]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&vistas[entrada]),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&vistas[saida]),
                },
            ],
        })
    };
    // A semente escreve A (e liga B como entrada, que ela não lê); o salto j lê j%2 e escreve o outro.
    let mut jfa_grupos = vec![jfa_grupo(0, SEMENTES[1], SEMENTES[0])];
    for j in 0..saltos.len() {
        jfa_grupos.push(jfa_grupo(j + 1, SEMENTES[j % 2], SEMENTES[(j + 1) % 2]));
    }
    Jfa {
        semente: jfa_semente,
        salto: jfa_salto,
        grupos: jfa_grupos,
    }
}
