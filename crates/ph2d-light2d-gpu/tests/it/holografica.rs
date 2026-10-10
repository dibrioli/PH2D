//! ⭐ W5c — **a cascata holográfica** (HRC, a do produto em todo nível) contra o gabarito e contra as cascatas
//! clássicas, nas cenas que passam pelas CASCATAS (a lâmpada direta da C3 não depende delas).

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Holografica, Solver};

use crate::cenas::{DEGRAUS, ceu_e_vidro, penumbra_pelas_cascatas, regiao_livre_longe};
use crate::comum::{dentro_de_opaco, gabarito, gpu, grade, ler_fluencia, rmse_rel};

// Os tetos da holográfica — medido + 10 % na RTX 5060 Ti (W5c; tabela no handoff):
// - a penumbra pelas cascatas no Alto: holográfica 0,0757 × clássica 0,1175; céu e vidro 0,0075;
// - junto a uma face (o pior de 6 desvios; longe · próprio): Alto 0,003 · 0,002 · Médio 0,007 ·
//   0,005 · Baixo 0,011 · 0,009 (a clássica até 0,046 · 0,215);
// - o G9 do rebote: 0,0092 · 0,0104 · 0,0071 (a clássica 0,044 · 0,074 · 0,131);
// - o pan (a grade por blocos, 3 vistas): 0,00061.
const TOL_PENUMBRA_HOLO: f64 = 0.0833;
const TOL_CEU_E_VIDRO_HOLO: f64 = 0.0083;
const TOL_FACE_HOLO: [[f64; 2]; 3] = [[0.0035, 0.0025], [0.0077, 0.0055], [0.0121, 0.0099]];
const TOL_G9_HOLO: [f64; 3] = [0.0101, 0.0115, 0.0079];
const TOL_PAN_HOLO: f64 = 0.00067;

/// ⭐ **A holográfica bate a clássica na penumbra** das cascatas (a afirmação do artigo, Fig. 8) e iguala-o no
/// céu e vidro.
#[test]
fn g3_a_holografica_bate_a_classica_na_penumbra() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let g = grade(lado, texel);
    let cena = penumbra_pelas_cascatas();
    let r = gabarito(&cena, &g);
    let dentro = dentro_de_opaco(&cena, &g);
    let (classica, _) = rodar(&gpu, Config::padrao(g), &cena);
    let (holo, _) = rodar(
        &gpu,
        com(Config::padrao(g), Some(HOLOGRAFICA_PADRAO)),
        &cena,
    );
    let (ec, eu) = (
        rmse_rel(&classica, &r, |k| !dentro(k)),
        rmse_rel(&holo, &r, |k| !dentro(k)),
    );
    eprintln!("G3 holográfica · penumbra pelas cascatas: holográfica {eu:.4} · clássica {ec:.4}");
    assert!(eu <= TOL_PENUMBRA_HOLO && eu < ec, "{eu:.4} × {ec:.4}");
    let cena = ceu_e_vidro();
    let r = gabarito(&cena, &g);
    let dentro = dentro_de_opaco(&cena, &g);
    let (holo, _) = rodar(
        &gpu,
        com(Config::padrao(g), Some(HOLOGRAFICA_PADRAO)),
        &cena,
    );
    let e = rmse_rel(&holo, &r, |k| !dentro(k));
    eprintln!("G3 holográfica · céu e vidro: {e:.4}");
    assert!(e <= TOL_CEU_E_VIDRO_HOLO, "{e:.4}");
}

