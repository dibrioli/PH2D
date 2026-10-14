//! ⭐ W5h — **o miolo dos opacos pela DIFUSÃO** na placa (`c3_miolo.wgsl`): depois da `c3_fina`, a
//! pirâmide (classifica → puxa até ao nível mais grosso → relaxa e desce até à fina), num passe de
//! cômputo com o carimbo `C3_MIOLO`. Módulo-filho do [`super`].

use super::*;
use crate::config::Miolo;
use crate::passlist::{FATIAS_DA_BASE, FLUENCIA, Opcionais};

/// O espelho de `Nivel` do `c3_miolo.wgsl` (48 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeNivel {
    inicio: [u32; 2],
    fim: [u32; 2],
    outro_inicio: [u32; 2],
    outro_fim: [u32; 2],
    kh2: f32,
    suavidade: f32,
    /// ⭐ W5n: 1 = a região é menor que a grade (o `miolo_classifica` deixa a moldura ao anel).
    moldura: f32,
    /// ⭐ W5q: 1 = o `puxa` do nível 1 lê um ladrilho apagado como fora ([`crate::ladrilhos`]).
    ladrilhos: f32,
}

/// ⭐ W5o — o instrumento das fases (ficheiro irmão pelo tecto de 700 LOC).
#[path = "solver_miolo_fases.rs"]
mod fases;
use fases::Fases;

/// ⭐ W5o — o espelho de `NivelBase` do `c3_miolo_base.wgsl` (48 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeNivelBase {
    inicio: [u32; 2],
    fim: [u32; 2],
    tamanho: [u32; 2],
    kh2: f32,
    pares: u32,
    deslocamento: u32,
    _p: [u32; 3],
}

/// ⭐ W5o — os níveis que a `Base` leva (índice = o nível).
const NIVEIS_DA_BASE: usize = 17;

/// ⭐ W5o — o espelho de `Base` do `c3_miolo_base.wgsl` (832 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeBase {
    b: u32,
    l: u32,
    suaviza: u32,
    _p: u32,
    niveis: [UniformeNivelBase; NIVEIS_DA_BASE],
}

/// ⭐ W5o — a memória de grupo por texel da base: 13 `f32` (o conhecido, os dois lados, o defeito).
const BYTES_POR_TEXEL_DA_BASE: u32 = 52;

/// Uma caixa `[inicio, fim)` num nível.
type Caixa = ([u32; 2], [u32; 2]);

#[derive(Clone, Copy)]
enum Tipo {
    Classifica,
    Puxa,
    Inicia,
    Relaxa,
    Desce,
    Final,
    // ⭐ W5n: o V-ciclo de correção.
    Defeito,
    Restringe,
    ErroInicia,
    ErroRelaxa,
    ErroSobe,
    Soma,
    // ⭐ W5o: a base num despacho (`c3_miolo_base.wgsl`).
    Base,
    ErroBase,
}

/// As entradas do `c3_miolo.wgsl`, na ordem de [`Tipo`].
const ENTRADAS: [&str; 12] = [
    "miolo_classifica",
    "miolo_puxa",
    "miolo_inicia",
    "miolo_relaxa",
    "miolo_desce",
    "miolo_final",
    "miolo_defeito",
    "miolo_restringe",
    "miolo_erro_inicia",
    "miolo_erro_relaxa",
    "miolo_erro_sobe",
    "miolo_soma",
];

struct Passo {
    tipo: Tipo,
    nivel: u32,
    /// O índice em [`FASES`].
    fase: usize,
    grupo: wgpu::BindGroup,
}

pub(super) struct DoMiolo {
    pipelines: Vec<wgpu::ComputePipeline>,
    passos: Vec<Passo>,
    uniforme: wgpu::Buffer,
    niveis: u32,
    miolo: Miolo,
    /// ⭐ W5n: o [`Config::anel_fora_da_regiao`].
    anel: bool,
    /// ⭐ W5o: o nível `b` de onde a base resolve num despacho (`None` = um despacho por relaxação).
    base: Option<u32>,
    /// Os pares de relaxações da cascata por nível (índice = o nível).
    pares: Vec<u32>,
    /// O tamanho da grade fina e a região dela neste quadro.
    tamanho: [u32; 2],
    regiao: Caixa,
    fases: Option<Fases>,
}

