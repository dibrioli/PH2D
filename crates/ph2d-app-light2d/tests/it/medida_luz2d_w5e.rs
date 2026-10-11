//! **A rodada da W5e — o governador acima de 1080p e a memória** (`docs/Luz2D/01_plano.md` §7; método
//! `docs/DevOps/MEDIR_VELOCIDADE.md`). À mão:
//!
//! ```text
//! cargo test -p ph2d-app-light2d --profile smoke --test it medida_luz2d_w5e_bytes -- --ignored --nocapture
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5e_placa -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A0. os bytes que a `PassList` DECLARA (sem placa), a vista de fábrica, por nível × resolução: o
//!     solver da vista, o entorno, e a parte deles que segue a grade das cascatas (a que o zoom não
//!     encolhe: a grade cobre o pior zoom da oitava e a margem) contra a da janela;
//! A1. na placa: os bytes que o alocador do wgpu diz e o tempo (o MÍNIMO de [`RODADAS`] intercaladas,
//!     o `loadavg` ao lado) da cena 3 por nível × resolução.

use kurbo::Affine;
use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::{
    Degrau, MARGEM_DO_ENTORNO, captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela,
};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_host::WindowSize;
use ph2d_light2d_gpu::{Config, Grade, HOLOGRAFICA_PADRAO, PassList};
use ph2d_render::Camera2d;

pub(crate) const RESOLUCOES: [(&str, (u32, u32)); 3] = [
    ("1080p", (1920, 1080)),
    ("1440p", (2560, 1440)),
    ("2160p", (3840, 2160)),
];

/// A vista de fábrica numa janela `alvo`.
pub(crate) fn fabrica(alvo: (u32, u32)) -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(alvo.0, alvo.1))
}

/// A configuração do PRODUTO (`LuzDoQuadro::gravar`) para o degrau `dg` na vista `a`.
pub(crate) fn config_do_produto(a: Affine, alvo: (u32, u32), dg: Degrau) -> Option<Config> {
    let g = grade_do_quadro(a, alvo, dg)?;
    Some(Config {
        fina: fina_do_quadro(a, dg),
        captura: captura_do_quadro(a, dg),
        holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
        rebote: true,
        rebotes_por_quadro: dg.rebotes_por_quadro,
        ..Config::padrao(g)
    })
}

/// A configuração do ENTORNO do produto (`LuzDoQuadro::preparar_entorno`).
pub(crate) fn config_do_entorno(a: Affine, alvo: (u32, u32), dg: Degrau) -> Option<Config> {
    if dg.entorno <= 0.0 || !dg.holografica {
        return None;
    }
    let de = Degrau {
        c0_px: dg.c0_px * dg.entorno,
        margem: MARGEM_DO_ENTORNO,
        ..dg
    };
    let g = grade_do_quadro(a, alvo, de)?;
    Some(Config {
        holografica: Some(HOLOGRAFICA_PADRAO),
        rebote: true,
        ..Config::padrao(g)
    })
}

fn mb(b: u64) -> f64 {
    b as f64 / 1e6
}

/// A fração da área de `g` dentro da janela.
fn fracao_na_janela(g: Grade, a: Affine, alvo: (u32, u32)) -> f64 {
    let (i, f) = regiao_da_janela(g, a, alvo);
    let dentro = f64::from(f[0] - i[0]) * f64::from(f[1] - i[1]);
    dentro / (f64::from(g.largura) * f64::from(g.altura))
}

/// Quantas vezes cada configuração corre (em passes alternados, ida e volta); vale o MÍNIMO.
const RODADAS: usize = 4;
/// Quadros por leitura (o primeiro aquece).
const QUADROS: usize = 3;

/// Os bytes vivos no alocador do wgpu (a fila esvaziada antes).
fn alocado(gpu: &ph2d_gpu::GpuContext) -> Option<u64> {
    gpu.queue.submit([]);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    gpu.device
        .generate_allocator_report()
        .map(|r| r.total_allocated_bytes)
}

/// O que a placa fez com uma configuração: a recusa do solver, os erros que o dispositivo deu ao
/// criá-lo e ao gravar um quadro, os bytes que o alocador ganhou e o tempo (`por nível + fixo`).
struct Leitura {
    recusa: Option<String>,
    erro: Option<String>,
    mb: f64,
    ms: [f64; 2],
}

