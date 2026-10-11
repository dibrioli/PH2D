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
