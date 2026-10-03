//! ADR-0177, P4: o espaço do núcleo dos efeitos de vizinhança, pelo oráculo
//! (`docs/Painter/ferramentas/oraculo_camadas_gimp/corre_vizinhanca.sh`). A entrada NOSSA numa camada,
//! uma camada de ajuste Gaussian por cima; o Krita a 8 bits (o alvo) borra em tons de ecrã
//! pré-multiplicados, o GIMP (o CONTROLO) em luz — nenhum filtro dele tem `trc`. A régua distingue os
//! dois: o modelo em luz fecha o GIMP e fica a dezenas de degraus do nosso.

use super::*;
use crate::layers::LayerStack;
use ph2d_painter_effects::adjustments::{AdjustmentParams, GaussianBlurParams, gaussian_weights};

static KRITA: &[u8] = include_bytes!(
    "../../../../docs/Painter/ferramentas/oraculo_camadas_gimp/krita_6.0.4_vizinhanca.bin"
);
static GIMP: &[u8] = include_bytes!(
    "../../../../docs/Painter/ferramentas/oraculo_camadas_gimp/gimp_3.2.6_vizinhanca.bin"
);

/// `(w, h, entrada, [(linha RUN, píxeis)])`.
type Fixtura = (u32, u32, &'static [u8], Vec<(&'static str, &'static [u8])>);

fn le(f: &'static [u8]) -> Fixtura {
    let mut at = 0;
    let mut linha = || {
        let fim = at + f[at..].iter().position(|&b| b == b'\n').unwrap();
        let s = std::str::from_utf8(&f[at..fim]).unwrap();
        at = fim + 1;
        s
    };
    let (mut w, mut h) = (0u32, 0u32);
    loop {
        let l = linha();
        if l == "#FIM" {
            break;
        }
        if let Some(r) = l.strip_prefix("# entrada ...: W=") {
            let mut it = r
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty());
            w = it.next().unwrap().parse().unwrap();
            h = it.next().unwrap().parse().unwrap();
        }
    }
    assert_eq!(linha(), "ENTRADA");
    let n = (w * h * 4) as usize;
    let entrada = &f[at..at + n];
    at += n;
    let mut corridas = Vec::new();
    while at < f.len() {
        let fim = at + f[at..].iter().position(|&b| b == b'\n').unwrap();
        let nome = std::str::from_utf8(&f[at..fim]).unwrap();
        corridas.push((nome, &f[fim + 1..fim + 1 + n]));
        at = fim + 1 + n;
    }
    (w, h, entrada, corridas)
}

fn corrida(fx: &Fixtura, prefixo: &str) -> &'static [u8] {
    fx.3.iter()
        .find(|(nome, _)| nome.starts_with(prefixo))
        .unwrap_or_else(|| panic!("a fixtura não tem a corrida «{prefixo}»"))
        .1
}

/// O compositor de produção: a entrada numa camada e um Gaussian de `raio` por cima.
fn desfoca(w: u32, h: u32, entrada: &[u8], raio: f32) -> Vec<u8> {
    let mut s = LayerStack::new();
    let b = s.add_raster("entrada", w, h).unwrap();
    let p = AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: raio });
    let a = s.add_adjustment(p.kind()).unwrap();
    if let Some(LayerKind::Adjustment(adj)) = s.get_mut(a).map(|l| &mut l.kind) {
        adj.params = p;
    }
    let mut src = MapPixelSource::default();
    src.insert(
        b,
        LayerImage {
            width: w,
            height: h,
            rgba8: entrada.to_vec(),
        },
    );
    composite(&s, &src, w, h)
}

