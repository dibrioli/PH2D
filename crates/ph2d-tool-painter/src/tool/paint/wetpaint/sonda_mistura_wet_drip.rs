//! **Sonda (ignorada): o Wet Drip com as caixas K–M** (prompt 16, Fases B–E). A cena e as réguas dos portões
//! ([`super::mistura_wet_drip_tests`]) nas quatro combinações das caixas e na lei de antes do «Glaze
//! layering», molhado e seco, mais as réguas do Rebelle (`ferramentas/mistura_wet_drip_regua.py`: o tetraedro
//! branco·vermelho·azul·vermelho⊙azul, a `N32`, a orla de baixo e a fronteira) e as fotografias.
//!
//! ```text
//! PH2D_SONDA_OUT=<dir> cargo test -p ph2d-tool-painter --profile smoke --lib sonda_mistura_wet_drip -- --ignored --nocapture
//! ```
//! `PH2D_SONDA_SO=<rótulo,…>` filtra as variantes; as fotografias saem como `faseE_<rótulo>_<estado>_<momento>.ppm`
//! (`ferramentas/prancha_mistura_wet_drip.py` faz as pranchas).
use super::mistura_wet_drip_tests::{self as wd, Tintas, Var};
use ph2d_brush_presets::Biblioteca;

fn r3(c: [f64; 3]) -> [i32; 3] {
    c.map(|v| v.round() as i32)
}

fn srgb(l: f64) -> f64 {
    let l = l.clamp(0.0, 1.0);
    if l <= 0.003_130_8 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    }
}

fn lum(c: [f64; 3]) -> f64 {
    0.2126 * wd::lin(c[0]) + 0.7152 * wd::lin(c[1]) + 0.0722 * wd::lin(c[2])
}

fn tinta(t: &[u8], i: usize) -> bool {
    255.0 - wd::px(t, i).iter().copied().fold(255.0, f64::min) > 25.0
}

fn percentil(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() as f64 - 1.0) * p).round() as usize]
}

