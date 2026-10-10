//! **A rodada da W4b — a grade FINA da luz direta** (`docs/DevOps/MEDIR_VELOCIDADE.md`): um lote, as
//! variantes no MESMO processo (configurações do solver, nunca `const` recompilada), intercaladas
//! com a ordem rodada, vale o MÍNIMO. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w4b -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A. o custo de placa por lâmpada (0 · 1 · 4 · 8) a 1920×1080, Low e Medium, por célula fina
//!    (sem · 2 px · 1 px) e refino (bordas · tudo);
//! B. a escada (a régua `a_escada_da_sombra`) por célula fina;
//! C. quantos texels da grade caem sob os PAINÉIS (o canvas de fábrica com as duas colunas);
//! D. a faixa escura na face ACESA da parede da cena 1 (o interior do opaco recebe 0).

use bevy_ecs::world::World;
use ph2d_app_light2d::{
    Degrau, fina_do_quadro, grade_do_quadro, qualidade::degrau_do, regiao_da_janela, smoke,
};
use ph2d_ecs::{GlobalTransform, Transform};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Bordas, Config, REFINO_PADRAO, Refino, Solver, TemposMs};
use ph2d_render::Camera2d;
use vello::kurbo::{Affine, Point};

use crate::comum::{Fluencia, fluencia_lida, gpu};

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 9;
/// O canvas de fábrica a 1920×1080 com as duas colunas abertas (`draw_area` de
/// `ph2d-editor-core/src/screens/layout.rs`: hierarquia 308 + 4, inspector 304 + 4, barra de menus
/// 22), em píxeis da janela.
const CANVAS: [f64; 4] = [312.0, 22.0, 1612.0, 1080.0];

fn afim() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

/// O degrau do nível com a célula fina `fino` (`None` = sem grade fina: a das cascatas).
fn degrau(nivel: NivelDaLuz, fino: Option<f64>) -> Degrau {
    let d = degrau_do(nivel);
    Degrau {
        fino_px: fino.unwrap_or(d.c0_px),
        ..d
    }
}

/// A configuração do PRODUTO para `degrau` (a do `LuzDoQuadro::gravar`), com o refino `refino`.
fn config(gpu: &GpuContext, degrau: Degrau, refino: Refino) -> Config {
    let g = grade_do_quadro(afim(), FHD, degrau).expect("grade");
    Config {
        fina: fina_do_quadro(afim(), degrau),
        refino,
        ..Config::padrao_para(&gpu.device, g)
    }
}

/// O solver do PRODUTO: a configuração e a região da janela.
fn solver(gpu: &GpuContext, cfg: Config) -> Solver {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    let (a, b) = regiao_da_janela(s.grade_da_fluencia(), afim(), FHD);
    s.regiao_fina(a, b);
    s
}

/// As variantes do refino que a rodada compara: o padrão e cada escolha dele desfeita.
fn variantes_do_refino() -> Vec<(&'static str, Refino)> {
    let Refino::Bordas(p) = REFINO_PADRAO else {
        unreachable!("o padrão é pelas bordas")
    };
    let b = |nome, b: Bordas| (nome, Refino::Bordas(b));
    vec![
        b("padrão", p),
        b(
            "partida (0,25 · joelho · passo fino · 8+5)",
            Bordas {
                limiar: 0.25,
                joelho: true,
                visibilidade_no_passo_fino: true,
                raios: 8,
                ..p
            },
        ),
        b("limiar 0,25", Bordas { limiar: 0.25, ..p }),
        b("com joelho", Bordas { joelho: true, ..p }),
        b(
            "sem certificar",
            Bordas {
                certificar: false,
                ..p
            },
        ),
        b(
            "visibilidade no passo fino",
            Bordas {
                visibilidade_no_passo_fino: true,
                ..p
            },
        ),
        b("raios 8+5", Bordas { raios: 8, ..p }),
        b(
            "raios 2+6",
            Bordas {
                raios: 2,
                bissecoes: 6,
                ..p
            },
        ),
        ("tudo", Refino::Tudo),
    ]
}

/// A cena 1 com `n` lâmpadas (as da `medida_custo_por_lampada`, espalhadas) e a ambiente.
fn cena_com(n: usize) -> CenaDeLuz {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 1));
    let fora: Vec<_> = w
        .query::<(bevy_ecs::entity::Entity, &ph2d_light2d_ecs::Luz)>()
        .iter(&w)
        .filter(|(_, l)| l.tipo == ph2d_light2d_ecs::TipoDeLuz::Ponto)
        .map(|(e, _)| e)
        .collect();
    for e in fora {
        w.despawn(e);
    }
    for k in 0..n {
        let mut t = Transform::IDENTITY;
        t.translation.x = -4.0 + 0.5 * k as f32;
        t.translation.y = if k % 2 == 0 { 1.8 } else { -1.8 };
        w.spawn((
            ph2d_light2d_ecs::Luz {
                intensidade: 3.0,
                ..ph2d_light2d_ecs::Luz::default()
            },
            t,
            GlobalTransform::from_transform(t),
        ));
    }
    cena_de_luz(&w)
}