fn ler(
    gpu: &ph2d_gpu::GpuContext,
    cena: &ph2d_light2d_ecs::CenaDeLuz,
    a: Affine,
    alvo: (u32, u32),
    dg: Degrau,
) -> Leitura {
    use ph2d_light2d_gpu::Solver;
    let antes = alocado(gpu).unwrap_or(0);
    let validacao = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let memoria = gpu.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let cfg = Config {
        aritmetica: Config::padrao_para(
            &gpu.device,
            config_do_produto(a, alvo, dg).expect("grade").grade,
        )
        .aritmetica,
        ..config_do_produto(a, alvo, dg).expect("grade")
    };
    let feito = Solver::novo(gpu, cfg).map(|mut s| {
        let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
        s.regiao_fina(i, f);
        let e = config_do_entorno(a, alvo, dg).and_then(|c| Solver::novo(gpu, c).ok());
        (s, e)
    });
    let mut ms = [f64::INFINITY; 2];
    let mut recusa = None;
    let mut depois = antes;
    match feito {
        Err(e) => recusa = Some(format!("{e}")),
        Ok((mut s, mut e)) => {
            depois = alocado(gpu).unwrap_or(0);
            let sem_ceu = ph2d_light2d_ecs::CenaDeLuz {
                ceu: [0.0; 3],
                ..cena.clone()
            };
            for _ in 0..QUADROS {
                if let Some(e) = e.as_mut() {
                    e.quadro(gpu, &sem_ceu);
                }
                s.quadro(gpu, cena);
                if let Some(t) = s.tempos(gpu) {
                    let te = e
                        .as_ref()
                        .and_then(|e| e.tempos(gpu))
                        .map_or(0.0, |t| t.total());
                    if t.total() + te < ms[0] + ms[1] {
                        ms = [t.por_nivel(), t.fixo() + te];
                    }
                }
            }
        }
    }
    let erro = [
        pollster::block_on(memoria.pop()),
        pollster::block_on(validacao.pop()),
    ]
    .into_iter()
    .flatten()
    .next()
    .map(|e| {
        format!("{e:?}")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    });
    Leitura {
        recusa,
        erro,
        mb: depois.saturating_sub(antes) as f64 / 1e6,
        ms,
    }
}

/// A1 — a placa por nível × resolução (cena 3, a vista de fábrica). As configurações correm uma de
/// cada vez (a 2160p o Alto pede ~4,7 GB: juntas não cabiam ao lado do resto da máquina) em passes de
/// ida e volta; o tempo é o MÍNIMO e só vale com o `loadavg` ao lado.
#[test]
#[ignore = "rodada de medição W5e (placa): memória alocada, erros e tempo por nível × resolução"]
fn medida_luz2d_w5e_placa() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    let mut w = bevy_ecs::world::World::new();
    assert!(ph2d_app_light2d::smoke::encenar(&mut w, 3));
    let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
    eprintln!(
        "## A1. Na placa (cena 3, vista de fábrica; max_texture_dimension_2d {}) · loadavg antes {}",
        gpu.device.limits().max_texture_dimension_2d,
        crate::medida_luz2d_w4b::loadavg()
    );
    let configs: Vec<_> = RESOLUCOES
        .iter()
        .flat_map(|&(r, alvo)| NivelDaLuz::ALL.map(|n| (r, alvo, n)))
        .collect();
    let mut leituras: Vec<Option<Leitura>> = configs.iter().map(|_| None).collect();
    for rodada in 0..RODADAS {
        let ordem: Vec<usize> = if rodada % 2 == 0 {
            (0..configs.len()).collect()
        } else {
            (0..configs.len()).rev().collect()
        };
        for i in ordem {
            let (_, alvo, n) = configs[i];
            let lado = gpu.device.limits().max_texture_dimension_2d;
            let tetos = ph2d_app_light2d::teto::Tetos {
                lado,
                recusou: None,
            };
            let k = ph2d_app_light2d::teto::fina_que_cabe(fabrica(alvo), alvo, degrau_do(n), tetos)
                .expect("cabe");
            let l = ler(
                &gpu,
                &cena,
                fabrica(alvo),
                alvo,
                degrau_do(n).com_fina_engrossada(k),
            );
            leituras[i] = Some(match leituras[i].take() {
                None => l,
                Some(v) => Leitura {
                    ms: if l.ms[0] + l.ms[1] < v.ms[0] + v.ms[1] {
                        l.ms
                    } else {
                        v.ms
                    },
                    erro: v.erro.or(l.erro),
                    ..v
                },
            });
        }
    }
    eprintln!("(a fina engrossada pelos tetos: a 2160p `k ≥ 1`, ver a tabela A0)");
    eprintln!("| res | nível | alocado MB | por nível ms | fixo ms | total ms | recusa / erro |");
    eprintln!("|---|---|---|---|---|---|---|");
    for ((r, _, n), l) in configs.iter().zip(&leituras) {
        let l = l.as_ref().expect("lida");
        eprintln!(
            "| {r} | {n:?} | {:.0} | {:.2} | {:.2} | {:.2} | {} |",
            l.mb,
            l.ms[0],
            l.ms[1],
            l.ms[0] + l.ms[1],
            l.recusa
                .clone()
                .or_else(|| l.erro.clone())
                .unwrap_or_else(|| "—".into())
        );
    }
    eprintln!("loadavg depois {}", crate::medida_luz2d_w4b::loadavg());
}

