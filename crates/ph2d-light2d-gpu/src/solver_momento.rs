//! ⭐ P14 «L» — o primeiro momento `M = [Mx, My]` na placa (`docs/PixelArt/P14_plano.md` §4; o WGSL em
//! `c3_momento.wgsl`). Só existe com receptor de normal à vista (`CenaDeLuz::com_normal`): sem ele nada
//! se aloca nem se despacha, e o quadro é o de sempre. Dois passes depois da C3: a c0 de `M` (nível 0
//! das cascatas, lendo o nível 1 que a c0 leu) e a fina de `M` (o `luz_do_texel` com `MOMENTO`).
//! Na holográfica a c0 de `M` nasce no laço dos quadrantes ([`MomentoHrc`], plano §11 «P14c-placa 5»).
//! Módulo-filho do [`super`] (o tecto de 700 LOC do `solver.rs`).

use super::*;
use crate::passlist::{FLUENCIA, Opcionais};
use crate::passlist_momento::{NOME_CONHECIDO_DO_MIOLO, NOME_UNIFORME_DO_MIOLO};

const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// O módulo das cascatas (C2, C3 e o momento): a fábrica devolve o MESMO de cada descrição.
pub(super) fn modulo_rc(device: &crate::Placa<'_>, cfg: &Config) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("luz2d c2_rc.wgsl"),
        source: wgpu::ShaderSource::Wgsl(
            cfg.modulo_rc
                .unwrap_or_else(|| {
                    shaders::rc(
                        cfg.aritmetica,
                        cfg.armazenamento,
                        cfg.hrc_niveis_em_textura(),
                    )
                })
                .into(),
        ),
    })
}

pub(super) struct Momento {
    /// A c0 de `M` pelas cascatas clássicas; `None` com a holográfica (a c0 de `M` nasce no [`MomentoHrc`]).
    c0: Option<(wgpu::ComputePipeline, wgpu::BindGroup)>,
    /// ⭐ P14c-placa 5 — a c0 de `M` no laço dos quadrantes da holográfica.
    pub(super) hrc: Option<MomentoHrc>,
    /// A fina sem e com os distantes.
    fina: [Option<wgpu::ComputePipeline>; 2],
    fina_grupo: wgpu::BindGroup,
    texturas: [wgpu::Texture; 4],
    vistas: [wgpu::TextureView; 4],
    /// A lista do solver com o momento ([`PassList::com_momento`]): as quatro texturas nascem dela.
    lista: PassList,
    /// O que a lista declara a mais que a do solver (texturas e, na holográfica, os buffers).
    bytes: u64,
    /// ⭐ P14c-placa 6 — o miolo de `M` (com o miolo de `F`).
    miolo: Option<MioloDoMomento>,
}

/// ⭐ P14c-placa 6 — `M` DENTRO dos opacos pela difusão do miolo (a lei do gabarito, que difunde `[F; M]`):
/// depois do miolo de `F`, a pirâmide dele (livre até o quadro seguinte) corre outra vez para `Mx` e para
/// `My`, com pipelines PRÓPRIAS (`miolo_classifica_momento`: o rgb da fina de `M`, o peso do conhecido de
/// `F`). O conhecido fino e o uniforme são próprios; sem o anel de fora da região (o uniforme de `F` tem-no):
/// com a região menor que a grade, as vizinhas de fora dela não contam (Neumann, a lei da W5h).
struct MioloDoMomento {
    xy: [super::miolo::DoMiolo; 2],
    conhecido: wgpu::Texture,
}

