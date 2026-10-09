//! O solver: C0 (captura) → C1 (campo de distância, JFA) → C2 (cascatas) → fluência.
//!
//! Toda textura e todo buffer fixo nascem da [`PassList`], num único sítio ([`criar_textura`] e
//! [`criar_buffer`]); o gate G5 confere-o contra o alocador do wgpu.

use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::CenaDeLuz;

use crate::captura::{Forma, RelatorioC0, capturar};
use crate::config::{Config, Precisao};
use crate::passlist::{
    CASCATAS, COBERTURA, EMISSAO, PASSO_UNIFORME, PassList, SEMENTES, TAU_DENTRO, Textura,
};
use crate::shaders;
use crate::tempos::{self, Tempos, TemposMs};

#[derive(Clone, Debug, PartialEq)]
pub enum ErroSolver {
    /// `aritmetica: F16` num dispositivo sem `SHADER_F16`.
    SemShaderF16,
    ConfigInvalida(&'static str),
    /// A grade passa o `max_texture_dimension_2d` do dispositivo (um limite do HARDWARE).
    GradeGrandeDemais {
        lado: u32,
        limite: u32,
    },
}

impl std::fmt::Display for ErroSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SemShaderF16 => write!(
                f,
                "aritmética f16 pedida e o dispositivo não tem SHADER_F16"
            ),
            Self::ConfigInvalida(o) => write!(f, "configuração inválida: {o}"),
            Self::GradeGrandeDemais { lado, limite } => {
                write!(
                    f,
                    "grade de {lado} texels passa o limite do dispositivo ({limite})"
                )
            }
        }
    }
}

impl std::error::Error for ErroSolver {}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeC0 {
    tamanho: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeJfa {
    fino: [u32; 2],
    tamanho: [u32; 2],
    fator: u32,
    salto: u32,
    _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeCascata {
    tamanho: [u32; 2],
    nivel: u32,
    topo: u32,
    t_inicio: f32,
    t_fim: f32,
    texel_m: f32,
    fator_jfa: u32,
    /// Onde fica a amostra anterior ao início: a última do intervalo de baixo.
    recuo: f32,
    _pad: [f32; 3],
    ceu: [f32; 4],
}

/// O ÚNICO sítio onde o solver cria uma textura.
fn criar_textura(device: &wgpu::Device, t: &Textura) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(t.nome),
        size: wgpu::Extent3d {
            width: t.tamanho[0],
            height: t.tamanho[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: t.formato,
        usage: t.usos,
        view_formats: &[],
    })
}

/// O ÚNICO sítio onde o solver cria um buffer.
fn criar_buffer(
    device: &wgpu::Device,
    nome: &'static str,
    bytes: u64,
    usos: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(nome),
        size: bytes,
        usage: usos,
        mapped_at_creation: false,
    })
}

/// A geometria da cena: cresce por potências de dois, nunca encolhe.
struct Geometria {
    formas: wgpu::Buffer,
    vertices: wgpu::Buffer,
    cap_formas: usize,
    cap_vertices: usize,
    grupo: wgpu::BindGroup,
}

pub struct Solver {
    cfg: Config,
    n: u32,
    lista: PassList,
    texturas: Vec<wgpu::Texture>,
    alvos_c0: [wgpu::TextureView; 3],
    uni_c0: wgpu::Buffer,
    uni_cascatas: wgpu::Buffer,
    c0_layout: wgpu::BindGroupLayout,
    c0: wgpu::RenderPipeline,
    geometria: Option<Geometria>,
    jfa_semente: wgpu::ComputePipeline,
    jfa_salto: wgpu::ComputePipeline,
    jfa_grupos: Vec<wgpu::BindGroup>,
    rc: wgpu::ComputePipeline,
    rc_grupos: Vec<wgpu::BindGroup>,
    tempos: Option<Tempos>,
}

fn entrada_uniforme(binding: u32, vis: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: vis,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn entrada_textura(binding: u32, amostra: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: amostra,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn entrada_escrita(binding: u32, formato: wgpu::TextureFormat) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: formato,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    }
}

