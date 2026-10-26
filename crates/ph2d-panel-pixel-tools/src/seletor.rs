//! **O seletor de cor abaixo da paleta** — as cinco leis posição → cor do menu ≡ da paleta do
//! Aseprite 1.3.18 (observadas no Trial, `docs/PixelArt/handoffs/` P2), puras e testáveis.
//!
//! Cada modo guarda três parâmetros ([`Params`]): dois mudam no quadro (`u` → direita, `v` → baixo,
//! ambos `0..=1`), o terceiro na barra de baixo. A cor sai deles ([`cor`]); quando ela muda por fora
//! (paleta, conta-gotas, `X`), [`de_rgb`] reconstrói-os guardando o que a cor não diz (a matiz de um
//! cinza, a saturação do preto).

use ph2d_editor_core::pixel_panel::PixelSelector;

/// Os três parâmetros do modo, todos em `0..=1` (a matiz é a fracção de volta).
///
/// - Tint/Shade/Tone e RGB wheel: matiz, saturação, brilho (HSV).
/// - Spectrum: matiz, saturação, claridade (HSL).
/// - RYB wheel: matiz NO CÍRCULO RYB, saturação, brilho.
/// - Normal map: `x`, `y` do ponto no quadro e o brilho.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Default for Params {
    /// Vermelho puro (a matiz 0 com tudo cheio).
    fn default() -> Self {
        Self {
            a: 0.0,
            b: 1.0,
            c: 1.0,
        }
    }
}

/// A rotação dos discos: a matiz 0 (vermelho) cai a −30° do eixo `+x`, o verde (120°) em cima.
const RODA_OFFSET: f64 = 30.0 / 360.0;

fn byte(x: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "presa a 0..=255 antes da conversão"
    )]
    let b = (x * 255.0).round().clamp(0.0, 255.0) as u8;
    b
}

fn rgb8(c: [f64; 3]) -> [u8; 3] {
    c.map(byte)
}

fn unit(c: [u8; 3]) -> [f64; 3] {
    c.map(|x| f64::from(x) / 255.0)
}

/// HSV → RGB, tudo em `0..=1` (a matiz em fracção de volta).
#[must_use]
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [f64; 3] {
    let h = h.rem_euclid(1.0) * 6.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v - c;
    #[expect(clippy::cast_possible_truncation, reason = "0..6")]
    let (r, g, b) = match h.floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r + m, g + m, b + m]
}

