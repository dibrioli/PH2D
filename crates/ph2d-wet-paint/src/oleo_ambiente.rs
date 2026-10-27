//! **Os ambientes do Óleo Real** (W1c, [`49_realshader/oraculo/`](../../../docs/Painter/49_realshader/oraculo/)):
//! o REFLEXO da tinta é o de um HDRI (o «Environment» do Rebelle), não o de uma lâmpada só. Cada `.amb` é um HDRI CC0 do
//! Blender pré-filtrado por `prefiltra_ambiente.py` (irradiância exacta + brilho GGX em 4 rugosidades),
//! na moldura do Blender (z para cima), escalado para o plano virado para cima = 1. W2 E2: o `PH2DAMB3`
//! traz também a radiância BRUTA da metade de cima, que vira as [`Sectores`] do DIFUSO (o céu visível de
//! cada fatia do GTAO integrado contra o HDRI).

use std::sync::OnceLock;

/// O ambiente que a tinta reflecte. `Classico` = o céu uniforme de antes (sem reflexo do céu).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ambiente {
    Classico,
    #[default]
    Estudio,
    Interior,
    Patio,
}

impl Ambiente {
    pub const TODOS: [Self; 4] = [Self::Classico, Self::Estudio, Self::Interior, Self::Patio];

    #[must_use]
    pub fn indice(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub fn de_indice(i: u8) -> Option<Self> {
        Self::TODOS.get(usize::from(i)).copied()
    }

    pub(crate) fn mapa(self) -> Option<&'static Mapa> {
        static MAPAS: [OnceLock<Mapa>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];
        let (k, bytes): (usize, &'static [u8]) = match self {
            Self::Classico => return None,
            Self::Estudio => (0, include_bytes!("../ambientes/estudio.amb")),
            Self::Interior => (1, include_bytes!("../ambientes/interior.amb")),
            Self::Patio => (2, include_bytes!("../ambientes/patio.amb")),
        };
        Some(MAPAS[k].get_or_init(|| Mapa::de(bytes)))
    }
}

pub(crate) struct Mapa {
    w: usize,
    h: usize,
    /// Azimute (moldura do Blender) do lado claro, pesado pelo cosseno: roda-se até à luz.
    pub(crate) claro: f64,
    rug: Vec<f64>,
    irr: Vec<[f32; 3]>,
    niveis: Vec<Vec<[f32; 3]>>,
    /// A irradiância do plano virado para cima, por canal: a escala do brilho (o reflexo não tinge o branco).
    cima: [f64; 3],
    /// O céu por SECTOR de fatia (`PH2DAMB3`); `None` num `PH2DAMB2` (o difuso fica o céu uniforme).
    pub(crate) sectores: Option<Sectores>,
}

/// **O céu por sector** (W2 E2): para cada azimute `u` do mapa (`K` colunas) a radiância média da CUNHA de
/// azimute de uma fatia (`π/4` = 2π / (2·[`crate::oleo_real`] fatias)), integrada ao longo da elevação
/// medida do zénite `α` em somas ACUMULADAS `∫₀^α L·sinα·cosα dα` e `∫₀^α L·sin²α dα` (`M+1` nós até ao
/// horizonte): o termo `cos(α−γ)` da normal projectada na fatia sai em forma fechada
/// (`cosα·cosγ + sinα·sinγ`), O(1) por lado de fatia. Céu uniforme `L = 1` dá `sin²α/2` e `α/2 − sin2α/4`,
/// os dois pedaços da integral de Jimenez — por isso o HDRI uniforme reduz-se à visibilidade.
pub(crate) struct Sectores {
    k: usize,
    m: usize,
    /// `[j·k + b]`, por canal: `(∫ L sinα cosα, ∫ L sin²α)` até ao nó `j` (`α = j·π/(2m)`); o canal `3` é o
    /// céu UNIFORME `L = 1` pela MESMA interpolação (o denominador do ganho: o erro dela cancela-se).
    c: Vec<[[f64; 4]; 2]>,
}

/// A largura (em fracção da volta) da cunha de azimute de cada lado de uma fatia: `FATIAS = 4` ⇒ 8 lados.
pub(crate) const CUNHA: f64 = 1.0 / 8.0;

