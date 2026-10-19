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

/// A entrada sem e com os distantes (ficheiro irmão pelo tecto de 700 LOC).
#[path = "solver_c3_par.rs"]
mod par;
use par::Par;
/// ⭐ W5d: a borda exata da captura e os cantos (ficheiro irmão pelo tecto de 700 LOC).
#[path = "solver_c3_borda.rs"]
mod borda;
/// ⭐ W5p: as constantes `override` da C3 (a montagem e a régua do compilador).
#[path = "solver_c3_constantes.rs"]
pub(crate) mod constantes;

/// O que os passes C3 pegam do solver ao nascer.
pub(super) struct Entradas<'a> {
    /// O dispositivo com a fábrica das pipelines (a grade nova não as recompila).
    pub placa: crate::Placa<'a>,
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
    /// ⭐ W5q: os ladrilhos do anel (com o miolo).
    pub ladrilhos: Option<&'a wgpu::Buffer>,
}

pub(super) struct C3 {
    /// O grupo 0: a cena capturada e o uniforme da c0 (a marcha da c0, sem as saídas das cascatas).
    cena: wgpu::BindGroup,
    visibilidade: Option<(Par, wgpu::BindGroup)>,
    preenchimento: Option<Preenchimento>,
    fina: Option<Par>,
    grupo_fina: wgpu::BindGroup,
    /// ⭐ W5n: o anel fora da região (o miolo no corpo inteiro, `c3_anel.wgsl`).
    anel: Option<(Par, wgpu::BindGroup)>,
    /// ⭐ W5c: a FONTE do rebote (a luz que cada superfície recebe, para o quadro seguinte) — os
    /// grupos 2 (a c0) e 3 (a saída).
    rebote: Option<(Par, [wgpu::BindGroup; 2])>,
    /// ⭐ W5d: a borda exata da captura (a pipeline, o grupo 3 e o tamanho dela).
    borda: Option<borda::Borda>,
    /// A grade das cascatas (a fonte do rebote cobre-a inteira).
    tamanho_grosso: [u32; 2],
    uniforme: wgpu::Buffer,
    /// O uniforme deste quadro: a grade fina inteira por omissão, a região da janela com
    /// [`Self::regiao`].
    u: UniformeFina,
}

fn layout(
    device: crate::Placa<'_>,
    rotulo: &str,
    entradas: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(rotulo),
        entries: entradas,
    })
}

/// Uma textura de 1 × 1 para uma ligação que o shader cita e este solver não alimenta.
fn textura_vazia(
    device: &wgpu::Device,
    rotulo: &str,
    formato: wgpu::TextureFormat,
    uso: wgpu::TextureUsages,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(rotulo),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: formato,
            usage: uso,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn vista(binding: u32, v: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    }
}

