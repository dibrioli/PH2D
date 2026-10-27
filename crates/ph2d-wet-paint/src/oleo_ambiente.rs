//! **Os ambientes do Óleo Real** (W1c, [`49_realshader/oraculo/`](../../../docs/Painter/49_realshader/oraculo/)):
//! o REFLEXO da tinta é o de um HDRI (o «Environment» do Rebelle), não o de uma lâmpada só. Cada `.amb` é um HDRI CC0 do
//! Blender pré-filtrado por `prefiltra_ambiente.py` (irradiância exacta + brilho GGX em 4 rugosidades),
//! na moldura do Blender (z para cima), escalado para o plano virado para cima = 1.

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
}

impl Mapa {
    fn de(b: &[u8]) -> Self {
        assert_eq!(&b[..8], b"PH2DAMB2", "ambiente: cabeçalho");
        let u = |i: usize| u32::from_le_bytes(b[i..i + 4].try_into().expect("u32")) as usize;
        let f = |i: usize| f64::from(f32::from_le_bytes(b[i..i + 4].try_into().expect("f32")));
        let (w, h, n) = (u(8), u(12), u(16));
        let claro = f(20);
        let mut p = 24;
        let rug = (0..n).map(|k| f(p + 4 * k)).collect();
        p += 4 * n;
        let mut le = |m: usize| {
            let v: Vec<[f32; 3]> = (0..m)
                .map(|k| [0, 1, 2].map(|c| f(p + 12 * k + 4 * c) as f32))
                .collect();
            p += 12 * m;
            v
        };
        let irr = le((w / 2) * (h / 2));
        let niveis = (0..n).map(|_| le(w * h)).collect();
        assert_eq!(p, b.len(), "ambiente: tamanho");
        let mut m = Self {
            w,
            h,
            claro,
            rug,
            irr,
            niveis,
            cima: [1.0; 3],
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
