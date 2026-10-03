//! Os gates da lei do céu — a quadratura crua, o forno e o oráculo (o Cycles).

use crate::prefiltro::{fontes_irr, irradiancia};
use crate::{Ceu, Embarcado, Panorama, Rgb};

fn luma(c: Rgb) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Direcções espalhadas pela esfera (Fibonacci), fora das grelhas do atlas.
fn direcoes(n: usize) -> Vec<[f32; 3]> {
    let phi = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    (0..n)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.37) / n as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = phi * i as f32 + 0.11;
            [r * a.cos(), y, r * a.sin()]
        })
        .collect()
}

fn quantis(mut v: Vec<f32>) -> (f32, f32, f32) {
    v.sort_by(f32::total_cmp);
    let q = |p: f32| v[((v.len() - 1) as f32 * p) as usize];
    (q(0.5), q(0.99), *v.last().expect("não vazio"))
}

/// O erro relativo de `a` contra `b`, na luma, com um piso de `2 %` da média do céu (um céu escuro
/// não pode ter erro relativo infinito por um resíduo do ruído da quadratura).
fn rel(a: Rgb, b: Rgb, piso: f32) -> f32 {
    (0..3)
        .map(|i| (a[i] - b[i]).abs() / (b[i].max(0.0) + piso))
        .fold(0.0, f32::max)
}

#[test]
fn os_embarcados_decodificam() {
    for e in Embarcado::TODOS {
        let p = e.panorama();
        assert_eq!((p.largura, p.altura), (1024, 512), "{e:?}");
        assert!(p.rgb.iter().flatten().all(|c| c.is_finite() && *c >= 0.0), "{e:?}");
        assert!(luma(p.media()) > 0.0, "{e:?}");
    }
}

/// ⭐ **O forno:** um céu da mesma radiância `L` devolve `L` em toda a direcção, a toda a rugosidade, e
/// a irradiância normalizada também é `L`. ⚠️ A tolerância é a do recurso: o atlas é `f16` (`2⁻¹¹`).
#[test]
fn um_ceu_chapado_devolve_l() {
    let l = [0.3, 0.7, 1.9];
    let p = Panorama {
        largura: 128,
        altura: 64,
        rgb: vec![l; 128 * 64],
    };
    let c = Ceu::novo(&p);
    for d in direcoes(97) {
        for a in [0.0, 0.003, 0.05, 0.2, 0.5, 1.0] {
            let r = c.radiance(d, a);
            assert!(rel(r, l, 0.0) < 5.0e-4, "radiância {r:?} em {d:?} α={a}");
        }
        let e = c.irradiance(d);
        assert!(rel(e, l, 0.0) < 2.0e-3, "irradiância {e:?} em {d:?}");
    }
}

/// ⭐ **A verdade SEM ruído**: o lóbulo somado sobre TODOS os texels do panorama cru, cada um com o
/// seu ângulo sólido — `Σ L·Ω·K / Σ Ω·K`, `K(l) = (R·l)·D(h)`, `h = (R + l)/|R + l|`. É o limite da
/// amostragem do `prefiltro::lobo` (a densidade dela em `l` é `D/4` quando `N = V`).
///
/// ⛔ Medido (02/10): a quadratura POR AMOSTRAS não serve de régua num céu com lâmpadas pequenas —
/// no interior, `16 384` e `65 536` amostras discordavam `8 %` na mediana a `α = 0,7`.
pub(crate) fn verdade(p: &Panorama, r: [f32; 3], alpha: f32) -> Rgb {
    let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    let r = r.map(|c| f64::from(c / l));
    let a2 = f64::from(alpha) * f64::from(alpha);
    let (mut s, mut w) = ([0.0f64; 3], 0.0f64);
    for y in 0..p.altura {
        let om = p.angulo_solido(y);
        for x in 0..p.largura {
            let d = p.direcao(x, y).map(f64::from);
            let rl = r[0] * d[0] + r[1] * d[1] + r[2] * d[2];
            if rl <= 0.0 {
                continue;
            }
            let h = [r[0] + d[0], r[1] + d[1], r[2] + d[2]];
            let hl2 = h[0] * h[0] + h[1] * h[1] + h[2] * h[2];
            let nh2 = (r[0] * h[0] + r[1] * h[1] + r[2] * h[2]).powi(2) / hl2;
            let den = nh2 * (a2 - 1.0) + 1.0;
            let k = om * rl * a2 / (std::f64::consts::PI * den * den);
            let c = p.rgb[(y * p.largura + x) as usize];
            for i in 0..3 {
                s[i] += f64::from(c[i]) * k;
            }
            w += k;
        }
    }
    s.map(|v| (v / w) as f32)
}

