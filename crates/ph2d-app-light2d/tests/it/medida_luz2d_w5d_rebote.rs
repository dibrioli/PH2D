//! **A rodada da W5d, bloco B — o rebote** (irmão de `medida_luz2d_w5d.rs` pelo tecto de 700 LOC, W5e):
//! a convergência e o rebote de fora da grade (as réguas da W5c), o entorno, o preço dele e os rebotes
//! por quadro. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5d_rebote -- --ignored --nocapture --test-threads=1
//! ```

use bevy_ecs::world::World;
use ph2d_app_light2d::{
    captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela, smoke,
};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};
use ph2d_render::Camera2d;
use vello::kurbo::Point;

use crate::comum::{fluencia_lida, gpu};
use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5d::*;

/// B. O rebote (a linha de base e as variantes): a convergência em quadros (K9) e o rebote de FORA
/// da grade — as réguas da W5c, por `PH2D_W5D_REBOTE=1`.
#[test]
#[ignore = "a rodada da W5d, bloco B: corre-se à mão"]
fn medida_luz2d_w5d_rebote() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!("# Rodada W5d · rebote · loadavg antes {}", loadavg());
    crate::medida_luz2d_w5c::convergencia(&gpu);
    crate::medida_luz2d_w5c::fora_da_grade(&gpu);
    entorno(&gpu);
    custo_do_entorno(&gpu);
    rebotes_por_quadro(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// B1. O preço do entorno por nível, na vista de fábrica a 1920×1080 (cenas 3 e 5): o tempo de placa
/// (o MÍNIMO de 7 rodadas intercaladas) da vista e do entorno, e a memória de cada um.
fn custo_do_entorno(gpu: &GpuContext) {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_app_light2d::{ENTORNO_PADRAO, MARGEM_DO_ENTORNO};
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    eprintln!("\n## B1. O preço do entorno (ms, o mínimo de 7; MB alocados)");
    let a = vista(1.0);
    let cenas: Vec<CenaDeLuz> = [3, 5]
        .iter()
        .map(|&n| {
            let mut w = World::new();
            assert!(smoke::encenar(&mut w, n));
            cena_de_luz(&w)
        })
        .collect();
    for (nome, nivel) in [
        ("Low", NivelDaLuz::Baixa),
        ("Medium", NivelDaLuz::Media),
        ("High", NivelDaLuz::Alta),
    ] {
        let d = degrau_do(nivel);
        let g = grade_do_quadro(a, FHD, d).expect("grade");
        let de = ph2d_app_light2d::Degrau {
            c0_px: d.c0_px * ENTORNO_PADRAO,
            margem: MARGEM_DO_ENTORNO,
            ..d
        };
        let ge = grade_do_quadro(a, FHD, de).expect("grade do entorno");
        let mut pares: Vec<(Solver, Solver)> = cenas
            .iter()
            .map(|_| {
                let mut s = solver(
                    gpu,
                    a,
                    Config {
                        fina: fina_do_quadro(a, d),
                        captura: captura_do_quadro(a, d),
                        holografica: Some(HOLOGRAFICA_PADRAO),
                        rebote: true,
                        ..Config::padrao_para(&gpu.device, g)
                    },
                );
                let mut e = Solver::novo(
                    gpu,
                    Config {
                        holografica: Some(HOLOGRAFICA_PADRAO),
                        rebote: true,
                        ..Config::padrao_para(&gpu.device, ge)
                    },
                )
                .expect("o entorno");
                e.regiao_fina([0; 2], [0; 2]);
                let gv = s.config().grade;
                e.mascarar_miolo(
                    gpu,
                    Some([
                        gv.origem_m[0],
                        gv.origem_m[1],
                        gv.origem_m[0] + f64::from(gv.largura) * gv.texel_m,
                        gv.origem_m[1] + f64::from(gv.altura) * gv.texel_m,
                    ]),
                );
                s.somar_entorno(gpu, Some(&e));
                (s, e)
            })
            .collect();
        let mut minimo = vec![[f64::INFINITY; 2]; cenas.len()];
        for rodada in 0..RODADAS {
            for j in 0..cenas.len() {
                let i = (j + rodada) % cenas.len();
                let (s, e) = &mut pares[i];
                let sem_ceu = CenaDeLuz {
                    ceu: [0.0; 3],
                    ..cenas[i].clone()
                };
                for _ in 0..3 {
                    e.quadro(gpu, &sem_ceu);
                    s.quadro(gpu, &cenas[i]);
                }
                let (ts, te) = (
                    s.tempos(gpu).expect("TIMESTAMP_QUERY"),
                    e.tempos(gpu).expect("TIMESTAMP_QUERY"),
                );
                minimo[i][0] = minimo[i][0].min(ts.total());
                minimo[i][1] = minimo[i][1].min(te.total());
            }
        }
        let (s, e) = &pares[0];
        eprintln!(
            "- {nome}: a vista {}×{} · {:.0} MB · o entorno {}×{} · {:.0} MB · cena 3 {:.2} + {:.2} ms · cena 5 {:.2} + {:.2} ms",
            g.largura,
            g.altura,
            s.pass_list().bytes_alocados() as f64 / 1e6,
            ge.largura,
            ge.altura,
            e.pass_list().bytes_alocados() as f64 / 1e6,
            minimo[0][0],
            minimo[0][1],
            minimo[1][0],
            minimo[1][1]
        );
    }
}

/// B1. O ENTORNO: um 2.º solver mais grosso (`fator` × o texel) e 4× mais largo, sem céu e com a
/// emissão do miolo (a grade da vista) apagada — a c0 dele soma-se à da vista. A mesma régua da W5c
/// (o ponto junto à borda esquerda da vista de 2 m, o vermelho contra a série).
fn entorno(gpu: &GpuContext) {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    use ph2d_light2d_ref::{OpcoesDoRebote, Rebote};
    eprintln!("\n## B1. O entorno (o vermelho junto à borda esquerda, contra a série; 80 quadros)");
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let c3 = cena_de_luz(&w);
    let sem_ceu = CenaDeLuz {
        ceu: [0.0; 3],
        ..c3.clone()
    };
    let serie = Rebote::resolver(
        &c3,
        OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 200,
        },
    );
    for centro_x in [-2.0_f32, 0.0, 2.0] {
        let a = Camera2d::new([centro_x, 0.0], 2.0)
            .world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        let d = degrau_do(NivelDaLuz::Media);
        let g = grade_do_quadro(a, FHD, d).expect("grade");
        let esquerda = (a.inverse() * Point::new(0.0, 540.0)).x;
        let p = [esquerda.max(-2.7) + 0.3, 0.0];
        let gab = 2.0 * std::f64::consts::PI * serie.luz_em(p)[0];
        let mut linha = format!("- câmera em x = {centro_x} · ponto {:.2}:", p[0]);
        for fator in [0.0_f64, 4.0, 8.0] {
            let cfg = Config {
                fina: fina_do_quadro(a, d),
                captura: captura_do_quadro(a, d),
                holografica: Some(HOLOGRAFICA_PADRAO),
                rebote: true,
                ..Config::padrao_para(&gpu.device, g)
            };
            let mut s = Solver::novo(gpu, cfg).expect("solver");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
            s.regiao_fina(i, f);
            let mut e = (fator > 0.0).then(|| {
                let de = ph2d_app_light2d::Degrau {
                    c0_px: d.c0_px * fator,
                    margem: 1.5,
                    ..d
                };
                let ge = grade_do_quadro(a, FHD, de).expect("grade do entorno");
                let mut e = Solver::novo(
                    gpu,
                    Config {
                        holografica: Some(HOLOGRAFICA_PADRAO),
                        rebote: true,
                        ..Config::padrao_para(&gpu.device, ge)
                    },
                )
                .expect("o entorno");
                e.regiao_fina([0; 2], [0; 2]);
                let gv = s.config().grade;
                assert!(e.mascarar_miolo(
                    gpu,
                    Some([
                        gv.origem_m[0],
                        gv.origem_m[1],
                        gv.origem_m[0] + f64::from(gv.largura) * gv.texel_m,
                        gv.origem_m[1] + f64::from(gv.altura) * gv.texel_m,
                    ])
                ));
                e
            });
            for _ in 0..80 {
                if let Some(e) = e.as_mut() {
                    e.quadro(gpu, &sem_ceu);
                }
                assert!(s.somar_entorno(gpu, e.as_ref()));
                s.quadro(gpu, &c3);
            }
            let fl = fluencia_lida(gpu, &s);
            let nome = if fator > 0.0 {
                format!("entorno {fator}×")
            } else {
                "sem entorno".into()
            };
            linha += &format!(" · {nome} {:+.3}", (fl.amostra_canal(p, 0) - gab) / gab);
        }
        eprintln!("{linha}");
    }
}

