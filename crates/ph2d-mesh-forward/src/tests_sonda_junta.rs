//! ⭐ **A JUNTA na CPU** — a busca do produto (`sonda_marcha`, `sonda_le.wgsl`) portada passo a passo sobre
//! a captura do cromo lida de volta, na vista [`JUNTA`]: o que ela aceita nos px da FAIXA entre os dois
//! reflexos (onde o raio, pela geometria, não acerta nada), por leitor da distância e por franja — a
//! causa isolada sem tocar no WGSL.

use crate::gpu::sondas_impl::{
    ARESTA, ARESTA_PASSOS, ESPESSURA, FRANJA, LADO, MARCHA_MAX, MARCHA_MIN, PASSO, REFINO,
};
use crate::tests_chao_tapa::metal;
use crate::tests_contacto::norm;
use crate::tests_reflexo_junta::faixa;
use crate::tests_reflexo_perto::{JUNTA, desenha, desenha_ate, desenhista, oraculo};
use crate::tests_sonda_cpu::{Nivel, camada, le, texel};

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `sonda_aresta`: o desvio pelo 2.º momento passa `aresta` da distância média.
fn aresta(g: [f32; 3], k: f32) -> bool {
    let m = g[0] / g[1].max(1.0e-6);
    g[2] / g[1].max(1.0e-6) - m * m > k * k * m * m
}

/// `sonda_marcha` com o leitor `ler` (a distância e o quadrado dela vezes a cobertura, a cobertura):
/// `(direcção, peso)`.
pub(crate) fn marcha(
    ler: &dyn Fn([f32; 3]) -> [f32; 3],
    q: [f32; 3],
    r: [f32; 3],
    franja: f32,
    k_aresta: f32,
    traco: &mut Option<Vec<String>>,
) -> ([f32; 3], f32) {
    let ql = dot(q, q).sqrt();
    let qh = q.map(|x| x / ql.max(1.0e-6));
    let ct = dot(qh, r).clamp(-1.0, 1.0);
    let th = ct.acos();
    if th < 1.0e-4 {
        return (r, 0.0);
    }
    let w = norm([0, 1, 2].map(|e| r[e] - ct * qh[e]));
    let st = th.sin();
    let u = |a: f32| [0, 1, 2].map(|e| a.cos() * qh[e] + a.sin() * w[e]);
    let lam = |a: f32| ql * st / (th - a).sin();
    let atras = |a: f32| {
        let g = ler(u(a));
        g[1] > 0.5 && lam(a) >= g[0] / g[1]
    };
    let texel = std::f32::consts::FRAC_PI_2 / ((LADO - 2) as f32 * 0.5);
    let n = ((th / (PASSO * texel)).ceil() as u32).clamp(MARCHA_MIN, MARCHA_MAX);
    let (mut ant, mut frente) = (0.0f32, true);
    let mut melhor = (r, 0.0f32);
    for k in 1..=n {
        let a = th * k as f32 / (n + 1) as f32;
        let at = atras(a);
        if at && frente {
            let (mut lo, mut hi) = (ant, a);
            for _ in 0..REFINO {
                let m = 0.5 * (lo + hi);
                if atras(m) {
                    hi = m;
                } else {
                    lo = m;
                }
            }
            let gl = ler(u(lo));
            let (mut uh, mut gh) = (u(hi), ler(u(hi)));
            let mut s = 1;
            while s <= ARESTA_PASSOS && aresta(gh, k_aresta) {
                uh = u(hi + s as f32 * texel);
                gh = ler(uh);
                s += 1;
            }
            let l = lam(hi);
            let fora = l - gh[0] / gh[1].max(1.0e-6);
            if let Some(t) = traco.as_mut() {
                let g0 = ler(u(hi));
                let m0 = g0[0] / g0[1].max(1.0e-6);
                t.push(format!(
                    "  passo {k}/{n} hi {:.3}° · λ {l:.3} · lida {m0:.3} (cob {:.2}, desvio/média {:.3}) · {} passos → \
                     pura {:.3} (cob {:.2}) · fora/λ {:.3} · cob lo {:.2}",
                    hi.to_degrees(),
                    g0[1],
                    (g0[2] / g0[1].max(1.0e-6) - m0 * m0).max(0.0).sqrt() / m0.max(1.0e-6),
                    s - 1,
                    gh[0] / gh[1].max(1.0e-6),
                    gh[1],
                    fora / l,
                    gl[1]
                ));
            }
            if aresta(gh, k_aresta) {
                frente = false;
                ant = a;
                continue;
            }
            if gl[1] > 0.5 && fora <= ESPESSURA * l {
                return (uh, 1.0);
            }
            let peso = 1.0 - smoothstep(0.0, franja * l, fora);
            if peso > melhor.1 {
                melhor = (uh, peso);
            }
        }
        frente = !at;
        ant = a;
    }
    melhor
}