/// ⭐⭐ **O pré-filtro é a CONVOLUÇÃO** — o atlas (níveis em `√α` + bilinear + amostragem filtrada)
/// contra a [`verdade`] somada sobre o panorama cru, em direcções e `α` fora das grelhas. Céus com sol
/// (o pôr do sol), com detalhe fino (a floresta) e com lâmpadas pequenas (o interior).
///
/// ⚠️ A partir de `α = 0,03`: abaixo, o lóbulo é mais estreito do que um texel do panorama e a soma
/// por texels deixa de ser a verdade — ali a régua é o espelho (`o_espelho_e_o_panorama`).
///
/// Medido (02/10, 17 níveis, `f16`): `α ≥ 0,09` — mediana `≤ 0,74 %`, máximo `≤ 5,5 %` (o interior);
/// `α = 0,03` (nível amostrado) — mediana `≤ 1,3 %`, máximo `≤ 30 %` (a borda de uma lâmpada).
#[test]
fn o_pre_filtro_e_a_convolucao() {
    use rayon::prelude::*;
    for e in [Embarcado::Floresta, Embarcado::Por, Embarcado::Interior] {
        let p = e.panorama();
        let t0 = std::time::Instant::now();
        let c = Ceu::novo(&p);
        let ms = t0.elapsed().as_millis();
        let piso = 0.02 * luma(p.media());
        for a in [0.03f32, 0.09, 0.2, 0.42, 0.7, 0.95] {
            let erros: Vec<f32> = direcoes(120)
                .into_par_iter()
                .map(|d| rel(c.radiance(d, a), verdade(&p, d, a), piso))
                .collect();
            let (p50, p99, max) = quantis(erros);
            eprintln!("{e:?} α={a}: p50 {p50:.4} p99 {p99:.4} max {max:.4}  (atlas em {ms} ms)");
            let (b50, bmax) = if a >= 0.09 { (0.01, 0.08) } else { (0.02, 0.4) };
            assert!(p50 <= b50 && max <= bmax, "{e:?} α={a}: p50 {p50} max {max}");
        }
    }
}

/// ⭐ **O espelho (`α = 0`) é o panorama** lido bilinear — o que o octaédrico de `256` perde contra o
/// equiretangular de `1024`.
#[test]
fn o_espelho_e_o_panorama() {
    for e in [Embarcado::Floresta, Embarcado::Por, Embarcado::Interior] {
        let p = e.panorama();
        let c = Ceu::novo(&p);
        let piso = 0.02 * luma(p.media());
        let erros: Vec<f32> = direcoes(2000)
            .into_iter()
            .map(|d| rel(c.radiance(d, 0.0), p.radiancia(d), piso))
            .collect();
        let (p50, p99, max) = quantis(erros);
        eprintln!("{e:?} espelho: p50 {p50:.4} p99 {p99:.4} max {max:.4}");
        // Medido (02/10): a folhagem da floresta `7,9 %` (o octaédrico de `512` contra o `1K`); os
        // outros `≤ 0,5 %`.
        assert!(p50 <= 0.1, "{e:?}: p50 {p50}");
    }
}

/// ⭐⭐ **A irradiância é a SOMA sobre o panorama cru** (`1024 × 512`, sem redução).
#[test]
fn a_irradiancia_e_a_soma_crua() {
    for e in [Embarcado::Floresta, Embarcado::Por] {
        let p = e.panorama();
        let c = Ceu::novo(&p);
        let crua = fontes_irr(std::slice::from_ref(&p));
        let piso = 0.02 * luma(p.media());
        let erros: Vec<f32> = direcoes(300)
            .into_iter()
            .map(|n| rel(c.irradiance(n), irradiancia(&crua, n), piso))
            .collect();
        let (p50, p99, max) = quantis(erros);
        eprintln!("{e:?} irradiância: p50 {p50:.4} p99 {p99:.4} max {max:.4}");
        // Medido (02/10): mediana `≤ 0,3 %`, máximo `≤ 7,2 %`.
        assert!(p50 <= 0.005 && max <= 0.1, "{e:?}: p50 {p50} max {max}");
    }
}

struct Linha {
    ceu: String,
    tipo: String,
    n: [f32; 3],
    rgb: Rgb,
}

fn oraculo() -> Vec<Linha> {
    include_str!("../fixtures/oraculo_blender.csv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("ceu,"))
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            let f = |i: usize| c[i].parse::<f32>().expect("número");
            Linha {
                ceu: c[0].to_owned(),
                tipo: c[1].to_owned(),
                n: [f(2), f(3), f(4)],
                rgb: [f(5), f(6), f(7)],
            }
        })
        .collect()
}

