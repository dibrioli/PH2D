//! ⭐ **A cascata holográfica na placa** (W5c) — o plano dos passes do `c2_hrc.wgsl`
//! (por quadrante: T0–T2 traçados, T3..TN e R_{N−1}..R0 pelas fusões, a soma; depois a saída na c0),
//! montado UMA vez (os grupos e os uniformes não mudam com a cena) e regravado por quadro. Módulo-filho
//! do [`super`].

use super::*;
use crate::config::Holografica as Escolhas;
use crate::hrc::{self, Quadrante};

/// ⭐ W5z — os T e os R em textura ou em buffer (ficheiro irmão).
#[path = "solver_hrc_entorno.rs"]
mod entorno;
/// ⭐ W5q — o instrumento das etapas (ficheiro irmão).
#[path = "solver_hrc_fases.rs"]
mod fases;
pub(crate) use entorno::Fora;
#[path = "solver_hrc_niveis.rs"]
mod niveis;
use niveis::Nivel;
#[path = "solver_hrc_ligar.rs"]
mod ligar;
use ligar::{armazenado, berco, buf, dobrar, grupos_1d, vista};

/// O espelho de `Hrc` do `c2_hrc.wgsl`.
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
    /// ⭐ W5u: 1 = os fios fazem mais de uma entrada neste passe (a grade passa [`hrc::POR_FIO_DESDE`]).
    por_fio: u32,
    /// ⭐ §44: com os quadrantes juntos, o começo da faixa deste quadrante em `t_a`, `t_b`, `r_cima` e na escrita.
    bases: [u32; 4],
    /// ⭐ W5d: o miolo cuja emissão não se recolhe (texels da grade) e a c0 do entorno somada na
    /// saída (`desloc, escala, ligado`) — o `Hrc` do `c2_hrc.wgsl`.
    miolo: [f32; 4],
    entorno: [f32; 4],
    /// ⭐ bug 6 (handoff W6g §32): `(base, arquiva, _, miolo vácuo)`, o entorno pela borda (`X, Y, K,
    /// ligado` no mesmo quadrante), a janela do arquivo e a base de cada nível nele.
    arquivo: [u32; 4],
    fora: [u32; 4],
    janela: [u32; 4],
    fora_af: [f32; 4],
    fora_base: [[u32; 4]; 5],
    /// ⭐ §44: os grupos `[⌈Y/64⌉, nx, k]` deste quadrante no despacho junto e o primeiro deles.
    grupos: [u32; 4],
}

/// O miolo VAZIO (nenhum texel cai nele).
const SEM_MIOLO: [f32; 4] = [1.0, 1.0, 0.0, 0.0];

const _: () = assert!(size_of::<UniformeHrc>() == 256, "o `Hrc` do `c2_hrc.wgsl`");
// ⭐ §44: o `array<Hrc, 4>` do `c2_hrc.wgsl` são quatro fatias seguidas do uniforme.
const _: () = assert!(size_of::<UniformeHrc>() as u64 == PASSO_UNIFORME);

#[derive(Clone, Copy)]
enum Etapa {
    Traca = 0,
    Sobe = 1,
    Desce = 2,
    Soma = 3,
    Saida = 4,
    Ladrilhos = 5,
}

const ENTRADAS: [&str; 6] = [
    "hrc_traca",
    "hrc_sobe",
    "hrc_desce",
    "hrc_soma",
    "hrc_saida",
    "hrc_ladrilhos",
];

/// O que a holográfica pega do solver ao nascer.
pub(super) struct Entradas<'a> {
    pub device: crate::Placa<'a>,
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
    /// `(etapa, grupo 3, grupos de trabalho, a 1.ª fatia do uniforme)`, na ordem.
    passos: Vec<(Etapa, wgpu::BindGroup, [u32; 3], usize)>,
    /// ⭐ W5d: o uniforme de cada passo (o miolo e o entorno reescrevem-se neles) e o buffer.
    uniformes: Vec<UniformeHrc>,
    uniforme: wgpu::Buffer,
    /// ⭐ W5d: o grupo 2 da saída — a c0 do entorno (ou, sem ele, a emissão, que não se lê) e a amostra.
    g2_layout: wgpu::BindGroupLayout,
    g2: wgpu::BindGroup,
    amostra: wgpu::Sampler,
    vazia: wgpu::TextureView,
    /// ⭐ bug 6: o arquivo dos R deste solver (um ENTORNO enche-o; nos outros, 16 bytes que não se leem),
    /// a janela máxima, a grade, as janelas do miolo de agora e as bases dos níveis nelas.
    pub(super) arquivo: wgpu::Buffer,
    janela_max: Option<[u32; 2]>,
    grade: [u32; 2],
    pub(super) janelas: [hrc::Janela; 4],
    pub(super) bases: [[u32; hrc::NIVEIS_MAX]; 4],
    fases: Option<fases::FasesHrc>,
    /// ⭐ W5x — os buffers grandes (T, R, acumulado) com o rótulo: a régua do regime da `c2` mede cada um.
    pub(super) grandes: Vec<(&'static str, wgpu::Buffer)>,
}