impl MioloDoMomento {
    fn novo(s: &Solver, gpu: &GpuContext, lista: &PassList, fina: [&wgpu::Texture; 2]) -> Option<Self> {
        let f = s.miolo.as_ref()?;
        let [i0, ..] = Opcionais::de(&s.cfg).miolo?;
        let decl = lista.texturas.iter().find(|t| t.nome == NOME_CONHECIDO_DO_MIOLO)?;
        let conhecido = textura(&gpu.device, decl);
        let u = lista.buffers.iter().find(|b| b.nome == NOME_UNIFORME_DO_MIOLO)?;
        let uniforme = super::criar_buffer(&gpu.device, u.nome, u.bytes, u.usos);
        let cfg = Config {
            anel_fora_da_regiao: false,
            ..s.cfg
        };
        let device = crate::Placa {
            device: &gpu.device,
            fabrica: &s.fabrica,
            conta: None,
        };
        let padrao = wgpu::TextureViewDescriptor::default();
        let k0_de_f = s.texturas[i0].create_view(&padrao);
        let um = |k: usize| {
            let mut texturas = s.texturas.clone();
            texturas[FLUENCIA] = fina[k].clone();
            texturas[i0] = conhecido.clone();
            let vistas: Vec<wgpu::TextureView> =
                texturas.iter().map(|t| t.create_view(&padrao)).collect();
            // A «transmitida» é a OUTRA fina de `M` (alfa 1 em todo texel): `M` não atravessa os translúcidos.
            let outra = fina[1 - k].create_view(&padrao);
            let mut m = super::miolo::DoMiolo::novo(&super::miolo::Entradas {
                placa: device,
                cfg: &cfg,
                texturas: &texturas,
                vistas: &vistas,
                uniforme: uniforme.clone(),
                ladrilhos: &s.ladrilhos.as_ref()?.buffer,
                momento: Some([&k0_de_f, &outra]),
            })?;
            let (inicio, fim) = f.regiao_de();
            m.regiao(inicio, fim);
            Some(m)
        };
        Some(Self {
            xy: [um(0)?, um(1)?],
            conhecido,
        })
    }
}

/// Uma textura do momento, como a lista a declara.
fn textura(device: &wgpu::Device, t: &Textura) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(t.nome),
        size: wgpu::Extent3d {
            width: t.tamanho[0],
            height: t.tamanho[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: t.mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: t.formato,
        usage: t.usos,
        view_formats: &[],
    })
}

fn vista(binding: u32, v: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    }
}