fn resolve3(m: [[f64; 3]; 3], x: [f64; 3]) -> [f64; 3] {
    let det = |m: [[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(m);
    [0, 1, 2].map(|k| {
        let mut mk = m;
        for (r, row) in mk.iter_mut().enumerate() {
            row[k] = x[r];
        }
        det(mk) / d
    })
}

/// (d) do Rebelle: % das células de tinta fora do tetraedro (sRGB, o produto em luz linear) a tol 0,10 com
/// `w ≥ −0,10`, e as CIANO (coord. do vermelho < −0,10, `w ≥ −0,03`).
fn tetraedro(t: &[u8]) -> (f64, f64, [f64; 3]) {
    let rd = wd::VERMELHO.map(|v| f64::from(v) / 255.0);
    let bl = wd::AZUL.map(|v| f64::from(v) / 255.0);
    let pr = [0, 1, 2]
        .map(|c| srgb(wd::lin(f64::from(wd::VERMELHO[c])) * wd::lin(f64::from(wd::AZUL[c]))));
    let m = [0, 1, 2].map(|r| [rd[r] - 1.0, bl[r] - 1.0, pr[r] - 1.0]);
    let (mut n, mut fora, mut ciano, mut pior, mut pior_v) =
        (0usize, 0usize, 0usize, 0.0, [0.0; 3]);
    for i in 0..wd::W * wd::H {
        if !tinta(t, i) {
            continue;
        }
        n += 1;
        let c = wd::px(t, i);
        let lam = resolve3(m, c.map(|v| v / 255.0 - 1.0));
        let w = 1.0 - lam.iter().sum::<f64>();
        let neg = lam.iter().copied().fold(0.0, f64::min);
        fora += usize::from(neg < -0.10 && w >= -0.10);
        ciano += usize::from(lam[0] < -0.10 && w >= -0.03);
        if w >= -0.03 && neg < pior {
            (pior, pior_v) = (neg, c);
        }
    }
    let pc = |k: usize| 100.0 * k as f64 / n.max(1) as f64;
    (pc(fora), pc(ciano), pior_v)
}

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn linha_med(t: &[u8], y: usize, x0: usize, x1: usize) -> [f64; 3] {
    [0, 1, 2].map(|c| mediana((x0..x1).map(|x| wd::px(t, y * wd::W + x)[c]).collect()))
}

/// A orla de baixo do azul (x 200–400): a lum da linha MAIS ESCURA em y 222–250 contra o miolo (y 185–215).
fn orla(t: &[u8]) -> (f64, f64, usize) {
    let miolo = mediana((185..215).map(|y| lum(linha_med(t, y, 200, 400))).collect());
    let (mut esc, mut ye) = (f64::MAX, 0);
    for y in 222..250 {
        let l = lum(linha_med(t, y, 200, 400));
        if l < esc {
            (esc, ye) = (l, y);
        }
    }
    (esc, miolo, ye)
}

/// A fronteira lateral direita azul/vermelho (y 228–240): `pico − max(patamares)` de lum a ±20 px de onde
/// R − B troca de sinal (x 400–460).
fn fronteira(t: &[u8]) -> Option<(usize, f64, [f64; 3])> {
    let col = |x: usize| -> [f64; 3] {
        [0, 1, 2].map(|c| mediana((228..241).map(|y| wd::px(t, y * wd::W + x)[c]).collect()))
    };
    let hp: Vec<[f64; 3]> = (0..wd::W).map(col).collect();
    let hl: Vec<f64> = hp.iter().map(|c| lum(*c)).collect();
    let rb = |x: usize| hp[x][0] - hp[x][2];
    let xf = (400..460).find(|&x| rb(x).signum() != rb(x + 1).signum())?;
    let pat = |a: usize, b: usize| hl[a..b].iter().sum::<f64>() / (b - a) as f64;
    let (pa, pb) = (pat(xf - 40, xf - 20), pat(xf + 21, xf + 41));
    let k = (xf - 20..=xf + 20).max_by(|&a, &b| hl[a].total_cmp(&hl[b]))?;
    Some((xf, hl[k] - pa.max(pb), hp[k]))
}

fn grava_ppm(dir: &str, nome: &str, t: &[u8]) {
    let mut ppm = format!("P6 {} {} 255\n", wd::W, wd::H).into_bytes();
    for q in t.as_chunks::<4>().0 {
        ppm.extend_from_slice(&q[..3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), ppm).expect("grava a tela");
}

fn hash(t: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

fn variantes() -> Vec<(&'static str, Var)> {
    let v = |km, glaze, lei_de_antes| Var {
        km,
        glaze,
        lei_de_antes,
        ..Var::DUAS
    };
    vec![
        ("nenhuma", v(false, false, false)),
        ("km", v(true, false, false)),
        ("glaze", v(false, true, false)),
        ("duas", v(true, true, false)),
        ("antes_glaze", v(false, true, true)),
        ("antes_duas", v(true, true, true)),
    ]
}

/// Os limiares do índice e do pontilhado (o P99,9 de cada tinta SOZINHA sem caixas, Heavy Edge 0) e o
/// excesso «mais clara à mesma massa» do produto SEM caixas (o máximo e o P99,9).
fn controlos(b: &Biblioteca) -> (f64, f64) {
    let (mut ix, mut dv) = (Vec::new(), Vec::new());
    for seco in [false, true] {
        let v = Var {
            km: false,
            glaze: false,
            seco,
            borda: 0.0,
            lei_de_antes: false,
        };
        for tintas in [Tintas::SoVermelho, Tintas::SoAzul] {
            let c = wd::cena(b, v, tintas);
            for tl in &c.telas[1..] {
                ix.extend(wd::indices(tl).into_iter().filter(|&x| x >= 0.0));
                dv.extend(wd::desvios(tl).into_iter().filter(|&x| x >= 0.0));
            }
        }
    }
    let (lim_ix, lim_dv) = (percentil(&mut ix, 0.999), percentil(&mut dv, 0.999));
    println!(
        "D [controlo: cada tinta sozinha] índice P99,9 {lim_ix:.3} · pontilhado P99,9 {lim_dv:.1} níveis"
    );
    (lim_ix, lim_dv)
}

#[test]
#[ignore = "sonda: o Wet Drip com as caixas K–M; corre-se à mão"]
fn sonda_mistura_wet_drip() {
    let b = wd::biblioteca();
    let dir = std::env::var("PH2D_SONDA_OUT").ok();
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let (lim_ix, lim_dv) = controlos(&b);
    for (rot, v0) in variantes() {
        if so
            .as_deref()
            .is_some_and(|s| !s.split(',').any(|x| x == rot))
        {
            continue;
        }
        for seco in [false, true] {
            let est = if seco { "seco" } else { "molhado" };
            let mix = wd::cena(&b, Var { seco, ..v0 }, Tintas::Ambas);
            if let Some(d) = &dir {
                for (k, s) in wd::MOMENTOS.iter().enumerate() {
                    grava_ppm(d, &format!("faseE_{rot}_{est}_{s}"), &mix.telas[k]);
                }
            }
            let mut out = format!("D {rot} {est} (hash seco {:016x})", hash(&mix.telas[3]));
            for k in [2usize, 3] {
                let tl = &mix.telas[k];
                let ixs = wd::indices(tl);
                let fora = ixs.iter().filter(|&&x| x > lim_ix).count();
                let dvs = wd::desvios(tl);
                let pont = dvs.iter().filter(|&&x| x > lim_dv).count();
                let mut exc: Vec<f64> = wd::excessos(tl, &mix.massa[k - 2])
                    .into_iter()
                    .filter(|&x| x > -1.0)
                    .collect();
                let am = exc.iter().filter(|&&x| x > 0.0).count();
                let (tf, tc, tp) = tetraedro(tl);
                let (oe, om, oy) = orla(tl);
                let fr = fronteira(tl).map_or("sem troca R−B".to_string(), |(x, d, cor)| {
                    format!("x{x} {d:+.3} {:?}", r3(cor))
                });
                out += &format!(
                    " | {}: índice {fora} (máx {:.2}) · mais clara à massa: máx {:+.4} P99,9 {:+.4} (>0: {am}) · pontilhado {pont} · N32 {:.3} % · tetraedro fora {tf:.3} % ciano {tc:.3} % (pior {:?}) · orla y{oy} lum {oe:.3} vs miolo {om:.3} ({}) · fronteira {fr}",
                    wd::MOMENTOS[k],
                    ixs.iter().copied().fold(0.0, f64::max),
                    exc.iter().copied().fold(f64::MIN, f64::max),
                    percentil(&mut exc, 0.999),
                    wd::claras_n32(tl),
                    r3(tp),
                    if oe < om { "escurece" } else { "CLAREIA" }
                );
            }
            println!("{out}");
        }
    }
}
