//! **C3 na grade FINA** (W4b) — os passes depois das cascatas: o preenchimento do interior dos
//! opacos (W5a, F3), a visibilidade das lâmpadas diretas nas sondas da c0 e a grade fina (a c0
//! interpolada + a luz direta), com os grupos deles. A marcha lê a captura FINA dos opacos quando há
//! (W5a, F1 + F2). Módulo-filho do [`super`] (vê os auxiliares dele sem os abrir).

use super::*;
use crate::config::Refino;
use crate::passlist::{FLUENCIA, Opcionais};

/// O espelho de `Fina` do `c3_direta.wgsl` (48 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeFina {
    tamanho: [u32; 2],
    inicio: [u32; 2],
    fim: [u32; 2],
    inicio_grosso: [u32; 2],
    fim_grosso: [u32; 2],
    fina: u32,
    limiar: f32,
}

/// O espelho de `Preenche` do `c3_preenche.wgsl` (48 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformePreenche {
    celulas: [u32; 2],
    inicio: [u32; 2],
    fim: [u32; 2],
    celula: u32,
    salto: u32,
    fina: u32,
    _p: [u32; 3],
}

/// ⭐ W5a (F3): o JFA do complemento — a semente e os saltos, um grupo por passo (o uniforme de cada
/// um numa fatia do buffer).
struct Preenchimento {
    semente: wgpu::ComputePipeline,
    salto: wgpu::ComputePipeline,
    grupos: Vec<wgpu::BindGroup>,
    uniforme: wgpu::Buffer,
    saltos: Vec<u32>,
    u: UniformePreenche,
}

/// O que os passes C3 pegam do solver ao nascer.
pub(super) struct Entradas<'a> {
    pub gpu: &'a GpuContext,
    pub cfg: &'a Config,
    pub lista: &'a PassList,
    pub vistas: &'a [wgpu::TextureView],
    pub modulo: &'a wgpu::ShaderModule,
    pub lampadas_layout: &'a wgpu::BindGroupLayout,
    pub uni_cascatas: &'a wgpu::Buffer,
    pub uni_c3: &'a wgpu::Buffer,
    /// ⭐ W5a: o uniforme do preenchimento (com ele).
    pub uni_preenche: Option<&'a wgpu::Buffer>,
    pub sementes_finais: usize,
    pub vizinhos: &'a wgpu::Sampler,
}

pub(super) struct C3 {
    /// O grupo 0: a cena capturada e o uniforme da c0 (a marcha da c0, sem as saídas das cascatas).
    cena: wgpu::BindGroup,
    visibilidade: Option<(wgpu::ComputePipeline, wgpu::BindGroup)>,
    preenchimento: Option<Preenchimento>,
    fina: wgpu::ComputePipeline,
    grupo_fina: wgpu::BindGroup,
    uniforme: wgpu::Buffer,
    /// O uniforme deste quadro: a grade fina inteira por omissão, a região da janela com
    /// [`Self::regiao`].
    u: UniformeFina,
}

fn layout(
    device: &wgpu::Device,
    rotulo: &str,
    entradas: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(rotulo),
        entries: entradas,
    })
}

fn vista(binding: u32, v: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    }
}