impl Sectores {
    fn de(rad: &[[f32; 3]], k: usize, m: usize) -> Self {
        use core::f64::consts::FRAC_PI_2;
        let meia = (CUNHA * k as f64 / 2.0) as usize;
        assert_eq!(2 * meia, (CUNHA * k as f64) as usize, "ambiente: cunha");
        let da = FRAC_PI_2 / m as f64;
        let mut c = vec![[[0.0; 4]; 2]; (m + 1) * k];
        for j in 0..m {
            let (a0, a1) = (j as f64 * da, (j + 1) as f64 * da);
            let i1 = (libm::sin(a1).powi(2) - libm::sin(a0).powi(2)) / 2.0;
            let i2 =
                ((a1 - libm::sin(a1) * libm::cos(a1)) - (a0 - libm::sin(a0) * libm::cos(a0))) / 2.0;
            for b in 0..k {
                // A média da cunha centrada na coluna `b` (as pontas pesam meio: largura exacta).
                let mut t = [0.0f64, 0.0, 0.0, (2 * meia) as f64];
                for d in -(meia as isize)..=meia as isize {
                    let w = if d.unsigned_abs() == meia { 0.5 } else { 1.0 };
                    let x = (b as isize + d).rem_euclid(k as isize) as usize;
                    for (tc, &r) in t.iter_mut().zip(&rad[x + j * k]) {
                        *tc += w * f64::from(r);
                    }
                }
                let ant = c[b + j * k];
                let novo = &mut c[b + (j + 1) * k];
                for ch in 0..4 {
                    let l = t[ch] / (2 * meia) as f64;
                    novo[0][ch] = ant[0][ch] + l * i1;
                    novo[1][ch] = ant[1][ch] + l * i2;
                }
            }
        }
        Self { k, m, c }
    }

    /// As duas somas acumuladas no azimute `u` (fracção da volta, a do [`Mapa::amostra`]) até à elevação `α`
    /// do zénite (`0..=π/2`), bilinear.
    #[inline]
    pub(crate) fn ate(&self, u: f64, alfa: f64) -> [[f64; 4]; 2] {
        let fx = u.rem_euclid(1.0) * self.k as f64 - 0.5;
        let x0 = libm::floor(fx);
        let tx = fx - x0;
        let xa = (x0 as isize).rem_euclid(self.k as isize) as usize;
        let xb = (xa + 1) % self.k;
        let fy = (alfa / core::f64::consts::FRAC_PI_2 * self.m as f64).clamp(0.0, self.m as f64);
        let j0 = (libm::floor(fy) as usize).min(self.m - 1);
        let ty = fy - j0 as f64;
        let p = |x: usize, j: usize| &self.c[x + j * self.k];
        let (p00, p10, p01, p11) = (p(xa, j0), p(xb, j0), p(xa, j0 + 1), p(xb, j0 + 1));
        let mut out = [[0.0; 4]; 2];
        for (q, oq) in out.iter_mut().enumerate() {
            for (ch, o) in oq.iter_mut().enumerate() {
                let l0 = p00[q][ch] + (p10[q][ch] - p00[q][ch]) * tx;
                let l1 = p01[q][ch] + (p11[q][ch] - p01[q][ch]) * tx;
                *o = l0 + (l1 - l0) * ty;
            }
        }
        out
    }

    /// Um céu UNIFORME de radiância 1 (só para os gates: o HDRI uniforme tem de dar o Clássico).
    #[cfg(test)]
    pub(crate) fn uniforme() -> Self {
        Self::de(&vec![[1.0f32; 3]; 128 * 32], 128, 32)
    }
}