impl Holografica {
    pub(super) fn novo(e: &Entradas<'_>) -> Self {
        let device = e.device;
        let g = e.cfg.grade;
        let grade = [g.largura, g.altura];
        let vazia = tabela_vazia(&device, crate::passlist::NOME_TABELA_VAZIA_HRC);
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
                entrada_textura(8, FLOAT),
                entrada_tabela(10),
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
                // O `cantos` do `meio` (W6f): só a C3 o lê.
                vista(8, &e.vistas[COBERTURA]),
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: vazia.as_entire_binding(),
                },
            ],
        });
        let c0 = PassList::indice_da_c0(e.cfg);
        let em_textura = e.cfg.hrc_niveis_em_textura();
        let [n5, n6, n7, n8, n9] = niveis::layout(em_textura);
        let g3_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c2 holográfica"),
            entries: &[
                entrada_uniforme(4, wgpu::ShaderStages::COMPUTE),
                n5,
                n6,
                n7,
                n8,
                n9,
                armazenado(10, false),
                entrada_escrita(11, e.lista.texturas[c0].formato),
                armazenado(12, true),
                armazenado(16, false),
            ],
        });
        // ⭐ W5d: o entorno (grupo 2) — a c0 dele, que a saída soma; ⭐ bug 6: e o arquivo dos R (o
        // entorno escreve o seu na descida; a vista lê o do entorno no lugar do céu).
        let g2_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c2 holográfica entorno"),
            entries: &[
                entrada_textura(14, wgpu::TextureSampleType::Float { filterable: true }),
                wgpu::BindGroupLayoutEntry {
                    binding: 15,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                armazenado(17, false),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d c2 holográfica"),
            bind_group_layouts: &[Some(&cena_layout), None, Some(&g2_layout), Some(&g3_layout)],
            immediate_size: 0,
        });
        let amostra = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("luz2d c2 holográfica entorno"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        // Sem entorno a saída liga a EMISSÃO (o `entorno.w = 0` não a lê): uma textura vazia a mais mudava
        // o enchimento que o alocador dá às vizinhas (o G5).
        let vazia = e.vistas[EMISSAO].clone();
        let arquivo = (e.buffer)(hrc::NOME_ARQUIVO);
        let g2 = g2_de(device, &g2_layout, &vazia, &amostra, &arquivo);
        // ⭐ W5u: os fios que fazem mais de uma entrada (só em 3D).
        let em_3d = e.cfg.hrc_grade_3d && e.cfg.hrc_por_colunas;
        let cones = if em_3d {
            e.cfg.hrc_cones_por_fio.max(1)
        } else {
            1
        };
        let pares = if em_3d { e.cfg.hrc_pares_na_subida } else { 0 };
        // A decisão por grade vai no UNIFORME, não nas constantes: redimensionar não recompila a luz.
        let por_fio = em_3d && u64::from(g.largura) * u64::from(g.altura) >= hrc::POR_FIO_DESDE;
        let constantes = [
            ("PASSO_HRC", f64::from(e.escolhas.passo)),
            ("BORRAO_HRC", f64::from(u8::from(e.escolhas.borrao))),
            ("MAIS_UM_HRC", f64::from(u8::from(e.escolhas.mais_um))),
            (
                "MASCARA_C0",
                f64::from(u8::from(e.cfg.preenchimento.is_some())),
            ),
            ("COLUNAS_HRC", f64::from(u8::from(e.cfg.hrc_por_colunas))),
            (
                "ARCOS_TABELA",
                f64::from(u8::from(e.cfg.hrc_arcos_em_tabela)),
            ),
            (
                "SALTA_O_VAZIO_HRC",
                f64::from(u8::from(e.cfg.hrc_salta_o_vazio)),
            ),
            ("SONDA_HRC", f64::from(u8::from(e.cfg.hrc_fio_por_sonda))),
            ("GRADE3D_HRC", f64::from(u8::from(e.cfg.hrc_grade_3d))),
            ("CONES_HRC", f64::from(cones)),
            ("PARES_SOBE_HRC", f64::from(pares)),
            ("JUNTOS_HRC", f64::from(u8::from(e.cfg.hrc_juntos()))),
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

        // ⭐ W5p (`Config::hrc_nasce_duas_vezes`): os buffers grandes nascem, largam-se e nascem de novo —
        // a `c2` do High é rápida quando eles caem em memória já usada e largada (escrevê-los antes: igual).
        // ⭐ W5q: o acumulado renasce PRIMEIRO — por último era o único no regime lento (a soma 1,40 contra
        // 0,83 ms; handoff W5q §1).
        // ⭐ W5x (`Config::hrc_berco`): o 1.º nascimento é em texturas cruas — as páginas ficam comprimíveis.
        // ⭐ W5z: com os níveis em textura, nasce UMA vez (medido: a mesma `c2`, placa ociosa ou não).
        let n_t = Quadrante::de(0, grade).n.max(Quadrante::de(1, grade).n) as usize + 1;
        let duas_vezes = e.cfg.hrc_nasce_duas_vezes && !em_textura;
        if duas_vezes {
            let nomes = hrc::NOMES_T[..n_t]
                .iter()
                .copied()
                .chain(hrc::NOMES_R)
                .chain([hrc::NOME_ACUMULADO]);
            let relata = |quando| {
                if e.cfg.hrc_guarda_os_grandes {
                    crate::sonda_memoria::relatar_o_alocador(device.device, quando);
                }
            };
            if e.cfg.hrc_berco {
                let berco: Vec<Option<wgpu::Texture>> = nomes.map(|n| berco(e, n)).collect();
                relata("vivo");
                drop(berco);
            } else {
                let primeiros: Vec<wgpu::Buffer> = nomes.map(|n| (e.buffer)(n)).collect();
                drop(primeiros);
            }
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            relata("largado");
        }
        let cedo = duas_vezes.then(|| (e.buffer)(hrc::NOME_ACUMULADO));
        let nivel = |nome| Nivel::de(e, em_textura, nome);
        let t: Vec<Nivel> = hrc::NOMES_T[..n_t].iter().map(|&n| nivel(n)).collect();
        let r = hrc::NOMES_R.map(nivel);
        let acumulado = cedo.unwrap_or_else(|| (e.buffer)(hrc::NOME_ACUMULADO));
        if e.cfg.hrc_guarda_os_grandes {
            crate::sonda_memoria::relatar_o_alocador(device.device, "buffers");
        }
        let uniforme = (e.buffer)(hrc::NOME_UNIFORME);
        let arcos = (e.buffer)(hrc::NOME_ARCOS);
        if e.cfg.hrc_arcos_em_tabela {
            escrever_arcos(e, &arcos);
        }
        let [vazio_l, vazio_t, vazio_r] = hrc::NOMES_VAZIOS.map(nivel);
        let com_cobertura = (e.buffer)(hrc::NOME_COM_COBERTURA);

        let mut uniformes = Vec::new();
        let mut passos = Vec::new();
        // Um passe com o uniforme de cada quadrante dele (⭐ §44: quatro no despacho junto, um nos outros).
        let mut passo = |etapa: Etapa, us: &[UniformeHrc], ligados: [&Nivel; 5], grupos: [u32; 3]| {
            let k = uniformes.len() as u64;
            uniformes.extend_from_slice(us);
            let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c2 holográfica"),
                layout: &g3_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: fatia_uniforme(&uniforme, k, 4 * size_of::<UniformeHrc>() as u64),
                    },
                    ligados[0].ligar(5),
                    ligados[1].ligar(6),
                    ligados[2].ligar(7),
                    ligados[3].ligar(8),
                    ligados[4].ligar(9),
                    buf(10, &acumulado),
                    vista(11, &e.vistas[c0]),
                    buf(12, &arcos),
                    buf(16, &com_cobertura),
                ],
            });
            passos.push((etapa, grupo, grupos, k as usize));
        };
        let saida = UniformeHrc {
            quadrante: 0,
            nivel: 0,
            grade,
            eixo: grade,
            nx: 0,
            nx_baixo: 0,
            nx_cima: 0,
            topo: 0,
            total: 0,
            por_fio: 0,
            bases: [0; 4],
            miolo: SEM_MIOLO,
            entorno: [0.0; 4],
            arquivo: [0; 4],
            fora: [0; 4],
            janela: [0; 4],
            fora_af: [0.0; 4],
            fora_base: [[0; 4]; 5],
            grupos: [0; 4],
        };
        // ⭐ W5s: a máscara do vazio, da própria captura (a textura inteira: a grade efetiva), antes dos
        // quadrantes.
        if e.cfg.hrc_salta_o_vazio {
            let ge = e.cfg.grade_efetiva();
            passo(
                Etapa::Ladrilhos,
                &[saida],
                [&vazio_l, &vazio_l, &vazio_t, &vazio_l, &vazio_r],
                [
                    ge.largura.div_ceil(hrc::LADO_DO_VAZIO),
                    ge.altura.div_ceil(hrc::LADO_DO_VAZIO),
                    1,
                ],
            );
        }
        // ⭐ bug 6 (handoff W6g §32): um ENTORNO faz uma 2.ª passagem com o miolo VÁCUO, que arquiva os R
        // (sem soma nem saída: a c0 dele é a da 1.ª, a do rebote dele).
        let rodadas: &[bool] = if e.cfg.arquivo_da_borda.is_some() {
            &[false, true]
        } else {
            &[false]
        };
        // ⭐ §44: os quadrantes JUNTOS — cada etapa num despacho, cada quadrante na sua faixa dos níveis; a soma
        // continua um quadrante de cada vez (o acumulado soma-os na MESMA ordem: o mesmo bit).
        let juntos = e.cfg.hrc_juntos().then(|| hrc::faixas(grade).0);
        let n_max = (0..4).map(|q| Quadrante::de(q, grade).n).max().unwrap_or(3);
        let soma = 2 * n_max + 1;
        for &arquivando in rodadas {
            // As etapas de cada quadrante: `(chave na rodada, etapa, uniforme, ligados, grupos)`.
            let mut etapas: [Vec<(u32, Etapa, UniformeHrc, [&Nivel; 5], [u32; 3])>; 4] = Default::default();
            for q in 0..4 {
                let faixa = |bases: [u32; 4]| if juntos.is_some() { bases } else { [0; 4] };
                let ft = juntos.map_or([0; hrc::NIVEIS_MAX], |f| f.t[q as usize]);
                let fr = juntos.map_or(0, |f| f.r[q as usize]);
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
                    por_fio: 0,
                    bases: [0; 4],
                    miolo: SEM_MIOLO,
                    entorno: [0.0; 4],
                    arquivo: [0; 4],
                    fora: [0; 4],
                    janela: [0; 4],
                    fora_af: [0.0; 4],
                    fora_base: [[0; 4]; 5],
                    grupos: [0; 4],
                };
                let base = UniformeHrc {
                    arquivo: [0, 0, 0, u32::from(arquivando)],
                    ..base
                };
                let nx = |n: u32| quad.nx(n);
                // ⭐ W5t: em 3D (por colunas) os grupos são `[⌈Y/64⌉, nx, k]` (a traça só com um fio por sonda).
                let grupos_3d = |n: u32, k: u32| [quad.y.div_ceil(64), nx(n), k];
                for n in 0..=quad.n {
                    // ⭐ W5s: um fio por sonda na traça (o `SONDA_HRC`).
                    let por_sonda = n <= 2 && e.cfg.hrc_fio_por_sonda;
                    let total = if por_sonda {
                        u64::from(nx(n)) * u64::from(quad.y)
                    } else {
                        quad.entradas_t(n)
                    };
                    let u = UniformeHrc {
                        nivel: n,
                        nx: nx(n),
                        nx_baixo: if n > 0 { nx(n - 1) } else { 0 },
                        total: total as u32,
                        por_fio: u32::from(por_fio),
                        ..base
                    };
                    let (etapa, ler, bases) = if n <= 2 {
                        (Etapa::Traca, &vazio_l, [0, 0, 0, ft[n as usize]])
                    } else {
                        (Etapa::Sobe, &t[(n - 1) as usize], [ft[(n - 1) as usize], 0, 0, ft[n as usize]])
                    };
                    let u = UniformeHrc {
                        bases: faixa(bases),
                        ..u
                    };
                    let grupos = match (em_3d, n <= 2) {
                        (true, true) if por_sonda => grupos_3d(n, 1),
                        (true, false) if pares > 0 && por_fio => {
                            grupos_3d(n, ((1 << (n - 1)) + 1u32).div_ceil(pares))
                        }
                        (true, false) => grupos_3d(n, (1 << n) + 1),
                        _ => grupos_1d(total),
                    };
                    etapas[q as usize].push((
                        n,
                        etapa,
                        u,
                        [ler, &vazio_l, &t[n as usize], &vazio_l, &vazio_r],
                        grupos,
                    ));
                }
                for n in (0..quad.n).rev() {
                    let total = quad.entradas_r(n);
                    let topo = n + 1 == quad.n;
                    let u = UniformeHrc {
                        nivel: n,
                        nx: nx(n),
                        nx_cima: nx(n + 1),
                        topo: u32::from(topo),
                        bases: faixa([
                            ft[n as usize],
                            ft[(n + 1) as usize],
                            if topo { 0 } else { fr },
                            fr,
                        ]),
                        total: total as u32,
                        por_fio: u32::from(por_fio),
                        arquivo: if arquivando {
                            [0, 1, u32::from(e.cfg.hrc_recorta_o_arquivo), 1]
                        } else {
                            [0; 4]
                        },
                        ..base
                    };
                    // ⭐ §44: junto, o passe é de todos os quadrantes neste nível e o topo de um não é o de outro
                    // (a grade não quadrada): liga-se o R de cima, que o topo não lê.
                    let cima = if topo && juntos.is_none() {
                        &vazio_l
                    } else {
                        &r[((n + 1) % 2) as usize]
                    };
                    etapas[q as usize].push((
                        2 * n_max - n,
                        Etapa::Desce,
                        u,
                        [
                            &t[n as usize],
                            &t[(n + 1) as usize],
                            &vazio_t,
                            cima,
                            &r[(n % 2) as usize],
                        ],
                        if em_3d {
                            grupos_3d(n, (1u32 << n).div_ceil(if por_fio { cones } else { 1 }))
                        } else {
                            grupos_1d(total)
                        },
                    ));
                }
                if arquivando {
                    continue;
                }
                // ⭐ W5r: por colunas, um grupo por ladrilho 8×8 (o `hrc_soma`).
                let total = if e.cfg.hrc_por_colunas {
                    64 * u64::from(quad.x.div_ceil(8)) * u64::from(quad.y.div_ceil(8))
                } else {
                    u64::from(quad.x) * u64::from(quad.y)
                };
                let u = UniformeHrc {
                    nx: nx(0),
                    total: total as u32,
                    bases: faixa([0, 0, fr, 0]),
                    ..base
                };
                etapas[q as usize].push((
                    soma,
                    Etapa::Soma,
                    u,
                    [&vazio_l, &vazio_l, &vazio_t, &r[0], &vazio_r],
                    grupos_1d(total),
                ));
            }
            if juntos.is_none() {
                for (_, etapa, u, ligados, grupos) in etapas.iter().flatten() {
                    passo(*etapa, &[*u], *ligados, *grupos);
                }
            } else {
                for chave in 0..soma {
                    let membros = etapas.each_ref().map(|v| v.iter().find(|x| x.0 == chave));
                    let Some(&&(_, etapa, _, ligados, _)) = membros.iter().flatten().next() else {
                        continue;
                    };
                    let mut primeiro = 0u32;
                    let us: [UniformeHrc; 4] = std::array::from_fn(|q| {
                        let (u, [x, y, z]) = membros[q].map_or(
                            (
                                UniformeHrc {
                                    quadrante: q as u32,
                                    ..saida
                                },
                                [0; 3],
                            ),
                            |m| (m.2, m.4),
                        );
                        let u = UniformeHrc {
                            grupos: [x, y, z, primeiro],
                            ..u
                        };
                        primeiro += x * y * z;
                        u
                    });
                    passo(etapa, &us, ligados, dobrar(u64::from(primeiro)));
                }
                for (_, etapa, u, ligados, grupos) in etapas.iter().flatten().filter(|x| x.0 == soma) {
                    passo(*etapa, &[*u], *ligados, *grupos);
                }
            }
            if !arquivando {
                passo(
                    Etapa::Saida,
                    &[saida],
                    [&vazio_l, &vazio_l, &vazio_t, &vazio_l, &vazio_r],
                    [grade[0].div_ceil(8), grade[1].div_ceil(8), 1],
                );
            }
        }
        let h = Self {
            cena,
            pipelines,
            passos,
            uniformes,
            uniforme,
            g2_layout,
            g2,
            amostra,
            vazia,
            arquivo,
            janela_max: e.cfg.arquivo_da_borda,
            grade,
            janelas: [[0; 4]; 4],
            bases: [[0; hrc::NIVEIS_MAX]; 4],
            fases: None,
            grandes: if e.cfg.hrc_guarda_os_grandes {
                hrc::NOMES_T[..n_t]
                    .iter()
                    .copied()
                    .zip(t)
                    .chain(hrc::NOMES_R.into_iter().zip(r))
                    .filter_map(|(nome, n)| Some((nome, n.buffer()?)))
                    .chain([(hrc::NOME_ACUMULADO, acumulado)])
                    .collect()
            } else {
                Vec::new()
            },
        };
        h.escrever(e.queue);
        h
    }

    /// Os uniformes de todos os passos para a placa.
    fn escrever(&self, queue: &wgpu::Queue) {
        let mut uni = vec![0u8; self.uniformes.len() * PASSO_UNIFORME as usize];
        for (k, u) in self.uniformes.iter().enumerate() {
            let o = k * PASSO_UNIFORME as usize;
            uni[o..o + size_of::<UniformeHrc>()].copy_from_slice(bytemuck::bytes_of(u));
        }
        queue.write_buffer(&self.uniforme, 0, &uni);
    }

    /// Grava a cascata holográfica inteira no passe `p` (a c0 sai dela); a gravação `k` do quadro (⭐ W5s:
    /// a máscara do vazio só na 1.ª — a cobertura não muda entre os rebotes).
    pub(super) fn gravar(&self, p: &mut wgpu::ComputePass<'_>, k: u32) {
        p.set_bind_group(0, &self.cena, &[]);
        for (etapa, grupo, [x, y, z], _) in &self.passos {
            if k > 0 && matches!(etapa, Etapa::Ladrilhos) {
                continue;
            }
            p.set_pipeline(&self.pipelines[*etapa as usize]);
            p.set_bind_group(2, &self.g2, &[]);
            p.set_bind_group(3, grupo, &[]);
            p.dispatch_workgroups(*x, *y, *z);
        }
    }
}

