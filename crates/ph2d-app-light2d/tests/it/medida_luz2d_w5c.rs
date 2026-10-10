//! **A rodada da W5c — o rebote e a cascata holográfica** (`docs/Luz2D/01_plano.md` §2.4 e §7; método
//! `docs/DevOps/MEDIR_VELOCIDADE.md`): um lote, as variantes no MESMO processo (configurações do
//! solver, nunca `const` recompilada), intercaladas com a ordem rodada, vale o MÍNIMO. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5c -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A. o custo de placa e a memória por nível (Low · Medium · High) × variante (a W5b: as cascatas com
//!    a fusão antiga, sem rebote · as cascatas da W5c com o rebote · a holográfica sem e com o rebote,
//!    o produto) × cena (1–4), na vista de fábrica a 1920×1080;
//! B. a convergência do rebote em quadros (a cena 3): o quadro em que cada ponto chega a 5 % e a 1 %
//!    do convergido, por nível;
//! C. o erro contra o gabarito: a cena 3 com a série do rebote, e os vaga-lumes (a luz pequena — o
//!    limite medido da holográfica), por nível, clássica × holográfica;
//! D. o rebote de FORA da grade: a parede vermelha a sair da grade com o pan.

use bevy_ecs::world::World;
use ph2d_app_light2d::{
    captura_do_quadro, fina_do_quadro, grade_do_quadro, qualidade::degrau_do, smoke,
};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, MascaraDaFusao, Solver, TemposMs};
use ph2d_light2d_ref::{OpcoesDoRebote, Rebote};
use ph2d_render::Camera2d;
use vello::kurbo::Affine;

use crate::comum::{fluencia_lida, gpu};
use crate::medida_luz2d_w4b::loadavg;

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 7;
const NIVEIS: [(&str, NivelDaLuz); 3] = [
    ("Low", NivelDaLuz::Baixa),
    ("Medium", NivelDaLuz::Media),
    ("High", NivelDaLuz::Alta),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Variante {
    /// A W5b: as cascatas clássicas com a fusão antiga, sem rebote.
    W5b,
    /// As cascatas clássicas com a fusão da W5c e o rebote.
    Classica,
    /// A cascata holográfica sem rebote.
    HoloSemRebote,
    /// O produto da W5c: a cascata holográfica com o rebote.
    Holo,
}

fn fabrica() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

/// A configuração do PRODUTO (a do `LuzDoQuadro::gravar`) para o nível, com a variante.
fn config(gpu: &GpuContext, nivel: NivelDaLuz, v: Variante) -> Config {
    let a = fabrica();
    let d = degrau_do(nivel);
    let holo = matches!(v, Variante::Holo | Variante::HoloSemRebote);
    let base = Config {
        fina: fina_do_quadro(a, d),
        captura: captura_do_quadro(a, d),
        holografica: holo.then_some(HOLOGRAFICA_PADRAO),
        rebote: matches!(v, Variante::Classica | Variante::Holo),
        ..Config::padrao_para(&gpu.device, grade_do_quadro(a, FHD, d).expect("grade"))
    };
    if v == Variante::W5b {
        Config {
            mascara_da_fusao: MascaraDaFusao::Nenhuma,
            fronteira_na_face: false,
            ..base
        }
    } else {
        base
    }
}

fn solver(gpu: &GpuContext, cfg: Config) -> Solver {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    let (i, f) = ph2d_app_light2d::regiao_da_janela(s.grade_da_fluencia(), fabrica(), FHD);
    s.regiao_fina(i, f);
    s
}

fn cena(n: u32) -> CenaDeLuz {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, n));
    cena_de_luz(&w)
}

