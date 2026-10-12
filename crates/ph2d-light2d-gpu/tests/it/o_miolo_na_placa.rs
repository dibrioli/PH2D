//! ⭐⭐ W5h — **o miolo da placa é o do gabarito** (a difusão, `ph2d_light2d_ref::OpcoesDoMiolo`):
//! dentro de um disco opaco iluminado de lado, o texel fino da placa contra o gabarito (validado pelo
//! núcleo de Poisson no `o_miolo_do_disco`), longe do anel (a borda, que a W5g já mede).

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Grade, MIOLO_PADRAO, Miolo, Solver};
use ph2d_light2d_ref::{Gabarito, OpcoesDoMiolo};

use crate::comum::{disco, gpu, grade, ler_fluencia, oclusor, regular};

const C: [f32; 2] = [2.6, 2.0];
const R: f32 = 0.6;

fn cena() -> CenaDeLuz {
    CenaDeLuz {
        emissores: vec![disco([1.0, 2.3], 0.15, [1.0; 3], f32::INFINITY)],
        oclusores: vec![oclusor(regular(C, R, 96, 0.1), 1.0)],
        luzes: 1,
        ..Default::default()
    }
}

fn config(miolo: Option<Miolo>) -> Config {
    Config {
        fina: 2,
        miolo,
        ..Config::padrao(grade(128, 0.04))
    }
}

/// O pior erro (sobre o máximo da luz na borda) a `[0,1 R, 0,7 R]` do centro e a média quadrática a
/// `[0,1 R, 0,9 R]` — junto à borda o erro é o do anel (a luz na borda, que a W5g mede), não o do miolo.
fn erro(flu: &[[f32; 3]], g: &Grade, gab: &Gabarito) -> (f64, f64) {
    (
        erro_na_faixa(flu, g, gab, 0.1, 0.7).0,
        erro_na_faixa(flu, g, gab, 0.1, 0.9).1,
    )
}

fn erro_na_faixa(flu: &[[f32; 3]], g: &Grade, gab: &Gabarito, a: f64, b: f64) -> (f64, f64) {
    let r = f64::from(R);
    let g_max = (0..360)
        .map(|k| {
            let a = f64::from(k).to_radians();
            let p = [
                f64::from(C[0]) + 1.02 * r * a.cos(),
                f64::from(C[1]) + 1.02 * r * a.sin(),
            ];
            gab.fluencia(p)[1]
        })
        .fold(0.0, f64::max);
    let (mut pior, mut soma, mut n) = (0.0_f64, 0.0, 0.0);
    for j in 0..g.altura {
        for i in 0..g.largura {
            let p = g.centro(i, j);
            let d = (p[0] - f64::from(C[0])).hypot(p[1] - f64::from(C[1]));
            if !(a * r..=b * r).contains(&d) {
                continue;
            }
            let e = (f64::from(flu[(j * g.largura + i) as usize][1]) - gab.fluencia(p)[1]).abs()
                / g_max;
            pior = pior.max(e);
            soma += e * e;
            n += 1.0;
        }
    }
    (pior, (soma / n).sqrt())
}

fn rodar(gpu: &ph2d_gpu::GpuContext, cfg: Config, c: &CenaDeLuz) -> (Vec<[f32; 3]>, Grade, f64) {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    s.quadro(gpu, c);
    let ms = s.tempos(gpu).map_or(f64::NAN, |t| t.c3_miolo);
    (ler_fluencia(gpu, &s), s.grade_da_fluencia(), ms)
}

/// Os tetos do pior erro no miolo fundo e da média quadrática — MEDIDOS (W5h, `sonda_relaxacoes_do_miolo`,
/// RTX 5060 Ti): com o `MIOLO_PADRAO` (4 → 32) 0,039 · 0,022 nesta grade; convergido (32 → 32) 0,025 ·
/// 0,011; o controlo sem o miolo 0,60.
const TETO: f64 = 0.05;
const TETO_RMS: f64 = 0.03;

/// O teto da média quadrática SEM o miolo (o exterior mais próximo, `PH2D_LIGHT2D_MIOLO=0`) — MEDIDO
/// (W5k, `sonda_o_translucido_contra_o_gabarito`): 0,044 no opaco, 0,036 a 0,9, 0,012 a ½. O pior dele é
/// o salto do eixo medial e do terminador (0,42 já no opaco) e não entra.
const TETO_RMS_SEM_MIOLO: f64 = 0.05;

