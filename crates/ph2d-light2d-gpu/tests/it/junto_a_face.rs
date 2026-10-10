//! ⭐ W5c — **a luz das cascatas junto a uma face** (o ABERTO 1 da W5a): o pior erro, contra o
//! gabarito, a menos de 1,5 texel de uma face, com a face em qualquer posição dentro do texel e em
//! texel par ou ímpar (a rede das sondas de cada cascata é de 2^i texels: uma face que cai sobre uma
//! sonda de cima é o caso que a fusão erra). Dois casos: a luz que vem de LONGE a chegar junto a uma
//! face que não brilha, e o termo PRÓPRIO de uma face que brilha.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_gpu::{Config, MascaraDaFusao, Solver};
use ph2d_light2d_ref::luz_em;

use crate::cenas::DEGRAUS;
use crate::comum::{gpu, grade, ler_fluencia};
use crate::g9_rebote::luz_na_placa;

/// A placa emissora em cima (`y ∈ [cima, cima + 0,2]`) e o chão que bloqueia em baixo
/// (`y ∈ [baixo − 0,2, baixo]`), de `x = 0,3` a `4,7`.
fn cena(baixo: f32, cima: f32) -> CenaDeLuz {
    let ret = |y0: f32, y1: f32| vec![[0.3, y0], [4.7, y0], [4.7, y1], [0.3, y1]];
    CenaDeLuz {
        emissores: vec![Emissor::Poligono {
            vertices: ret(cima, cima + 0.2),
            radiancia: [1.0; 3],
            alcance_m: f32::INFINITY,
        }],
        oclusores: vec![Oclusor {
            vertices: ret(baixo - 0.2, baixo),
            opacidade: 1.0,
            albedo: [0.0; 3],
        }],
        luzes: 1,
        ..Default::default()
    }
}

/// O pior erro relativo nos pontos a menos de 1,5 texel das duas faces: `(longe, próprio)`.
pub fn pior_junto_as_faces(
    gpu: &ph2d_gpu::GpuContext,
    cfg: Config,
    baixo: f32,
    cima: f32,
) -> (f64, f64) {
    let c = cena(baixo, cima);
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    s.quadro(gpu, &c);
    let flu = ler_fluencia(gpu, &s);
    let gf = s.grade_da_fluencia();
    let t = cfg.grade.texel_m;
    let mut pior = (0.0_f64, 0.0_f64);
    for k in 1..=6 {
        let d = 0.25 * f64::from(k) * t;
        for x in [1.3, 2.5, 3.7] {
            let p = [x, f64::from(baixo) + d];
            let e = (luz_na_placa(&flu, &gf, p)[0] / luz_em(&c, p)[0] - 1.0).abs();
            pior.0 = pior.0.max(e);
            let q = [x, f64::from(cima) - d];
            let e = (luz_na_placa(&flu, &gf, q)[0] / luz_em(&c, q)[0] - 1.0).abs();
            pior.1 = pior.1.max(e);
        }
    }
    pior
}

// O pior erro junto a uma face, com a face em 0 · ¼ · ½ · ¾ · 1 · 1½ texel (Alto · Médio · Baixo),
// medido + 10 % na RTX 5060 Ti (W5c). A luz de LONGE: 0,023 · 0,030 · 0,046 — o RC de antes dava
// 0,434 · 0,373 · 0,325 (a sonda de cima sobre a face zerava a fusão). O termo PRÓPRIO: 0,144 ·
// 0,146 · 0,215 (o pior é a face em 1½ texel, igual ao de antes: 0,147 · 0,150 · 0,153).
const TOL_LONGE: [f64; 3] = [0.026, 0.033, 0.051];
const TOL_PROPRIO: [f64; 3] = [0.159, 0.161, 0.237];