impl C3 {
    pub(super) fn novo(e: &Entradas<'_>) -> Self {
        let device = &e.gpu.device;
        let cfg = e.cfg;
        let op = Opcionais::de(cfg);
        // A marcha da luz direta lê a captura FINA dos opacos quando há (W5a).
        let [cob, tau] = op.captura.unwrap_or([COBERTURA, TAU_DENTRO]);
        let cena_layout = layout(
            device,
            "luz2d c3 cena",
            &[
                entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                entrada_textura(1, FLOAT),
                entrada_textura(2, wgpu::TextureSampleType::Uint),
                entrada_textura(5, FLOAT),
                entrada_textura(6, FLOAT),
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        );
        let cena = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c3 cena"),
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
                vista(2, &e.vistas[e.sementes_finais]),
                vista(5, &e.vistas[cob]),
                vista(6, &e.vistas[tau]),
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(e.vizinhos),
                },
            ],
        });
        // O passo da marcha da luz direta: `passo_direto` texels FINOS (o das cascatas sem o passo
        // fino); junto à borda, a fração `passo_na_borda` dele.
        let fino = |passo_fino: bool| {
            f64::from(cfg.passo_perto)
                * if passo_fino {
                    f64::from(cfg.passo_direto) / f64::from(cfg.fina)
                } else {
                    1.0
                }
        };
        // As constantes de TODO passe da C3: a captura ligada e o preenchimento.
        let comuns = [
            ("ESCALA_DA_CAPTURA", f64::from(cfg.escala_da_captura())),
            ("COM_LUZ", 0.0),
            ("PASSO_NA_BORDA", f64::from(cfg.passo_na_borda)),
            (
                "PREENCHE_A_GRADE_INTEIRA",
                f64::from(u8::from(cfg.preenche_a_grade_inteira)),
            ),
            ("PREENCHE", f64::from(u8::from(cfg.preenchimento.is_some()))),
            ("CELULA_PREENCHE", f64::from(cfg.preenchimento.unwrap_or(1))),
        ];
        let bordas = match cfg.refino {
            Refino::Bordas(b) => Some(b),
            Refino::Tudo => None,
        };
        let pipeline_em = |rotulo: &str,
                           grupos: &[Option<&wgpu::BindGroupLayout>],
                           entrada: &str,
                           constantes: &[(&str, f64)]| {
            let constantes: Vec<(&str, f64)> = constantes.iter().chain(&comuns).copied().collect();
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(rotulo),
                bind_group_layouts: grupos,
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(rotulo),
                layout: Some(&pl),
                module: e.modulo,
                entry_point: Some(entrada),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &constantes,
                    zero_initialize_workgroup_memory: true,
                },
                cache: None,
            })
        };
        let pipeline = |rotulo: &str,
                        g2: &wgpu::BindGroupLayout,
                        entrada: &str,
                        constantes: &[(&str, f64)]| {
            pipeline_em(
                rotulo,
                &[Some(&cena_layout), Some(e.lampadas_layout), Some(g2)],
                entrada,
                constantes,
            )
        };

        let visibilidade = bordas.filter(|_| cfg.usa_visibilidade()).map(|b| {
            let g2 = layout(
                device,
                "luz2d c3 visibilidade",
                &[
                    entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rg32Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            );
            let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 visibilidade"),
                layout: &g2,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: e.uni_c3.as_entire_binding(),
                    },
                    vista(3, &e.vistas[op.visibilidade.expect("usa_visibilidade")]),
                ],
            });
            let constantes = [
                ("PASSO", fino(b.visibilidade_no_passo_fino)),
                ("RAIOS_DIRETOS", f64::from(b.raios)),
                ("BISSECOES", f64::from(b.bissecoes)),
            ];
            (
                pipeline("luz2d c3 visibilidade", &g2, "c3_visibilidade", &constantes),
                grupo,
            )
        });

        let c0 = PassList::indice_da_c0(cfg);
        let formato = e.lista.texturas[FLUENCIA].formato;
        let mut entradas = vec![
            entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
            entrada_textura(1, FLOAT),
            entrada_escrita(4, formato),
        ];
        let mut grupo = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: e.uni_c3.as_entire_binding(),
            },
            vista(1, &e.vistas[c0]),
            vista(4, &e.vistas[FLUENCIA]),
        ];
        if let (Some(_), Some(v)) = (&visibilidade, op.visibilidade) {
            entradas.push(entrada_textura(2, wgpu::TextureSampleType::Uint));
            grupo.push(vista(2, &e.vistas[v]));
        }
        // ⭐ W5a (F3): o preenchimento (sem ele, as sementes do campo ocupam o lugar: o
        // `PREENCHE = false` nunca as lê).
        entradas.push(entrada_textura(5, wgpu::TextureSampleType::Uint));
        grupo.push(vista(
            5,
            &e.vistas[op.preenche_final(cfg).unwrap_or(e.sementes_finais)],
        ));
        let g2 = layout(device, "luz2d c3 fina", &entradas);
        let grupo_fina = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c3 fina"),
            layout: &g2,
            entries: &grupo,
        });
        // ⭐ Os raios da luz direta marcham no passo da grade FINA: com o das cascatas as amostras
        // entram e saem do canto do oclusor conforme o texel fino anda, e a borda ondula (medido na
        // W4b, Low: 0,53 → 0,06 px no contorno de 50 %).
        let fina = match bordas.filter(|_| visibilidade.is_some()) {
            Some(b) => pipeline(
                "luz2d c3 fina",
                &g2,
                "c3_fina",
                &[
                    ("PASSO", fino(true)),
                    ("RAIOS_DIRETOS", f64::from(b.raios)),
                    ("BISSECOES", f64::from(b.bissecoes)),
                    ("JOELHO", f64::from(u8::from(b.joelho))),
                    ("CURVATURA", f64::from(u8::from(b.curvatura))),
                ],
            ),
            None => pipeline(
                "luz2d c3 fina",
                &g2,
                "c3_fina_exata",
                &[("PASSO", fino(true))],
            ),
        };

        // ⭐ W5a (F3): o JFA do complemento — grupo 3 (o 1 e o 2 ficam vazios).
        let preenchimento = cfg
            .grade_do_preenchimento()
            .zip(op.preenche)
            .zip(e.uni_preenche)
            .map(|(((celulas, saltos), p), uniforme)| {
                let g3 = layout(
                    device,
                    "luz2d c3 preenche",
                    &[
                        entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                        entrada_textura(1, wgpu::TextureSampleType::Uint),
                        entrada_escrita(2, wgpu::TextureFormat::R32Uint),
                    ],
                );
                let grupos = [Some(&cena_layout), None, None, Some(&g3)];
                let bytes = size_of::<UniformePreenche>() as u64;
                let grupo = |j: usize, entrada: usize, saida: usize| {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("luz2d c3 preenche"),
                        layout: &g3,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: fatia_uniforme(uniforme, j as u64, bytes),
                            },
                            vista(1, &e.vistas[entrada]),
                            vista(2, &e.vistas[saida]),
                        ],
                    })
                };
                // A semente escreve a 1.ª (e liga a 2.ª como entrada, que ela não lê); o salto j lê
                // j % 2 e escreve a outra.
                let mut gs = vec![grupo(0, p[1], p[0])];
                for j in 0..saltos.len() {
                    gs.push(grupo(j + 1, p[j % 2], p[(j + 1) % 2]));
                }
                Preenchimento {
                    semente: pipeline_em(
                        "luz2d c3 preenche semente",
                        &grupos,
                        "c3_preenche_semente",
                        &[],
                    ),
                    salto: pipeline_em(
                        "luz2d c3 preenche salto",
                        &grupos,
                        "c3_preenche_salto",
                        &[],
                    ),
                    grupos: gs,
                    uniforme: uniforme.clone(),
                    saltos,
                    u: UniformePreenche {
                        celulas,
                        inicio: [0; 2],
                        fim: celulas,
                        celula: cfg.preenchimento.unwrap_or(1),
                        salto: 0,
                        fina: cfg.fina,
                        _p: [0; 3],
                    },
                }
            });

        let gf = cfg.grade_da_fluencia();
        let mut c3 = Self {
            cena,
            visibilidade,
            preenchimento,
            fina,
            grupo_fina,
            uniforme: e.uni_c3.clone(),
            u: UniformeFina {
                tamanho: [gf.largura, gf.altura],
                inicio: [0; 2],
                fim: [0; 2],
                inicio_grosso: [0; 2],
                fim_grosso: [0; 2],
                fina: cfg.fina,
                limiar: bordas.map_or(0.0, |b| b.limiar),
            },
        };
        c3.regiao(cfg, [0; 2], [gf.largura, gf.altura]);
        c3
    }

    /// ⭐ Só a REGIÃO `[inicio, fim)` da grade fina se calcula (a da janela; o resto da grade
    /// existe para as cascatas verem o que está fora dela), e as sondas da c0 que a cobrem.
    pub(super) fn regiao(&mut self, cfg: &Config, inicio: [u32; 2], fim: [u32; 2]) {
        let u = &mut self.u;
        let grossa = [cfg.grade.largura, cfg.grade.altura];
        for k in 0..2 {
            u.fim[k] = fim[k].min(u.tamanho[k]);
            u.inicio[k] = inicio[k].min(u.fim[k]);
            // Duas sondas de folga: a da célula e a vizinha que a curvatura lê.
            u.inicio_grosso[k] = (u.inicio[k] / cfg.fina).saturating_sub(2);
            u.fim_grosso[k] = (u.fim[k].div_ceil(cfg.fina) + 2).min(grossa[k]);
        }
        // As células do preenchimento que cobrem a região (ou a grade inteira).
        if let Some(p) = &mut self.preenchimento {
            for k in 0..2 {
                if cfg.preenche_a_grade_inteira {
                    (p.u.inicio[k], p.u.fim[k]) = (0, p.u.celulas[k]);
                } else {
                    p.u.inicio[k] = u.inicio[k] / p.u.celula;
                    p.u.fim[k] = u.fim[k].div_ceil(p.u.celula).min(p.u.celulas[k]);
                }
            }
        }
    }

    /// Grava os dois passes (o da visibilidade fica vazio sem ela: os carimbos dele valem 0).
    pub(super) fn gravar(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        lampadas: &wgpu::BindGroup,
        tempos: Option<&Tempos>,
    ) {
        gpu.queue
            .write_buffer(&self.uniforme, 0, bytemuck::bytes_of(&self.u));
        let grupos =
            |a: [u32; 2], b: [u32; 2]| [(b[0] - a[0]).div_ceil(8), (b[1] - a[1]).div_ceil(8)];
        let grossa = grupos(self.u.inicio_grosso, self.u.fim_grosso);
        let fina = grupos(self.u.inicio, self.u.fim);
        {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C3 preenche"),
                timestamp_writes: tempos.map(|t| t.escrita_compute(tempos::C3_PREENCHE)),
            });
            if let Some(pr) = &self.preenchimento {
                // Os uniformes de cada passo: a região deste quadro e o salto.
                let mut uni = vec![0u8; PASSO_UNIFORME as usize * pr.grupos.len()];
                for (j, salto) in std::iter::once(0)
                    .chain(pr.saltos.iter().copied())
                    .enumerate()
                {
                    let u = UniformePreenche { salto, ..pr.u };
                    let o = j * PASSO_UNIFORME as usize;
                    uni[o..o + size_of::<UniformePreenche>()]
                        .copy_from_slice(bytemuck::bytes_of(&u));
                }
                gpu.queue.write_buffer(&pr.uniforme, 0, &uni);
                let celulas = grupos(pr.u.inicio, pr.u.fim);
                p.set_bind_group(0, &self.cena, &[]);
                for (j, g) in pr.grupos.iter().enumerate() {
                    p.set_pipeline(if j == 0 { &pr.semente } else { &pr.salto });
                    p.set_bind_group(3, g, &[]);
                    p.dispatch_workgroups(celulas[0], celulas[1], 1);
                }
            }
        }
        {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C3 visibilidade"),
                timestamp_writes: tempos.map(|t| t.escrita_compute(tempos::C3_VISIBILIDADE)),
            });
            if let Some((pipe, g2)) = &self.visibilidade {
                p.set_pipeline(pipe);
                p.set_bind_group(0, &self.cena, &[]);
                p.set_bind_group(1, lampadas, &[]);
                p.set_bind_group(2, g2, &[]);
                p.dispatch_workgroups(grossa[0], grossa[1], 1);
            }
        }
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d C3 fina"),
            timestamp_writes: tempos.map(|t| t.escrita_compute(tempos::C3_FINA)),
        });
        p.set_pipeline(&self.fina);
        p.set_bind_group(0, &self.cena, &[]);
        p.set_bind_group(1, lampadas, &[]);
        p.set_bind_group(2, &self.grupo_fina, &[]);
        p.dispatch_workgroups(fina[0], fina[1], 1);
    }
}
