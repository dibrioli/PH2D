//! **O material no sombreado do Óleo Real** (filho de [`super`]): o Principled SIMPLIFICADO do Cycles sobre a
//! tinta (W3: Metallic · Roughness · IOR Level · Coat · Coat Roughness · Sheen · Subsurface · Subsurface Radius),
//! a partir do [`MaterialCelula`] da célula combinado com o ajuste GLOBAL do cartão Real Oil ([`MaterialCartao`],
//! regra em [`crate::material::efectivo`]). Medido contra o Cycles em `docs/Painter/49_realshader/oraculo/`
//! (README §E3-B). Com a célula NEUTRA e o cartão no padrão o sombreado é o de antes, bit a bit. O reflexo usa o
//! Environment (o Principled não tem mapa de reflexo à parte).

use crate::material::MaterialCelula;

/// A Roughness do cartão no PADRÃO: a `Afinacao::rugosidade` de antes da W3 (0,28, a régua M4) — a imagem de antes.
pub const RUGOSIDADE_PADRAO: f64 = 0.28;

/// O material GLOBAL do cartão Real Oil, os 8 controlos do Blender em valores ABSOLUTOS. [`Self::NEUTRO`] é o
/// padrão do cartão (= a imagem de antes). Conversão do cartão de antes da W3 (multiplicadores): Roughness =
/// `0,28 × m`, IOR Level = `0,5 × s`, Coat Roughness = `0,03 × r`; Translucency → Subsurface
/// ([`Self::poe_campo_antigo`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCartao {
    /// Metallic (`0..1`, max com a célula).
    pub metal: f64,
    /// Roughness absoluta (`0..1`).
    pub rugosidade: f64,
    /// IOR Level (`0,5` = IOR 1,5).
    pub nivel_ior: f64,
    /// Coat (`0..1`, max).
    pub verniz: f64,
    /// Coat Roughness absoluta.
    pub verniz_rugosidade: f64,
    /// Sheen (`0..1`, max).
    pub veludo: f64,
    /// Subsurface (`0..1`, max).
    pub subsuperficie: f64,
    /// Subsurface Radius, px de tela.
    pub raio_subsuperficie: f64,
}

impl MaterialCartao {
    pub const NEUTRO: Self = Self {
        metal: 0.0,
        rugosidade: RUGOSIDADE_PADRAO,
        nivel_ior: 0.5,
        verniz: 0.0,
        // O neutro da célula em `f64` (a razão padrão/neutro é `1` exacto: a célula passa ao bit).
        verniz_rugosidade: MaterialCelula::NEUTRO.verniz_rugosidade as f64,
        veludo: 0.0,
        subsuperficie: 0.0,
        raio_subsuperficie: crate::material::RAIO_SUBSUPERFICIE_PADRAO as f64,
    };

    /// Os 8 campos, pela ordem da UI (`PAINTER_WETPAINT_OLEO_FIELDS[8..]`) e com os nomes da macro.
    pub const CAMPOS: [&'static str; 8] = [
        "metal",
        "rugosidade",
        "nivel_ior",
        "verniz",
        "verniz_rugosidade",
        "veludo",
        "subsuperficie",
        "raio_subsuperficie",
    ];

    /// Os valores de [`Self::CAMPOS`], pela mesma ordem.
    #[must_use]
    pub const fn valores(&self) -> [f64; 8] {
        [
            self.metal,
            self.rugosidade,
            self.nivel_ior,
            self.verniz,
            self.verniz_rugosidade,
            self.veludo,
            self.subsuperficie,
            self.raio_subsuperficie,
        ]
    }

    /// Põe o campo `k` de [`Self::CAMPOS`] (fora da faixa = sem efeito).
    pub fn poe(&mut self, k: usize, v: f64) {
        let alvo = match k {
            0 => &mut self.metal,
            1 => &mut self.rugosidade,
            2 => &mut self.nivel_ior,
            3 => &mut self.verniz,
            4 => &mut self.verniz_rugosidade,
            5 => &mut self.veludo,
            6 => &mut self.subsuperficie,
            7 => &mut self.raio_subsuperficie,
            _ => return,
        };
        *alvo = v;
    }

    /// O LEITOR TOLERANTE do cartão de antes da W3 (multiplicadores, nomes da macro sem o `mat_`): converte
    /// `rugosidade`/`especular`/`verniz_rugosidade` (× o padrão) e `translucidez` (→ Subsurface); os outros
    /// campos do cartão de então (metal, verniz, veludo) têm o mesmo sentido; os que saíram são ignorados.
    pub fn poe_campo_antigo(&mut self, nome: &str, v: f64) {
        match nome {
            "metal" => self.metal = v,
            "rugosidade" => self.rugosidade = RUGOSIDADE_PADRAO * v,
            "especular" => self.nivel_ior = Self::NEUTRO.nivel_ior * v,
            "verniz" => self.verniz = v,
            "verniz_rugosidade" => self.verniz_rugosidade = Self::NEUTRO.verniz_rugosidade * v,
            "veludo" => self.veludo = v,
            "translucidez" => self.subsuperficie = v,
            _ => {}
        }
    }
}

