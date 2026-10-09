//! ⏱ **A RODADA DA W2** — um lote só, intercalado (`docs/DevOps/MEDIR_VELOCIDADE.md`): todas as
//! variantes no MESMO processo, aquecidas fora da régua, em rodadas com a ordem rodada; vale o
//! MÍNIMO do relógio da placa (mediana ao lado). Para cada caso: ms por passe, bytes alocados e
//! por quadro (da `PassList`), e o erro contra o gabarito (RMSE relativo, total e na região livre).
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-light2d-gpu --profile smoke --test it medida_luz2d -- --ignored --nocapture
//! ```

use ph2d_light2d_gpu::{Config, Fusao, PassList, Precisao, Solver};

use crate::cenas::{DEGRAUS, ceu_e_vidro, enxame, enxame_de_luzes, penumbra, regiao_livre};
use crate::comum::{gabarito, gpu, grade, ler_fluencia, rmse_rel, tem_f16};
use crate::medicao::{Caso, loadavg, perfil, rodadas};

const RODADAS: usize = 20;

#[test]
#[ignore = "régua: corre-se à mão, no perfil smoke"]
fn medida_luz2d() {
    let Some(gpu) = gpu() else { return };
    let info = gpu.adapter.get_info();
    let alto = |cfg: fn(Config) -> Config| cfg(Config::padrao(grade(DEGRAUS[0].1, DEGRAUS[0].2)));
    let mut variantes: Vec<(String, Config, &str)> = vec![
        (
            "base (bilinear · l0 1 · passo 1 · jfa 2 · f32/f16)".into(),
            alto(|c| c),
            "penumbra",
        ),
        (
            "fusão clássica".into(),
            alto(|c| Config {
                fusao: Fusao::Classica,
                ..c
            }),
            "penumbra",
        ),
        (
            "fusão vizinha intercalada".into(),
            alto(|c| Config {
                fusao: Fusao::VizinhaIntercalada,
                ..c
            }),
            "penumbra",
        ),
        (
            "intervalo c0 ½".into(),
            alto(|c| Config {
                intervalo_c0: 0.5,
                ..c
            }),
            "penumbra",
        ),
        (
            "intervalo c0 2".into(),
            alto(|c| Config {
                intervalo_c0: 2.0,
                ..c
            }),
            "penumbra",
        ),
        (
            "passo ½".into(),
            alto(|c| Config {
                passo_perto: 0.5,
                ..c
            }),
            "penumbra",
        ),
        (
            "jfa 1".into(),
            alto(|c| Config { fator_jfa: 1, ..c }),
            "penumbra",
        ),
        (
            "jfa 4".into(),
            alto(|c| Config { fator_jfa: 4, ..c }),
            "penumbra",
        ),
        (
            "cascatas f32".into(),
            alto(|c| Config {
                armazenamento: Precisao::F32,
                ..c
            }),
            "penumbra",
        ),
        ("céu e vidro · base".into(), alto(|c| c), "ceu"),
        (
            "céu e vidro · intervalo c0 ½".into(),
            alto(|c| Config {
                intervalo_c0: 0.5,
                ..c
            }),
            "ceu",
        ),
        (
            "céu e vidro · intervalo c0 2".into(),
            alto(|c| Config {
                intervalo_c0: 2.0,
                ..c
            }),
            "ceu",
        ),
        (
            "céu e vidro · passo ½".into(),
            alto(|c| Config {
                passo_perto: 0.5,
                ..c
            }),
            "ceu",
        ),
        (
            "céu e vidro · fusão clássica".into(),
            alto(|c| Config {
                fusao: Fusao::Classica,
                ..c
            }),
            "ceu",
        ),
    ];
    if tem_f16(&gpu) {
        variantes.push((
            "aritmética f16".into(),
            alto(|c| Config {
                aritmetica: Precisao::F16,
                ..c
            }),
            "penumbra",
        ));
    }
    for &(degrau, lado, texel) in &DEGRAUS[1..] {
        let base = Config::padrao(grade(lado, texel));
        variantes.push((format!("{degrau} base"), base, "penumbra"));
        variantes.push((
            format!("{degrau} passo ½"),
            Config {
                passo_perto: 0.5,
                ..base
            },
            "penumbra",
        ));
        variantes.push((format!("céu e vidro · {degrau} base"), base, "ceu"));
        variantes.push((
            format!("céu e vidro · {degrau} passo ½"),
            Config {
                passo_perto: 0.5,
                ..base
            },
            "ceu",
        ));
    }
    let mut casos: Vec<Caso> = variantes
        .iter()
        .map(|(nome, cfg, cena)| Caso {
            nome: nome.clone(),
            solver: Solver::novo(&gpu, *cfg).expect("solver"),
            cena: if *cena == "ceu" {
                ceu_e_vidro()
            } else {
                penumbra()
            },
        })
        .collect();
    let n_variantes = casos.len();
    for n in [1, 10, 100, 1_000, 10_000] {
        casos.push(Caso {
            nome: format!("S2 · {n} luzes na mesma geometria"),
            solver: Solver::novo(&gpu, alto(|c| c)).expect("solver"),
            cena: enxame_de_luzes(n),
        });
    }
    for n in [1, 10, 100, 1_000, 10_000] {
        for (rotulo, f) in [("jfa 2", 2), ("jfa 4", 4)] {
            casos.push(Caso {
                nome: format!("S3 · {n} emissores, área fixa · {rotulo}"),
                solver: Solver::novo(
                    &gpu,
                    Config {
                        fator_jfa: f,
                        ..alto(|c| c)
                    },
                )
                .expect("solver"),
                cena: enxame(n),
            });
        }
    }

    let carga0 = loadavg();
    let Some(r) = rodadas(&gpu, &mut casos, RODADAS) else {
        println!("a placa não tem TIMESTAMP_QUERY — régua impossível");
        return;
    };
    let carga1 = loadavg();

    println!(
        "\n## Rodada W2 — {} · {:?} · driver {} {}",
        info.name, info.backend, info.driver, info.driver_info
    );
    println!(
        "perfil {} · {RODADAS} rodadas intercaladas · loadavg {carga0} → {carga1} · SHADER_F16: {}\n",
        perfil(),
        tem_f16(&gpu)
    );
    println!(
        "| caso | grade | cascatas | C0 ms | C1 ms | C2 ms | RC = C1+C2 ms (mín · med) | MB alocados | MB/quadro | RMSE rel. | RMSE livre |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|");
    let mut referencias: Vec<(u32, &str, Vec<[f64; 3]>)> = Vec::new();
    for (k, (caso, x)) in casos.iter_mut().zip(&r).enumerate() {
        let cfg = *caso.solver.config();
        let g = cfg.grade;
        let mb = |b: u64| b as f64 / (1u64 << 20) as f64;
        let alocado = caso.solver.pass_list().bytes_alocados();
        let por_quadro = PassList::bytes_por_quadro(&cfg);
        let (rmse, livre) = if k < n_variantes {
            let qual = if variantes[k].2 == "ceu" {
                "ceu"
            } else {
                "penumbra"
            };
            let pos = referencias
                .iter()
                .position(|(l, q, _)| *l == g.largura && *q == qual);
            let i = pos.unwrap_or_else(|| {
                referencias.push((g.largura, qual, gabarito(&caso.cena, &g)));
                referencias.len() - 1
            });
            caso.solver.quadro(&gpu, &caso.cena);
            let f = ler_fluencia(&gpu, &caso.solver);
            let rf = &referencias[i].2;
            let livre = if qual == "penumbra" {
                format!(
                    "{:.4}",
                    rmse_rel(&f, rf, regiao_livre(rf, g.largura, g.texel_m))
                )
            } else {
                "—".into()
            };
            (format!("{:.4}", rmse_rel(&f, rf, |_| true)), livre)
        } else {
            ("—".into(), "—".into())
        };
        println!(
            "| {} | {}² | {} | {:.4} | {:.4} | {:.4} | {:.4} · {:.4} | {:.2} | {:.1} | {rmse} | {livre} |",
            x.nome,
            g.largura,
            caso.solver.n_cascatas(),
            x.c0[0],
            x.c1[0],
            x.c2[0],
            x.rc[0],
            x.rc[1],
            mb(alocado),
            mb(por_quadro),
        );
    }
    println!("\n### C2 por cascata (ms, mínimo), Alto");
    for x in r.iter().take(3) {
        let niveis: Vec<String> = x.c2_por_nivel.iter().map(|v| format!("{v:.4}")).collect();
        println!("- {}: {}", x.nome, niveis.join(" · "));
    }
    let k6 = r[0].c2[0] / r[1].c2[0];
    println!("\nK6: C2 com bilinear fix / C2 clássica = {k6:.2}× (limite 2×)");
    let variacao = |xs: &[f64]| {
        let (mn, mx) = xs
            .iter()
            .fold((f64::INFINITY, 0.0_f64), |(a, b), &v| (a.min(v), b.max(v)));
        100.0 * (mx - mn) / mn
    };
    let s2: Vec<f64> = r[n_variantes..n_variantes + 5]
        .iter()
        .map(|x| x.rc[0])
        .collect();
    println!(
        "K3 (S2, número de luzes, mesma geometria): variação do RC 1 → 10 000 = {:.2} % (limite 5 %)",
        variacao(&s2)
    );
    for (k, rotulo) in ["jfa 2", "jfa 4"].iter().enumerate() {
        let s3: Vec<f64> = r[n_variantes + 5..]
            .iter()
            .skip(k)
            .step_by(2)
            .map(|x| x.rc[0])
            .collect();
        println!(
            "S3 (geometria muda com N, área emissora fixa) · {rotulo}: variação do RC = {:.2} %",
            variacao(&s3)
        );
    }
}