/// A2 — o que o wgpu EXPÕE da memória do dispositivo: o limite de dimensão do adaptador, o relatório
/// do alocador e a RECUSA pelo orçamento do driver (`MemoryBudgetThresholds::for_resource_creation`,
/// uma percentagem do orçamento que a API nativa reporta — `VK_EXT_memory_budget`). Cada limiar numa
/// instância própria; uma textura de 64 MB sob o escopo `OutOfMemory`. Não enche a placa.
#[test]
#[ignore = "rodada de medição W5e (placa): o que o wgpu expõe da memória"]
fn medida_luz2d_w5e_orcamento() {
    eprintln!("## A2. A memória que o wgpu expõe");
    for limiar in [None, Some(1u8), Some(50), Some(100)] {
        let instancia = wgpu::Instance::new(wgpu::InstanceDescriptor {
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                for_resource_creation: limiar,
                for_device_loss: None,
            },
            ..wgpu::InstanceDescriptor::new_without_display_handle_from_env()
        });
        let Ok(gpu) = ph2d_gpu::GpuContext::new(instancia, None) else {
            eprintln!("- limiar {limiar:?}: o dispositivo não nasce");
            continue;
        };
        let escopo = gpu.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let _t = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("w5e/orcamento"),
            size: wgpu::Extent3d {
                width: 4096,
                height: 2048,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let erro = pollster::block_on(escopo.pop());
        eprintln!(
            "- limiar {limiar:?}: {} · adaptador max_texture_dimension_2d {} · alocado {:?} MB",
            erro.map_or("criada".into(), |e| format!("RECUSADA ({e})")),
            gpu.adapter.limits().max_texture_dimension_2d,
            alocado(&gpu).map(|b| b / 1_000_000)
        );
    }
}

#[test]
#[ignore = "rodada de medição W5e (sem placa): bytes declarados por nível × resolução"]
fn medida_luz2d_w5e_bytes() {
    eprintln!("## A0. Os bytes DECLARADOS (PassList, sem placa), vista de fábrica");
    eprintln!(
        "| res | nível | grade (cascatas) | fluência | vista MB | entorno MB | total MB | na janela (cascatas · fluência) | maiores |"
    );
    eprintln!("|---|---|---|---|---|---|---|---|---|");
    for (nome, alvo) in RESOLUCOES {
        let a = fabrica(alvo);
        for nivel in NivelDaLuz::ALL {
            let dg = degrau_do(nivel);
            let cfg = config_do_produto(a, alvo, dg).expect("grade");
            let pl = PassList::de(&cfg, true);
            let ent = config_do_entorno(a, alvo, dg).map(|c| PassList::de(&c, true));
            let (vb, eb) = (
                pl.bytes_alocados(),
                ent.as_ref().map_or(0, PassList::bytes_alocados),
            );
            let gc = cfg.grade_efetiva();
            let gf = cfg.grade_da_fluencia();
            let mut maiores: Vec<_> = pl.texturas.iter().map(|t| (t.bytes(), t.nome)).collect();
            maiores.sort_unstable_by(|x, y| y.cmp(x));
            let top = maiores
                .iter()
                .take(4)
                .map(|(b, n)| format!("{n} {:.0}", mb(*b)))
                .collect::<Vec<_>>()
                .join(" · ");
            eprintln!(
                "| {nome} | {nivel:?} | {}×{} | {}×{} | {:.0} | {:.0} | {:.0} | {:.0} % · {:.0} % | {top} |",
                gc.largura,
                gc.altura,
                gf.largura,
                gf.altura,
                mb(vb),
                mb(eb),
                mb(vb + eb),
                100.0 * fracao_na_janela(gc, a, alvo),
                100.0 * fracao_na_janela(gf, a, alvo),
            );
        }
    }
}