impl Default for MaterialCartao {
    fn default() -> Self {
        Self::NEUTRO
    }
}

/// O VELUDO (Sheen) relativo, à luz branca de difuso `e` e normal de cosseno com a vista `nz`: `s·(k₀ +
/// k₁·(1−nz)^p)·e`, as constantes ajustadas ao Sheen do Principled (rugosidade 0,5) na cena do dono
/// (`oraculo/README.md` §E3: sheen/e 0,031 de frente → 0,108 a `1−nz ≈ 0,6`).
pub(super) fn veludo(s: f64, nz: f64, e: f64) -> f64 {
    s * (VELUDO_K0 + VELUDO_K1 * libm::pow((1.0 - nz).clamp(0.0, 1.0), VELUDO_P)) * e
}
pub(super) const VELUDO_K0: f64 = 0.031;
pub(super) const VELUDO_K1: f64 = 0.20;
pub(super) const VELUDO_P: f64 = 1.83;

/// SUBSURFACE: o raio de cada canal ÷ o Subsurface Radius (o `Radius` de omissão do Principled — o vermelho
/// espalha-se mais longe; a cor do espalhamento é a da tinta).
pub const PERFIL_SUBSUPERFICIE: [f64; 3] = [1.0, 0.2, 0.1];

/// O alcance (células) do espalhamento de raio `raio` px a `ratio` px por célula: `3σ` do canal mais largo.
#[must_use]
pub fn alcance_subsuperficie(raio: f64, ratio: usize) -> usize {
    let r = raio.clamp(0.0, f64::from(crate::material::RAIO_SUBSUPERFICIE_MAX));
    libm::ceil(3.0 * r * PERFIL_SUBSUPERFICIE[0] / ratio.max(1) as f64) as usize
}

/// O MAIOR raio efectivo (px) da Subsurface que a janela `(x0, y0, x1, y1)` lê — as suas colunas, com as linhas
/// alargadas pelo alcance do tecto (a 1.ª passagem corre lá com o raio de cada célula) —, `None` sem Subsurface
/// (nem no cartão nem em célula de camada visível): o alcance do espalhamento é o deste raio, não o do tecto.
pub(super) fn raio_subsuperficie_max(
    camadas: &[(&crate::grid::Grid, bool)],
    o: &super::OleoReal,
    ratio: usize,
    (x0, y0, x1, y1): (usize, usize, usize, usize),
) -> Option<f64> {
    let k = &o.material;
    let t = alcance_subsuperficie(f64::INFINITY, ratio);
    let (y0, y1) = (y0.saturating_sub(t).max(1), (y1 + t).min(camadas[0].0.h));
    let mut r = (k.subsuperficie > 0.0).then_some(k.raio_subsuperficie);
    for &(g, v) in camadas {
        if !v || g.material.is_empty() {
            continue;
        }
        for y in y0..=y1 {
            for c in &g.material[x0 + y * g.s..=x1 + y * g.s] {
                if c.subsuperficie > 0.0 || k.subsuperficie > 0.0 {
                    let e = crate::material::efectivo(c, k).raio_subsuperficie;
                    r = Some(r.map_or(e, |r: f64| r.max(e)));
                }
            }
        }
    }
    r
}

/// A irradiância relativa da célula (o difuso de uma tinta cinza `0,5`, plano iluminado = 1) — a luz que a
/// Subsurface espalha: os mesmos termos difusos do [`super::termos_com`], das peças `lc` da célula.
fn irradiancia(
    o: &super::OleoReal,
    campo: &super::CampoAltura,
    x: isize,
    y: isize,
    lc: &super::LuzCeu,
) -> f64 {
    let n = super::normal(campo, x, y);
    let l = super::para_a_luz(o);
    let super::LuzCeu { luz, v, dir } = *lc;
    let ndl = (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]).max(0.0);
    super::Termos {
        vis: super::forca_da_fenda(v, o.fenda),
        luz,
        sol: (1.0 - o.forca_ambiente) * (ndl / l[2]) * luz,
        espec: [0.0; 3],
        dir,
        metal: 0.0,
        sob_verniz: 1.0,
        sss: [0.0; 3],
    }
    .e(o)
}