/// O erro do atlas contra o oráculo, com a normal transformada por `t` (a identidade é a medição; o
/// resto são os CONTROLOS — orientações erradas que a fixtura tem de reprovar).
fn erro_contra_oraculo(t: impl Fn([f32; 3]) -> [f32; 3]) -> Vec<(String, String, f32, f32)> {
    let linhas = oraculo();
    let mut out = Vec::new();
    for e in [Embarcado::Floresta, Embarcado::Por, Embarcado::Interior] {
        let p = e.panorama();
        let c = Ceu::novo(&p);
        let piso = 0.02 * luma(p.media());
        for tipo in ["difusa", "espelho"] {
            let erros: Vec<f32> = linhas
                .iter()
                .filter(|l| l.ceu == e.chave() && l.tipo == tipo)
                .map(|l| {
                    let n = t(l.n);
                    let nosso = if tipo == "difusa" {
                        c.irradiance(n)
                    } else {
                        // O reflexo da vista `(0, 0, 1)` em `n`.
                        let k = 2.0 * n[2];
                        c.radiance([k * n[0], k * n[1], k * n[2] - 1.0], 0.0)
                    };
                    rel(nosso, l.rgb, piso)
                })
                .collect();
            assert!(erros.len() > 150, "a fixtura tem as linhas de {e:?}/{tipo}");
            let (p50, _, max) = quantis(erros);
            out.push((e.chave().to_owned(), tipo.to_owned(), p50, max));
        }
    }
    out
}

/// ⭐⭐⭐ **A orientação é a do Blender** — a esfera difusa (`E(n)/π`) e a esfera espelho (o
/// panorama no reflexo) contra o Cycles, sobre os MESMOS EXR. Os CONTROLOS: o céu espelhado em `x`
/// e o céu de pernas para o ar têm de dar erros muito maiores.
#[test]
fn a_orientacao_e_a_do_blender() {
    // Medido (02/10): difusa — mediana `≤ 0,7 %` (o ruído do Cycles a `2048` amostras); espelho —
    // `≤ 0,6 %`, e a folhagem da floresta `8,6 %`. Os controlos: `≥ 49 %`.
    for (ceu, tipo, p50, max) in erro_contra_oraculo(|n| n) {
        eprintln!("oráculo {ceu}/{tipo}: p50 {p50:.4} max {max:.4}");
        let b = if tipo == "difusa" { 0.01 } else { 0.1 };
        assert!(p50 <= b, "oráculo {ceu}/{tipo}: p50 {p50}");
    }
    let controlos = [
        |n: [f32; 3]| [-n[0], n[1], n[2]],
        |n: [f32; 3]| [n[0], -n[1], n[2]],
        |n: [f32; 3]| [n[2], n[1], -n[0]],
    ];
    for (i, t) in controlos.into_iter().enumerate() {
        for (ceu, tipo, p50, max) in erro_contra_oraculo(t) {
            eprintln!("CONTROLO {i} {ceu}/{tipo}: p50 {p50:.4} max {max:.4}");
            assert!(p50 >= 0.2, "o controlo {i} {ceu}/{tipo} tem de reprovar: p50 {p50}");
        }
    }
}


/// ⚙️ **Instrumento** (não é gate): o erro do atlas contra a convolução, decomposto por nível —
/// (A) no centro dos texels ao `α` do nível: só a amostragem; (B) em direcções quaisquer: + a leitura
/// bilinear; (C) a meio caminho do nível anterior: + a interpolação em `√α`. Foi ele que mediu os
/// `LADOS`, os `NIVEIS`, a `ALFA_EXACTA` e as `amostras`.
#[test]
#[ignore = "instrumento"]
fn instrumento_decomposicao() {
    use rayon::prelude::*;
    for e in [Embarcado::Floresta, Embarcado::Interior] {
        let p = e.panorama();
        let c = Ceu::novo(&p);
        let piso = 0.02 * luma(p.media());
        for k in 1..crate::NIVEIS {
            let n = crate::LADOS[k];
            let a = (k as f32 / (crate::NIVEIS - 1) as f32).powi(2);
            // (A) no centro dos texels, ao α do nível: só a amostragem.
            let centros: Vec<[f32; 3]> = (0..40u32)
                .map(|i| crate::prefiltro::centro(1 + (i * 37) % n, 1 + (i * 53 + 7) % n, n))
                .collect();
            let ea = quantis(centros.par_iter().map(|d| rel(c.radiance(*d, a), verdade(&p, *d, a), piso)).collect());
            // (B) direcções quaisquer, ao α do nível: amostragem + bilinear.
            let eb = quantis(direcoes(40).par_iter().map(|d| rel(c.radiance(*d, a), verdade(&p, *d, a), piso)).collect());
            // (C) entre este nível e o anterior: + interpolação em √α.
            let am = ((k as f32 - 0.5) / (crate::NIVEIS - 1) as f32).powi(2);
            let ec = quantis(direcoes(40).par_iter().map(|d| rel(c.radiance(*d, am), verdade(&p, *d, am), piso)).collect());
            eprintln!("{e:?} k={k} α={a:.4} lado {n}: A p50 {:.4} max {:.4} | B p50 {:.4} max {:.4} | C(α={am:.4}) p50 {:.4} max {:.4}", ea.0, ea.2, eb.0, eb.2, ec.0, ec.2);
        }
    }
}