/// B2. Os rebotes por quadro (K9: ≤ 10 quadros a 1 %): a convergência da cena 3 na vista de fábrica
/// (o quadro em que o PIOR ponto fica a 5 % · 1 % do convergido, 120 quadros) e o custo (o mínimo de
/// [`RODADAS`]), por nível, com 1 e 2 rebotes por quadro.
fn rebotes_por_quadro(gpu: &GpuContext) {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    eprintln!("\n## B2. Rebotes por quadro (cena 3, vista de fábrica)");
    let a = vista(1.0);
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let c3 = cena_de_luz(&w);
    let pts = crate::medida_luz2d_w5c::pontos_da_cena_3();
    for (nome, nivel) in [
        ("Low", NivelDaLuz::Baixa),
        ("Medium", NivelDaLuz::Media),
        ("High", NivelDaLuz::Alta),
    ] {
        let d = degrau_do(nivel);
        let cfg = |n: u32| Config {
            fina: fina_do_quadro(a, d),
            captura: captura_do_quadro(a, d),
            holografica: Some(HOLOGRAFICA_PADRAO),
            rebote: true,
            rebotes_por_quadro: n,
            ..Config::padrao_para(&gpu.device, grade_do_quadro(a, FHD, d).expect("grade"))
        };
        let mut linha = format!("- {nome}:");
        let mut fins = Vec::new();
        for n in [1, 2] {
            let mut s = solver(gpu, a, cfg(n));
            let mut historia: Vec<Vec<f64>> = Vec::new();
            for _ in 0..120 {
                s.quadro(gpu, &c3);
                let f = fluencia_lida(gpu, &s);
                historia.push(pts.iter().map(|&p| f.amostra(p)).collect());
            }
            let fim = historia.last().expect("120").clone();
            let quadro_a = |tol: f64| {
                historia
                    .iter()
                    .position(|h| {
                        h.iter()
                            .zip(&fim)
                            .all(|(x, y)| (x - y).abs() <= tol * y.abs())
                    })
                    .map_or(0, |q| q + 1)
            };
            linha += &format!(
                " · {n} por quadro: 5 % no {} · 1 % no {}",
                quadro_a(0.05),
                quadro_a(0.01)
            );
            fins.push(fim);
        }
        let dif = fins[0]
            .iter()
            .zip(&fins[1])
            .map(|(x, y)| (x - y).abs() / x.abs().max(1e-9))
            .fold(0.0, f64::max);
        linha += &format!(" · o convergido difere {:.2} %", 100.0 * dif);
        let mut ss = [solver(gpu, a, cfg(1)), solver(gpu, a, cfg(2))];
        let mut minimo = [f64::INFINITY; 2];
        for rodada in 0..RODADAS {
            for j in 0..2 {
                let i = (j + rodada) % 2;
                for _ in 0..3 {
                    ss[i].quadro(gpu, &c3);
                }
                minimo[i] = minimo[i].min(ss[i].tempos(gpu).expect("TIMESTAMP_QUERY").total());
            }
        }
        linha += &format!(" · custo {:.2} → {:.2} ms", minimo[0], minimo[1]);
        eprintln!("{linha}");
    }
}