/// ⭐ W5k — **o translúcido da placa é o do gabarito, com e sem o miolo** (`mix(transmitida,
/// exterior/miolo, α)`; sem o miolo a mistura é a da `c3_fina`, com ele a do `miolo_final`): o disco a
/// 0,9 e a ½ contra o gabarito no miolo fundo — com o miolo nos tetos do OPACO (medido: 0,034 · 0,019 a
/// 0,9; 0,015 · 0,008 a ½) —, e a 0,999 a menos de 1 % do opaco (até a W5k, PRETO). Controlo da face: a
/// metade de trás da lâmpada vista através do corpo (`visibilidade_na_face`) — tapada, o rms a 0,9 era
/// 0,17 sem o miolo e 0,09 com ele.
#[test]
fn o_translucido_da_placa_e_o_do_gabarito() {
    let Some(gpu) = gpu() else { return };
    let com_alfa = |a: f32| {
        let mut c = cena();
        c.oclusores[0].opacidade = a;
        c
    };
    for (miolo, suavidade) in [(Some(MIOLO_PADRAO), 1.0), (None, 0.0)] {
        let op = OpcoesDoMiolo {
            suavidade,
            penetracao_m: f64::INFINITY,
        };
        for a in [0.9, 0.5] {
            let (flu, g, _) = rodar(&gpu, config(miolo), &com_alfa(a));
            let (pior, rms) = erro(&flu, &g, &Gabarito::com_miolo(&com_alfa(a), op));
            let ok = if miolo.is_some() {
                pior <= TETO && rms <= TETO_RMS
            } else {
                rms <= TETO_RMS_SEM_MIOLO
            };
            assert!(ok, "miolo {} a {a}: a placa não é o gabarito: pior {pior:.4} · rms {rms:.4}", miolo.is_some());
        }
        let (um, g, _) = rodar(&gpu, config(miolo), &com_alfa(1.0));
        let (quase, ..) = rodar(&gpu, config(miolo), &com_alfa(0.999));
        let no_disco = |k: usize| {
            let p = g.centro(k as u32 % g.largura, k as u32 / g.largura);
            (p[0] - f64::from(C[0])).hypot(p[1] - f64::from(C[1])) < 0.95 * f64::from(R)
        };
        let topo = (0..um.len()).filter(|&k| no_disco(k)).map(|k| um[k][1]).fold(0.0, f32::max);
        let salto = (0..um.len())
            .filter(|&k| no_disco(k))
            .map(|k| (quase[k][1] - um[k][1]).abs() / topo)
            .fold(0.0, f32::max);
        assert!(salto <= 0.01, "miolo {}: de 1 a 0,999 o interior saltou {salto:.5}", miolo.is_some());
    }
}

/// Sonda (`--ignored --nocapture`): o erro da placa contra o gabarito no miolo fundo, por opacidade.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_o_translucido_contra_o_gabarito() {
    let Some(gpu) = gpu() else { return };
    for (miolo, suavidade) in [(Some(MIOLO_PADRAO), 1.0), (None, 0.0)] {
        for a in [1.0_f32, 0.9, 0.5, 0.1, 0.001] {
            let mut c = cena();
            c.oclusores[0].opacidade = a;
            let op = OpcoesDoMiolo {
                suavidade,
                penetracao_m: f64::INFINITY,
            };
            let (flu, g, _) = rodar(&gpu, config(miolo), &c);
            let gab = Gabarito::com_miolo(&c, op);
            let (pior, rms) = erro(&flu, &g, &gab);
            let p = [f64::from(C[0]) - 0.3 * f64::from(R), f64::from(C[1])];
            let (i, j) = (((p[0] - g.origem_m[0]) / g.texel_m) as u32, ((p[1] - g.origem_m[1]) / g.texel_m) as u32);
            let placa = flu[(j * g.largura + i) as usize][1];
            eprintln!(
                "miolo {} α {a}: pior {pior:.4} rms {rms:.4} · no ponto da face placa {placa:.4} gabarito {:.4}",
                miolo.is_some(),
                gab.fluencia(g.centro(i, j))[1]
            );
            if (a - 0.9).abs() < 1e-6 || (a - 0.5).abs() < 1e-6 {
                let volta: Vec<String> = (0..12)
                    .map(|k| {
                        let ang = f64::from(k * 30).to_radians();
                        let p = [f64::from(C[0]) + 0.5 * f64::from(R) * ang.cos(), f64::from(C[1]) + 0.5 * f64::from(R) * ang.sin()];
                        let (i, j) = (((p[0] - g.origem_m[0]) / g.texel_m) as u32, ((p[1] - g.origem_m[1]) / g.texel_m) as u32);
                        let q = g.centro(i, j);
                        format!("{}°:{:.3}/{:.3}", k * 30, flu[(j * g.largura + i) as usize][1], gab.fluencia(q)[1])
                    })
                    .collect();
                eprintln!("   volta a 0,5R (placa/gabarito): {}", volta.join(" "));
            }
        }
    }
}