#[test]
#[ignore = "a rodada da W5c: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w5c() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W5c · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
        loadavg()
    );
    custo(&gpu);
    convergencia(&gpu);
    erro(&gpu);
    fora_da_grade(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// A. O custo de placa e a memória.
fn custo(gpu: &GpuContext) {
    let cenas: Vec<(&str, CenaDeLuz)> = (1..=4)
        .map(|n| {
            (
                ["cena 1", "cena 2", "cena 3 (rebote)", "cena 4"][n as usize - 1],
                cena(n),
            )
        })
        .collect();
    let mut variantes: Vec<(String, Solver)> = Vec::new();
    for (nome, nivel) in NIVEIS {
        for v in [
            Variante::W5b,
            Variante::Classica,
            Variante::HoloSemRebote,
            Variante::Holo,
        ] {
            variantes.push((
                format!("{nome} · {v:?}"),
                solver(gpu, config(gpu, nivel, v)),
            ));
        }
    }
    let n = variantes.len() * cenas.len();
    let mut minimo: Vec<Option<TemposMs>> = vec![None; n];
    for rodada in 0..RODADAS {
        for k in 0..n {
            let i = (k + rodada * 5) % n;
            let (v, c) = (i / cenas.len(), i % cenas.len());
            let s = &mut variantes[v].1;
            // Quatro quadros: o rebote (um por quadro) e os tempos de placa assentam.
            for _ in 0..4 {
                s.quadro(gpu, &cenas[c].1);
            }
            let t = s.tempos(gpu).expect("TIMESTAMP_QUERY");
            if minimo[i].as_ref().is_none_or(|m| t.total() < m.total()) {
                minimo[i] = Some(t);
            }
        }
    }
    eprintln!(
        "\n## A. Custo de placa (ms, MÍNIMO de {RODADAS} rodadas intercaladas), 1920×1080, vista de fábrica"
    );
    eprintln!(
        "| variante | {} | cascatas · rebote (C0 + C3) na cena 3 | MB alocados |",
        cenas.iter().map(|c| c.0).collect::<Vec<_>>().join(" | ")
    );
    eprintln!("|---|{}---|---|", "---|".repeat(cenas.len()));
    for (v, (nome, s)) in variantes.iter().enumerate() {
        let t = |c: usize| minimo[v * cenas.len() + c].clone().expect("medido");
        let tot: Vec<String> = (0..cenas.len())
            .map(|c| format!("{:.2}", t(c).total()))
            .collect();
        let t3 = t(2);
        eprintln!(
            "| {nome} | {} | {:.2} · {:.3} + {:.3} | {:.1} |",
            tot.join(" | "),
            t3.c2_total(),
            t3.c0_rebote,
            t3.c3_rebote,
            s.pass_list().bytes_alocados() as f64 / 1e6
        );
    }
    eprintln!(
        "(orçamento do governador: {:.2} ms)",
        ph2d_app_light2d::qualidade::ORCAMENTO_MS
    );
}

/// Os pontos da cena 3: junto à vermelha, à verde, ao chão, o meio e junto ao teto.
fn pontos_da_cena_3() -> [[f64; 2]; 5] {
    let [sw, sh] = smoke::SALA_DO_REBOTE.map(f64::from);
    let (x, y) = (0.5 * sw, 0.5 * sh);
    [
        [-x + 0.3, 0.0],
        [x - 0.3, 0.0],
        [-1.5, -y + 0.15],
        [0.0, -0.8],
        [1.5, y - 0.15],
    ]
}

/// B. A convergência do rebote em quadros (um rebote por quadro).
fn convergencia(gpu: &GpuContext) {
    eprintln!(
        "\n## B. Convergência do rebote (cena 3): o quadro em que o PIOR ponto fica a 5 % · 1 % do convergido (120 quadros)"
    );
    let c3 = cena(3);
    let pts = pontos_da_cena_3();
    for (nome, nivel) in NIVEIS {
        let mut s = solver(gpu, config(gpu, nivel, Variante::Holo));
        let mut historia: Vec<Vec<f64>> = Vec::new();
        for _ in 0..120 {
            s.quadro(gpu, &c3);
            let f = fluencia_lida(gpu, &s);
            historia.push(pts.iter().map(|&p| f.amostra(p)).collect());
        }
        let fim = historia.last().expect("120 quadros").clone();
        let quadro_a = |tol: f64| {
            historia
                .iter()
                .position(|h| {
                    h.iter()
                        .zip(&fim)
                        .all(|(a, b)| (a - b).abs() <= tol * b.abs())
                })
                .map_or(0, |q| q + 1)
        };
        eprintln!(
            "- {nome}: 5 % no quadro {} · 1 % no quadro {}",
            quadro_a(0.05),
            quadro_a(0.01)
        );
    }
}

/// C. O erro contra o gabarito.
fn erro(gpu: &GpuContext) {
    eprintln!(
        "\n## C. Erro contra o gabarito (o pior ponto, do canal verde, relativo à luz LOCAL)"
    );
    let c3 = cena(3);
    let serie = Rebote::resolver(
        &c3,
        OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 200,
        },
    );
    let pts = pontos_da_cena_3();
    let gab: Vec<f64> = pts
        .iter()
        .map(|&p| 2.0 * std::f64::consts::PI * serie.luz_em(p)[1])
        .collect();
    let c2 = cena(2);
    // Os vaga-lumes: no chão, junto a cada enxame (fora dele) — a luz de MUITAS luzes pequenas.
    let vaga: Vec<[f64; 2]> = (0..smoke::ENXAMES)
        .map(|k| {
            let c = [
                f64::from(smoke::CENTROS_DOS_ENXAMES[0][k]),
                f64::from(smoke::CENTROS_DOS_ENXAMES[1][k]),
            ];
            [c[0] - 1.4, c[1] + 0.3]
        })
        .collect();
    let gab2: Vec<f64> = vaga
        .iter()
        .map(|&p| ph2d_light2d_ref::fluencia(&c2, p)[1])
        .collect();
    let casos: Vec<(String, NivelDaLuz, Variante)> = NIVEIS
        .iter()
        .map(|&(n, l)| (format!("{n} clássica"), l, Variante::Classica))
        .chain(
            NIVEIS
                .iter()
                .map(|&(n, l)| (format!("{n} holográfica"), l, Variante::Holo)),
        )
        .collect();
    for (nome, nivel, v) in casos {
        let mut s = solver(gpu, config(gpu, nivel, v));
        for _ in 0..120 {
            s.quadro(gpu, &c3);
        }
        let f = fluencia_lida(gpu, &s);
        let e3 = pts
            .iter()
            .zip(&gab)
            .map(|(&p, g)| (f.amostra(p) - g).abs() / g)
            .fold(0.0, f64::max);
        let mut s = solver(gpu, config(gpu, nivel, v));
        s.quadro(gpu, &c2);
        let f = fluencia_lida(gpu, &s);
        let e2: Vec<String> = vaga
            .iter()
            .zip(&gab2)
            .map(|(&p, g)| format!("{:+.3}", (f.amostra(p) - g) / g))
            .collect();
        eprintln!(
            "- {nome}: cena 3 (rebote) pior {e3:.4} · vaga-lumes {}",
            e2.join(" ")
        );
    }
}