const FLOAT: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };

fn fatia_uniforme(buf: &wgpu::Buffer, indice: u64, bytes: u64) -> wgpu::BindingResource<'_> {
    wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        buffer: buf,
        offset: indice * PASSO_UNIFORME,
        size: wgpu::BufferSize::new(bytes),
    })
}

fn validar(cfg: &Config) -> Result<(), ErroSolver> {
    let g = &cfg.grade;
    if !(g.texel_m.is_finite() && g.texel_m > 0.0) {
        return Err(ErroSolver::ConfigInvalida(
            "texel_m tem de ser finito e > 0",
        ));
    }
    if !(g.origem_m[0].is_finite() && g.origem_m[1].is_finite()) {
        return Err(ErroSolver::ConfigInvalida("origem_m tem de ser finita"));
    }
    if g.largura == 0 || g.altura == 0 {
        return Err(ErroSolver::ConfigInvalida(
            "a grade tem de ter ao menos um texel",
        ));
    }
    if !(cfg.intervalo_c0.is_finite() && cfg.intervalo_c0 > 0.0) {
        return Err(ErroSolver::ConfigInvalida(
            "intervalo_c0 tem de ser finito e > 0",
        ));
    }
    if cfg.fator_jfa == 0 {
        return Err(ErroSolver::ConfigInvalida("fator_jfa tem de ser ≥ 1"));
    }
    if !(cfg.passo_perto.is_finite() && cfg.passo_perto > 0.0) {
        return Err(ErroSolver::ConfigInvalida(
            "passo_perto tem de ser finito e > 0",
        ));
    }
    Ok(())
}