#[test]
#[ignore = "a rodada da W4b: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w4b() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W4b · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
        loadavg()
    );
    custo(&gpu);
    escada(&gpu);
    paineis();
    faixa_da_parede(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// A. O custo de placa por lâmpada.
fn custo(gpu: &GpuContext) {
    let lampadas = [0_usize, 1, 4, 8];
    let cenas: Vec<CenaDeLuz> = lampadas.iter().map(|&n| cena_com(n)).collect();
    let mut variantes: Vec<(String, Solver)> = Vec::new();
    for (nome, nivel) in [("Low", NivelDaLuz::Baixa), ("Medium", NivelDaLuz::Media)] {
        let cfg = config(gpu, degrau(nivel, None), REFINO_PADRAO);
        variantes.push((format!("{nome} · sem fina"), solver(gpu, cfg)));
        for (rnome, refino) in variantes_do_refino() {
            let cfg = config(gpu, degrau(nivel, Some(1.0)), refino);
            variantes.push((
                format!("{nome} · fina 1 px (×{}) · {rnome}", cfg.fina),
                solver(gpu, cfg),
            ));
        }
    }
    let n = variantes.len() * cenas.len();
    let mut minimo: Vec<Option<TemposMs>> = vec![None; n];
    for rodada in 0..RODADAS {
        for k in 0..n {
            let i = (k + rodada * 7) % n;
            let (v, c) = (i / cenas.len(), i % cenas.len());
            let s = &mut variantes[v].1;
            s.quadro(gpu, &cenas[c]);
            s.quadro(gpu, &cenas[c]);
            let t = s.tempos(gpu).expect("TIMESTAMP_QUERY");
            if minimo[i].as_ref().is_none_or(|m| t.total() < m.total()) {
                minimo[i] = Some(t);
            }
        }
    }
    eprintln!("\n## A. Custo de placa (ms, MÍNIMO de {RODADAS} rodadas intercaladas), 1920×1080");
    eprintln!(
        "| variante | 0 lâmpadas | 1 | 4 | 8 | ms/lâmpada | C3 vis · fina (8) | cascatas (8) | MB alocados |"
    );
    eprintln!("|---|---|---|---|---|---|---|---|---|");
    for (v, (nome, s)) in variantes.iter().enumerate() {
        let t = |c: usize| minimo[v * cenas.len() + c].clone().expect("medido");
        let tot: Vec<f64> = (0..cenas.len()).map(|c| t(c).total()).collect();
        let t8 = t(3);
        eprintln!(
            "| {nome} | {:.2} | {:.2} | {:.2} | {:.2} | {:.3} | {:.2} · {:.2} | {:.2} | {:.1} |",
            tot[0],
            tot[1],
            tot[2],
            tot[3],
            (tot[3] - tot[0]) / 8.0,
            t8.c3_visibilidade,
            t8.c3_fina,
            t8.c0 + t8.rc(),
            s.pass_list().bytes_alocados() as f64 / 1e6
        );
    }
}

/// B. A escada por célula fina (a régua do gate, no caminho de configuração do produto).
fn escada(gpu: &GpuContext) {
    eprintln!("\n## B. Escada (cena 4, lâmpada pequena): degrau rms (px) · média (px)");
    eprintln!("| variante | 10 % | 50 % | 90 % | média 50 % |");
    eprintln!("|---|---|---|---|---|");
    let (com, sem) = crate::a_escada_da_sombra::cena4_com_e_sem();
    let (cc, cs) = (cena_de_luz(&com), cena_de_luz(&sem));
    for (nome, nivel) in [
        ("Low", NivelDaLuz::Baixa),
        ("Medium", NivelDaLuz::Media),
        ("High", NivelDaLuz::Alta),
    ] {
        let mut vs: Vec<(String, Degrau, Refino)> =
            vec![("sem fina".into(), degrau(nivel, None), REFINO_PADRAO)];
        for (rnome, refino) in variantes_do_refino() {
            vs.push((
                format!("fina 1 px · {rnome}"),
                degrau(nivel, Some(1.0)),
                refino,
            ));
        }
        for (vnome, d, refino) in vs {
            let cfg = config(gpu, d, refino);
            let flu = |c: &CenaDeLuz| -> Fluencia {
                let mut s = solver(gpu, cfg);
                s.quadro(gpu, c);
                fluencia_lida(gpu, &s)
            };
            let e = crate::a_escada_da_sombra::escada_de(&flu(&cc), &flu(&cs));
            eprintln!(
                "| {nome} · {vnome} | {:.3} | {:.3} | {:.3} | {:.2} |",
                e[0].degrau_px, e[1].degrau_px, e[2].degrau_px, e[1].media_px
            );
        }
    }
}

/// C. A fração da grade cujo texel cai FORA do canvas (sob os painéis), por nível.
fn paineis() {
    eprintln!("\n## C. Texels fora da janela e sob os painéis (canvas {CANVAS:?} de 1920×1080)");
    eprintln!(
        "| nível | grade | fora da janela | fora do canvas | grade fina (1 px) | fora da janela | fora do canvas |"
    );
    eprintln!("|---|---|---|---|---|---|---|");
    let afim = afim();
    let janela = [0.0, 0.0, f64::from(FHD.0), f64::from(FHD.1)];
    let fora = |g: ph2d_light2d_gpu::Grade, r: [f64; 4]| {
        let mut n = 0u64;
        for j in 0..g.altura {
            for i in 0..g.largura {
                let p = afim * Point::new(g.centro(i, j)[0], g.centro(i, j)[1]);
                if !(r[0]..r[2]).contains(&p.x) || !(r[1]..r[3]).contains(&p.y) {
                    n += 1;
                }
            }
        }
        100.0 * n as f64 / (f64::from(g.largura) * f64::from(g.altura))
    };
    for (nome, nivel) in [
        ("Low", NivelDaLuz::Baixa),
        ("Medium", NivelDaLuz::Media),
        ("High", NivelDaLuz::Alta),
    ] {
        let d = degrau(nivel, Some(1.0));
        let g = grade_do_quadro(afim, FHD, d).expect("grade");
        let cfg = Config {
            fina: fina_do_quadro(afim, d),
            ..Config::padrao(g)
        };
        let (ge, gf) = (cfg.grade_efetiva(), cfg.grade_da_fluencia());
        eprintln!(
            "| {nome} | {} | {:.1} % | {:.1} % | {} | {:.1} % | {:.1} % |",
            ge.largura * ge.altura,
            fora(ge, janela),
            fora(ge, CANVAS),
            gf.largura * gf.altura,
            fora(gf, janela),
            fora(gf, CANVAS)
        );
    }
}

/// D. A faixa escura junto à face ACESA da parede da cena 1 (`x = −0,95`): a razão placa ÷ gabarito
/// em faixas de ¼ px a partir da face, média e mínimo ao longo dela.
fn faixa_da_parede(gpu: &GpuContext) {
    eprintln!("\n## D. Faixa escura na face acesa da parede (cena 1): placa ÷ gabarito");
    eprintln!(
        "| variante | largura < 0,9 (px) | mínimo da média | desvio ao longo da face a 2 px |"
    );
    eprintln!("|---|---|---|---|");
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 1));
    let cena = cena_de_luz(&w);
    let [a, b, ..] = afim().as_coeffs();
    let ppm = a.hypot(b);
    let face = -0.95_f64;
    let ys: Vec<f64> = (0..)
        .map(|k| -0.1 + f64::from(k) / ppm)
        .take_while(|&y| y <= 1.3)
        .collect();
    let gab: Vec<Vec<f64>> = (0..80)
        .map(|k| {
            let x = face - (0.25 + 0.25 * f64::from(k)) / ppm;
            ys.iter()
                .map(|&y| ph2d_light2d_ref::fluencia(&cena, [x, y])[1])
                .collect()
        })
        .collect();
    for (nome, nivel) in [
        ("Low", NivelDaLuz::Baixa),
        ("Medium", NivelDaLuz::Media),
        ("High", NivelDaLuz::Alta),
    ] {
        for (fnome, fino) in [("sem fina", None), ("fina 1 px", Some(1.0))] {
            let cfg = config(gpu, degrau(nivel, fino), REFINO_PADRAO);
            let mut s = solver(gpu, cfg);
            s.quadro(gpu, &cena);
            let f = fluencia_lida(gpu, &s);
            let mut largura = 0.0;
            let mut minimo = f64::INFINITY;
            let mut desvio2 = 0.0;
            for (k, g) in gab.iter().enumerate() {
                let d = 0.25 + 0.25 * k as f64;
                let x = face - d / ppm;
                let r: Vec<f64> = ys
                    .iter()
                    .zip(g)
                    .map(|(&y, &gy)| f.amostra([x, y]) / gy.max(1e-9))
                    .collect();
                let m = r.iter().sum::<f64>() / r.len() as f64;
                minimo = minimo.min(m);
                if m < 0.9 {
                    largura = d;
                }
                if (d - 2.0).abs() < 1e-9 {
                    desvio2 =
                        (r.iter().map(|x| (x - m).powi(2)).sum::<f64>() / r.len() as f64).sqrt();
                }
            }
            eprintln!(
                "| {nome} · {fnome} (×{}) | {largura:.2} | {minimo:.3} | {desvio2:.4} |",
                cfg.fina
            );
        }
    }
}
