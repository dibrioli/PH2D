//! ⭐ **G9 na placa** (W5c, a lei do plano §2.4): um rebote por quadro converge para o gabarito com
//! a série resolvida, na sala vermelha e verde (S8), em cada degrau; e **o rebote não muda com a
//! vista** (G11): o pan da grade e a passagem de oitava nem o fazem piscar nem o põem a reaprender
//! de zero — a fonte do quadro anterior é lida no MUNDO, não no índice do texel.

use std::f64::consts::TAU;

use ph2d_light2d_ecs::{CenaDeLuz, Oclusor};
use ph2d_light2d_gpu::{Config, Grade, MascaraDaFusao, Solver};
use ph2d_light2d_ref::{OpcoesDoRebote, Rebote, luz_em};

use crate::cenas::DEGRAUS;
use crate::comum::{disco, gpu, grade, ler_fluencia};

/// O canto de baixo à esquerda do interior da sala, no mundo.
const O: [f32; 2] = [0.5, 0.5];

fn caixa(x0: f32, y0: f32, x1: f32, y1: f32, albedo: [f32; 3]) -> Oclusor {
    let [ox, oy] = O;
    Oclusor {
        vertices: vec![
            [ox + x0, oy + y0],
            [ox + x1, oy + y0],
            [ox + x1, oy + y1],
            [ox + x0, oy + y1],
        ],
        opacidade: 1.0,
        albedo,
    }
}

/// A sala S8 (a do gabarito, `g9_o_rebote.rs`), com o interior `[0, 4] × [0, 2,5]` a partir de `O`.
pub fn sala() -> CenaDeLuz {
    let branco = [1.0; 3];
    CenaDeLuz {
        emissores: vec![disco(
            [O[0] + 2.0, O[1] + 2.2],
            0.1,
            [1.0; 3],
            f32::INFINITY,
        )],
        oclusores: vec![
            caixa(-0.2, -0.2, 4.2, 0.0, branco),
            caixa(-0.2, 2.5, 4.2, 2.7, branco),
            caixa(-0.2, 0.0, 0.0, 2.5, [1.0, 0.0, 0.0]),
            caixa(4.0, 0.0, 4.2, 2.5, [0.0, 1.0, 0.0]),
        ],
        ..Default::default()
    }
}

/// Junto ao chão (perto da vermelha, no meio, perto da verde) e no meio da sala.
pub fn pontos() -> [[f64; 2]; 4] {
    let (ox, oy) = (f64::from(O[0]), f64::from(O[1]));
    [
        [ox + 0.3, oy + 0.15],
        [ox + 2.0, oy + 0.15],
        [ox + 3.7, oy + 0.15],
        [ox + 1.3, oy + 1.2],
    ]
}

/// A luz recebida (`F/2π`) em `p`, bilinear entre centros de texel da grade fina.
pub fn luz_na_placa(flu: &[[f32; 3]], g: &Grade, p: [f64; 2]) -> [f64; 3] {
    let u = [
        (p[0] - g.origem_m[0]) / g.texel_m - 0.5,
        (p[1] - g.origem_m[1]) / g.texel_m - 0.5,
    ];
    let (i, j) = (u[0].floor(), u[1].floor());
    let (fx, fy) = (u[0] - i, u[1] - j);
    let w = g.largura as usize;
    let em = |di: f64, dj: f64| {
        let (x, y) = ((i + di) as usize, (j + dj) as usize);
        flu[y * w + x].map(f64::from)
    };
    let mut l = [0.0; 3];
    for (di, dj, peso) in [
        (0.0, 0.0, (1.0 - fx) * (1.0 - fy)),
        (1.0, 0.0, fx * (1.0 - fy)),
        (0.0, 1.0, (1.0 - fx) * fy),
        (1.0, 1.0, fx * fy),
    ] {
        let v = em(di, dj);
        for c in 0..3 {
            l[c] += peso * v[c] / TAU;
        }
    }
    l
}

fn com_rebote(g: Grade, rebote: bool) -> Config {
    Config {
        rebote,

        ..Config::padrao(g)
    }
}

/// `quadros` quadros da `cena` e a luz nos `pontos`.
fn depois_de(
    gpu: &ph2d_gpu::GpuContext,
    s: &mut Solver,
    cena: &CenaDeLuz,
    quadros: u32,
) -> [[f64; 3]; 4] {
    for _ in 0..quadros {
        s.quadro(gpu, cena);
    }
    let flu = ler_fluencia(gpu, s);
    let g = s.grade_da_fluencia();
    pontos().map(|p| luz_na_placa(&flu, &g, p))
}