pub(super) struct Entradas<'a> {
    pub placa: crate::Placa<'a>,
    pub cfg: &'a Config,
    pub texturas: &'a [wgpu::Texture],
    pub vistas: &'a [wgpu::TextureView],
    pub uniforme: wgpu::Buffer,
    pub ladrilhos: &'a wgpu::Buffer,
}

/// O buffer dos ladrilhos, só leitura (o miolo e o anel).
fn de_leitura(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn mip(t: &wgpu::Texture, nivel: u32) -> wgpu::TextureView {
    t.create_view(&wgpu::TextureViewDescriptor {
        base_mip_level: nivel,
        mip_level_count: Some(1),
        ..Default::default()
    })
}

impl DoMiolo {
    pub(super) fn novo(e: &Entradas<'_>) -> Option<Self> {
        let cfg = e.cfg;
        let miolo = cfg.miolo?;
        let niveis = cfg.niveis_do_miolo()?;
        let [i0, ik, ia, ib, it, id] = Opcionais::de(cfg).miolo?;
        let device = e.placa;
        // Um módulo e um layout por formato da `saida` (o WGSL fixa-o na declaração).
        let por_formato = |formato: wgpu::TextureFormat| {
            let nome = match formato {
                wgpu::TextureFormat::Rgba32Float => "rgba32float",
                _ => "rgba16float",
            };
            let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("luz2d c3 miolo"),
                source: wgpu::ShaderSource::Wgsl(shaders::C3_MIOLO.replace("FORMATO", nome).into()),
            });
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("luz2d c3 miolo"),
                entries: &[
                    entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                    entrada_textura(1, FLOAT),
                    entrada_textura(2, FLOAT),
                    entrada_textura(3, FLOAT),
                    entrada_escrita(4, formato),
                    entrada_textura(5, FLOAT),
                    de_leitura(6),
                ],
            });
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("luz2d c3 miolo"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            (modulo, layout, pl)
        };
        let fluencia = e.texturas[FLUENCIA].format();
        let formatos = [
            fluencia,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Rgba32Float,
        ];
        let mut feitos: Vec<(wgpu::TextureFormat, _)> = Vec::new();
        for f in formatos {
            if !feitos.iter().any(|(g, _)| *g == f) {
                feitos.push((f, por_formato(f)));
            }
        }
        let de = |f: wgpu::TextureFormat| &feitos.iter().find(|(g, _)| *g == f).expect("formato").1;
        let pipeline = |entrada: &str, f: wgpu::TextureFormat| {
            let (modulo, _, pl) = de(f);
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entrada),
                layout: Some(pl),
                module: modulo,
                entry_point: Some(entrada),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let (f16, f32) = (
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Rgba32Float,
        );
        let saida_de = |t: Tipo| match t {
            Tipo::Classifica | Tipo::Final => fluencia,
            Tipo::Puxa => f16,
            _ => f32,
        };
        let mut pipelines: Vec<wgpu::ComputePipeline> = (0..ENTRADAS.len())
            .map(|i| {
                let formato = match i {
                    0 | 5 => fluencia,
                    1 => f16,
                    _ => f32,
                };
                pipeline(ENTRADAS[i], formato)
            })
            .collect();
        // ⭐ W5o: a base — o nível mais fino `b ≥ 2` de onde os níveis até ao topo cabem na memória de
        // grupo do dispositivo.
        let gf = cfg.grade_da_fluencia();
        let tamanho_em =
            |l: u32| [gf.largura, gf.altura].map(|x| ((x >> 1).max(1) >> (l - 1)).max(1));
        let cap = device.limits().max_compute_workgroup_storage_size / BYTES_POR_TEXEL_DA_BASE;
        let base = (miolo.base_no_grupo && niveis >= 2 && (niveis as usize) < NIVEIS_DA_BASE)
            .then(|| {
                (2..=niveis).find(|&b| {
                    (b..=niveis)
                        .map(|l| tamanho_em(l)[0] * tamanho_em(l)[1])
                        .sum::<u32>()
                        <= cap
                })
            })
            .flatten();
        // As pirâmides começam no nível 1: o mip `l − 1` é o nível `l`.
        let piramide = |t: usize| -> Vec<wgpu::TextureView> {
            (1..=niveis).map(|l| mip(&e.texturas[t], l - 1)).collect()
        };
        let (kv, av, bv, dv) = (piramide(ik), piramide(ia), piramide(ib), piramide(id));
        let k0 = &e.vistas[i0];
        let fl = &e.vistas[FLUENCIA];
        let transmitida = &e.vistas[it];
        // O conhecido do nível `l` (0 = o da fina).
        let k = |l: u32| if l == 0 { k0 } else { &kv[l as usize - 1] };
        let a = |l: u32| &av[l as usize - 1];
        let b = |l: u32| &bv[l as usize - 1];
        let d = |l: u32| &dv[l as usize - 1];
        let tam = size_of::<UniformeNivel>() as u64;
        let base_grupo = base.map(|b| {
            let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("luz2d c3 miolo base"),
                source: wgpu::ShaderSource::Wgsl(
                    shaders::C3_MIOLO_BASE
                        .replace("CAP", &cap.to_string())
                        .into(),
                ),
            });
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("luz2d c3 miolo base"),
                entries: &[
                    entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                    entrada_textura(1, FLOAT),
                    entrada_textura(2, FLOAT),
                    entrada_escrita(3, f32),
                ],
            });
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("luz2d c3 miolo base"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            for entrada in ["miolo_base", "miolo_erro_base"] {
                pipelines.push(
                    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some(entrada),
                        layout: Some(&pl),
                        module: &modulo,
                        entry_point: Some(entrada),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    }),
                );
            }
            let vista = |binding, v| wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(v),
            };
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 miolo base"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: fatia_uniforme(
                            &e.uniforme,
                            2 * u64::from(niveis + 1),
                            size_of::<UniformeBase>() as u64,
                        ),
                    },
                    vista(1, &e.vistas[ik]),
                    vista(2, &e.vistas[id]),
                    vista(3, a(b)),
                ],
            })
        });
        // As fatias: `2l` lê a região do nível de BAIXO no par «outro» (puxa); `2l + 1`, a de cima.
        // O binding 5 é a transmitida — ou, na `miolo_soma`, a solução u do nível 1.
        let grupo = |tipo: Tipo, fatia: u64, v: [&wgpu::TextureView; 4]| {
            let quinto = if matches!(tipo, Tipo::Soma) {
                a(1)
            } else {
                transmitida
            };
            let (_, layout, _) = de(saida_de(tipo));
            let entrada = |binding, vista| wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(vista),
            };
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 miolo"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: fatia_uniforme(&e.uniforme, fatia, tam),
                    },
                    entrada(1, v[0]),
                    entrada(2, v[1]),
                    entrada(3, v[2]),
                    entrada(4, v[3]),
                    entrada(5, quinto),
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: e.ladrilhos.as_entire_binding(),
                    },
                ],
            })
        };
        let resto = |l: u32| 2 * u64::from(l) + 1;
        let mut passos = Vec::new();
        let mut por = |tipo: Tipo, nivel: u32, fase: usize, v: [&wgpu::TextureView; 4]| {
            let fatia = if matches!(tipo, Tipo::Puxa | Tipo::Restringe) {
                2 * u64::from(nivel)
            } else {
                resto(nivel)
            };
            let grupo = match (tipo, &base_grupo) {
                (Tipo::Base | Tipo::ErroBase, Some(g)) => g.clone(),
                _ => grupo(tipo, fatia, v),
            };
            passos.push(Passo {
                tipo,
                nivel,
                fase,
                grupo,
            });
        };
        por(Tipo::Classifica, 0, 0, [fl, a(1), a(1), k0]);
        for l in 1..=niveis {
            por(Tipo::Puxa, l, 0, [k(l - 1), a(1), a(1), k(l)]);
        }
        // Um par de relaxações no nível `l` (a → b → a): o resultado fica na `a`.
        let no_nivel = |l: u32| {
            let n = miolo
                .relaxa
                .saturating_mul(1 << (l - 1).min(16))
                .min(miolo.relaxa_max.max(miolo.relaxa));
            if l == niveis { 2 * n } else { n }
        };
        let pares = |l: u32| no_nivel(l).div_ceil(2);
        // ⭐ W5o: com a base, os níveis `[b, topo]` saem dela; os despachos fazem de `b − 1` para baixo.
        let fora = base.map_or(niveis, |b| b - 1);
        match base {
            Some(b) => por(Tipo::Base, b, 1, [k0; 4]),
            None => por(
                Tipo::Inicia,
                niveis,
                1,
                [k(niveis - 1), k(niveis), k(niveis), a(niveis)],
            ),
        }
        for l in (1..=fora).rev() {
            if l < niveis {
                por(Tipo::Desce, l, 2, [a(l + 1), k(l), k(l + 1), a(l)]);
            }
            let fase = if l == niveis { 1 } else { 2 };
            for _ in 0..pares(l) {
                por(Tipo::Relaxa, l, fase, [a(l), k(l), k(l), b(l)]);
                por(Tipo::Relaxa, l, fase, [b(l), k(l), k(l), a(l)]);
            }
        }
        // ⭐ W5n: os V-ciclos sobre a solução do nível 1 (os níveis acima guardam o ERRO na a/b).
        let ciclos = if niveis > 1 { miolo.ciclos } else { 0 };
        for _ in 0..ciclos {
            por(Tipo::Defeito, 1, 3, [a(1), k(1), k(1), d(1)]);
            for l in 2..=base.unwrap_or(niveis) {
                por(Tipo::Restringe, l, 3, [d(l - 1), k(l), k(l - 1), d(l)]);
            }
            match base {
                Some(b) => por(Tipo::ErroBase, b, 4, [k0; 4]),
                None => por(
                    Tipo::ErroInicia,
                    niveis,
                    4,
                    [d(niveis), k(niveis), k(niveis), a(niveis)],
                ),
            }
            for l in (2..=fora).rev() {
                if l < niveis {
                    por(Tipo::ErroSobe, l, 5, [a(l + 1), k(l), k(l + 1), a(l)]);
                }
                let (n, fase) = if l == niveis {
                    (pares(l), 4)
                } else {
                    (miolo.suaviza, 5)
                };
                for _ in 0..n {
                    por(Tipo::ErroRelaxa, l, fase, [a(l), k(l), d(l), b(l)]);
                    por(Tipo::ErroRelaxa, l, fase, [b(l), k(l), d(l), a(l)]);
                }
            }
            por(Tipo::Soma, 1, 6, [a(2), k(1), k(2), b(1)]);
            por(Tipo::Relaxa, 1, 6, [b(1), k(1), k(1), a(1)]);
        }
        por(Tipo::Final, 0, 7, [a(1), k0, k(1), fl]);
        let pares = (0..=niveis)
            .map(|l| if l == 0 { 0 } else { pares(l) })
            .collect();
        Some(Self {
            pipelines,
            passos,
            uniforme: e.uniforme.clone(),
            niveis,
            miolo,
            anel: cfg.anel_fora_da_regiao,
            base,
            pares,
            tamanho: [gf.largura, gf.altura],
            regiao: ([0; 2], [gf.largura, gf.altura]),
            fases: None,
        })
    }

    pub(super) fn regiao(&mut self, inicio: [u32; 2], fim: [u32; 2]) {
        let fim = [0, 1].map(|k| fim[k].min(self.tamanho[k]));
        self.regiao = ([0, 1].map(|k| inicio[k].min(fim[k])), fim);
    }

    /// O tamanho do nível `l` (o mip `l − 1` da pirâmide, que começa na metade da fina).
    fn tamanho_em(&self, l: u32) -> [u32; 2] {
        if l == 0 {
            return self.tamanho;
        }
        self.tamanho.map(|x| ((x >> 1).max(1) >> (l - 1)).max(1))
    }

    /// A região é menor que a grade e o anel de fora dela corre.
    fn parcial(&self) -> bool {
        self.anel && self.regiao != ([0; 2], self.tamanho)
    }

    /// Onde o `puxa` do nível `l` sobe o conhecido: a grade inteira — ou, sem o anel, a região
    /// reduzida (a W5h).
    fn conhecido_em(&self, l: u32) -> Caixa {
        if self.anel {
            return ([0; 2], self.tamanho_em(l));
        }
        let (i, f) = self.regiao;
        let t = self.tamanho_em(l);
        (
            i.map(|x| x >> l),
            [0, 1].map(|k| f[k].div_ceil(1 << l).min(t[k])),
        )
    }

    /// ⭐ W5n — onde cada nível se RESOLVE (índice = o nível): a fina na região; acima dela o nível
    /// INTEIRO (o V-ciclo e o conhecido do corpo inteiro, com o anel de fora da região do
    /// `c3_anel.wgsl`: o resultado na região é o bit da grade inteira) — ou, sem o anel, a região
    /// reduzida (a W5h).
    fn regioes(&self) -> Vec<Caixa> {
        std::iter::once(self.regiao)
            .chain((1..=self.niveis).map(|l| self.conhecido_em(l)))
            .collect()
    }

    /// `texel_m`: o lado de um texel da grade fina neste quadro (o `reposicionar` muda-o).
    pub(super) fn gravar(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        texel_m: f64,
        tempos: Option<&Tempos>,
    ) {
        let fatias = 2 * u64::from(self.niveis + 1);
        let mut uni = vec![0u8; (PASSO_UNIFORME * (fatias + FATIAS_DA_BASE)) as usize];
        let pen = f64::from(self.miolo.penetracao_m);
        let regioes = self.regioes();
        let inteira = |l: u32| self.conhecido_em(l);
        let kh2_em = |l: u32| {
            let h = texel_m * f64::from(1u32 << l);
            if pen.is_finite() && pen > 0.0 {
                (h / pen).powi(2) as f32
            } else {
                0.0
            }
        };
        if let Some(b) = self.base {
            let mut u = UniformeBase {
                b,
                l: self.niveis,
                suaviza: self.miolo.suaviza,
                ..Zeroable::zeroed()
            };
            let mut deslocamento = 0;
            for l in b..=self.niveis {
                let tamanho = self.tamanho_em(l);
                let (inicio, fim) = regioes[l as usize];
                u.niveis[l as usize] = UniformeNivelBase {
                    inicio,
                    fim,
                    tamanho,
                    kh2: kh2_em(l),
                    pares: self.pares[l as usize],
                    deslocamento,
                    _p: [0; 3],
                };
                deslocamento += tamanho[0] * tamanho[1];
            }
            let o = (fatias * PASSO_UNIFORME) as usize;
            uni[o..o + size_of::<UniformeBase>()].copy_from_slice(bytemuck::bytes_of(&u));
        }
        for l in 0..=self.niveis {
            let kh2 = kh2_em(l);
            // A fatia `2l` (puxa) sobe o conhecido da grade inteira; a `2l + 1` resolve a caixa do
            // nível e lê a do de cima (desce).
            let acima = regioes[(l + 1).min(self.niveis) as usize];
            for (fatia, (inicio, fim), (outro_inicio, outro_fim)) in [
                (2 * l, inteira(l), inteira(l.saturating_sub(1))),
                (2 * l + 1, regioes[l as usize], acima),
            ] {
                let u = UniformeNivel {
                    inicio,
                    fim,
                    outro_inicio,
                    outro_fim,
                    kh2,
                    suavidade: self.miolo.suavidade.clamp(0.0, 1.0),
                    moldura: f32::from(u8::from(self.parcial())),
                    ladrilhos: f32::from(u8::from(fatia == 2 && self.parcial())),
                };
                let o = fatia as usize * PASSO_UNIFORME as usize;
                uni[o..o + size_of::<UniformeNivel>()].copy_from_slice(bytemuck::bytes_of(&u));
            }
        }
        gpu.queue.write_buffer(&self.uniforme, 0, &uni);
        let tudo = [(0, 0..self.passos.len())];
        let corridas = self.fases.as_ref().map_or(&tudo[..], |f| &f.corridas[..]);
        for (c, (_, r)) in corridas.iter().enumerate() {
            let par = 2 * c as u32;
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C3 miolo"),
                timestamp_writes: match &self.fases {
                    Some(f) => Some(wgpu::ComputePassTimestampWrites {
                        query_set: &f.set,
                        beginning_of_pass_write_index: Some(par),
                        end_of_pass_write_index: Some(par + 1),
                    }),
                    None => tempos.map(|t| t.escrita_compute(tempos::C3_MIOLO)),
                },
            });
            for passo in &self.passos[r.clone()] {
                if matches!(passo.tipo, Tipo::Base | Tipo::ErroBase) {
                    p.set_pipeline(&self.pipelines[passo.tipo as usize]);
                    p.set_bind_group(0, &passo.grupo, &[]);
                    p.dispatch_workgroups(1, 1, 1);
                    continue;
                }
                let (inicio, fim) = match passo.tipo {
                    Tipo::Puxa => inteira(passo.nivel),
                    _ => regioes[passo.nivel as usize],
                };
                let w = fim[0].saturating_sub(inicio[0]);
                let h = fim[1].saturating_sub(inicio[1]);
                if w == 0 || h == 0 {
                    continue;
                }
                p.set_pipeline(&self.pipelines[passo.tipo as usize]);
                p.set_bind_group(0, &passo.grupo, &[]);
                p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
            }
        }
        if let Some(f) = &self.fases {
            let n = 2 * f.corridas.len() as u32;
            enc.resolve_query_set(&f.set, 0..n, &f.resolve, 0);
            enc.copy_buffer_to_buffer(&f.resolve, 0, &f.leitura, 0, u64::from(n) * 8);
            super::rebote::carimbos_vazios(enc, tempos, tempos::C3_MIOLO);
        }
    }
}