#[test]
fn o_miolo_da_placa_e_o_do_gabarito() {
    let Some(gpu) = gpu() else { return };
    let c = cena();
    let gab = Gabarito::novo(&c);
    let (flu, g, _) = rodar(&gpu, config(Some(MIOLO_PADRAO)), &c);
    let (pior, rms) = erro(&flu, &g, &gab);
    // Controlo: o exterior mais próximo (sem o miolo) não é a difusão.
    let (flu0, g0, _) = rodar(&gpu, config(None), &c);
    let (pior0, _) = erro(&flu0, &g0, &gab);
    eprintln!("miolo na placa: pior {pior:.4} · rms {rms:.4} (controlo sem o miolo: {pior0:.4})");
    assert!(pior0 > 3.0 * TETO, "o controlo não distingue: {pior0:.4}");
    assert!(
        pior <= TETO && rms <= TETO_RMS,
        "o miolo da placa não é o do gabarito: pior {pior:.4} (teto {TETO}) · rms {rms:.4} (teto {TETO_RMS})"
    );
}

#[test]
fn a_penetracao_da_placa_e_a_do_gabarito() {
    let Some(gpu) = gpu() else { return };
    let c = cena();
    let gab = Gabarito::com_miolo(
        &c,
        OpcoesDoMiolo {
            suavidade: 1.0,
            penetracao_m: 0.15,
        },
    );
    let m = Miolo {
        penetracao_m: 0.15,
        ..MIOLO_PADRAO
    };
    let (flu, g, _) = rodar(&gpu, config(Some(m)), &c);
    let (pior, rms) = erro(&flu, &g, &gab);
    eprintln!("penetração 0,15 m: pior {pior:.4} · rms {rms:.4}");
    assert!(
        pior <= TETO && rms <= TETO_RMS,
        "penetração 0,15 m: pior {pior:.4} · rms {rms:.4}"
    );
}

/// A suavidade é a mistura LINEAR das duas leis, texel a texel (o exterior mais próximo tem, no eixo
/// medial, o salto que a média da área do texel alisa — a régua dele é o `o_interior_do_opaco`).
#[test]
fn a_suavidade_da_placa_mistura_as_duas_leis() {
    let Some(gpu) = gpu() else { return };
    let c = cena();
    let com = |suavidade| {
        let m = Miolo {
            suavidade,
            ..MIOLO_PADRAO
        };
        rodar(&gpu, config(Some(m)), &c).0
    };
    let (zero, um, meio) = (com(0.0), com(1.0), com(0.5));
    // Só os texels do disco (fora dele a luz junto à lâmpada é ordens de grandeza maior).
    let g = rodar(&gpu, config(None), &c).1;
    let no_disco: Vec<usize> = (0..meio.len())
        .filter(|&k| {
            let p = g.centro(k as u32 % g.largura, k as u32 / g.largura);
            (p[0] - f64::from(C[0])).hypot(p[1] - f64::from(C[1])) < 0.95 * f64::from(R)
        })
        .collect();
    let topo = no_disco
        .iter()
        .map(|&k| zero[k][1].max(um[k][1]))
        .fold(0.0_f32, f32::max);
    let pior = no_disco
        .iter()
        .map(|&k| (meio[k][1] - 0.5 * (zero[k][1] + um[k][1])).abs() / topo)
        .fold(0.0_f32, f32::max);
    let diferem = no_disco
        .iter()
        .filter(|&&k| (zero[k][1] - um[k][1]).abs() > 0.05 * topo)
        .count();
    eprintln!(
        "suavidade ½ × a média das duas: pior {pior:.5} ({diferem} texels onde as leis diferem)"
    );
    assert!(
        diferem > 100,
        "o controlo não distingue as duas leis: {diferem} texels"
    );
    // O f16 da fluência: ~2⁻¹¹ relativo, nas três leituras.
    assert!(
        pior <= 2e-3,
        "a suavidade ½ não é a média das duas leis: {pior:.5}"
    );
}