/// A média gaussiana de `f(k)` em `k ∈ [lo, hi]` à volta de `c`, de desvio `s` (renormalizada nas bordas).
fn gauss(c: usize, lo: usize, hi: usize, s: f64, f: impl Fn(usize) -> f64) -> f64 {
    if s < 0.25 {
        return f(c);
    }
    let r = libm::ceil(3.0 * s) as usize;
    let k = -0.5 / (s * s);
    let (mut acc, mut pes) = (0.0, 0.0);
    for i in c.saturating_sub(r).max(lo)..=(c + r).min(hi) {
        let d = i as f64 - c as f64;
        let p = libm::exp(k * d * d);
        acc += p * f(i);
        pes += p;
    }
    acc / pes
}

/// **A lei da Subsurface** (W3 M1, `oraculo/README.md` §M-SSS): a luz que ENTRA ao lado e sai aqui — a
/// irradiância desfocada por canal (gaussiana separável, desvio = o raio efectivo × [`PERFIL_SUBSUPERFICIE`],
/// à Jimenez com o raio da célula em cada passagem), somada só onde a vizinhança é mais clara (`max(0, B − E)`:
/// a fenda e a sombra enchem-se da cor da tinta, a encosta iluminada fica — o Cycles, Random Walk). Devolve, por
/// célula da janela do `campo`, o que se SOMA ao difuso de cada canal antes do peso; zero fora de `(x0..x1,
/// y0..y1)` e onde a célula não tem Subsurface. Devolve também as [`super::LuzCeu`] da janela da irradiância
/// `(x0, y0, w, h)` — o sombreado da mesma célula reusa-as.
#[allow(clippy::type_complexity)]
pub(super) fn espalha(
    rep: &impl super::Reparte,
    campo: &super::CampoAltura,
    o: &super::OleoReal,
    (ratio, rr): (usize, usize),
    (x0, y0, x1, y1): (usize, usize, usize, usize),
    (gw, gh): (usize, usize),
) -> (
    Vec<[f32; 3]>,
    Vec<super::LuzCeu>,
    (usize, usize, usize, usize),
) {
    let (ex0, ey0) = (x0.saturating_sub(rr).max(1), y0.saturating_sub(rr).max(1));
    let (ex1, ey1) = ((x1 + rr).min(gw), (y1 + rr).min(gh));
    let (ew, eh, dw) = (ex1 + 1 - ex0, ey1 + 1 - ey0, x1 + 1 - x0);
    let mut e = vec![0f64; ew * eh];
    let nada = super::LuzCeu {
        luz: 0.0,
        v: 0.0,
        dir: None,
    };
    let mut luz_ceu = vec![nada; ew * eh];
    rep.por_linhas(ew, &mut e, &mut luz_ceu, &|y, linha, lcs| {
        for (x, (q, lc)) in linha.iter_mut().zip(lcs.iter_mut()).enumerate() {
            let (cx, cy) = ((ex0 + x) as isize, (ey0 + y) as isize);
            *lc = super::luz_ceu(o, campo, cx, cy);
            *q = irradiancia(o, campo, cx, cy, lc);
        }
    });
    let tecto = f64::from(crate::material::RAIO_SUBSUPERFICIE_MAX);
    let mat = |x: usize, y: usize| {
        let m = crate::material::efectivo(&campo.material_em(x, y), &o.material);
        let r = m.raio_subsuperficie.clamp(0.0, tecto) / ratio.max(1) as f64;
        (m.subsuperficie, PERFIL_SUBSUPERFICIE.map(|p| p * r))
    };
    // Passagem 1 (linhas): só as colunas da janela pedida, em todas as linhas que a 2.ª lê.
    let mut hz = vec![[0f64; 3]; dw * eh];
    super::por_linha(rep, dw, &mut hz, &|y, linha| {
        for (x, q) in linha.iter_mut().enumerate() {
            let (cx, cy) = (x0 + x, ey0 + y);
            let s = mat(cx, cy).1;
            for c in 0..3 {
                q[c] = gauss(cx, ex0, ex1, s[c], |i| e[i - ex0 + y * ew]);
            }
        }
    });
    let mut out = vec![[0f32; 3]; campo.w * campo.h];
    super::por_linha(rep, campo.w, &mut out, &|y, linha| {
        let cy = campo.y0 + y;
        if cy < y0 || cy > y1 {
            return;
        }
        for cx in x0..=x1 {
            let (w, s) = mat(cx, cy);
            if w <= 0.0 {
                continue;
            }
            let e0 = e[cx - ex0 + (cy - ey0) * ew];
            let q = &mut linha[cx - campo.x0];
            for c in 0..3 {
                let b = gauss(cy, ey0, ey1, s[c], |j| hz[cx - x0 + (j - ey0) * dw][c]);
                q[c] = (b - e0).max(0.0) as f32;
            }
        }
    });
    (out, luz_ceu, (ex0, ey0, ew, eh))
}

#[cfg(test)]
#[path = "oleo_material_tests.rs"]
mod oleo_material_tests;
