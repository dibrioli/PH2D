//! ⭐ **A cascata holográfica na placa** (W5c) — o plano dos passes do `c2_hrc.wgsl`
//! (por quadrante: T0–T2 traçados, T3..TN e R_{N−1}..R0 pelas fusões, a soma; depois a saída na c0),
//! montado UMA vez (os grupos e os uniformes não mudam com a cena) e regravado por quadro. Módulo-filho
//! do [`super`].

use super::*;
use crate::config::Holografica as Escolhas;
use crate::hrc::{self, Quadrante};

/// O espelho de `Hrc` do `c2_hrc.wgsl` (56 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeHrc {
    quadrante: u32,
    nivel: u32,
    grade: [u32; 2],
    eixo: [u32; 2],
    nx: u32,
    nx_baixo: u32,
    nx_cima: u32,
    topo: u32,
    total: u32,
    _p: [u32; 3],
}

#[derive(Clone, Copy)]
enum Etapa {
    Traca = 0,
    Sobe = 1,
    Desce = 2,
    Soma = 3,
    Saida = 4,
}

const ENTRADAS: [&str; 5] = [
    "hrc_traca",
    "hrc_sobe",
    "hrc_desce",
    "hrc_soma",
    "hrc_saida",
];

/// O que a holográfica pega do solver ao nascer.
pub(super) struct Entradas<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub cfg: &'a Config,
    pub escolhas: Escolhas,
    pub lista: &'a PassList,
    pub vistas: &'a [wgpu::TextureView],
    pub modulo: &'a wgpu::ShaderModule,
    pub uni_cascatas: &'a wgpu::Buffer,
    pub vizinhos: &'a wgpu::Sampler,
    pub buffer: &'a dyn Fn(&'static str) -> wgpu::Buffer,
}

pub(super) struct Holografica {
    cena: wgpu::BindGroup,
    pipelines: Vec<wgpu::ComputePipeline>,
    /// `(etapa, grupo 3, grupos de trabalho)`, na ordem.
    passos: Vec<(Etapa, wgpu::BindGroup, [u32; 2])>,
}

fn armazenado(binding: u32, leitura: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: leitura },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn vista(binding: u32, v: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    }
}

fn buf(binding: u32, x: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: x.as_entire_binding(),
    }
}

/// Grupos de trabalho de 64 para `total` entradas, dobrados em 2D (65 535 por eixo).
fn grupos_1d(total: u64) -> [u32; 2] {
    let g = total.div_ceil(64).max(1);
    let x = g.min(65_535);
    [x as u32, g.div_ceil(x) as u32]
}