impl Momento {
    /// `None` no entorno `so_a_c0` (ninguém lê a fluência dele; com a borda do bug 6 a luz dele chega à vista
    /// pelos R, e o `M` da vista já a tem).
    pub(super) fn novo(s: &Solver, gpu: &GpuContext) -> Option<Self> {
        let cfg = &s.cfg;
        if cfg.so_a_c0 {
            return None;
        }
        let device = crate::Placa {
            device: &gpu.device,
            fabrica: &s.fabrica,
            conta: None,
        };
        let modulo = modulo_rc(&device, cfg);
        let lista = s.lista.com_momento(cfg);
        let base = s.lista.texturas.len();
        let texturas: [wgpu::Texture; 4] =
            std::array::from_fn(|k| textura(&gpu.device, &lista.texturas[base + k]));
        let vistas = texturas
            .each_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        // ── a c0 de M ──
        let g2_c0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d momento c0"),
            entries: &[entrada_escrita(34, FORMATO), entrada_escrita(35, FORMATO)],
        });
        let rc_layout = s.rc.get_bind_group_layout(0);
        let pipeline = |rotulo: &str,
                        grupos: &[Option<&wgpu::BindGroupLayout>],
                        entrada: &str,
                        constantes: &[(&str, f64)]| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(rotulo),
                bind_group_layouts: grupos,
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(rotulo),
                layout: Some(&pl),
                module: &modulo,
                entry_point: Some(entrada),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: constantes,
                    zero_initialize_workgroup_memory: true,
                },
                cache: None,
            })
        };
        let (c0, hrc) = match &s.hrc {
            Some(h) => (
                None,
                Some(MomentoHrc::novo(&device, gpu, h, &modulo, cfg, &vistas)),
            ),
            None => {
                let c0_grupo = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("luz2d momento c0"),
                    layout: &g2_c0,
                    entries: &[vista(34, &vistas[0]), vista(35, &vistas[1])],
                });
                let c0 = pipeline(
                    "luz2d momento c0",
                    &[Some(&rc_layout), Some(&s.lampadas_layout), Some(&g2_c0)],
                    "cascata_momento",
                    &c2::constantes(cfg),
                );
                (Some((c0, c0_grupo)), None)
            }
        };

        // ── a fina de M: as leituras da fina de hoje (0, 1, 2, 5) e o momento (30–33) ──
        let op = Opcionais::de(cfg);
        let sementes = SEMENTES[cfg.saltos_jfa().len() % 2];
        let visibilidade = op
            .visibilidade
            .filter(|_| c3::constantes::com_visibilidade(cfg))
            .unwrap_or(sementes);
        let preenche = op.preenche_final(cfg).unwrap_or(sementes);
        let ler = |b: usize| s.texturas[b].create_view(&wgpu::TextureViewDescriptor::default());
        let (c0v, visv, prev) = (
            ler(PassList::indice_da_c0(cfg)),
            ler(visibilidade),
            ler(preenche),
        );
        let g2_fina = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d momento fina"),
            entries: &[
                entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                entrada_textura(1, FLOAT),
                entrada_textura(2, wgpu::TextureSampleType::Uint),
                entrada_textura(5, wgpu::TextureSampleType::Uint),
                entrada_textura(30, FLOAT),
                entrada_textura(31, FLOAT),
                entrada_escrita(32, FORMATO),
                entrada_escrita(33, FORMATO),
            ],
        });
        let fina_grupo = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d momento fina"),
            layout: &g2_fina,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: s.c3.uniforme.as_entire_binding(),
                },
                vista(1, &c0v),
                vista(2, &visv),
                vista(5, &prev),
                vista(30, &vistas[0]),
                vista(31, &vistas[1]),
                vista(32, &vistas[2]),
                vista(33, &vistas[3]),
            ],
        });
        let (entrada, proprias) = c3::constantes::da_fina(cfg);
        let exata = f64::from(u8::from(entrada == "c3_fina_exata"));
        let grupos = [
            Some(&s.c3.cena_layout),
            Some(&s.lampadas_layout),
            Some(&g2_fina),
        ];
        let fina_com = |distantes: f64| {
            let c: Vec<(&str, f64)> = proprias
                .iter()
                .copied()
                .chain(
                    c3::constantes::comuns(cfg)
                        .into_iter()
                        .filter(|(k, _)| proprias.iter().all(|(o, _)| o != k)),
                )
                .chain([
                    ("DISTANTES", distantes),
                    ("MOMENTO", 1.0),
                    ("MOMENTO_EXATA", exata),
                ])
                .collect();
            pipeline("luz2d momento fina", &grupos, "c3_momento", &c)
        };
        let fina_p = [Some(fina_com(0.0)), cfg.distantes.then(|| fina_com(1.0))];
        let miolo = MioloDoMomento::novo(s, gpu, &lista, [&texturas[2], &texturas[3]]);
        Some(Self {
            miolo,
            bytes: lista.bytes_alocados() - s.lista.bytes_alocados(),
            c0,
            hrc,
            fina: fina_p,
            fina_grupo,
            texturas,
            vistas,
            lista,
        })
    }

    /// Os dois passes, depois da C3 (a c0 de `M` lê o nível 1 das cascatas, intacto até o quadro seguinte).
    pub(super) fn gravar(&self, s: &Solver, enc: &mut wgpu::CommandEncoder, distantes: bool) {
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d momento"),
            timestamp_writes: None,
        });
        let [w, h] = [s.cfg.grade.largura, s.cfg.grade.altura];
        p.set_bind_group(1, &s.lampadas_grupo, &[]);
        if let Some((c0, grupo)) = &self.c0 {
            p.set_pipeline(c0);
            p.set_bind_group(0, &s.rc_grupos[0], &[]);
            p.set_bind_group(2, grupo, &[]);
            p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
        let fina = if distantes { &self.fina[1] } else { &None };
        let fina = fina.as_ref().or(self.fina[0].as_ref()).expect("a fina");
        // Só a região da fina de `F` (o `c3_momento` desloca por `fina.inicio`, como a `c3_fina`).
        let [gx, gy] = s.c3.grupos_da_fina();
        p.set_pipeline(fina);
        p.set_bind_group(0, &s.c3.cena, &[]);
        p.set_bind_group(2, &self.fina_grupo, &[]);
        p.dispatch_workgroups(gx, gy, 1);
    }

    /// ⭐ P14c-placa 6 — o miolo de `M`, depois do de `F` (a pirâmide dele está livre).
    pub(super) fn gravar_miolo(&self, gpu: &GpuContext, enc: &mut wgpu::CommandEncoder, texel_m: f64) {
        for m in self.miolo.iter().flat_map(|m| &m.xy) {
            m.gravar(gpu, enc, texel_m, None);
        }
    }

    pub(super) fn regiao(&mut self, inicio: [u32; 2], fim: [u32; 2]) {
        for m in self.miolo.iter_mut().flat_map(|m| &mut m.xy) {
            m.regiao(inicio, fim);
        }
    }

    /// `[Mx, My]` na grade da fluência.
    pub(super) fn vistas(&self) -> [&wgpu::TextureView; 2] {
        [&self.vistas[2], &self.vistas[3]]
    }

    /// `[Mx, My]` (as texturas, para quem as lê de volta).
    pub(super) fn texturas(&self) -> [&wgpu::Texture; 2] {
        [&self.texturas[2], &self.texturas[3]]
    }

    /// O que a lista do momento declara a mais (as quatro texturas e, na holográfica, o acumulado e a grade).
    pub(super) fn bytes(&self) -> u64 {
        self.bytes
    }

    /// As texturas do momento com o rótulo de cada uma (as da lista que o solver não guarda).
    pub(super) fn recursos(&self) -> impl Iterator<Item = (&'static str, &wgpu::Texture)> {
        let proprias = self.texturas.len() + usize::from(self.miolo.is_some());
        let base = self.lista.texturas.len() - proprias;
        self.lista.texturas[base..]
            .iter()
            .map(|t| t.nome)
            .zip(self.texturas.iter().chain(self.miolo.as_ref().map(|m| &m.conhecido)))
    }
}