/// O CONTROLO: o nosso núcleo em LUZ pré-multiplicada (a porta dos blurs até à P4).
fn desfoca_em_luz(w: u32, h: u32, entrada: &[u8], raio: f32) -> Vec<u8> {
    use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_unit};
    let (pesos, half) = gaussian_weights(raio);
    let (w, h, half) = (w as i32, h as i32, half as i32);
    let mut pm: Vec<[f32; 4]> = entrada
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            let a = f32::from(p[3]) / 255.0;
            let l = |c: u8| srgb_to_linear_unit(f32::from(c) / 255.0) * a;
            [l(p[0]), l(p[1]), l(p[2]), a]
        })
        .collect();
    for (dx, dy) in [(1, 0), (0, 1)] {
        let src = pm.clone();
        for y in 0..h {
            for x in 0..w {
                let mut c = [0.0f32; 4];
                for k in -half..=half {
                    let sx = (x + k * dx).clamp(0, w - 1);
                    let sy = (y + k * dy).clamp(0, h - 1);
                    let s = src[(sy * w + sx) as usize];
                    for ch in 0..4 {
                        c[ch] += s[ch] * pesos[k.unsigned_abs() as usize];
                    }
                }
                pm[(y * w + x) as usize] = c;
            }
        }
    }
    pm.iter()
        .flat_map(|c| {
            let a = c[3].clamp(0.0, 1.0);
            let e = |v: f32| {
                if a > 1e-6 {
                    linear_to_srgb_byte(v / a)
                } else {
                    0
                }
            };
            [e(c[0]), e(c[1]), e(c[2]), (a * 255.0).round() as u8]
        })
        .collect()
}

/// `(pior desvio de cor, canais de cor a mais de 1, pior desvio de alfa)`; a cor de um píxel que o
/// oráculo deixa transparente é indefinida.
fn desvio(oraculo: &[u8], nosso: &[u8]) -> (u8, usize, u8) {
    let (mut pior, mut acima, mut alfa) = (0u8, 0usize, 0u8);
    for (o, n) in oraculo
        .as_chunks::<4>()
        .0
        .iter()
        .zip(nosso.as_chunks::<4>().0)
    {
        alfa = alfa.max(o[3].abs_diff(n[3]));
        if o[3] == 0 {
            continue;
        }
        for k in 0..3 {
            let d = o[k].abs_diff(n[k]);
            pior = pior.max(d);
            acima += usize::from(d > 1);
        }
    }
    (pior, acima, alfa)
}

#[test]
fn the_gaussian_is_kritas_8_bit_blur_to_the_byte() {
    // O Gaussian do Krita de raio R é o nosso `gaussian_weights(R)` (σ = R/3) em tons de ecrã
    // pré-multiplicados: imagem inteira, bordas e a cor escondida sob alfa 0 incluídas.
    let fx = le(KRITA);
    let krita = corrida(&fx, "RUN Gaussian gaussian_blur krita-u8 horizRadius=9");
    let nosso = desfoca(fx.0, fx.1, fx.2, 9.0);
    assert_eq!(
        desvio(krita, &nosso),
        (0, 0, 0),
        "o nosso Gaussian × o do Krita"
    );
}

#[test]
fn the_gimp_blurs_in_light_and_the_ruler_tells_the_two_apart() {
    // σ = 3 = o nosso raio 9. O GIMP trunca o núcleo mais longe que os nossos 3σ: 12 canais de borda
    // passam de 1 (medido 03/10); o resto fecha a ≤1.
    let fx = le(GIMP);
    let gimp = corrida(&fx, "RUN Gaussian gegl:gaussian-blur u8 std-dev-x=3.0");
    let (pior, acima, alfa) = desvio(gimp, &desfoca_em_luz(fx.0, fx.1, fx.2, 9.0));
    assert!(
        pior <= 7 && acima <= 12 && alfa <= 1,
        "o controlo não fecha: o GIMP × o nosso núcleo em luz = {pior} ({acima} canais > 1), alfa {alfa}"
    );
    let (pior, _, _) = desvio(gimp, &desfoca(fx.0, fx.1, fx.2, 9.0));
    assert!(
        pior >= 60,
        "a régua não distingue luz de ecrã: o GIMP × o nosso = {pior}"
    );
}