/// D. O rebote de FORA da grade: a cena 3 vista com a câmera deslocada para a direita até a parede
/// vermelha sair da grade (a margem é o desenho das cascatas; o mundo fora da grade da W5b não tem
/// albedo). Um ponto da sala junto à borda esquerda da vista, contra o gabarito (que vê a sala toda).
fn fora_da_grade(gpu: &GpuContext) {
    eprintln!(
        "\n## D. O rebote de fora da grade (cena 3, a câmera deslocada; o ponto junto à borda esquerda da vista)"
    );
    let c3 = cena(3);
    let serie = Rebote::resolver(
        &c3,
        OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 200,
        },
    );
    for centro_x in [-2.0_f32, -1.0, 0.0, 1.0, 2.0] {
        // A vista de 2 m de altura (a do dono na W5b): a grade cobre poucos metros à volta do centro.
        let a = Camera2d::new([centro_x, 0.0], 2.0)
            .world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        let d = degrau_do(NivelDaLuz::Media);
        let g = grade_do_quadro(a, FHD, d).expect("grade");
        let cfg = Config {
            fina: fina_do_quadro(a, d),
            captura: captura_do_quadro(a, d),
            holografica: Some(HOLOGRAFICA_PADRAO),
            rebote: true,
            ..Config::padrao_para(&gpu.device, g)
        };
        let mut s = Solver::novo(gpu, cfg).expect("solver");
        let (i, f) = ph2d_app_light2d::regiao_da_janela(s.grade_da_fluencia(), a, FHD);
        s.regiao_fina(i, f);
        for _ in 0..80 {
            s.quadro(gpu, &c3);
        }
        let fl = fluencia_lida(gpu, &s);
        // A borda esquerda da vista no mundo, e um ponto da sala 0,3 m para dentro dela.
        let esquerda = (a.inverse() * vello::kurbo::Point::new(0.0, 540.0)).x;
        let p = [esquerda.max(-2.7) + 0.3, 0.0];
        let gab = 2.0 * std::f64::consts::PI * serie.luz_em(p)[0];
        let fora = g.origem_m[0] > -3.2;
        eprintln!(
            "- câmera em x = {centro_x}: a grade começa em {:.2} m (a parede vermelha {}) · ponto {:.2} · vermelho {:+.3} do gabarito",
            g.origem_m[0],
            if fora { "FORA dela" } else { "dentro" },
            p[0],
            (fl.amostra_canal(p, 0) - gab) / gab
        );
    }
}