impl Solver {
    /// A lista declarada: a de [`PassList::de`], e com o momento ([`PassList::com_momento`]) enquanto a
    /// cena tem normal à vista.
    #[must_use]
    pub fn pass_list(&self) -> &PassList {
        self.momento.as_ref().map_or(&self.lista, |m| &m.lista)
    }

    /// ⭐ P14 «L» — `[Mx, My]` do último quadro, na grade da [`Self::fluencia`]; `None` sem normal.
    #[must_use]
    pub fn momento(&self) -> Option<[&wgpu::TextureView; 2]> {
        self.momento.as_ref().map(Momento::vistas)
    }

    /// [`Self::momento`] como texturas (para lê-las de volta).
    #[must_use]
    pub fn momento_texturas(&self) -> Option<[&wgpu::Texture; 2]> {
        self.momento.as_ref().map(Momento::texturas)
    }

    /// Os bytes do momento residente (0 sem normal à vista).
    #[must_use]
    pub fn bytes_do_momento(&self) -> u64 {
        self.momento.as_ref().map_or(0, Momento::bytes)
    }

    /// Liga o momento quando a cena tem normal à vista e solta-o quando não tem.
    pub(super) fn acertar_o_momento(&mut self, gpu: &GpuContext, com_normal: bool) {
        if !com_normal {
            self.momento = None;
        } else if self.momento.is_none() {
            self.momento = Momento::novo(self, gpu);
        }
    }
}

/// ⭐ P14c-placa 5 — `M` na holográfica (`hrc_momento.wgsl`): a `hrc_soma_momento` logo depois de cada
/// `hrc_soma` (os R do quadrante ainda lá) e a `hrc_saida_momento` depois da `hrc_saida`. Pipelines PRÓPRIAS
/// com o grupo 1 (as da holográfica não o têm: nenhuma ligação nova nelas); os grupos 0, 2 e 3 são os dela.
pub(super) struct MomentoHrc {
    soma: (wgpu::ComputePipeline, wgpu::BindGroup),
    saida: (wgpu::ComputePipeline, wgpu::BindGroup),
    _acumulado: wgpu::Buffer,
    _grade: wgpu::Buffer,
}