impl C3 {
    pub(super) fn novo(e: &Entradas<'_>) -> Self {
        let device = e.placa;
        let cfg = e.cfg;
        let op = Opcionais::de(cfg);
        // A marcha da luz direta lê a captura FINA dos opacos quando há (W5a).
        let [cob, tau] = op.captura.unwrap_or([COBERTURA, TAU_DENTRO]);
        // ⭐ W5d: com a borda exata, a C3 lê a saída do `c3_borda` (a cobertura no vermelho).
        let cob_c3 = op.borda.unwrap_or(cob);
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
                entrada_textura(8, FLOAT),
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
                vista(5, &e.vistas[cob_c3]),
                vista(6, &e.vistas[tau]),
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(e.vizinhos),
                },
                // ⭐ W6f: as cunhas espalhadas (sem elas, a cobertura: o `meio` não as lê).
                vista(8, &e.vistas[op.cantos.map_or(cob_c3, |c| c[1])]),
            ],
        });
        let fino = |passo_fino: bool| constantes::passo(cfg, passo_fino);
        let comuns = constantes::comuns(cfg);
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
            let em = |d: f64| {
                let c: Vec<(&str, f64)> = constantes
                    .iter()
                    .copied()
                    .chain([("DISTANTES", d)])
                    .collect();
                pipeline_em(
                    rotulo,
                    &[Some(&cena_layout), Some(e.lampadas_layout), Some(g2)],
                    entrada,
                    &c,
                )
            };
            Par::em_paralelo(cfg.distantes, em)
        };

        let visibilidade = bordas
            .filter(|_| cfg.usa_visibilidade() && !cfg.so_a_c0)
            .map(|b| {
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
        // ⭐ W5p: a luz de um texel tem UM laço de lâmpadas para a fina e a exata (`luz_das_lampadas`),
        // que cita as visibilidades: sem o passe delas, as SEMENTES (inteiras, nunca se leem aqui) — uma
        // textura nova a mais é uma alocação que a `PassList` não declara (o G5).
        entradas.push(entrada_textura(2, wgpu::TextureSampleType::Uint));
        grupo.push(vista(
            2,
            &e.vistas[op
                .visibilidade
                .filter(|_| visibilidade.is_some())
                .unwrap_or(e.sementes_finais)],
        ));
        // ⭐ W5a (F3): o preenchimento (sem ele, as sementes do campo ocupam o lugar: o
        // `PREENCHE = false` nunca as lê).
        entradas.push(entrada_textura(5, wgpu::TextureSampleType::Uint));
        grupo.push(vista(
            5,
            &e.vistas[op.preenche_final(cfg).unwrap_or(e.sementes_finais)],
        ));
        // ⭐ W5k: a transmitida dos translúcidos para o miolo; sem ele a fina mistura-a no sítio e a
        // ligação recebe uma textura de 1 × 1 que nunca se escreve.
        entradas.push(entrada_escrita(6, formato));
        let transmitida = Opcionais::de(cfg).miolo.map(|m| m[4]);
        let vazia = transmitida.is_none().then(|| {
            textura_vazia(
                device.device,
                "luz2d/tex/transmitida_vazia",
                formato,
                wgpu::TextureUsages::STORAGE_BINDING,
            )
        });
        grupo.push(vista(
            6,
            transmitida.map_or_else(|| vazia.as_ref().expect("a vazia"), |t| &e.vistas[t]),
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
        debug_assert_eq!(visibilidade.is_some(), constantes::com_visibilidade(cfg));
        let (entrada_fina, constantes_fina) = constantes::da_fina(cfg);
        // ⭐ W5o: a fina e o anel compilam ao mesmo tempo; o entorno (`so_a_c0`) não os tem.
        let grupos_anel = op
            .miolo
            .zip(e.ladrilhos)
            .filter(|_| cfg.anel_fora_da_regiao)
            .map(|(m, l)| {
                super::miolo::grupo_do_anel(device, &entradas, &grupo, e.vistas, m[0], l)
            });
        let entrada_anel = format!("{entrada_fina}_anel");
        // ⭐ W5p: a fina e o anel (os únicos que levam segundos a frio no compilador do driver) compilam
        // nos seus fios enquanto este monta o resto da C3 (a abertura em série: +0,3 s).
        let (fina, anel, preenchimento, rebote, borda) = std::thread::scope(|s| {
            let anel = grupos_anel.as_ref().map(|(l, _)| {
                s.spawn(|| pipeline("luz2d c3 anel", l, &entrada_anel, &constantes_fina))
            });
            let fina = (!cfg.so_a_c0).then(|| {
                s.spawn(|| pipeline("luz2d c3 fina", &g2, entrada_fina, &constantes_fina))
            });
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

            // ⭐ W5c: a fonte do rebote, na grade das cascatas inteira (a marcha e o passo da luz direta).
            let rebote = op.rebote.map(|[_, fonte]| {
                let g2 = layout(device, "luz2d c3 rebote c0", &[entrada_textura(1, FLOAT)]);
                let g3 = layout(
                    device,
                    "luz2d c3 rebote saida",
                    &[entrada_escrita(3, wgpu::TextureFormat::Rgba16Float)],
                );
                let grupo = |l: &wgpu::BindGroupLayout, b: u32, v: usize| {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("luz2d c3 rebote"),
                        layout: l,
                        entries: &[vista(b, &e.vistas[v])],
                    })
                };
                let grupos = [grupo(&g2, 1, c0), grupo(&g3, 3, fonte)];
                let mut constantes = vec![("PASSO", fino(true))];
                if let Some(b) = bordas {
                    constantes.push(("RAIOS_DIRETOS", f64::from(b.raios)));
                    constantes.push(("BISSECOES", f64::from(b.bissecoes)));
                }
                let em = |d: f64| {
                    let c: Vec<(&str, f64)> = constantes
                        .iter()
                        .copied()
                        .chain([("DISTANTES", d)])
                        .collect();
                    pipeline_em(
                        "luz2d c3 rebote",
                        &[
                            Some(&cena_layout),
                            Some(e.lampadas_layout),
                            Some(&g2),
                            Some(&g3),
                        ],
                        "c3_rebote",
                        &c,
                    )
                };
                (Par::em_paralelo(cfg.distantes, em), grupos)
            });

            // ⭐ W5d: a borda exata (e, W6f, os cantos) — `solver_c3_borda.rs`.
            let borda = op.borda.map(|b| {
                borda::Borda::nova(e, b, cob, (op.cantos, op.vertices_vazia), &pipeline_em)
            });
            (
                fina.map(|h| h.join().expect("a compilação da fina")),
                anel.map(|h| h.join().expect("a compilação do anel")),
                preenchimento,
                rebote,
                borda,
            )
        });
        let anel = anel.zip(grupos_anel.map(|(_, g)| g));

        let gf = cfg.grade_da_fluencia();
        let mut c3 = Self {
            cena,
            visibilidade,
            preenchimento,
            fina,
            grupo_fina,
            anel,
            rebote,
            borda,
            tamanho_grosso: [cfg.grade.largura, cfg.grade.altura],
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

    /// Grava os dois passes (o da visibilidade fica vazio sem ela: os carimbos dele valem 0) —
    /// `distantes`: alguma lâmpada tem objetos fora da grade neste quadro.
    /// ⭐ W5d — só a FONTE do rebote (o 1.º de dois rebotes no quadro), com a borda exata antes; sem
    /// carimbos.
    pub(super) fn gravar_so_a_fonte(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        (lampadas, distantes): (&wgpu::BindGroup, bool),
    ) {
        let Some((pipe, [g2, g3])) = &self.rebote else {
            return;
        };
        gpu.queue
            .write_buffer(&self.uniforme, 0, bytemuck::bytes_of(&self.u));
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d C3 rebote (1.º)"),
            timestamp_writes: None,
        });
        if let Some(b) = &self.borda {
            b.gravar(&mut p);
        }
        p.set_pipeline(pipe.de(distantes));
        p.set_bind_group(0, &self.cena, &[]);
        p.set_bind_group(1, lampadas, &[]);
        p.set_bind_group(2, g2, &[]);
        p.set_bind_group(3, g3, &[]);
        let [w, h] = self.tamanho_grosso;
        p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
    }

    pub(super) fn gravar(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        (lampadas, distantes, reflete): (&wgpu::BindGroup, bool, bool),
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
            // ⭐ W5d: a borda exata antes de tudo o que a lê (o carimbo dela conta no preenche).
            if let Some(b) = &self.borda {
                b.gravar(&mut p);
            }
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
                p.set_pipeline(pipe.de(distantes));
                p.set_bind_group(0, &self.cena, &[]);
                p.set_bind_group(1, lampadas, &[]);
                p.set_bind_group(2, g2, &[]);
                p.dispatch_workgroups(grossa[0], grossa[1], 1);
            }
        }
        {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C3 rebote"),
                timestamp_writes: tempos.map(|t| t.escrita_compute(tempos::C3_REBOTE)),
            });
            if let Some((pipe, [g2, g3])) = self.rebote.as_ref().filter(|_| reflete) {
                p.set_pipeline(pipe.de(distantes));
                p.set_bind_group(0, &self.cena, &[]);
                p.set_bind_group(1, lampadas, &[]);
                p.set_bind_group(2, g2, &[]);
                p.set_bind_group(3, g3, &[]);
                let [w, h] = self.tamanho_grosso;
                p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
            }
        }
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d C3 fina"),
            timestamp_writes: tempos.map(|t| t.escrita_compute(tempos::C3_FINA)),
        });
        // O entorno (`so_a_c0`) não tem a fina.
        let Some(par) = &self.fina else { return };
        p.set_pipeline(par.de(distantes));
        p.set_bind_group(0, &self.cena, &[]);
        p.set_bind_group(1, lampadas, &[]);
        p.set_bind_group(2, &self.grupo_fina, &[]);
        p.dispatch_workgroups(fina[0], fina[1], 1);
        // ⭐ W5n: com a região menor que a grade, o anel e o conhecido de fora dela (na grade inteira).
        if let Some((pipe, g2)) = self.anel.as_ref().filter(|_| self.regiao_parcial()) {
            let [w, h] = self.u.tamanho;
            p.set_pipeline(pipe.de(distantes));
            p.set_bind_group(2, g2, &[]);
            p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
    }

    /// A região deste quadro é menor que a grade fina.
    pub(super) fn regiao_parcial(&self) -> bool {
        self.u.inicio != [0; 2] || self.u.fim != self.u.tamanho
    }
}