/// Sonda (à mão): a cena 3 ponto a ponto, por nível e fusão (com o rebote).
#[test]
#[ignore = "sonda: imprime"]
fn sonda_cena_3_ponto_a_ponto() {
    let Some(gpu) = gpu() else { return };
    let c3 = cena(3);
    let serie = Rebote::resolver(
        &c3,
        OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 200,
        },
    );
    let pts = pontos_da_cena_3();
    let gab: Vec<[f64; 3]> = pts
        .iter()
        .map(|&p| serie.luz_em(p).map(|l| 2.0 * std::f64::consts::PI * l))
        .collect();
    let sem: Vec<f64> = pts
        .iter()
        .map(|&p| ph2d_light2d_ref::fluencia(&c3, p)[1])
        .collect();
    eprintln!(
        "gabarito (verde): {:?} · sem rebote {:?}",
        gab.iter().map(|g| g[1]).collect::<Vec<_>>(),
        sem
    );
    for (nome, nivel) in NIVEIS {
        for antiga in [false, true] {
            let mut cfg = config(&gpu, nivel, Variante::Classica);
            if antiga {
                // (sonda: a captura e a grade fina da grade das cascatas)
                cfg.captura = 1;
                cfg.fina = 1;
            }
            let mut s = solver(&gpu, cfg);
            for _ in 0..120 {
                s.quadro(&gpu, &c3);
            }
            let f = fluencia_lida(&gpu, &s);
            let e: Vec<String> = pts
                .iter()
                .zip(&gab)
                .map(|(&p, g)| format!("{:+.3}", (f.amostra(p) - g[1]) / g[1]))
                .collect();
            eprintln!(
                "{nome} captura 1 {antiga}: {} (texel {:.4} m)",
                e.join(" "),
                s.config().grade.texel_m
            );
        }
    }
}

/// Sonda (à mão): a cena 3 no Alto clássico, sem a caixa · sem a ambiente · sem as duas.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_cena_3_ablacao() {
    let Some(gpu) = gpu() else { return };
    let base = cena(3);
    let variantes: Vec<(&str, CenaDeLuz)> = vec![
        ("inteira", base.clone()),
        ("sem a caixa", {
            let mut c = base.clone();
            c.oclusores.pop();
            c
        }),
        ("sem a ambiente", {
            let mut c = base.clone();
            c.ceu = [0.0; 3];
            c
        }),
        ("sem as duas", {
            let mut c = base.clone();
            c.oclusores.pop();
            c.ceu = [0.0; 3];
            c
        }),
    ];
    let pts = pontos_da_cena_3();
    for (nome, c) in variantes {
        let serie = Rebote::resolver(
            &c,
            OpcoesDoRebote {
                passo_m: 0.02,
                rebotes: 200,
            },
        );
        for v in [Variante::Classica, Variante::Holo] {
            let nivel = NivelDaLuz::Alta;
            let mut s = solver(&gpu, config(&gpu, nivel, v));
            for _ in 0..120 {
                s.quadro(&gpu, &c);
            }
            let f = fluencia_lida(&gpu, &s);
            let e: Vec<String> = pts
                .iter()
                .map(|&p| {
                    let g = 2.0 * std::f64::consts::PI * serie.luz_em(p)[1];
                    format!("{:+.3}", (f.amostra(p) - g) / g)
                })
                .collect();
            eprintln!("{nome} · {nivel:?} {v:?}: {}", e.join(" "));
        }
    }
}