impl MomentoHrc {
    fn novo(
        device: &crate::Placa<'_>,
        gpu: &GpuContext,
        h: &holo::Holografica,
        modulo: &wgpu::ShaderModule,
        cfg: &Config,
        vistas: &[wgpu::TextureView; 4],
    ) -> Self {
        let acumulado_entrada = || wgpu::BindGroupLayoutEntry {
            binding: 40,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let g1_soma = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d momento hrc soma"),
            entries: &[acumulado_entrada()],
        });
        let g1_saida = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d momento hrc saída"),
            entries: &[
                acumulado_entrada(),
                entrada_escrita(44, FORMATO),
                entrada_escrita(45, FORMATO),
                entrada_uniforme(46, wgpu::ShaderStages::COMPUTE),
            ],
        });
        let base = &h.pipelines[0];
        let [g0, g2, g3] = [0, 2, 3].map(|i| base.get_bind_group_layout(i));
        let pipeline = |entrada: &str, grupos: &[Option<&wgpu::BindGroupLayout>]| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(entrada),
                bind_group_layouts: grupos,
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entrada),
                layout: Some(&pl),
                module: modulo,
                entry_point: Some(entrada),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &h.constantes,
                    zero_initialize_workgroup_memory: true,
                },
                cache: None,
            })
        };
        let acumulado = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(crate::passlist_momento::NOME_ACUMULADO_DO_MOMENTO),
            size: 2 * 16 * u64::from(cfg.grade.largura) * u64::from(cfg.grade.altura),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let grade = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(crate::passlist_momento::NOME_GRADE_DO_MOMENTO),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(
            &grade,
            0,
            bytemuck::bytes_of(&[cfg.grade.largura, cfg.grade.altura, 0, 0]),
        );
        let acumulado_ligado = || wgpu::BindGroupEntry {
            binding: 40,
            resource: acumulado.as_entire_binding(),
        };
        let grupo_soma = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d momento hrc soma"),
            layout: &g1_soma,
            entries: &[acumulado_ligado()],
        });
        let grupo_saida = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d momento hrc saída"),
            layout: &g1_saida,
            entries: &[
                acumulado_ligado(),
                vista(44, &vistas[0]),
                vista(45, &vistas[1]),
                wgpu::BindGroupEntry {
                    binding: 46,
                    resource: grade.as_entire_binding(),
                },
            ],
        });
        Self {
            soma: (
                pipeline(
                    "hrc_soma_momento",
                    &[Some(&g0), Some(&g1_soma), Some(&g2), Some(&g3)],
                ),
                grupo_soma,
            ),
            saida: (
                pipeline("hrc_saida_momento", &[Some(&g0), Some(&g1_saida)]),
                grupo_saida,
            ),
            _acumulado: acumulado,
            _grade: grade,
        }
    }

    /// Depois do passo `etapa` da holográfica (com os grupos 0, 2 e 3 dele ligados): o do momento, se houver.
    pub(super) fn depois(&self, p: &mut wgpu::ComputePass<'_>, etapa: usize, [x, y, z]: [u32; 3]) {
        let (pipeline, grupo) = match etapa {
            3 => &self.soma,
            4 => &self.saida,
            _ => return,
        };
        p.set_pipeline(pipeline);
        p.set_bind_group(1, grupo, &[]);
        p.dispatch_workgroups(x, y, z);
    }
}

/// [`MomentoHrc::depois`] quando há momento.
pub(super) fn depois(
    m: Option<&MomentoHrc>,
    p: &mut wgpu::ComputePass<'_>,
    etapa: usize,
    grupos: [u32; 3],
) {
    if let Some(m) = m {
        m.depois(p, etapa, grupos);
    }
}
