//! **A rodada da W5c — o rebote e o Ultra** (`docs/Luz2D/01_plano.md` §2.4 e §7; método
//! `docs/DevOps/MEDIR_VELOCIDADE.md`): um lote, as variantes no MESMO processo (configurações do
//! solver, nunca `const` recompilada), intercaladas com a ordem rodada, vale o MÍNIMO. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5c -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A. o custo de placa e a memória por nível (Low · Medium · High · Ultra) × variante (a W5b: a fusão
//!    antiga, sem rebote · a W5c sem rebote · a W5c) × cena (1–4), na vista de fábrica a 1920×1080;
//! B. a convergência do rebote em quadros (a cena 3): o quadro em que cada ponto chega a 5 % e a 1 %
//!    do convergido, por nível;
//! C. o erro contra o gabarito: a cena 3 com a série do rebote, e os vaga-lumes (a luz pequena — o
//!    limite medido do Ultra), por nível.

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
const NIVEIS: [(&str, NivelDaLuz); 4] = [
    ("Low", NivelDaLuz::Baixa),
    ("Medium", NivelDaLuz::Media),
    ("High", NivelDaLuz::Alta),
    ("Ultra", NivelDaLuz::Ultra),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Variante {
    /// A fusão de antes (sem a máscara pelo fim, sem a fronteira na face), sem rebote.
    W5b,
    /// A fusão da W5c, sem rebote.
    SemRebote,
    /// O produto da W5c.
    W5c,
}

fn fabrica() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

/// A configuração do PRODUTO (a do `LuzDoQuadro::gravar`) para o nível, com a variante.
fn config(gpu: &GpuContext, nivel: NivelDaLuz, v: Variante) -> Config {
    let a = fabrica();
    let d = degrau_do(nivel);
    let base = Config {
        fina: fina_do_quadro(a, d),
        captura: captura_do_quadro(a, d),
        holografica: d.holografica.then_some(HOLOGRAFICA_PADRAO),
        rebote: v == Variante::W5c,
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
        for v in [Variante::W5b, Variante::SemRebote, Variante::W5c] {
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

/// Os pontos da cena 3: junto à vermelha, à verde, ao chão no meio, e as faces da caixa.
fn pontos_da_cena_3() -> [[f64; 2]; 5] {
    let [sw, sh] = smoke::SALA_DO_REBOTE.map(f64::from);
    let (x, y) = (0.5 * sw, 0.5 * sh);
    [
        [-x + 0.3, 0.0],
        [x - 0.3, 0.0],
        [-1.5, -y + 0.15],
        [-0.45, -y + 0.5],
        [0.45, -y + 0.5],
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
        let mut s = solver(gpu, config(gpu, nivel, Variante::W5c));
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
    for (nome, nivel) in NIVEIS {
        let mut s = solver(gpu, config(gpu, nivel, Variante::W5c));
        for _ in 0..120 {
            s.quadro(gpu, &c3);
        }
        let f = fluencia_lida(gpu, &s);
        let e3 = pts
            .iter()
            .zip(&gab)
            .map(|(&p, g)| (f.amostra(p) - g).abs() / g)
            .fold(0.0, f64::max);
        let mut s = solver(gpu, config(gpu, nivel, Variante::W5c));
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