/// O erro relativo à luz LOCAL (a do gabarito no ponto), no pior canal.
fn erro(placa: [f64; 3], gab: [f64; 3]) -> f64 {
    (0..3)
        .map(|c| (placa[c] - gab[c]).abs() / gab[c].max(1e-9))
        .fold(0.0, f64::max)
}

/// Quadros até convergir na placa: a série do gabarito cai a ~0,79 por rebote nesta sala
/// (`sonda_do_rebote`): 60 deixam ~1e-6.
const QUADROS: u32 = 60;

// Os tetos da placa contra o gabarito com a série (Alto · Médio · Baixo), o pior ponto — medido
// + 10 % na RTX 5060 Ti (W5c: 0,0442 · 0,0743 · 0,1306; tabela no handoff). O Baixo é o limite do
// RC clássico a 4 px por célula junto a uma face (o chão perde ~2 % por rebote, `sonda_fonte_…`).
const TOL_G9: [f64; 3] = [0.049, 0.082, 0.144];

/// ⭐ **G9**: o rebote converge para o gabarito em todo degrau, e o chão tinge-se. Controlo: sem o
/// rebote a placa dá a luz SEM rebote do gabarito (branca), longe da série.
#[test]
fn g9_o_rebote_converge_para_o_gabarito_por_degrau() {
    let Some(gpu) = gpu() else { return };
    let cena = sala();
    let serie = Rebote::resolver(
        &cena,
        OpcoesDoRebote {
            passo_m: 0.01,
            rebotes: 300,
        },
    );
    let gab = pontos().map(|p| serie.luz_em(p));
    let sem = pontos().map(|p| luz_em(&cena, p));
    let mut falhas = Vec::new();
    for (k, (degrau, lado, texel)) in DEGRAUS.into_iter().enumerate() {
        let g = grade(lado, texel);
        let mut s = Solver::novo(&gpu, com_rebote(g, true)).expect("solver");
        let placa = depois_de(&gpu, &mut s, &cena, QUADROS);
        let mut d = Solver::novo(&gpu, com_rebote(g, false)).expect("solver");
        let desligado = depois_de(&gpu, &mut d, &cena, 2);
        for i in 0..4 {
            let e = erro(placa[i], gab[i]);
            let e_sem = erro(desligado[i], gab[i]);
            eprintln!(
                "G9 {degrau} ponto {i}: placa {:.4?} gabarito {:.4?} erro {e:.4} · sem rebote {e_sem:.3} (gabarito sem {:.4?})",
                placa[i], gab[i], sem[i]
            );
            if e > TOL_G9[k] {
                falhas.push(format!("G9 {degrau} ponto {i}: {e:.4} > {}", TOL_G9[k]));
            }
            // O controlo: sem o rebote a resposta muda > 5× o teto (no Baixo o efeito do rebote é
            // ~6× o erro do nível: 0,873 contra 0,144).
            if e_sem <= 5.0 * TOL_G9[k] {
                falhas.push(format!("o controlo tem de estar longe: {e_sem}"));
            }
        }
        if placa[0][0] <= 1.3 * placa[0][1] || placa[2][1] <= 1.3 * placa[2][0] {
            falhas.push(format!(
                "{degrau}: o chão não tingiu {:?} {:?}",
                placa[0], placa[2]
            ));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// O espaçamento da cascata de topo (o passo do pan: a origem encaixa nele, `grade.rs` da família).
fn bloco(cfg: &Config) -> f64 {
    f64::from(1u32 << (cfg.n_cascatas() - 1))
}

/// ⭐ **O pan e a oitava** (G11 do rebote): convergido numa vista, a grade anda no mundo um bloco por
/// quadro (como a da família) e a luz de cada ponto fica a menos de `TOL_PAN` da convergida —
/// nenhum quadro pisca. Depois a oitava (o texel dobra): o 1.º quadro não reaprende de zero (sem a
/// fonte lida no mundo ele cairia à luz sem rebote, ~12 % da convergida).
#[test]
fn o_rebote_nao_pisca_no_pan_nem_reaprende_na_oitava() {
    let Some(gpu) = gpu() else { return };
    let cena = sala();
    let texel = 0.02;
    let g = Grade {
        origem_m: [-2.56, -2.56],
        texel_m: texel,
        largura: 512,
        altura: 512,
    };
    let cfg = com_rebote(g, true);
    let passo = bloco(&cfg) * texel;
    let mut s = Solver::novo(&gpu, cfg).expect("solver");
    let convergida = depois_de(&gpu, &mut s, &cena, QUADROS);
    // A régua só mede o rebote se ele existe: a convergida tem de passar do dobro da luz sem ele.
    let sem = pontos().map(|p| luz_em(&cena, p));
    for i in 0..4 {
        assert!(
            convergida[i][0] > 2.0 * sem[i][0],
            "sem rebote o pan não mede nada: {:?} × {:?}",
            convergida[i],
            sem[i]
        );
    }
    let mut pior: f64 = 0.0;
    for (dx, dy) in [(1, 0), (1, 0), (0, 1), (-1, 1), (-1, -1), (0, -1)] {
        let o = s.config().grade.origem_m;
        s.reposicionar(
            [o[0] + f64::from(dx) * passo, o[1] + f64::from(dy) * passo],
            texel,
        )
        .expect("reposicionar");
        let agora = depois_de(&gpu, &mut s, &cena, 1);
        for i in 0..4 {
            pior = pior.max(erro(agora[i], convergida[i]));
        }
    }
    eprintln!("pan do rebote: pior {pior:.5} da luz convergida");
    assert!(
        pior <= TOL_PAN,
        "o rebote piscou no pan: {pior:.5} > {TOL_PAN}"
    );

    // A oitava: a mesma origem, o texel dobra.
    let o = s.config().grade.origem_m;
    s.reposicionar(o, 2.0 * texel).expect("oitava");
    let primeiro = depois_de(&gpu, &mut s, &cena, 1);
    let mut longe = Solver::novo(
        &gpu,
        com_rebote(
            Grade {
                origem_m: o,
                texel_m: 2.0 * texel,
                ..g
            },
            true,
        ),
    )
    .expect("solver");
    let alvo = depois_de(&gpu, &mut longe, &cena, QUADROS);
    let e = (0..4)
        .map(|i| erro(primeiro[i], alvo[i]))
        .fold(0.0, f64::max);
    eprintln!("oitava: o 1.º quadro a {e:.4} da convergida no texel novo");
    assert!(
        e <= TOL_OITAVA,
        "o rebote reaprendeu na oitava: {e:.4} > {TOL_OITAVA}"
    );
}

// Medido na RTX 5060 Ti (W5c): o pan 0,00000 (a fonte lida no mundo cai nos MESMOS texels: a origem
// encaixa no bloco); o teto é o ruído do f16 da fonte. A oitava 0,0593 + 10 % (o texel dobra: a fonte
// do quadro anterior é lida pela bilinear das que a TÊM).
const TOL_PAN: f64 = 1e-3;
const TOL_OITAVA: f64 = 0.066;

/// Sonda (à mão, `--ignored --nocapture`): UMA parede sob o céu (o caso da fórmula fechada do
/// gabarito) — a luz a várias alturas, sem e com o rebote, sem e com a fusão mascarada.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_meio_ceu_na_placa() {
    let Some(gpu) = gpu() else { return };
    let mut cena = CenaDeLuz {
        oclusores: vec![Oclusor {
            vertices: vec![[0.06, 1.0], [5.06, 1.0], [5.06, 1.2], [0.06, 1.2]],
            opacidade: 1.0,
            albedo: [0.5; 3],
        }],
        ..Default::default()
    };
    cena.ceu = [1.0; 3];
    let alturas = [0.005, 0.015, 0.03, 0.06, 0.15, 0.5];
    let serie = Rebote::resolver(
        &cena,
        OpcoesDoRebote {
            passo_m: 0.01,
            rebotes: 40,
        },
    );
    for (degrau, lado, texel) in DEGRAUS {
        let g = grade(lado, texel);
        for mascara in [
            MascaraDaFusao::Nenhuma,
            MascaraDaFusao::PelaSonda,
            MascaraDaFusao::PeloFim,
        ] {
            for rebote in [false, true] {
                let cfg = Config {
                    rebote,
                    mascara_da_fusao: mascara,
                    ..Config::padrao(g)
                };
                let mut s = Solver::novo(&gpu, cfg).expect("solver");
                for _ in 0..40 {
                    s.quadro(&gpu, &cena);
                }
                let flu = ler_fluencia(&gpu, &s);
                let gf = s.grade_da_fluencia();
                let linha: Vec<String> = alturas
                    .iter()
                    .map(|&d| {
                        let p = [2.56, 1.2 + d];
                        let placa = luz_na_placa(&flu, &gf, p)[0];
                        let gab = if rebote {
                            serie.luz_em(p)[0]
                        } else {
                            luz_em(&cena, p)[0]
                        };
                        format!("{d}: {:+.3}", placa / gab - 1.0)
                    })
                    .collect();
                eprintln!(
                    "{degrau} {mascara:?} rebote {rebote}: {}",
                    linha.join(" · ")
                );
            }
        }
    }
}

/// Sonda (à mão): a sala, rebote a rebote — o quadro `n` tem `n − 1` rebotes.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_sala_rebote_a_rebote() {
    let Some(gpu) = gpu() else { return };
    let cena = sala();
    for (degrau, lado, texel) in DEGRAUS {
        let g = grade(lado, texel);
        let cfg = Config {
            rebote: true,
            ..Config::padrao(g)
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        for n in 1..=8u32 {
            let placa = depois_de(&gpu, &mut s, &cena, 1);
            let serie = Rebote::resolver(
                &cena,
                OpcoesDoRebote {
                    passo_m: 0.01,
                    rebotes: n - 1,
                },
            );
            let e: Vec<String> = pontos()
                .iter()
                .zip(&placa)
                .map(|(&p, l)| {
                    let r = serie.luz_em(p);
                    format!("{:+.3}/{:+.3}", l[0] / r[0] - 1.0, l[2] / r[2] - 1.0)
                })
                .collect();
            eprintln!("{degrau} quadro {n}: (R/B) {}", e.join(" · "));
        }
    }
}

/// A fonte do rebote (rgba16f) texel a texel.
fn ler_fonte(gpu: &ph2d_gpu::GpuContext, s: &Solver) -> Vec<[f32; 4]> {
    let tex = s.fonte_do_rebote().expect("com o rebote");
    crate::comum::ler_bytes(gpu, tex)
        .as_chunks::<8>()
        .0
        .iter()
        .map(|t| {
            let c = |k: usize| half::f16::from_le_bytes([t[2 * k], t[2 * k + 1]]).to_f32();
            [c(0), c(1), c(2), c(3)]
        })
        .collect()
}

/// Sonda (à mão): a FONTE ao longo das faces da sala (o 1.º texel de dentro de cada parede) contra a
/// luz do gabarito na face, no quadro 3 (a fonte tem 2 rebotes).
#[test]
#[ignore = "sonda: imprime"]
fn sonda_fonte_ao_longo_das_faces() {
    let Some(gpu) = gpu() else { return };
    let cena = sala();
    let (ox, oy) = (f64::from(O[0]), f64::from(O[1]));
    for quadros in [2u32, 3, 6] {
        let serie = Rebote::resolver(
            &cena,
            OpcoesDoRebote {
                passo_m: 0.01,
                rebotes: quadros - 1,
            },
        );
        for (degrau, lado, texel) in DEGRAUS {
            let g = grade(lado, texel);
            let cfg = Config {
                rebote: true,
                ..Config::padrao(g)
            };
            let mut s = Solver::novo(&gpu, cfg).expect("solver");
            for _ in 0..quadros {
                s.quadro(&gpu, &cena);
            }
            let f = ler_fonte(&gpu, &s);
            let gr = s.config().grade;
            let w = gr.largura as usize;
            // (nome, ponto na face, passo para DENTRO da parede)
            let faces: [(&str, fn(f64) -> [f64; 2], [f64; 2]); 3] = [
                ("chão", |t| [t * 4.0, 0.0], [0.0, -1.0]),
                ("vermelha", |t| [0.0, t * 2.5], [-1.0, 0.0]),
                ("teto", |t| [t * 4.0, 2.5], [0.0, 1.0]),
            ];
            for (nome, ponto, dentro) in faces {
                let linha: Vec<String> = [0.1, 0.3, 0.5, 0.7, 0.9]
                    .iter()
                    .map(|&t| {
                        let p = ponto(t);
                        let face = [ox + p[0], oy + p[1]];
                        let q = [
                            face[0] + dentro[0] * 0.5 * gr.texel_m,
                            face[1] + dentro[1] * 0.5 * gr.texel_m,
                        ];
                        let i = ((q[0] - gr.origem_m[0]) / gr.texel_m) as usize;
                        let j = ((q[1] - gr.origem_m[1]) / gr.texel_m) as usize;
                        let v = f[j * w + i];
                        let fora = [face[0] - dentro[0] * 1e-6, face[1] - dentro[1] * 1e-6];
                        let r = serie.luz_em(fora);
                        format!("{t}: {:+.3}", f64::from(v[2]) / r[2] - 1.0)
                    })
                    .collect();
                eprintln!(
                    "quadro {quadros} {degrau} {nome} (B): {}",
                    linha.join(" · ")
                );
            }
        }
    }
}

/// Sonda (à mão): o TETO emissor uniforme e o chão só a bloquear — a luz que chega junto à face do
/// chão vinda de longe (o transporte dos rebotes ≥ 2), placa × gabarito, sem e com a máscara.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_teto_emissor_sobre_o_chao() {
    let Some(gpu) = gpu() else { return };
    let (ox, oy) = (O[0], O[1]);
    let cena = CenaDeLuz {
        emissores: vec![ph2d_light2d_ecs::Emissor::Poligono {
            vertices: vec![
                [ox - 0.2, oy + 2.5],
                [ox + 4.2, oy + 2.5],
                [ox + 4.2, oy + 2.7],
                [ox - 0.2, oy + 2.7],
            ],
            radiancia: [1.0; 3],
            alcance_m: f32::INFINITY,
        }],
        oclusores: vec![Oclusor {
            vertices: vec![
                [ox - 0.2, oy - 0.2],
                [ox + 4.2, oy - 0.2],
                [ox + 4.2, oy],
                [ox - 0.2, oy],
            ],
            opacidade: 1.0,
            albedo: [0.0; 3],
        }],
        luzes: 1,
        ..Default::default()
    };
    let alturas = [0.003, 0.01, 0.03, 0.1, 0.3, 1.0];
    for (degrau, lado, texel) in DEGRAUS {
        let g = grade(lado, texel);
        for mascara in [
            MascaraDaFusao::Nenhuma,
            MascaraDaFusao::PelaSonda,
            MascaraDaFusao::PeloFim,
        ] {
            let cfg = Config {
                mascara_da_fusao: mascara,
                ..Config::padrao(g)
            };
            let mut s = Solver::novo(&gpu, cfg).expect("solver");
            s.quadro(&gpu, &cena);
            let flu = ler_fluencia(&gpu, &s);
            let gf = s.grade_da_fluencia();
            let linha: Vec<String> = alturas
                .iter()
                .map(|&d| {
                    let p = [f64::from(ox) + 1.0, f64::from(oy) + d];
                    let placa = luz_na_placa(&flu, &gf, p)[0];
                    // E logo ABAIXO do teto que brilha (o termo próprio de uma face que emite).
                    let q = [f64::from(ox) + 1.0, f64::from(oy) + 2.5 - d];
                    let teto = luz_na_placa(&flu, &gf, q)[0];
                    format!(
                        "{d}: {:+.3} | teto {:+.3}",
                        placa / luz_em(&cena, p)[0] - 1.0,
                        teto / luz_em(&cena, q)[0] - 1.0
                    )
                })
                .collect();
            eprintln!("{degrau} {mascara:?}: {}", linha.join(" · "));
        }
    }
}

/// Sonda (à mão): o perfil da fonte e da emissão numa coluna através do chão (x = 2,5 m), quadro 2.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_perfil_do_chao() {
    let Some(gpu) = gpu() else { return };
    let cena = sala();
    let serie = Rebote::resolver(
        &cena,
        OpcoesDoRebote {
            passo_m: 0.01,
            rebotes: 1,
        },
    );
    for (degrau, lado, texel) in DEGRAUS {
        let g = grade(lado, texel);
        let cfg = Config {
            rebote: true,
            ..Config::padrao(g)
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        for _ in 0..2 {
            s.quadro(&gpu, &cena);
        }
        let f = ler_fonte(&gpu, &s);
        let e: Vec<[f32; 4]> = crate::comum::ler_bytes(&gpu, s.cena_capturada()[0])
            .as_chunks::<8>()
            .0
            .iter()
            .map(|t| {
                let c = |k: usize| half::f16::from_le_bytes([t[2 * k], t[2 * k + 1]]).to_f32();
                [c(0), c(1), c(2), c(3)]
            })
            .collect();
        let gr = s.config().grade;
        let w = gr.largura as usize;
        let i = ((2.5 + 0.5 - gr.origem_m[0]) / gr.texel_m) as usize;
        let face = 0.5 / gr.texel_m;
        eprintln!("{degrau}: face do chão na linha {face}");
        for j in (face as usize).saturating_sub(4)..(face as usize + 3) {
            let y = gr.origem_m[1] + (j as f64 + 0.5) * gr.texel_m;
            let p = [
                gr.origem_m[0] + (i as f64 + 0.5) * gr.texel_m,
                y.max(0.5 + 1e-6),
            ];
            eprintln!(
                "  linha {j}: fonte {:.4} (a {}) · emissão {:.4} · gabarito na face/ponto {:.4}",
                f[j * w + i][2],
                f[j * w + i][3],
                e[j * w + i][2],
                serie.luz_em(p)[2]
            );
        }
    }
}
