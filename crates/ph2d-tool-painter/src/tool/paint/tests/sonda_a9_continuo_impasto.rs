//! **Sonda da lista 26 A9** (smoke do dono 15/10): Impasto + Shape + Texture Color, Spacing alto com o
//! Continuous ≠ sem ele. Mede pelo PRODUTO (o `PainterTool`, o traço da `capsula.rs`): (a) Spacing S sem
//! a caixa, (b) S com ela, (c) 0,025 sem ela — o alvo que o Continuous promete. Isola cada factor.
//! `PH2D_SONDA_OUT=<pasta> cargo test -p ph2d-tool-painter --profile smoke --lib sonda_a9 -- --ignored --nocapture`

use super::capsula::{
    H, Meio, W, correlacao, dif_cor as dif_cor_r, matiz as matiz_r, miolo as miolo_r, pincel,
    regua, traco, vermelho_azul,
};
use super::*;
use ph2d_painter_brush::TextureKind;
use ph2d_painter_brush::texture::TextureMapping;

const TAM: f32 = 40.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cor {
    /// Sem rampa: o Shape recorta o pigmento.
    Off,
    /// Só a Texture Color, vermelho → azul.
    Textura,
    /// Só a Falloff Color, vermelho → azul.
    Queda,
    /// As duas camadas de fábrica, com a rampa de fábrica (o caso do dono).
    Fabrica,
}

#[derive(Clone, Copy, Debug)]
struct Caso {
    meio: Meio,
    kind: TextureKind,
    cor: Cor,
    tiled: bool,
    relevo: bool,
}