/// A rodada das relaxações: o erro e o custo de cada par (fina, grossa), intercalados no MESMO
/// processo, o mínimo de 5.
#[test]
#[ignore = "a medida das relaxações do miolo: corre-se à mão"]
fn sonda_relaxacoes_do_miolo() {
    let Some(gpu) = gpu() else { return };
    let c = cena();
    let gab = Gabarito::novo(&c);
    let pares = [(32, 32), (2, 32), (4, 32), (8, 32), (4, 64), (2, 64)];
    // A pequena (256² finos) e a do tamanho de 1080p (1920² finos, o disco com 240 texels de raio).
    for lado in [128_u32, 960] {
        eprintln!("grade fina {}²:", 2 * lado);
        let mut ms = vec![f64::INFINITY; pares.len()];
        let mut erros = vec![(0.0, 0.0); pares.len()];
        for _ in 0..3 {
            for (k, &(relaxa, relaxa_max)) in pares.iter().enumerate() {
                let m = Miolo {
                    relaxa,
                    relaxa_max,
                    ..MIOLO_PADRAO
                };
                let cfg = Config {
                    grade: grade(lado, 5.12 / f64::from(lado)),
                    ..config(Some(m))
                };
                let (flu, g, t) = rodar(&gpu, cfg, &c);
                ms[k] = ms[k].min(t);
                erros[k] = erro(&flu, &g, &gab);
            }
        }
        for (k, (relaxa, max)) in pares.iter().enumerate() {
            eprintln!(
                "relaxa {relaxa:>2} → {max:>2}: pior {:.4} · rms {:.4} · {:.3} ms",
                erros[k].0, erros[k].1, ms[k]
            );
        }
    }
}

/// O ANEL da placa (os texels de dentro com uma vizinha de fora) contra o gabarito no pé deles.
#[test]
#[ignore = "a medida do anel do miolo: corre-se à mão"]
fn sonda_anel_do_miolo() {
    let Some(gpu) = gpu() else { return };
    let c = cena();
    let gab = Gabarito::novo(&c);
    let (flu, g, _) = rodar(&gpu, config(None), &c);
    let v = regular(C, R, 96, 0.1);
    let dentro = |p: [f64; 2]| {
        let p = [p[0] as f32, p[1] as f32];
        let mut d = false;
        let mut j = v.len() - 1;
        for i in 0..v.len() {
            let (a, b) = (v[i], v[j]);
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                d = !d;
            }
            j = i;
        }
        d
    };
    let r = f64::from(R);
    for lado_de_dentro in [true, false] {
        let (mut soma, mut n, mut pior) = (0.0, 0.0, (0.0_f64, 0.0, 0.0, 0.0));
        let mut g_max = 0.0_f64;
        for j in 1..g.altura - 1 {
            for i in 1..g.largura - 1 {
                let p = g.centro(i, j);
                if dentro(p) != lado_de_dentro {
                    continue;
                }
                let anel = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(di, dj)| {
                    dentro(g.centro((i as i32 + di) as u32, (j as i32 + dj) as u32))
                        != lado_de_dentro
                });
                if !anel {
                    continue;
                }
                let a = (p[1] - f64::from(C[1])).atan2(p[0] - f64::from(C[0]));
                // O pé na borda (o polígono de 96 lados pelo raio do apótema na direção) — ou o centro
                // do texel de fora.
                let lado = std::f64::consts::TAU / 96.0;
                let rb = r * (lado / 2.0).cos() / ((a - 0.1).rem_euclid(lado) - lado / 2.0).cos();
                let pe = if lado_de_dentro {
                    [
                        f64::from(C[0]) + (rb + 1e-5) * a.cos(),
                        f64::from(C[1]) + (rb + 1e-5) * a.sin(),
                    ]
                } else {
                    p
                };
                let gpu_v = f64::from(flu[(j * g.largura + i) as usize][1]);
                let ref_v = gab.fluencia(pe)[1];
                g_max = g_max.max(ref_v);
                let e = gpu_v - ref_v;
                soma += e;
                n += 1.0;
                if e.abs() > pior.0.abs() {
                    pior = (e, a.to_degrees(), gpu_v, ref_v);
                }
            }
        }
        eprintln!(
            "{}: {n} texels · viés médio {:.4} (g_max {g_max:.3}) · pior {pior:?}",
            if lado_de_dentro {
                "anel de dentro"
            } else {
                "anel de fora"
            },
            soma / n
        );
    }
}