impl Mapa {
    fn de(b: &[u8]) -> Self {
        let v3 = &b[..8] == b"PH2DAMB3";
        assert!(v3 || &b[..8] == b"PH2DAMB2", "ambiente: cabeçalho");
        let u = |i: usize| u32::from_le_bytes(b[i..i + 4].try_into().expect("u32")) as usize;
        let f = |i: usize| f64::from(f32::from_le_bytes(b[i..i + 4].try_into().expect("f32")));
        let (w, h, n) = (u(8), u(12), u(16));
        let claro = f(20);
        let mut p = 24;
        let rug = (0..n).map(|k| f(p + 4 * k)).collect();
        p += 4 * n;
        let le = |p: &mut usize, m: usize| {
            let v: Vec<[f32; 3]> = (0..m)
                .map(|k| [0, 1, 2].map(|c| f(*p + 12 * k + 4 * c) as f32))
                .collect();
            *p += 12 * m;
            v
        };
        let irr = le(&mut p, (w / 2) * (h / 2));
        let niveis = (0..n).map(|_| le(&mut p, w * h)).collect();
        let sectores = v3.then(|| {
            let (k, m) = (u(p), u(p + 4));
            p += 8;
            Sectores::de(&le(&mut p, k * m), k, m)
        });
        assert_eq!(p, b.len(), "ambiente: tamanho");
        let mut m = Self {
            w,
            h,
            claro,
            rug,
            irr,
            niveis,
            cima: [1.0; 3],
            sectores,
        };
        m.cima = m.irradiancia_bruta([0.0, 0.0, 1.0]);
        m
    }

    /// Bilinear numa equirectangular `w×h` (linha 0 = zénite), `d` unitário na moldura do mapa. Acima do
    /// centro da 1.ª linha mistura-se até ao POLO (a média dessa linha): no zénite o azimute é arbitrário
    /// (`atan2(±0, …)`), e a tinta lisa saía 8 % mais clara com o estúdio.
    fn amostra(t: &[[f32; 3]], w: usize, h: usize, d: [f64; 3]) -> [f64; 3] {
        let u = libm::atan2(d[1], -d[0]) / core::f64::consts::TAU + 0.5;
        let v = libm::atan2(d[2], libm::hypot(d[0], d[1])) / core::f64::consts::PI + 0.5;
        let fx = u * w as f64 - 0.5;
        let fy_bruto = (1.0 - v) * h as f64 - 0.5;
        let fy = fy_bruto.clamp(0.0, (h - 1) as f64);
        let (x0, y0) = (libm::floor(fx), libm::floor(fy));
        let (tx, ty) = (fx - x0, fy - y0);
        let xi = |k: f64| (k as isize).rem_euclid(w as isize) as usize;
        let (xa, xb) = (xi(x0), xi(x0 + 1.0));
        let (ya, yb) = (y0 as usize, (y0 as usize + 1).min(h - 1));
        let polo = (fy_bruto < 0.0).then(|| (-fy_bruto / 0.5).min(1.0));
        [0, 1, 2].map(|c| {
            let p = |x: usize, y: usize| f64::from(t[x + y * w][c]);
            let l0 = p(xa, ya) + (p(xb, ya) - p(xa, ya)) * tx;
            let l1 = p(xa, yb) + (p(xb, yb) - p(xa, yb)) * tx;
            let s = l0 + (l1 - l0) * ty;
            match polo {
                None => s,
                Some(k) => {
                    let m = (0..w).map(|x| p(x, 0)).sum::<f64>() / w as f64;
                    s + (m - s) * k
                }
            }
        })
    }

    fn irradiancia_bruta(&self, n: [f64; 3]) -> [f64; 3] {
        Self::amostra(&self.irr, self.w / 2, self.h / 2, n)
    }

    /// O brilho pré-filtrado na direcção `r` à rugosidade `rug` (interpolada entre os níveis), na mesma
    /// escala relativa da irradiância.
    pub(crate) fn brilho(&self, r: [f64; 3], rug: f64) -> [f64; 3] {
        let n = self.rug.len();
        let k = self
            .rug
            .iter()
            .rposition(|&x| x <= rug)
            .unwrap_or(0)
            .min(n - 2);
        let t = ((rug - self.rug[k]) / (self.rug[k + 1] - self.rug[k])).clamp(0.0, 1.0);
        let a = Self::amostra(&self.niveis[k], self.w, self.h, r);
        let b = Self::amostra(&self.niveis[k + 1], self.w, self.h, r);
        [0, 1, 2].map(|c| (a[c] + (b[c] - a[c]) * t) / self.cima[c].max(1e-9))
    }
}