/// ⭐ **A luz das cascatas junto a uma face é a do gabarito** em todo alinhamento (o ABERTO 1).
#[test]
fn a_luz_junto_a_uma_face_e_a_do_gabarito() {
    let Some(gpu) = gpu() else { return };
    let mut falhas = Vec::new();
    for (k, (degrau, lado, texel)) in DEGRAUS.into_iter().enumerate() {
        let t = texel as f32;
        for desvio in [0.0_f32, 0.25, 0.5, 0.75, 1.0, 1.5] {
            let cfg = Config::padrao(grade(lado, texel));
            let (l, p) = pior_junto_as_faces(&gpu, cfg, (40.0 + desvio) * t, (100.0 + desvio) * t);
            eprintln!("junto à face · {degrau} · desvio {desvio}: longe {l:.4} · próprio {p:.4}");
            if l > TOL_LONGE[k] || p > TOL_PROPRIO[k] {
                falhas.push(format!(
                    "{degrau} desvio {desvio}: longe {l:.4} · próprio {p:.4}"
                ));
            }
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// Sonda (à mão, `--ignored --nocapture`): as variantes da fusão e do início × o desvio da face.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_junto_a_face() {
    let Some(gpu) = gpu() else { return };
    let variantes = [
        (MascaraDaFusao::Nenhuma, false),
        (MascaraDaFusao::PelaSonda, false),
        (MascaraDaFusao::PeloFim, false),
        (MascaraDaFusao::PelaSonda, true),
        (MascaraDaFusao::PeloFim, true),
    ];
    for (degrau, lado, texel) in DEGRAUS {
        let t = texel as f32;
        for (mascara, inicio) in variantes {
            let mut linha = Vec::new();
            for desvio in [0.0_f32, 0.25, 0.5, 0.75, 1.0, 1.5] {
                let baixo = 40.0 * t + desvio * t;
                let cima = 100.0 * t + desvio * t;
                let cfg = Config {
                    mascara_da_fusao: mascara,
                    fronteira_na_face: inicio,
                    ..Config::padrao(grade(lado, texel))
                };
                let (l, p) = pior_junto_as_faces(&gpu, cfg, baixo, cima);
                linha.push(format!("{desvio}: {l:.3}/{p:.3}"));
            }
            eprintln!(
                "{degrau} {mascara:?} início {inicio} (longe/próprio): {}",
                linha.join(" · ")
            );
        }
    }
}

/// Sonda (à mão): junto à face, a cascata holográfica × a clássica, por degrau e desvio.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_junto_a_face_holografica() {
    let Some(gpu) = gpu() else { return };
    for (degrau, lado, texel) in DEGRAUS {
        let t = texel as f32;
        for h in [None, Some(ph2d_light2d_gpu::HOLOGRAFICA_PADRAO)] {
            let mut linha = Vec::new();
            for desvio in [0.0_f32, 0.25, 0.5, 0.75, 1.0, 1.5] {
                let cfg = Config {
                    holografica: h,
                    ..Config::padrao(grade(lado, texel))
                };
                let (l, p) =
                    pior_junto_as_faces(&gpu, cfg, (40.0 + desvio) * t, (100.0 + desvio) * t);
                linha.push(format!("{desvio}: {l:.3}/{p:.3}"));
            }
            eprintln!(
                "{degrau} holográfica {} (longe/próprio): {}",
                h.is_some(),
                linha.join(" · ")
            );
        }
    }
}

/// Sonda (à mão): o ANEL do G3 (a região livre à volta de uma luz pequena) por variante.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_anel_por_variante() {
    use crate::cenas::{penumbra_pelas_cascatas, regiao_livre_longe};
    use crate::comum::{gabarito, rmse_rel};
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let cena = penumbra_pelas_cascatas();
    let g = grade(lado, texel);
    let sem = gabarito(
        &CenaDeLuz {
            oclusores: vec![],
            ..cena.clone()
        },
        &g,
    );
    let r = gabarito(&cena, &g);
    let longe = regiao_livre_longe(lado, texel);
    for mascara in [
        MascaraDaFusao::Nenhuma,
        MascaraDaFusao::PelaSonda,
        MascaraDaFusao::PeloFim,
    ] {
        for fronteira in [false, true] {
            let cfg = Config {
                mascara_da_fusao: mascara,
                fronteira_na_face: fronteira,
                ..Config::padrao(g)
            };
            let mut s = Solver::novo(&gpu, cfg).expect("solver");
            s.quadro(&gpu, &cena);
            let flu = ler_fluencia(&gpu, &s);
            let livre = |k: usize| longe(k) && (r[k][0] - sem[k][0]).abs() <= 1e-9 * sem[k][0];
            let e = rmse_rel(&flu, &r, livre);
            let (mut d, mut m) = (0.0, 0.0);
            for k in (0..r.len()).filter(|&k| livre(k)) {
                d += f64::from(flu[k][0]) - r[k][0];
                m += r[k][0];
            }
            eprintln!(
                "anel {mascara:?} fronteira {fronteira}: {e:.4} · viés {:+.4}",
                d / m
            );
        }
    }
}