/// Sonda (imprime): nos px da faixa, quantos a busca ACEITA (peso `1`) e o peso médio que lhes dá, por
/// leitor (bilinear = o produto, o texel mais próximo) e franja; com a azul e sem ela.
#[test]
#[ignore = "precisa de aparelho"]
fn sonda_da_junta_na_cpu() {
    let v = &JUNTA;
    let Some(mut fw) = desenhista(v) else {
        return;
    };
    let px = oraculo(v);
    let f = faixa(v, &px);
    let c = v.pecas[0].0;
    let vista = norm(v.de.map(|x| -x));
    // Os px do cromo que DEVEM acertar uma vizinha (a mais de 2 px do contorno dela): os que a lei perde.
    let acertam: Vec<usize> = (0..px.len())
        .filter(|&k| crate::tests_reflexo_perto::vizinha_refletida(v, &px[k]).is_some())
        .collect();
    for (nome, n) in [("com a azul", v.pecas.len()), ("sem a azul", 2)] {
        let _ = desenha_ate(v, &mut fw, metal(0.0), n, true);
        let dist: Vec<Nivel> = camada(&fw, 1);
        let bil = |d: [f32; 3]| {
            let g = le(&dist, d, 0.0);
            [g[0], g[1], g[2]]
        };
        let viz = |d: [f32; 3]| {
            let g = texel(&dist, d, 0);
            [g[0], g[1], g[2]]
        };
        for (leitor, ler) in [
            ("bilinear", &bil as &dyn Fn([f32; 3]) -> [f32; 3]),
            ("texel", &viz),
        ] {
            for (franja, k_aresta) in [
                (FRANJA, f32::INFINITY),
                (FRANJA, 0.2),
                (FRANJA, ARESTA),
                (FRANJA, 0.05),
                (0.0, ARESTA),
            ] {
                let corre = |quais: &[usize]| {
                    let (mut aceita, mut peso) = (0usize, 0.0f32);
                    for &k in quais {
                        let p = &px[k];
                        let r: [f32; 3] =
                            std::array::from_fn(|e| vista[e] - 2.0 * dot(vista, p.n) * p.n[e]);
                        let q = [0, 1, 2].map(|e| p.p[e] - c[e]);
                        let (_, w) = marcha(ler, q, r, franja.max(1.0e-6), k_aresta, &mut None);
                        aceita += usize::from(w >= 1.0);
                        peso += w;
                    }
                    (aceita, peso / quais.len().max(1) as f32)
                };
                let (fa, fp) = corre(&f);
                let (aa, ap) = corre(&acertam);
                eprintln!(
                    "{nome} · {leitor} · franja {franja} · aresta {k_aresta}: faixa {} px ACEITES {fa} (peso {fp:.3}) · \
                     dos {} que acertam, aceites {aa} (peso {ap:.3})",
                    f.len(),
                    acertam.len()
                );
            }
        }
    }
    let _ = desenha(v, &mut fw, metal(0.0), true);
}

/// Sonda (imprime): os px em volta da verde que a azul atrás MUDA (`> 0,1`) sem acertar nada pela geometria,
/// e a busca de alguns deles com e sem a azul, cruzamento a cruzamento.
#[test]
#[ignore = "precisa de aparelho"]
fn sonda_da_tira() {
    let v = &crate::tests_reflexo_perto::SOBREPOSTA;
    let Some(mut fw) = desenhista(v) else {
        return;
    };
    let px = oraculo(v);
    let c = v.pecas[0].0;
    let vista = norm(v.de.map(|x| -x));
    let com = desenha(v, &mut fw, metal(0.0), true);
    let dcom = camada(&fw, 1);
    let sem = desenha_ate(v, &mut fw, metal(0.0), 2, true);
    let dsem = camada(&fw, 1);
    let lin = crate::tests_contacto::linear;
    let tira: Vec<usize> = crate::tests_reflexo_junta::em_volta_da_verde(v, &px)
        .into_iter()
        .filter(|&k| {
            let i = (px[k].j * LADO + px[k].i) as usize * 4 + 1;
            (lin(com[i]) - lin(sem[i])).abs() > 0.1
        })
        .collect();
    eprintln!("{} px mudam", tira.len());
    for &k in tira.iter().step_by((tira.len() / 6).max(1)) {
        let p = &px[k];
        let r: [f32; 3] = std::array::from_fn(|e| vista[e] - 2.0 * dot(vista, p.n) * p.n[e]);
        let q = [0, 1, 2].map(|e| p.p[e] - c[e]);
        let i = (p.j * LADO + p.i) as usize * 4 + 1;
        eprintln!(
            "px ({}, {}): com {:.3} · sem {:.3} · acerta {:?}",
            p.i,
            p.j,
            lin(com[i]),
            lin(sem[i]),
            crate::tests_reflexo_perto::vizinha_refletida(v, p)
        );
        for (nome, d) in [("com", &dcom), ("sem", &dsem)] {
            let ler = |x: [f32; 3]| {
                let g = le(d, x, 0.0);
                [g[0], g[1], g[2]]
            };
            let mut t = Some(Vec::new());
            let (_, w) = marcha(&ler, q, r, FRANJA, ARESTA, &mut t);
            eprintln!(" {nome}: peso {w:.3}");
            for l in t.unwrap_or_default() {
                eprintln!("{l}");
            }
        }
    }
}