/// O retângulo do mundo que a vista `a` mostra numa janela `alvo`: `[x0, y0, x1, y1]`.
fn visivel(a: Affine, alvo: (u32, u32)) -> [f32; 4] {
    let inv = a.inverse();
    let p = [
        inv * kurbo::Point::new(0.0, 0.0),
        inv * kurbo::Point::new(f64::from(alvo.0), f64::from(alvo.1)),
    ];
    [
        p[0].x.min(p[1].x) as f32,
        p[0].y.min(p[1].y) as f32,
        p[0].x.max(p[1].x) as f32,
        p[0].y.max(p[1].y) as f32,
    ]
}

/// `n` pontos espalhados no retângulo `r` pela treliça do passo áureo (determinística).
fn espalhados(n: u32, r: [f32; 4]) -> Vec<[f32; 2]> {
    let k = f64::from(n).sqrt().ceil() as u32;
    let celulas = k * k;
    let alvo = ((f64::from(celulas) * 0.618_034) as u32).max(1);
    let gcd = |mut a: u32, mut b: u32| {
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    };
    let passo = (1..=alvo)
        .rev()
        .find(|&p| gcd(p, celulas) == 1)
        .unwrap_or(1);
    let (w, h) = ((r[2] - r[0]) / k as f32, (r[3] - r[1]) / k as f32);
    (0..n)
        .map(|i| {
            let c = (u64::from(i) * u64::from(passo) % u64::from(celulas)) as u32;
            [
                r[0] + ((c % k) as f32 + 0.5) * w,
                r[1] + ((c / k) as f32 + 0.5) * h,
            ]
        })
        .collect()
}

fn quadrado(c: [f32; 2], meio: f32, giro: f32) -> Vec<[f32; 2]> {
    (0..4)
        .map(|i| {
            let a = giro + std::f32::consts::FRAC_PI_2 * i as f32;
            let (s, co) = a.sin_cos();
            [
                c[0] + meio * std::f32::consts::SQRT_2 * co,
                c[1] + meio * std::f32::consts::SQRT_2 * s,
            ]
        })
        .collect()
}