/// ⭐ W5r — a tabela dos arcos (`hrc_arcos`), uma vez: um despacho no seu próprio envio.
fn escrever_arcos(e: &Entradas<'_>, arcos: &wgpu::Buffer) {
    let device = e.device;
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("luz2d c2 holográfica arcos"),
        entries: &[armazenado(13, false)],
    });
    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("luz2d c2 holográfica arcos"),
        bind_group_layouts: &[None, None, None, Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("hrc_arcos"),
        layout: Some(&pl),
        module: e.modulo,
        entry_point: Some("hrc_arcos"),
        compilation_options: Default::default(),
        cache: None,
    });
    let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("luz2d c2 holográfica arcos"),
        layout: &layout,
        entries: &[buf(13, arcos)],
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("luz2d c2 holográfica arcos"),
    });
    {
        let mut p = enc.begin_compute_pass(&Default::default());
        p.set_pipeline(&pipeline);
        p.set_bind_group(3, &grupo, &[]);
        p.dispatch_workgroups(((arcos.size() / 4) as u32).div_ceil(64), 1, 1);
    }
    e.queue.submit([enc.finish()]);
}

fn g2_de(
    device: crate::Placa<'_>,
    layout: &wgpu::BindGroupLayout,
    v: &wgpu::TextureView,
    amostra: &wgpu::Sampler,
    arquivo: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("luz2d c2 holográfica entorno"),
        layout,
        entries: &[
            vista(14, v),
            wgpu::BindGroupEntry {
                binding: 15,
                resource: wgpu::BindingResource::Sampler(amostra),
            },
            buf(17, arquivo),
        ],
    })
}