impl Solver {
    /// # Errors
    /// [`ErroSolver`] se a configuração é inválida, pede f16 sem `SHADER_F16`, ou a grade passa o
    /// limite de textura do dispositivo.
    pub fn novo(gpu: &GpuContext, pedido: Config) -> Result<Self, ErroSolver> {
        validar(&pedido)?;
        let device = &gpu.device;
        if pedido.aritmetica == Precisao::F16
            && !device.features().contains(wgpu::Features::SHADER_F16)
        {
            return Err(ErroSolver::SemShaderF16);
        }
        let cfg = Config {
            grade: pedido.grade_efetiva(),
            ..pedido
        };
        let limite = device.limits().max_texture_dimension_2d;
        let lado = cfg.grade.largura.max(cfg.grade.altura);
        if lado > limite {
            return Err(ErroSolver::GradeGrandeDemais { lado, limite });
        }
        let n = cfg.n_cascatas();
        let saltos = cfg.saltos_jfa();
        let com_tempos = device.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let lista = PassList::de(&cfg, com_tempos);

        let texturas: Vec<wgpu::Texture> = lista
            .texturas
            .iter()
            .map(|t| criar_textura(device, t))
            .collect();
        let buffer = |nome: &'static str| {
            let b = lista
                .buffers
                .iter()
                .find(|b| b.nome == nome)
                .expect("buffer declarado na PassList");
            criar_buffer(device, b.nome, b.bytes, b.usos)
        };
        let uni_c0 = buffer("luz2d/buf/c0");
        let uni_jfa = buffer("luz2d/buf/jfa");
        let uni_cascatas = buffer("luz2d/buf/cascatas");
        let tempos = com_tempos.then(|| Tempos {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("luz2d/carimbos"),
                ty: wgpu::QueryType::Timestamp,
                count: tempos::n_carimbos(n),
            }),
            resolve: buffer("luz2d/buf/tempos_resolve"),
            leitura: buffer("luz2d/buf/tempos_leitura"),
            n,
            periodo_ns: f64::from(gpu.queue.get_timestamp_period()),
        });

        let vistas: Vec<wgpu::TextureView> = texturas
            .iter()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()))
            .collect();

        // ── C0 ──
        let c0_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c0"),
            entries: &[
                entrada_uniforme(0, wgpu::ShaderStages::VERTEX),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let c0_modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("luz2d c0_captura.wgsl"),
            source: wgpu::ShaderSource::Wgsl(shaders::C0.into()),
        });
        let c0_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d c0"),
            bind_group_layouts: &[Some(&c0_layout)],
            immediate_size: 0,
        });
        // Emissão do dono: o maior (um só, no caso comum). Cobertura: soma. τ dentro: multiplica.
        let soma = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let maior = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Max,
        };
        let multiplica = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Zero,
            dst_factor: wgpu::BlendFactor::Src,
            operation: wgpu::BlendOperation::Add,
        };
        let c0 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("luz2d c0 captura"),
            layout: Some(&c0_pl),
            vertex: wgpu::VertexState {
                module: &c0_modulo,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &c0_modulo,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: lista.texturas[EMISSAO].formato,
                        blend: Some(wgpu::BlendState {
                            color: maior,
                            alpha: maior,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: lista.texturas[COBERTURA].formato,
                        blend: Some(wgpu::BlendState {
                            color: soma,
                            alpha: soma,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: lista.texturas[TAU_DENTRO].formato,
                        blend: Some(wgpu::BlendState {
                            color: multiplica,
                            alpha: multiplica,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // ── C1 ──
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
        gpu.queue.write_buffer(&uni_jfa, 0, &uni);
        let jfa_grupo = |j: usize, entrada: usize, saida: usize| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c1"),
                layout: &jfa_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: fatia_uniforme(
                            &uni_jfa,
                            j as u64,
                            size_of::<UniformeJfa>() as u64,
                        ),
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
        let sementes_finais = SEMENTES[saltos.len() % 2];

        // ── C2 ──
        let formato = lista.texturas[CASCATAS[0]].formato;
        let rc_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let rc_modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("luz2d c2_rc.wgsl"),
            source: wgpu::ShaderSource::Wgsl(shaders::rc(cfg.aritmetica, cfg.armazenamento).into()),
        });
        let rc_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d c2"),
            bind_group_layouts: &[Some(&rc_layout)],
            immediate_size: 0,
        });
        let rc = compute(
            "luz2d c2 cascata",
            &rc_pl,
            &rc_modulo,
            "cascata",
            &[
                ("FUSAO", cfg.fusao.codigo()),
                ("PASSO", f64::from(cfg.passo_perto)),
            ],
        );
        // O `textureGather` dos 4 vizinhos da cobertura; a borda prende-se ao texel da borda.
        let vizinhos = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("luz2d c2 vizinhos"),
            ..Default::default()
        });
        let rc_grupos = (0..n)
            .map(|i| {
                let saida = CASCATAS[((n - 1 - i) % 2) as usize];
                let superior = CASCATAS[((n - i) % 2) as usize];
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("luz2d c2"),
                    layout: &rc_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: fatia_uniforme(
                                &uni_cascatas,
                                u64::from(i),
                                size_of::<UniformeCascata>() as u64,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&vistas[EMISSAO]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&vistas[sementes_finais]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&vistas[superior]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::TextureView(&vistas[saida]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(&vistas[COBERTURA]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::TextureView(&vistas[TAU_DENTRO]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::Sampler(&vizinhos),
                        },
                    ],
                })
            })
            .collect();

        gpu.queue.write_buffer(
            &uni_c0,
            0,
            bytemuck::bytes_of(&UniformeC0 {
                tamanho: [fino[0] as f32, fino[1] as f32],
                _pad: [0.0; 2],
            }),
        );

        let alvos_c0 = [
            vistas[EMISSAO].clone(),
            vistas[COBERTURA].clone(),
            vistas[TAU_DENTRO].clone(),
        ];
        Ok(Self {
            cfg,
            n,
            lista,
            texturas,
            alvos_c0,
            uni_c0,
            uni_cascatas,
            c0_layout,
            c0,
            geometria: None,
            jfa_semente,
            jfa_salto,
            jfa_grupos,
            rc,
            rc_grupos,
            tempos,
        })
    }

    /// A configuração efetiva (a grade já estendida).
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.cfg
    }

    #[must_use]
    pub fn pass_list(&self) -> &PassList {
        &self.lista
    }

    #[must_use]
    pub fn n_cascatas(&self) -> u32 {
        self.n
    }

    /// A FLUÊNCIA (rgb) do último quadro, um texel por sonda da c0, na grade efetiva.
    #[must_use]
    pub fn fluencia(&self) -> &wgpu::Texture {
        &self.texturas[PassList::indice_da_fluencia(&self.cfg)]
    }

    /// A cena como a C0 a capturou: `[emissão, cobertura, τ dentro]` — para quem precisa de a medir.
    #[must_use]
    pub fn cena_capturada(&self) -> [&wgpu::Texture; 3] {
        [
            &self.texturas[EMISSAO],
            &self.texturas[COBERTURA],
            &self.texturas[TAU_DENTRO],
        ]
    }

    /// Bytes da geometria residente (formas + vértices, na capacidade atual).
    #[must_use]
    pub fn bytes_da_geometria_residente(&self) -> u64 {
        self.geometria.as_ref().map_or(0, |g| {
            (g.cap_formas * size_of::<Forma>() + g.cap_vertices * size_of::<[f32; 2]>()) as u64
        })
    }

    fn subir_geometria(&mut self, gpu: &GpuContext, formas: &[Forma], vertices: &[[f32; 2]]) {
        let device = &gpu.device;
        let precisa = |cap: usize, n: usize| cap < n.max(1);
        let refazer = self.geometria.as_ref().is_none_or(|g| {
            precisa(g.cap_formas, formas.len()) || precisa(g.cap_vertices, vertices.len())
        });
        if refazer {
            let (cf, cv) = self
                .geometria
                .as_ref()
                .map_or((1, 1), |g| (g.cap_formas, g.cap_vertices));
            let cap_formas = cf.max(formas.len().next_power_of_two());
            let cap_vertices = cv.max(vertices.len().next_power_of_two());
            let usos = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
            let b_formas = criar_buffer(
                device,
                "luz2d/buf/formas",
                (cap_formas * size_of::<Forma>()) as u64,
                usos,
            );
            let b_vertices = criar_buffer(
                device,
                "luz2d/buf/vertices",
                (cap_vertices * size_of::<[f32; 2]>()) as u64,
                usos,
            );
            let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c0"),
                layout: &self.c0_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uni_c0.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: b_formas.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: b_vertices.as_entire_binding(),
                    },
                ],
            });
            self.geometria = Some(Geometria {
                formas: b_formas,
                vertices: b_vertices,
                cap_formas,
                cap_vertices,
                grupo,
            });
        }
        let g = self.geometria.as_ref().expect("geometria criada acima");
        if !formas.is_empty() {
            gpu.queue
                .write_buffer(&g.formas, 0, bytemuck::cast_slice(formas));
        }
        if !vertices.is_empty() {
            gpu.queue
                .write_buffer(&g.vertices, 0, bytemuck::cast_slice(vertices));
        }
    }

    /// Grava um quadro inteiro em `enc` (as subidas vão pela fila, antes do submit de `enc`).
    pub fn gravar(
        &mut self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        cena: &CenaDeLuz,
    ) -> RelatorioC0 {
        let cap = capturar(cena, &self.cfg.grade);
        self.subir_geometria(gpu, &cap.formas, &cap.vertices);

        let mut uni = vec![0u8; (PASSO_UNIFORME * u64::from(self.n)) as usize];
        for i in 0..self.n {
            let (t_inicio, t_fim) = self.cfg.intervalo(i);
            let recuo = if i == 0 {
                0.0
            } else {
                let (a, b) = self.cfg.intervalo(i - 1);
                self.cfg.passo_perto.min(b - a)
            };
            let [r, g, b] = cena.ceu;
            let u = UniformeCascata {
                tamanho: [self.cfg.grade.largura, self.cfg.grade.altura],
                nivel: i,
                topo: u32::from(i + 1 == self.n),
                t_inicio,
                t_fim,
                texel_m: self.cfg.grade.texel_m as f32,
                fator_jfa: self.cfg.fator_jfa,
                recuo,
                _pad: [0.0; 3],
                ceu: [r, g, b, 0.0],
            };
            let o = i as usize * PASSO_UNIFORME as usize;
            uni[o..o + size_of::<UniformeCascata>()].copy_from_slice(bytemuck::bytes_of(&u));
        }
        gpu.queue.write_buffer(&self.uni_cascatas, 0, &uni);

        let geo = self
            .geometria
            .as_ref()
            .expect("subir_geometria cria a geometria");
        {
            let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("luz2d C0 captura"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.alvos_c0[0],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.alvos_c0[1],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.alvos_c0[2],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: self.tempos.as_ref().map(|t| t.escrita_render(tempos::C0)),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let total = u32::try_from(cap.formas.len()).expect("formas cabem em u32");
            if total > 0 {
                p.set_pipeline(&self.c0);
                p.set_bind_group(0, &geo.grupo, &[]);
                p.draw(0..4, 0..total);
            }
        }
        {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C1 campo"),
                timestamp_writes: self.tempos.as_ref().map(|t| t.escrita_compute(tempos::C1)),
            });
            let [w, h] = self.cfg.grade_jfa();
            let grupos = [w.div_ceil(8), h.div_ceil(8)];
            p.set_pipeline(&self.jfa_semente);
            p.set_bind_group(0, &self.jfa_grupos[0], &[]);
            p.dispatch_workgroups(grupos[0], grupos[1], 1);
            p.set_pipeline(&self.jfa_salto);
            for g in &self.jfa_grupos[1..] {
                p.set_bind_group(0, g, &[]);
                p.dispatch_workgroups(grupos[0], grupos[1], 1);
            }
        }
        let grupos = [
            self.cfg.grade.largura.div_ceil(8),
            self.cfg.grade.altura.div_ceil(8),
        ];
        for i in (0..self.n).rev() {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C2 cascata"),
                timestamp_writes: self
                    .tempos
                    .as_ref()
                    .map(|t| t.escrita_compute(tempos::da_cascata(i))),
            });
            p.set_pipeline(&self.rc);
            p.set_bind_group(0, &self.rc_grupos[i as usize], &[]);
            p.dispatch_workgroups(grupos[0], grupos[1], 1);
        }
        if let Some(t) = &self.tempos {
            t.resolver(enc);
        }
        cap.relatorio
    }

    /// Move a grade no MUNDO (pan, oitava de zoom) SEM realocar: as dimensões ficam, e a origem e o
    /// texel só entram pela captura e pelos uniformes de cada quadro (`gravar`). Quem ancora a
    /// origem (G11) é o chamador.
    ///
    /// # Errors
    /// [`ErroSolver::ConfigInvalida`] com origem ou texel não finitos (a grade fica a anterior).
    pub fn reposicionar(&mut self, origem_m: [f64; 2], texel_m: f64) -> Result<(), ErroSolver> {
        let mut cfg = self.cfg;
        cfg.grade.origem_m = origem_m;
        cfg.grade.texel_m = texel_m;
        validar(&cfg)?;
        self.cfg = cfg;
        Ok(())
    }

    /// Um quadro: grava e submete.
    pub fn quadro(&mut self, gpu: &GpuContext, cena: &CenaDeLuz) -> RelatorioC0 {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("luz2d quadro"),
            });
        let r = self.gravar(gpu, &mut enc, cena);
        gpu.queue.submit([enc.finish()]);
        r
    }

    /// Os tempos de placa do último quadro submetido; `None` sem `TIMESTAMP_QUERY`.
    #[must_use]
    pub fn tempos(&self, gpu: &GpuContext) -> Option<TemposMs> {
        self.tempos.as_ref()?.ler(&gpu.device)
    }
}