/// Sonda (à mão): a sala do G9 (`ph2d-light2d-gpu`, centrada) no solver do produto, e a da cena 3
/// variando UMA coisa de cada vez (as cores puras · a lâmpada da G9 · as dimensões da G9).
#[test]
#[ignore = "sonda: imprime"]
fn sonda_sala_g9_no_produto() {
    use ph2d_light2d_ecs::{Emissor, Oclusor};
    let Some(gpu) = gpu() else { return };
    let caixa = |x0: f32, y0: f32, x1: f32, y1: f32, albedo: [f32; 3]| Oclusor {
        vertices: vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
        opacidade: 1.0,
        albedo,
    };
    // Uma sala interior w × h centrada, paredes de e, lâmpada (y, raio, intensidade-fluência a 1 m).
    let sala = |w: f32, h: f32, e: f32, lamp: (f32, f32), cores: ([f32; 3], [f32; 3], [f32; 3])| {
        let (x, y) = (0.5 * w, 0.5 * h);
        CenaDeLuz {
            emissores: vec![Emissor::Disco {
                centro: [0.0, lamp.0],
                raio: lamp.1,
                radiancia: [1.0; 3],
                alcance_m: f32::INFINITY,
            }],
            oclusores: vec![
                caixa(-x - e, -y - e, x + e, -y, cores.0),
                caixa(-x - e, y, x + e, y + e, cores.0),
                caixa(-x - e, -y, -x, y, cores.1),
                caixa(x, -y, x + e, y, cores.2),
            ],
            luzes: 1,
            ..Default::default()
        }
    };
    let puras = ([1.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let da_cena = ([0.95; 3], [0.9, 0.1, 0.1], [0.1, 0.8, 0.15]);
    let _ = (puras, da_cena);
    let casos: Vec<(String, CenaDeLuz)> = [
        (4.0, 2.5, 0.2),
        (4.0, 2.5, 0.3),
        (6.0, 3.6, 0.2),
        (6.0, 3.6, 0.3),
        (5.0, 3.0, 0.2),
        (6.0, 2.5, 0.2),
        (4.0, 3.6, 0.2),
        (6.01, 3.61, 0.3),
    ]
    .into_iter()
    .map(|(w, h, e)| {
        (
            format!("{w}×{h} e {e}"),
            sala(w, h, e, (0.5 * h - 0.3, 0.1), puras),
        )
    })
    .collect();
    for (nome, c) in casos {
        let serie = Rebote::resolver(
            &c,
            OpcoesDoRebote {
                passo_m: 0.02,
                rebotes: 300,
            },
        );
        let pts = [[-1.5, 0.0], [1.5, 0.0], [0.0, -0.9], [0.0, 0.0]];
        let mut linha = Vec::new();
        for v in [Variante::Classica, Variante::Holo] {
            let mut s = solver(&gpu, config(&gpu, NivelDaLuz::Alta, v));
            for _ in 0..120 {
                s.quadro(&gpu, &c);
            }
            let f = fluencia_lida(&gpu, &s);
            let e = pts
                .iter()
                .map(|&p| {
                    let g = serie.fluencia(p)[1];
                    (f.amostra(p) - g) / g
                })
                .fold(0.0_f64, |m, x| if x.abs() > m.abs() { x } else { m });
            linha.push(format!("{v:?}: {e:+.3}"));
        }
        eprintln!("{nome}: {}", linha.join(" · "));
    }
}