impl Holografica {
    pub(super) fn novo(e: &Entradas<'_>) -> Self {
        let device = e.device;
        let g = e.cfg.grade;
        let grade = [g.largura, g.altura];
        let cena_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c2 holográfica cena"),
            entries: &[
                entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                entrada_textura(1, FLOAT),
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
        let cena = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c2 holográfica cena"),
            layout: &cena_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fatia_uniforme(
                        e.uni_cascatas,
                        0,
                        size_of::<UniformeCascata>() as u64,
                    ),
                },
                vista(1, &e.vistas[EMISSAO]),
                vista(5, &e.vistas[COBERTURA]),
                vista(6, &e.vistas[TAU_DENTRO]),
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(e.vizinhos),
                },
            ],
        });
        let c0 = PassList::indice_da_c0(e.cfg);
        let g3_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c2 holográfica"),
            entries: &[
                entrada_uniforme(4, wgpu::ShaderStages::COMPUTE),
                armazenado(5, true),
                armazenado(6, true),
                armazenado(7, false),
                armazenado(8, true),
                armazenado(9, false),
                armazenado(10, false),
                entrada_escrita(11, e.lista.texturas[c0].formato),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d c2 holográfica"),
            bind_group_layouts: &[Some(&cena_layout), None, None, Some(&g3_layout)],
            immediate_size: 0,
        });
        let constantes = [
            ("PASSO_HRC", f64::from(e.escolhas.passo)),
            ("BORRAO_HRC", f64::from(u8::from(e.escolhas.borrao))),
            ("MAIS_UM_HRC", f64::from(u8::from(e.escolhas.mais_um))),
            (
                "MASCARA_C0",
                f64::from(u8::from(e.cfg.preenchimento.is_some())),
            ),
        ];
        let pipelines = ENTRADAS
            .iter()
            .map(|entrada| {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(entrada),
                    layout: Some(&pl),
                    module: e.modulo,
                    entry_point: Some(entrada),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &constantes,
                        zero_initialize_workgroup_memory: true,
                    },
                    cache: None,
                })
            })
            .collect();

        let t: Vec<wgpu::Buffer> = (0..=Quadrante::de(0, grade).n.max(Quadrante::de(1, grade).n))
            .map(|n| (e.buffer)(hrc::NOMES_T[n as usize]))
            .collect();
        let r = hrc::NOMES_R.map(|nome| (e.buffer)(nome));
        let acumulado = (e.buffer)(hrc::NOME_ACUMULADO);
        let uniforme = (e.buffer)(hrc::NOME_UNIFORME);
        let [vazio_l, vazio_t, vazio_r] = hrc::NOMES_VAZIOS.map(|nome| (e.buffer)(nome));

        let mut uni = Vec::new();
        let mut passos = Vec::new();
        let mut passo =
            |etapa: Etapa, u: UniformeHrc, ligados: [&wgpu::Buffer; 5], grupos: [u32; 2]| {
                let k = passos.len() as u64;
                uni.resize(((k + 1) * PASSO_UNIFORME) as usize, 0);
                let o = (k * PASSO_UNIFORME) as usize;
                uni[o..o + size_of::<UniformeHrc>()].copy_from_slice(bytemuck::bytes_of(&u));
                let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("luz2d c2 holográfica"),
                    layout: &g3_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: fatia_uniforme(&uniforme, k, size_of::<UniformeHrc>() as u64),
                        },
                        buf(5, ligados[0]),
                        buf(6, ligados[1]),
                        buf(7, ligados[2]),
                        buf(8, ligados[3]),
                        buf(9, ligados[4]),
                        buf(10, &acumulado),
                        vista(11, &e.vistas[c0]),
                    ],
                });
                passos.push((etapa, grupo, grupos));
            };
        for q in 0..4 {
            let quad = Quadrante::de(q, grade);
            let base = UniformeHrc {
                quadrante: q,
                nivel: 0,
                grade,
                eixo: [quad.x, quad.y],
                nx: 0,
                nx_baixo: 0,
                nx_cima: 0,
                topo: 0,
                total: 0,
                _p: [0; 3],
            };
            let nx = |n: u32| quad.nx(n);
            for n in 0..=quad.n {
                let total = quad.entradas_t(n);
                let u = UniformeHrc {
                    nivel: n,
                    nx: nx(n),
                    nx_baixo: if n > 0 { nx(n - 1) } else { 0 },
                    total: total as u32,
                    ..base
                };
                let (etapa, ler) = if n <= 2 {
                    (Etapa::Traca, &vazio_l)
                } else {
                    (Etapa::Sobe, &t[(n - 1) as usize])
                };
                passo(
                    etapa,
                    u,
                    [ler, &vazio_l, &t[n as usize], &vazio_l, &vazio_r],
                    grupos_1d(total),
                );
            }
            for n in (0..quad.n).rev() {
                let total = quad.entradas_r(n);
                let topo = n + 1 == quad.n;
                let u = UniformeHrc {
                    nivel: n,
                    nx: nx(n),
                    nx_cima: nx(n + 1),
                    topo: u32::from(topo),
                    total: total as u32,
                    ..base
                };
                let cima = if topo {
                    &vazio_l
                } else {
                    &r[((n + 1) % 2) as usize]
                };
                passo(
                    Etapa::Desce,
                    u,
                    [
                        &t[n as usize],
                        &t[(n + 1) as usize],
                        &vazio_t,
                        cima,
                        &r[(n % 2) as usize],
                    ],
                    grupos_1d(total),
                );
            }
            let total = u64::from(quad.x) * u64::from(quad.y);
            let u = UniformeHrc {
                nx: nx(0),
                total: total as u32,
                ..base
            };
            passo(
                Etapa::Soma,
                u,
                [&vazio_l, &vazio_l, &vazio_t, &r[0], &vazio_r],
                grupos_1d(total),
            );
        }
        let u = UniformeHrc {
            quadrante: 0,
            nivel: 0,
            grade,
            eixo: grade,
            nx: 0,
            nx_baixo: 0,
            nx_cima: 0,
            topo: 0,
            total: 0,
            _p: [0; 3],
        };
        passo(
            Etapa::Saida,
            u,
            [&vazio_l, &vazio_l, &vazio_t, &vazio_l, &vazio_r],
            [grade[0].div_ceil(8), grade[1].div_ceil(8)],
        );
        e.queue.write_buffer(&uniforme, 0, &uni);
        Self {
            cena,
            pipelines,
            passos,
        }
    }

    /// Grava a cascata holográfica inteira no passe `p` (a c0 sai dela).
    pub(super) fn gravar(&self, p: &mut wgpu::ComputePass<'_>) {
        p.set_bind_group(0, &self.cena, &[]);
        for (etapa, grupo, [x, y]) in &self.passos {
            p.set_pipeline(&self.pipelines[*etapa as usize]);
            p.set_bind_group(3, grupo, &[]);
            p.dispatch_workgroups(*x, *y, 1);
        }
    }
}