/// ⭐ **A holográfica junto a uma face** — em todo alinhamento e degrau.
#[test]
fn a_holografica_junto_a_uma_face_e_o_gabarito() {
    let Some(gpu) = gpu() else { return };
    let mut falhas = Vec::new();
    for (k, (degrau, lado, texel)) in DEGRAUS.into_iter().enumerate() {
        let t = texel as f32;
        for desvio in [0.0_f32, 0.25, 0.5, 0.75, 1.0, 1.5] {
            let cfg = com(Config::padrao(grade(lado, texel)), Some(HOLOGRAFICA_PADRAO));
            let (l, p) = crate::junto_a_face::pior_junto_as_faces(
                &gpu,
                cfg,
                (40.0 + desvio) * t,
                (100.0 + desvio) * t,
            );
            if l > TOL_FACE_HOLO[k][0] || p > TOL_FACE_HOLO[k][1] {
                falhas.push(format!(
                    "{degrau} desvio {desvio}: longe {l:.4} · próprio {p:.4}"
                ));
            }
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// ⭐ **G9 na holográfica** — o rebote converge para o gabarito em todo degrau.
#[test]
fn g9_o_rebote_na_holografica() {
    use crate::g9_rebote::{luz_na_placa, pontos, sala};
    use ph2d_light2d_ref::{OpcoesDoRebote, Rebote};
    let Some(gpu) = gpu() else { return };
    let cena = sala();
    let serie = Rebote::resolver(
        &cena,
        OpcoesDoRebote {
            passo_m: 0.01,
            rebotes: 300,
        },
    );
    let mut falhas = Vec::new();
    for (k, (degrau, lado, texel)) in DEGRAUS.into_iter().enumerate() {
        let cfg = Config {
            rebote: true,
            ..com(Config::padrao(grade(lado, texel)), Some(HOLOGRAFICA_PADRAO))
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        for _ in 0..60 {
            s.quadro(&gpu, &cena);
        }
        let flu = ler_fluencia(&gpu, &s);
        let gf = s.grade_da_fluencia();
        for p in pontos() {
            let (l, r) = (luz_na_placa(&flu, &gf, p), serie.luz_em(p));
            let e = (0..3)
                .map(|c| (l[c] - r[c]).abs() / r[c])
                .fold(0.0, f64::max);
            if e > TOL_G9_HOLO[k] {
                falhas.push(format!("{degrau} {p:?}: {e:.4}"));
            }
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// ⭐ **A holográfica não muda com o pan** — a grade por blocos, os mesmos pontos do mundo.
#[test]
fn a_holografica_nao_muda_com_o_pan() {
    let Some(gpu) = gpu() else { return };
    let pior = pior_no_pan(&gpu, Some(HOLOGRAFICA_PADRAO));
    eprintln!("holográfica · pan: pior {pior:.5}");
    assert!(pior <= TOL_PAN_HOLO, "{pior:.5}");
}

pub fn com(cfg: Config, holografica: Option<Holografica>) -> Config {
    Config { holografica, ..cfg }
}

/// A fluência de um quadro e o tempo de placa das cascatas (ms).
pub fn rodar(gpu: &ph2d_gpu::GpuContext, cfg: Config, cena: &CenaDeLuz) -> (Vec<[f32; 3]>, f64) {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    s.quadro(gpu, cena);
    let ms = s.tempos(gpu).map_or(f64::NAN, |t| t.c2_total());
    (ler_fluencia(gpu, &s), ms)
}

/// Sonda (à mão, `--ignored --nocapture`): clássica × holográfica por degrau.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_hrc_contra_o_gabarito() {
    let Some(gpu) = gpu() else { return };
    for (nome, cena) in [
        ("penumbra pelas cascatas", penumbra_pelas_cascatas()),
        ("céu e vidro", ceu_e_vidro()),
    ] {
        for (degrau, lado, texel) in DEGRAUS {
            let g = grade(lado, texel);
            let r = gabarito(&cena, &g);
            let dentro = dentro_de_opaco(&cena, &g);
            let mut linha = Vec::new();
            for h in variantes() {
                let rotulo = format!("{h:?}");
                let (flu, ms) = rodar(&gpu, com(Config::padrao(g), h), &cena);
                let e = rmse_rel(&flu, &r, |k| !dentro(k));
                let livre = regiao_livre_longe(lado, texel);
                let e_livre = rmse_rel(&flu, &r, |k| !dentro(k) && livre(k));
                linha.push(format!("{rotulo} {e:.4} (longe {e_livre:.4}) {ms:.3} ms"));
            }
            eprintln!("{nome} · {degrau}: {}", linha.join(" · "));
        }
    }
}

/// Sonda (à mão): só o céu (exato: 2π·L), e uma luz pequena no vazio, por distância.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_hrc_casos_simples() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[1];
    let g = grade(lado, texel);
    let ceu = CenaDeLuz {
        ceu: [1.0; 3],
        luzes: 1,
        ..Default::default()
    };
    for h in variantes() {
        let (flu, _) = rodar(&gpu, com(Config::padrao(g), h), &ceu);
        let w = lado as usize;
        let amostras: Vec<String> = [(0, 0), (5, 5), (128, 128), (200, 50), (255, 255)]
            .iter()
            .map(|&(i, j)| format!("({i},{j}) {:.4}", flu[j * w + i][0] / std::f32::consts::TAU))
            .collect();
        eprintln!("só o céu · {h:?}: {}", amostras.join(" · "));
    }
    let luz = CenaDeLuz {
        emissores: vec![ph2d_light2d_ecs::Emissor::Poligono {
            vertices: crate::comum::regular([2.56, 2.56], 0.06, 16, 0.0),
            radiancia: [1.0; 3],
            alcance_m: f32::INFINITY,
        }],
        luzes: 1,
        ..Default::default()
    };
    let r = gabarito(&luz, &g);
    for h in variantes() {
        let (flu, _) = rodar(&gpu, com(Config::padrao(g), h), &luz);
        let w = lado as usize;
        let c = 128usize;
        let amostras: Vec<String> = [4usize, 6, 10, 20, 40, 80, 120]
            .iter()
            .flat_map(|&d| {
                [
                    (c + d, c),
                    (c, c + d),
                    (c - d, c),
                    (c + d * 7 / 10, c + d * 7 / 10),
                ]
            })
            .map(|(i, j)| {
                format!(
                    "{:+.3}",
                    f64::from(flu[j * w + i][0]) / r[j * w + i][0] - 1.0
                )
            })
            .collect();
        eprintln!(
            "luz pequena · {h:?} (d = 4·6·10·20·40·80·120; →,↑,←,↗): {}",
            amostras.join(" ")
        );
    }
}

/// Sonda (à mão): uma placa emissora grande (x ∈ [3, 5,12]), placa × gabarito a várias distâncias à
/// esquerda dela, em três alturas.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_hrc_placa_grande() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[1];
    let g = grade(lado, texel);
    let placa = CenaDeLuz {
        emissores: vec![ph2d_light2d_ecs::Emissor::Poligono {
            vertices: vec![[3.0, 0.0], [5.12, 0.0], [5.12, 5.12], [3.0, 5.12]],
            radiancia: [1.0; 3],
            alcance_m: f32::INFINITY,
        }],
        luzes: 1,
        ..Default::default()
    };
    let r = gabarito(&placa, &g);
    for h in variantes() {
        let (flu, _) = rodar(&gpu, com(Config::padrao(g), h), &placa);
        let w = lado as usize;
        for j in [128usize, 30, 220] {
            let linha: Vec<String> = [149usize, 148, 146, 140, 125, 100, 50, 10]
                .iter()
                .map(|&i| {
                    format!(
                        "{i}: {:+.3}",
                        f64::from(flu[j * w + i][0]) / r[j * w + i][0] - 1.0
                    )
                })
                .collect();
            eprintln!("placa · {h:?} · linha {j}: {}", linha.join(" "));
        }
    }
}

/// As variantes que as sondas comparam: a clássica e a holográfica com e sem o «+1».
fn variantes() -> [Option<Holografica>; 3] {
    [
        None,
        Some(HOLOGRAFICA_PADRAO),
        Some(Holografica {
            mais_um: false,
            ..HOLOGRAFICA_PADRAO
        }),
    ]
}

/// Sonda (à mão): o céu com SÓ o vidro (α 0,5) e com SÓ uma parede opaca.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_hrc_ceu_com_um_obstaculo() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[1];
    let g = grade(lado, texel);
    let vidro = crate::comum::oclusor(crate::comum::regular([2.6, 2.4], 0.4, 4, 0.3), 0.5);
    let parede = crate::comum::oclusor(crate::comum::regular([2.6, 2.4], 0.4, 4, 0.3), 1.0);
    for (nome, o) in [("vidro", vidro), ("parede", parede)] {
        let cena = CenaDeLuz {
            oclusores: vec![o],
            ceu: [1.0; 3],
            luzes: 1,
            ..Default::default()
        };
        let r = gabarito(&cena, &g);
        let dentro = dentro_de_opaco(&cena, &g);
        for h in variantes() {
            let (flu, _) = rodar(&gpu, com(Config::padrao(g), h), &cena);
            let e = rmse_rel(&flu, &r, |k| !dentro(k));
            let (mut d, mut m) = (0.0, 0.0);
            for k in (0..r.len()).filter(|&k| !dentro(k)) {
                d += f64::from(flu[k][0]) - r[k][0];
                m += r[k][0];
            }
            eprintln!("{nome} · {h:?}: {e:.4} · viés {:+.4}", d / m);
        }
    }
}

/// Sonda (à mão): o erro médio (com sinal) a meia distância de uma luz, pelo tamanho dela.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_hrc_pelo_tamanho_da_luz() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[1];
    let g = grade(lado, texel);
    for raio_texels in [1.5_f32, 3.0, 6.0, 12.0, 24.0] {
        let raio = raio_texels * texel as f32;
        let cena = CenaDeLuz {
            emissores: vec![ph2d_light2d_ecs::Emissor::Poligono {
                vertices: crate::comum::regular([2.56, 2.56], raio, 32, 0.1),
                radiancia: [1.0; 3],
                alcance_m: f32::INFINITY,
            }],
            luzes: 1,
            ..Default::default()
        };
        let r = gabarito(&cena, &g);
        let w = lado as usize;
        for h in variantes() {
            let (flu, _) = rodar(&gpu, com(Config::padrao(g), h), &cena);
            // O anel entre 2 e 8 raios do centro (mais 4 texels).
            let (mut d, mut m, mut se) = (0.0, 0.0, 0.0);
            for k in 0..r.len() {
                let (i, j) = ((k % w) as f32 + 0.5, (k / w) as f32 + 0.5);
                let dist = ((i - 128.0).powi(2) + (j - 128.0).powi(2)).sqrt();
                if dist > 2.0 * raio_texels + 4.0 && dist < 8.0 * raio_texels + 4.0 {
                    let e = f64::from(flu[k][0]) - r[k][0];
                    d += e;
                    se += e * e / (r[k][0] * r[k][0]);
                    m += r[k][0];
                }
            }
            eprintln!(
                "raio {raio_texels} texels · {h:?}: viés {:+.4} · rms rel {:.4}",
                d / m,
                (se / r.len() as f64).sqrt()
            );
        }
    }
}

/// O pior desvio da luz (sala com rebote, 60 pontos) com a grade deslocada 0 · 1 · 2 · 3 blocos da
/// cascata de topo clássica em x e em y (o encaixe da família), nos mesmos pontos do mundo.
fn pior_no_pan(gpu: &ph2d_gpu::GpuContext, h: Option<Holografica>) -> f64 {
    let texel = 0.02;
    let base = ph2d_light2d_gpu::Grade {
        origem_m: [-2.56, -2.56],
        texel_m: texel,
        largura: 512,
        altura: 512,
    };
    let bloco = f64::from(1u32 << (Config::padrao(base).n_cascatas() - 1)) * texel;
    let cena = crate::g9_rebote::sala();
    let mut ref_luz: Option<Vec<[f64; 3]>> = None;
    let mut pior: f64 = 0.0;
    for desloc in [0.0, 1.0, 2.0, 3.0] {
        let g = ph2d_light2d_gpu::Grade {
            origem_m: [
                base.origem_m[0] + desloc * bloco,
                base.origem_m[1] + (desloc % 2.0) * bloco,
            ],
            ..base
        };
        let cfg = Config {
            rebote: true,
            ..com(Config::padrao(g), h)
        };
        let mut s = Solver::novo(gpu, cfg).expect("solver");
        for _ in 0..40 {
            s.quadro(gpu, &cena);
        }
        let flu = ler_fluencia(gpu, &s);
        let gf = s.grade_da_fluencia();
        let pts: Vec<[f64; 3]> = (0..60)
            .map(|k| {
                let p = [0.6 + 0.065 * f64::from(k), 0.6 + 0.04 * f64::from(k)];
                crate::g9_rebote::luz_na_placa(&flu, &gf, p)
            })
            .collect();
        match &ref_luz {
            None => ref_luz = Some(pts),
            Some(r) => {
                for (a, b) in pts.iter().zip(r) {
                    pior = pior.max((a[0] - b[0]).abs() / b[0].max(1e-9));
                }
            }
        }
    }
    pior
}

/// Sonda (à mão): as escolhas da holográfica (o filtro em cruz e o passo dos traços curtos) no Alto.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_hrc_escolhas() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let g = grade(lado, texel);
    let escolhas = [
        HOLOGRAFICA_PADRAO,
        Holografica {
            borrao: false,
            ..HOLOGRAFICA_PADRAO
        },
        Holografica {
            passo: 1.0,
            ..HOLOGRAFICA_PADRAO
        },
        Holografica {
            passo: 0.25,
            ..HOLOGRAFICA_PADRAO
        },
    ];
    for (nome, cena) in [
        ("penumbra pelas cascatas", penumbra_pelas_cascatas()),
        ("céu e vidro", ceu_e_vidro()),
    ] {
        let r = gabarito(&cena, &g);
        let dentro = dentro_de_opaco(&cena, &g);
        let linha: Vec<String> = escolhas
            .iter()
            .map(|&h| {
                let (flu, ms) = rodar(&gpu, com(Config::padrao(g), Some(h)), &cena);
                format!(
                    "passo {} borrão {}: {:.4} {ms:.3} ms",
                    h.passo,
                    h.borrao,
                    rmse_rel(&flu, &r, |k| !dentro(k))
                )
            })
            .collect();
        eprintln!("{nome}: {}", linha.join(" · "));
    }
}

/// ⭐ **G4 (K3) na holográfica** — gate de RELÓGIO (a faixa dos flakes de carga): o custo das
/// cascatas com 1 · 10 · 100 · 1 000 · 10 000 luzes na MESMA geometria varia ≤ 5 %.
#[test]
fn g4_o_custo_da_holografica_nao_depende_do_numero_de_luzes() {
    use crate::cenas::enxame_de_luzes;
    use crate::medicao::{Caso, loadavg, rodadas};
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let mut casos: Vec<Caso> = [1u32, 10, 100, 1_000, 10_000]
        .iter()
        .map(|&n| Caso {
            nome: format!("{n} luzes"),
            solver: Solver::novo(
                &gpu,
                com(Config::padrao(grade(lado, texel)), Some(HOLOGRAFICA_PADRAO)),
            )
            .expect("solver"),
            cena: enxame_de_luzes(n),
        })
        .collect();
    let carga = loadavg();
    let Some(r) = rodadas(&gpu, &mut casos, 15) else {
        eprintln!("G4 holográfica: sem TIMESTAMP_QUERY — saltado");
        return;
    };
    let rc: Vec<f64> = r.iter().map(|x| x.rc[0]).collect();
    let (min, max) = rc
        .iter()
        .fold((f64::INFINITY, 0.0_f64), |(a, b), &v| (a.min(v), b.max(v)));
    eprintln!(
        "G4 holográfica · loadavg {carga} · {:?} ms · variação {:.2} %",
        rc,
        100.0 * (max - min) / min
    );
    assert!(
        (max - min) / min <= 0.05,
        "K3 na holográfica: {:.2} %",
        100.0 * (max - min) / min
    );
}

/// ⭐ **Um enxame denso não tem pintas pretas** (W5c, a foto da cena 2): quadradinhos que brilham com
/// frestas mais estreitas que uma sonda — o interior de cada um (o preenchimento do F3) lê a luz que o
/// rodeia, nunca zero, mesmo onde as quatro sondas de um texel caem dentro de quadradinhos.
#[test]
fn um_enxame_denso_nao_tem_pintas_pretas() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[1];
    let t = texel as f32;
    let mut emissores = Vec::new();
    let mut centros = Vec::new();
    for j in 0..14 {
        for i in 0..14 {
            // 3 texels de lado, frestas de 0,6 texel (mais estreitas que uma sonda).
            let c = [2.0 + 3.6 * t * i as f32, 2.0 + 3.6 * t * j as f32];
            let h = 1.5 * t;
            emissores.push(ph2d_light2d_ecs::Emissor::Poligono {
                vertices: vec![
                    [c[0] - h, c[1] - h],
                    [c[0] + h, c[1] - h],
                    [c[0] + h, c[1] + h],
                    [c[0] - h, c[1] + h],
                ],
                radiancia: [1.0; 3],
                alcance_m: f32::INFINITY,
            });
            centros.push([f64::from(c[0]), f64::from(c[1])]);
        }
    }
    let cena = CenaDeLuz {
        emissores,
        luzes: 1,
        ..Default::default()
    };
    // A captura FINA dos opacos do produto (2× no Alto): na fresta ela diz «fora» e as quatro sondas
    // grossas dizem «dentro» — o caso que a bilinear crua lia.
    let cfg = Config {
        captura: 2,
        ..com(Config::padrao(grade(lado, texel)), Some(HOLOGRAFICA_PADRAO))
    };
    let (flu, _) = rodar(&gpu, cfg, &cena);
    let g = grade(lado, texel);
    let valores: Vec<f64> = centros
        .iter()
        .map(|&p| crate::g9_rebote::luz_na_placa(&flu, &g, p)[0])
        .collect();
    let media = valores.iter().sum::<f64>() / valores.len() as f64;
    let minimo = valores.iter().copied().fold(f64::INFINITY, f64::min);
    eprintln!("enxame denso: o mínimo {minimo:.4} da média {media:.4}");
    // Medido (W5c): 0,40 da média com a cura, 0,21 com a sonda opaca a zero.
    assert!(
        minimo > 0.33 * media,
        "uma pinta preta: {minimo:.4} × {media:.4}"
    );
}