/// RGB → HSV; `None` na matiz de um cinza e na saturação do preto (a cor não as diz).
#[must_use]
pub fn rgb_to_hsv(c: [f64; 3]) -> (Option<f64>, Option<f64>, f64) {
    let [r, g, b] = c;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let s = (max > 0.0).then(|| d / max);
    let h = (d > 0.0).then(|| {
        let h = if max == r {
            ((g - b) / d).rem_euclid(6.0)
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        h / 6.0
    });
    (h, s, max)
}

/// HSL → RGB.
#[must_use]
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> [f64; 3] {
    let v = l + s * l.min(1.0 - l);
    let sv = if v > 0.0 { 2.0 * (1.0 - l / v) } else { 0.0 };
    hsv_to_rgb(h, sv, v)
}

/// RGB → HSL; `None` como em [`rgb_to_hsv`] (a saturação do branco e do preto).
#[must_use]
pub fn rgb_to_hsl(c: [f64; 3]) -> (Option<f64>, Option<f64>, f64) {
    let (h, _, v) = rgb_to_hsv(c);
    let min = c[0].min(c[1]).min(c[2]);
    let l = (v + min) * 0.5;
    let s = (l > 0.0 && l < 1.0).then(|| (v - l) / l.min(1.0 - l));
    (h, s, l)
}

/// **RYB → RGB pela interpolação trilinear do cubo RYB de Gossett & Chen** (*Paint Inspired Color
/// Mixing and Compositing for Visualization*, IEEE InfoVis 2004, tabela dos oito cantos).
#[must_use]
pub fn ryb_to_rgb([r, y, b]: [f64; 3]) -> [f64; 3] {
    // Os cantos (r, y, b) ∈ {0, 1}³, pela ordem do índice `r·4 + y·2 + b`.
    const CANTOS: [[f64; 3]; 8] = [
        [1.0, 1.0, 1.0],   // branco
        [0.163, 0.373, 0.6], // azul
        [1.0, 1.0, 0.0],   // amarelo
        [0.0, 0.66, 0.2],  // verde
        [1.0, 0.0, 0.0],   // vermelho
        [0.5, 0.0, 0.5],   // violeta
        [1.0, 0.5, 0.0],   // laranja
        [0.2, 0.094, 0.0], // preto
    ];
    let mut out = [0.0; 3];
    for (i, canto) in CANTOS.iter().enumerate() {
        let w = |bit: usize, t: f64| if i & bit != 0 { t } else { 1.0 - t };
        let peso = w(4, r) * w(2, y) * w(1, b);
        for k in 0..3 {
            out[k] += peso * canto[k];
        }
    }
    out
}

/// A cor pura da matiz `h` do círculo RYB (vermelho 0, laranja ⅙, amarelo ⅓, verde ½, azul ⅔,
/// violeta ⅚), levada ao canal máximo 1 — o brilho é da barra.
#[must_use]
pub fn ryb_hue(h: f64) -> [f64; 3] {
    let c = ryb_to_rgb(hsv_to_rgb(h, 1.0, 1.0));
    let m = c[0].max(c[1]).max(c[2]).max(f64::MIN_POSITIVE);
    c.map(|x| x / m)
}

/// A cor da roda RYB: o cinza do brilho `v` misturado com a cor pura por `s`.
fn ryb(h: f64, s: f64, v: f64) -> [f64; 3] {
    ryb_hue(h).map(|x| v * (1.0 - s + s * x))
}

/// O ponto `(u, v)` no disco unitário inscrito no quadro: `(dx, dy)` com `y` para CIMA, preso à borda.
fn no_disco(u: f64, v: f64) -> (f64, f64) {
    let (dx, dy) = ((u - 0.5) * 2.0, (0.5 - v) * 2.0);
    let r = dx.hypot(dy);
    if r > 1.0 { (dx / r, dy / r) } else { (dx, dy) }
}

/// O ponto `(u, v)` dentro do disco inscrito.
#[must_use]
pub fn dentro_do_disco(u: f64, v: f64) -> bool {
    ((u - 0.5) * 2.0).hypot((0.5 - v) * 2.0) <= 1.0
}

/// O modo desenha um disco (fora dele não há cor).
#[must_use]
pub const fn e_disco(m: PixelSelector) -> bool {
    !matches!(m, PixelSelector::TintShadeTone | PixelSelector::Spectrum)
}

/// O fundo do Normal map, fora do disco: a normal que olha para o ecrã.
pub const NORMAL_FUNDO: [u8; 3] = [128, 128, 255];

/// A cor da normal `(nx, ny)` (`y` para cima) no disco unitário.
fn normal(nx: f64, ny: f64) -> [f64; 3] {
    let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
    [nx, ny, nz].map(|n| n * 0.5 + 0.5)
}

/// **A cor dos parâmetros** no modo `m`.
#[must_use]
pub fn cor(m: PixelSelector, p: Params) -> [u8; 3] {
    rgb8(match m {
        PixelSelector::TintShadeTone | PixelSelector::RgbWheel => hsv_to_rgb(p.a, p.b, p.c),
        PixelSelector::Spectrum => hsl_to_rgb(p.a, p.b, p.c),
        PixelSelector::RybWheel => ryb(p.a, p.b, p.c),
        PixelSelector::NormalMap => {
            let (nx, ny) = no_disco(p.a, p.b);
            normal(nx, ny).map(|x| x * p.c)
        }
    })
}

/// **Um clique no quadro** em `(u, v)`: os parâmetros novos (o terceiro fica).
#[must_use]
pub fn no_quadro(m: PixelSelector, p: Params, u: f64, v: f64) -> Params {
    let (u, v) = (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
    match m {
        PixelSelector::TintShadeTone => Params { b: u, c: 1.0 - v, ..p },
        PixelSelector::Spectrum => Params {
            a: u,
            c: 1.0 - v,
            ..p
        },
        PixelSelector::RgbWheel | PixelSelector::RybWheel => {
            let (dx, dy) = no_disco(u, v);
            let angulo = dy.atan2(dx) / std::f64::consts::TAU;
            Params {
                a: (angulo + RODA_OFFSET).rem_euclid(1.0),
                b: dx.hypot(dy).min(1.0),
                ..p
            }
        }
        PixelSelector::NormalMap => {
            let (dx, dy) = no_disco(u, v);
            Params {
                a: dx * 0.5 + 0.5,
                b: 0.5 - dy * 0.5,
                ..p
            }
        }
    }
}

/// **Um clique na barra** do modo em `u`: matiz (Tint), saturação (Spectrum) ou brilho (rodas).
#[must_use]
pub fn na_barra(m: PixelSelector, p: Params, u: f64) -> Params {
    let u = u.clamp(0.0, 1.0);
    match m {
        PixelSelector::TintShadeTone => Params { a: u, ..p },
        PixelSelector::Spectrum => Params { b: u, ..p },
        _ => Params { c: u, ..p },
    }
}

/// **Onde a cor dos parâmetros cai** no quadro, `(u, v)`.
#[must_use]
pub fn ponto(m: PixelSelector, p: Params) -> (f64, f64) {
    match m {
        PixelSelector::TintShadeTone => (p.b, 1.0 - p.c),
        PixelSelector::Spectrum => (p.a, 1.0 - p.c),
        PixelSelector::RgbWheel | PixelSelector::RybWheel => {
            let ang = (p.a - RODA_OFFSET) * std::f64::consts::TAU;
            (0.5 + 0.5 * p.b * ang.cos(), 0.5 - 0.5 * p.b * ang.sin())
        }
        PixelSelector::NormalMap => (p.a, p.b),
    }
}

/// Onde os parâmetros caem na barra do modo.
#[must_use]
pub fn ponto_na_barra(m: PixelSelector, p: Params) -> f64 {
    match m {
        PixelSelector::TintShadeTone => p.a,
        PixelSelector::Spectrum => p.b,
        _ => p.c,
    }
}

/// **Os parâmetros de uma cor que chegou por fora**; o que a cor não diz fica como em `antes`.
#[must_use]
pub fn de_rgb(m: PixelSelector, c: [u8; 3], antes: Params) -> Params {
    // A cor que os parâmetros de antes já dão devolve-os iguais: sem isto o arrasto realimentava a
    // reconstrução e a roda Normal Map escurecia a cada quadro (smoke do dono 25/10).
    if cor(m, antes) == c {
        return antes;
    }
    let f = unit(c);
    match m {
        PixelSelector::TintShadeTone | PixelSelector::RgbWheel => {
            let (h, s, v) = rgb_to_hsv(f);
            Params {
                a: h.unwrap_or(antes.a),
                b: s.unwrap_or(antes.b),
                c: v,
            }
        }
        PixelSelector::Spectrum => {
            let (h, s, l) = rgb_to_hsl(f);
            Params {
                a: h.unwrap_or(antes.a),
                b: s.unwrap_or(antes.b),
                c: l,
            }
        }
        PixelSelector::RybWheel => ryb_de_rgb(f, antes),
        PixelSelector::NormalMap => {
            // cor = (n·0,5 + 0,5)·brilho com |n| = 1: o brilho é a raiz de 4Σf²·s² − 4Σf·s + 2 = 0
            // em s = 1/brilho (a maior; a outra dá brilho > 1). Sem raiz real, o maior canal.
            let (sf, sf2) = (f.iter().sum::<f64>(), f.iter().map(|x| x * x).sum::<f64>());
            let disc = 16.0 * sf * sf - 32.0 * sf2;
            let v = if sf2 > 0.0 && disc >= 0.0 {
                (8.0 * sf2 / (4.0 * sf + disc.sqrt())).min(1.0)
            } else {
                f[0].max(f[1]).max(f[2])
            };
            if v <= 0.0 {
                return Params { c: 0.0, ..antes };
            }
            let (nx, ny) = (f[0] / v * 2.0 - 1.0, f[1] / v * 2.0 - 1.0);
            let (dx, dy) = no_disco(0.5 + nx * 0.5, 0.5 - ny * 0.5);
            Params {
                a: 0.5 + dx * 0.5,
                b: 0.5 - dy * 0.5,
                c: v,
            }
        }
    }
}

/// O inverso da roda RYB (não tem fórmula fechada): brilho = o canal máximo; a matiz é a que, com a
/// saturação que o canal mínimo pede, chega mais perto — procurada em passos de ¼ de grau.
fn ryb_de_rgb(f: [f64; 3], antes: Params) -> Params {
    let v = f[0].max(f[1]).max(f[2]);
    let min = f[0].min(f[1]).min(f[2]);
    if v <= 0.0 {
        return Params { c: 0.0, ..antes };
    }
    if v - min <= f64::EPSILON {
        return Params {
            b: 0.0,
            c: v,
            ..antes
        };
    }
    const PASSOS: u32 = 1440;
    let mut melhor = (f64::INFINITY, antes.a, antes.b);
    for i in 0..PASSOS {
        let h = f64::from(i) / f64::from(PASSOS);
        let pura = ryb_hue(h);
        let m = pura[0].min(pura[1]).min(pura[2]);
        let s = ((1.0 - min / v) / (1.0 - m).max(f64::EPSILON)).clamp(0.0, 1.0);
        let c = ryb(h, s, v);
        let e: f64 = (0..3).map(|k| (c[k] - f[k]).powi(2)).sum();
        if e < melhor.0 {
            melhor = (e, h, s);
        }
    }
    Params {
        a: melhor.1,
        b: melhor.2,
        c: v,
    }
}

/// **A imagem do quadro**, `lado × lado` RGBA de alfa reto (fora do disco: transparente, ou o fundo
/// do Normal map).
#[must_use]
pub fn imagem_do_quadro(m: PixelSelector, p: Params, lado: u32) -> Vec<u8> {
    let n = f64::from(lado.max(1));
    let mut out = Vec::with_capacity((lado * lado * 4) as usize);
    for y in 0..lado {
        for x in 0..lado {
            let (u, v) = ((f64::from(x) + 0.5) / n, (f64::from(y) + 0.5) / n);
            if !e_disco(m) {
                out.extend_from_slice(&cor(m, no_quadro(m, p, u, v)));
                out.push(u8::MAX);
                continue;
            }
            // A borda do disco SUAVE: a cobertura é a distância à borda em píxeis da imagem
            // (meio píxel para cada lado), misturada com o fundo do modo.
            let raio = ((u - 0.5) * 2.0).hypot((0.5 - v) * 2.0);
            let cob = ((1.0 - raio) * n * 0.5 + 0.5).clamp(0.0, 1.0);
            let dentro = if raio <= 1.0 { (u, v) } else { ((u - 0.5) / raio + 0.5, (v - 0.5) / raio + 0.5) };
            let c = cor(m, no_quadro(m, p, dentro.0, dentro.1));
            if m == PixelSelector::NormalMap {
                let mistura = |k: usize| {
                    let (a, b) = (f64::from(c[k]), f64::from(NORMAL_FUNDO[k]));
                    byte((b + (a - b) * cob) / 255.0)
                };
                out.extend_from_slice(&[mistura(0), mistura(1), mistura(2), u8::MAX]);
            } else {
                out.extend_from_slice(&c);
                out.push(byte(cob));
            }
        }
    }
    out
}

/// **A imagem da barra** do modo, `largura × 1`.
#[must_use]
pub fn imagem_da_barra(m: PixelSelector, p: Params, largura: u32) -> Vec<u8> {
    let n = f64::from(largura.max(1));
    (0..largura)
        .flat_map(|x| {
            let [r, g, b] = cor(m, na_barra(m, p, (f64::from(x) + 0.5) / n));
            [r, g, b, u8::MAX]
        })
        .collect()
}

/// **A imagem da barra de alfa**: a cor `c` de transparente a opaca, `largura × 1`.
#[must_use]
pub fn imagem_do_alfa(c: [u8; 3], largura: u32) -> Vec<u8> {
    let n = f64::from(largura.max(1));
    (0..largura)
        .flat_map(|x| [c[0], c[1], c[2], byte((f64::from(x) + 0.5) / n)])
        .collect()
}

/// O alfa de um clique em `u` na barra de alfa.
#[must_use]
pub fn alfa(u: f64) -> u8 {
    byte(u.clamp(0.0, 1.0))
}