/// O pincel da `capsula.rs` armado com o caso; devolve `(tela, relevo)`.
fn corre(c: Caso, sp: f32, continuo: bool) -> (Vec<u8>, Vec<f32>) {
    let mut t = pincel(c.meio, c.kind, TAM, sp, continuo);
    if c.tiled {
        t.paint.brush.shape.mapping = TextureMapping::Tiled;
        t.paint.brush.shape.size = [6.0; 2];
    }
    if !c.relevo {
        t.paint.brush.impasto_depth = 0.0;
    }
    match c.cor {
        Cor::Off => t.set_shape_ramp_enabled(false),
        Cor::Fabrica => t.set_shape_ramp_enabled(true),
        Cor::Textura | Cor::Queda => {
            t.set_shape_color_ramp(vermelho_azul());
            t.set_shape_falloff_ramp(vermelho_azul());
            t.paint.shape_color_ramp_bw = false;
            t.paint.shape_camadas.textura = c.cor == Cor::Textura;
            t.paint.shape_camadas.queda = c.cor == Cor::Queda;
            t.set_shape_falloff_ramp(vermelho_azul()); // re-assa as LUTs
            t.set_shape_ramp_enabled(true);
        }
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    let px = traco(&mut t);
    let rel = t
        .layers
        .active()
        .and_then(|id| t.layer_height_view(id))
        .unwrap_or_default();
    (px, rel)
}

fn miolo() -> Vec<(usize, usize)> {
    miolo_r(TAM / 2.0)
}

fn matiz(px: &[u8]) -> Vec<f32> {
    matiz_r(px, TAM / 2.0)
}

fn dif_cor(a: &[u8], b: &[u8]) -> f32 {
    dif_cor_r(a, b, TAM / 2.0)
}

/// Quanto da variância da matiz é ENTRE linhas (listras ao longo do traço): 1 = só listras longas.
fn listras(px: &[u8]) -> f32 {
    let pts = miolo();
    let m = matiz(px);
    let media = m.iter().sum::<f32>() / m.len() as f32;
    let total: f32 = m.iter().map(|v| (v - media) * (v - media)).sum();
    let mut por_linha: std::collections::BTreeMap<usize, Vec<f32>> = Default::default();
    for (p, v) in pts.iter().zip(&m) {
        por_linha.entry(p.1).or_default().push(*v);
    }
    let dentro: f32 = por_linha
        .values()
        .map(|l| {
            let ml = l.iter().sum::<f32>() / l.len() as f32;
            l.iter().map(|v| (v - ml) * (v - ml)).sum::<f32>()
        })
        .sum();
    if total < 1e-9 {
        f32::NAN
    } else {
        1.0 - dentro / total
    }
}

/// O relevo: `Σ|a − t| / Σ t` e a correlação, no miolo.
fn dif_relevo(a: &[f32], t: &[f32]) -> (f32, f32) {
    if a.is_empty() || t.is_empty() {
        return (f32::NAN, f32::NAN);
    }
    let pts = miolo();
    let i = |&(x, y): &(usize, usize)| y * W as usize + x;
    let (va, vt): (Vec<f32>, Vec<f32>) = pts.iter().map(|p| (a[i(p)], t[i(p)])).unzip();
    let st: f32 = vt.iter().map(|v| v.abs()).sum();
    let d: f32 = va.iter().zip(&vt).map(|(a, t)| (a - t).abs()).sum();
    (d / st.max(1e-9), correlacao(&va, &vt))
}

fn grava(dir: &str, nome: &str, px: &[u8]) {
    let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
    for k in 0..(W * H) as usize {
        ppm.extend_from_slice(&px[k * 4..k * 4 + 3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), ppm).expect("grava");
}

fn grava_relevo(dir: &str, nome: &str, h: &[f32]) {
    if h.is_empty() {
        return;
    }
    let mx = h.iter().copied().fold(1e-9f32, f32::max);
    let mut ppm = format!("P5 {W} {H} 255\n").into_bytes();
    ppm.extend(h.iter().map(|v| (v / mx * 255.0).clamp(0.0, 255.0) as u8));
    std::fs::write(format!("{dir}/{nome}_relevo.pgm"), ppm).expect("grava");
}

#[test]
#[ignore = "sonda A9: o Continuous no Impasto com a Texture Color"]
fn sonda_a9_continuo_impasto() {
    use Meio::{Digital, Impasto};
    use TextureKind::{Checker, Noise, Stripes};
    let out = std::env::var("PH2D_SONDA_OUT").ok();
    println!(
        "mapping de fábrica do Shape: {:?}",
        PainterTool::default().paint.brush.shape.mapping
    );
    let c = |meio, kind, cor| Caso {
        meio,
        kind,
        cor,
        tiled: false,
        relevo: true,
    };
    let casos = [
        ("Imp Noise Textura", c(Impasto, Noise, Cor::Textura)),
        ("Imp Noise Fábrica", c(Impasto, Noise, Cor::Fabrica)),
        ("Imp Checker Textura", c(Impasto, Checker, Cor::Textura)),
        ("Imp Stripes Textura", c(Impasto, Stripes, Cor::Textura)),
        ("Imp Noise Off", c(Impasto, Noise, Cor::Off)),
        ("Imp Noise Queda", c(Impasto, Noise, Cor::Queda)),
        (
            "Imp None Textura",
            c(Impasto, TextureKind::None, Cor::Textura),
        ),
        ("Dig Noise Textura", c(Digital, Noise, Cor::Textura)),
        (
            "Imp Noise Textura Tiled",
            Caso {
                tiled: true,
                ..c(Impasto, Noise, Cor::Textura)
            },
        ),
        (
            "Imp Noise Textura relevo 0",
            Caso {
                relevo: false,
                ..c(Impasto, Noise, Cor::Textura)
            },
        ),
    ];
    println!(
        "| caso | S | caixa | dif tinta | tinta÷alvo | Δcor (/255) | corr matiz×alvo | listras (alvo) | Δrelevo | corr relevo |"
    );
    println!("|---|---:|---|---:|---:|---:|---:|---:|---:|---:|");
    for (nome, caso) in casos {
        let (alvo, alvo_h) = corre(caso, 0.025, false);
        let (m_alvo, l_alvo) = (matiz(&alvo), listras(&alvo));
        if let Some(d) = &out {
            let n = nome.replace(' ', "_");
            grava(d, &format!("{n}_alvo"), &alvo);
            grava_relevo(d, &format!("{n}_alvo"), &alvo_h);
        }
        for sp in [0.25f32, 0.5] {
            for continuo in [false, true] {
                let (px, h) = corre(caso, sp, continuo);
                let g = regua(&alvo, &px, TAM / 2.0, sp * TAM);
                let (dr, cr) = dif_relevo(&h, &alvo_h);
                println!(
                    "| {nome} | {sp} | {} | {:.3} | {:.2} | {:.3} | {:.3} | {:.2} ({:.2}) | {:.3} | {:.3} |",
                    if continuo { "sim" } else { "não" },
                    g.dif,
                    g.tinta,
                    dif_cor(&px, &alvo),
                    correlacao(&matiz(&px), &m_alvo),
                    listras(&px),
                    l_alvo,
                    dr,
                    cr
                );
                if let Some(d) = &out {
                    let n = nome.replace(' ', "_");
                    grava(d, &format!("{n}_{sp}_{continuo}"), &px);
                    grava_relevo(d, &format!("{n}_{sp}_{continuo}"), &h);
                }
            }
        }
    }
}