/// As cenas S1–S3 do protocolo (o relatório, «Luz 2D estado da arte»), na vista `r` do mundo: S1 uma
/// luz e um corpo; S2 `n` emissores de 2 cm na treliça (a S2 do Godot: uma `PointLight2D` cada); S3 uma
/// luz e `n` quadrados opacos de 10 cm.
fn cenas_s(r: [f32; 4]) -> Vec<(String, ph2d_light2d_ecs::CenaDeLuz)> {
    use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
    let centro = [(r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0];
    let luz = |c: [f32; 2], raio: f32, l: f32| Emissor::Disco {
        centro: c,
        raio,
        radiancia: [l; 3],
        alcance_m: f32::INFINITY,
    };
    let corpo = |c, meio| Oclusor {
        vertices: quadrado(c, meio, 0.3),
        opacidade: 1.0,
        albedo: [0.5; 3],
    };
    let base = CenaDeLuz {
        luzes: 1,
        ..Default::default()
    };
    let mut out = vec![(
        "S1 uma luz".to_owned(),
        CenaDeLuz {
            emissores: vec![luz(centro, 0.1, 4.0)],
            oclusores: vec![corpo([centro[0] + 1.0, centro[1]], 0.3)],
            ..base.clone()
        },
    )];
    let dentro = [r[0] + 0.5, r[1] + 0.5, r[2] - 0.5, r[3] - 0.5];
    for n in [1, 10, 100, 1_000, 10_000] {
        out.push((
            format!("S2 {n} sprites"),
            CenaDeLuz {
                emissores: espalhados(n, dentro)
                    .into_iter()
                    .map(|c| Emissor::Poligono {
                        vertices: quadrado(c, 0.02, 0.0),
                        radiancia: [4.0; 3],
                        alcance_m: f32::INFINITY,
                    })
                    .collect(),
                ..base.clone()
            },
        ));
    }
    for n in [1, 10, 100, 1_000, 10_000] {
        out.push((
            format!("S2L {n} lâmpadas"),
            CenaDeLuz {
                emissores: espalhados(n, dentro)
                    .into_iter()
                    .map(|c| luz(c, 0.02, 4.0))
                    .collect(),
                ..base.clone()
            },
        ));
    }
    for n in [0, 10, 100, 1_000] {
        out.push((
            format!("S3 {n} oclusores"),
            CenaDeLuz {
                emissores: vec![luz(centro, 0.1, 4.0)],
                oclusores: espalhados(n, dentro)
                    .into_iter()
                    .map(|c| corpo(c, 0.05))
                    .collect(),
                ..base.clone()
            },
        ));
    }
    out
}

/// S1–S3 e S7 do protocolo, NOSSO lado (o produto: o degrau Baixo do «Automático» de fábrica, a fina
/// que cabe, o entorno): por resolução UM solver, as cenas trocadas quadro a quadro em rodadas
/// rotadas — o MÍNIMO, o `loadavg` ao lado. A S4/S5 (penumbra, faísca) são os G3, a S6 o G11, a S8 o
/// G9 e o K9 (rodadas anteriores).
#[test]
#[ignore = "rodada de medição W5e (placa): S1–S3 e S7, o nosso lado contra o Godot"]
fn medida_luz2d_w5e_s() {
    use ph2d_light2d_gpu::Solver;
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    let rodadas: usize = std::env::var("PH2D_W5E_RODADAS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);
    eprintln!(
        "## S. O protocolo S1–S3 · S7 (ms de placa, o mínimo de {rodadas}; nível {}) · loadavg antes {}",
        std::env::var("PH2D_W5E_NIVEL").unwrap_or_else(|_| "Baixa".into()),
        crate::medida_luz2d_w4b::loadavg()
    );
    let nivel = match std::env::var("PH2D_W5E_NIVEL").as_deref() {
        Ok("Media") => NivelDaLuz::Media,
        Ok("Alta") => NivelDaLuz::Alta,
        _ => NivelDaLuz::Baixa,
    };
    let lado = gpu.device.limits().max_texture_dimension_2d;
    eprintln!("| res | cena | vista ms | entorno ms | total ms |");
    eprintln!("|---|---|---|---|---|");
    for (nome, alvo) in [
        ("720p", (1280, 720)),
        RESOLUCOES[0],
        RESOLUCOES[1],
        RESOLUCOES[2],
    ] {
        let a = fabrica(alvo);
        let tetos = ph2d_app_light2d::teto::Tetos {
            lado,
            recusou: None,
        };
        let k =
            ph2d_app_light2d::teto::fina_que_cabe(a, alvo, degrau_do(nivel), tetos).expect("cabe");
        let dg = degrau_do(nivel).com_fina_engrossada(k);
        let g = grade_do_quadro(a, alvo, dg).expect("grade");
        let cfg = Config {
            aritmetica: Config::padrao_para(&gpu.device, g).aritmetica,
            ..ph2d_app_light2d::teto::config_do_quadro(g, a, dg, true)
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
        s.regiao_fina(i, f);
        let mut e = config_do_entorno(a, alvo, dg).map(|c| {
            let mut e = Solver::novo(&gpu, c).expect("entorno");
            e.regiao_fina([0; 2], [0; 2]);
            e
        });
        let cenas = cenas_s(visivel(a, alvo));
        let mut minimo = vec![[f64::INFINITY; 2]; cenas.len()];
        for rodada in 0..rodadas {
            for j in 0..cenas.len() {
                let i = (j + rodada) % cenas.len();
                let sem_ceu = ph2d_light2d_ecs::CenaDeLuz {
                    ceu: [0.0; 3],
                    ..cenas[i].1.clone()
                };
                for _ in 0..QUADROS {
                    if let Some(e) = e.as_mut() {
                        e.quadro(&gpu, &sem_ceu);
                    }
                    s.quadro(&gpu, &cenas[i].1);
                }
                let tv = s.tempos(&gpu).map_or(f64::NAN, |t| t.total());
                let te = e
                    .as_ref()
                    .and_then(|e| e.tempos(&gpu))
                    .map_or(0.0, |t| t.total());
                if tv + te < minimo[i][0] + minimo[i][1] {
                    minimo[i] = [tv, te];
                }
            }
        }
        for ((c, _), m) in cenas.iter().zip(&minimo) {
            eprintln!(
                "| {nome}{} | {c} | {:.2} | {:.2} | {:.2} |",
                if k > 0 {
                    format!(" (fina ×{})", 1 << k)
                } else {
                    String::new()
                },
                m[0],
                m[1],
                m[0] + m[1]
            );
        }
    }
    eprintln!("loadavg depois {}", crate::medida_luz2d_w4b::loadavg());
}

/// S2 por PASSE (1080p, Baixo): quem escala com o número de emissores — o `TemposMs` inteiro do
/// mínimo de 5 leituras de cada cena.
#[test]
#[ignore = "rodada de medição W5e (placa): a S2 passe a passo"]
fn medida_luz2d_w5e_s2_por_passe() {
    use ph2d_light2d_gpu::Solver;
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let alvo = RESOLUCOES[0].1;
    let a = fabrica(alvo);
    let dg = degrau_do(NivelDaLuz::Baixa);
    let g = grade_do_quadro(a, alvo, dg).expect("grade");
    let cfg = Config {
        aritmetica: Config::padrao_para(&gpu.device, g).aritmetica,
        ..ph2d_app_light2d::teto::config_do_quadro(g, a, dg, true)
    };
    let mut s = Solver::novo(&gpu, cfg).expect("solver");
    let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
    s.regiao_fina(i, f);
    eprintln!(
        "## S2 passe a passo (1080p Baixo) · loadavg {}",
        crate::medida_luz2d_w4b::loadavg()
    );
    for (nome, cena) in cenas_s(visivel(a, alvo)) {
        if !nome.starts_with("S2 ") {
            continue;
        }
        let mut melhor: Option<ph2d_light2d_gpu::TemposMs> = None;
        for _ in 0..5 {
            let r = s.quadro(&gpu, &cena);
            if let Some(t) = s.tempos(&gpu)
                && melhor.as_ref().is_none_or(|m| t.total() < m.total())
            {
                melhor = Some(t);
            }
            if nome.ends_with("100 sprites") {
                eprintln!("  relatório C0: {} fora", r.fora.len());
            }
        }
        eprintln!("- {nome}: {:?}", melhor.expect("TIMESTAMP_QUERY"));
    }
}

/// O quadro da luz como o APP o faz (a preferência de fábrica: Alta com o «Automático»), na cena 6 a
/// 1930×1012: o relógio de parede de cada quadro (preparar + gravar + a espera da placa), o custo de
/// placa que o governador lê e o nível que ele escolhe.
#[test]
#[ignore = "rodada de medição W5e (placa): o quadro do app com o Automático"]
fn medida_luz2d_w5e_quadro_do_app() {
    use ph2d_app_light2d::LuzDoQuadro;
    use ph2d_editor_core::luz_edits::PreferenciaDaLuz;
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let alvo = (1930, 1012);
    let a = fabrica(alvo);
    let mut w = bevy_ecs::world::World::new();
    assert!(ph2d_app_light2d::smoke::encenar(&mut w, 3));
    let r = crate::comum::renderer(&gpu);
    let mut l = LuzDoQuadro::new();
    eprintln!(
        "## O quadro do app · loadavg {}",
        crate::medida_luz2d_w4b::loadavg()
    );
    for q in 0..120 {
        l.preferir(PreferenciaDaLuz {
            nivel: NivelDaLuz::Alta,
            automatica: true,
        });
        let t0 = std::time::Instant::now();
        assert!(l.preparar(&w, a, alvo));
        l.gravar(&gpu, &mut ph2d_ecs::PresentWorld::new(), &r);
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
        if q % 10 == 0 || q < 12 {
            eprintln!(
                "q{q}: {:.1} ms de parede · placa {:?} ms · {:?} · fina {:?}",
                t0.elapsed().as_secs_f64() * 1e3,
                l.custo_ms(),
                l.qualidade(),
                l.fina_engrossada()
            );
        }
    }
}