/// ⭐ **A holográfica numa grade QUALQUER** (mutação P17 da W5c): os portões acima correm em grades
/// de potência de 2; a do produto não é — a última coluna parcial de sondas de cada nível existe.
#[test]
fn a_holografica_numa_grade_que_nao_e_potencia_de_dois() {
    let Some(gpu) = gpu() else { return };
    let g = ph2d_light2d_gpu::Grade {
        origem_m: [0.0, 0.0],
        texel_m: 0.02,
        largura: 300,
        altura: 172,
    };
    let mut cena = CenaDeLuz {
        oclusores: vec![crate::comum::oclusor(
            crate::comum::regular([3.0, 1.7], 0.4, 4, 0.3),
            1.0,
        )],
        ceu: [1.0; 3],
        luzes: 1,
        ..Default::default()
    };
    cena.emissores.push(ph2d_light2d_ecs::Emissor::Poligono {
        vertices: crate::comum::regular([5.4, 2.6], 0.2, 8, 0.0),
        radiancia: [2.0; 3],
        alcance_m: f32::INFINITY,
    });
    let cfg = com(Config::padrao(g), Some(HOLOGRAFICA_PADRAO));
    let mut s = Solver::novo(&gpu, cfg).expect("solver");
    s.quadro(&gpu, &cena);
    let ge = s.config().grade;
    let flu = ler_fluencia(&gpu, &s);
    let r = gabarito(&cena, &ge);
    let dentro = dentro_de_opaco(&cena, &ge);
    let e = rmse_rel(&flu, &r, |k| !dentro(k));
    eprintln!("holográfica · grade {}×{}: {e:.4}", ge.largura, ge.altura);
    // Medido (W5c): 0,0032 (a grade efetiva é 320×192: 320 não é múltiplo de 128); com a contagem
    // truncada (P17) a última coluna parcial de sondas lê o céu: 0,0167.
    assert!(e <= 0.0035, "{e:.4}");
}