/// ⭐ W5n — o grupo 2 do anel (`c3_anel.wgsl`): o da `c3_fina` com a saída trocada pelo conhecido da
/// fina (`k0`) e a fluência LIDA no binding 7. Devolve o layout e o grupo.
pub(super) fn grupo_do_anel(
    placa: crate::Placa<'_>,
    entradas: &[wgpu::BindGroupLayoutEntry],
    grupo: &[wgpu::BindGroupEntry<'_>],
    vistas: &[wgpu::TextureView],
    k0: usize,
    ladrilhos: &wgpu::Buffer,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let mut e = entradas.to_vec();
    e.push(entrada_textura(7, FLOAT));
    e.push(de_leitura(8));
    let layout = placa.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("luz2d c3 anel"),
        entries: &e,
    });
    let vista = |binding, v| wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    };
    let mut g: Vec<wgpu::BindGroupEntry<'_>> =
        grupo.iter().filter(|x| x.binding != 4).cloned().collect();
    g.push(vista(4, &vistas[k0]));
    g.push(vista(7, &vistas[FLUENCIA]));
    g.push(wgpu::BindGroupEntry {
        binding: 8,
        resource: ladrilhos.as_entire_binding(),
    });
    let grupo = placa.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("luz2d c3 anel"),
        layout: &layout,
        entries: &g,
    });
    (layout, grupo)
}
