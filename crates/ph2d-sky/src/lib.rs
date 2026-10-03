//! ⭐⭐⭐ **O CÉU FOTOGRÁFICO (HDRI)** — um panorama equiretangular de luz real, pré-filtrado UMA vez
//! para responder às duas perguntas que a lei do material faz ao céu ([`ph2d_material::Environment`]):
//!
//! | pergunta | o que o atlas guarda | a régua |
//! |---|---|---|
//! | a radiância pela direcção `R`, já filtrada pelo lóbulo GGX de `α` (com `N = V = R`) | [`NIVEIS`] mapas octaédricos, um por `√α = k/(NIVEIS−1)` | a quadratura do lóbulo sobre o panorama cru |
//! | a irradiância normalizada `E(n)/π` | o ÚLTIMO nível (`α = 1`) — ver [`Ceu::irradiance`] | a soma sobre o panorama, e o Cycles (oráculo) |
//!
//! ⭐ **Uma porta por lei:** a CPU ([`Ceu::radiance`], [`Ceu::irradiance`]) e o WGSL ([`wgsl`]) fazem a
//! MESMA leitura bilinear do MESMO atlas, na mesma ordem de contas — quem desenha sobe o
//! [`Ceu::atlas`] para uma textura e lê-a texel a texel (nada de amostrador: o WebGL2 não filtra
//! `f32`, e um filtro da placa seria uma segunda resposta).
//!
//! ⚠️ **A orientação é a do Blender** (medida contra o Cycles: gate `a_orientacao_e_a_do_blender`): um
//! céu da Poly Haven aparece aqui como lá, com `y` para cima. O giro em torno de `y` é do chamador
//! ([`Orientado`]) e não refaz nada.

mod embarcados;
mod prefiltro;
pub mod sol;
pub mod wgsl;

pub use embarcados::Embarcado;
pub use sol::Sol;

/// Uma cor linear.
pub type Rgb = [f32; 3];

/// Quantos níveis de rugosidade o atlas guarda: `√α = k / (NIVEIS − 1)`.
pub const NIVEIS: usize = 17;

/// O lado (interior) do mapa octaédrico de cada nível. ⚠️ Cada um tem mais um texel de margem de cada
/// lado, preenchido pela dobra octaédrica: a leitura bilinear nunca sai do nível.
///
/// ⛔ Medido (02/10, a decomposição contra a convolução): com `9` níveis, a meio caminho entre o
/// espelho e o 1.º (`α ≈ 0,004`) o erro era `5–10 %` de mediana e `280 %` de máximo — a forma muda
/// depressa perto do espelho. O lado de cada um segue o lóbulo: o texel `≤ ¼` da largura dele; o
/// espelho é `512` porque o panorama é `1K` (`0,35°` por texel no equador) — a `256` a floresta
/// perdia `17 %` de mediana contra o Cycles.
pub const LADOS: [u32; NIVEIS] = [
    512, 512, 256, 128, 128, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
];

/// A largura do atlas: dois espelhos lado a lado. ⚠️ O WebGL2 garante `2048`.
pub const ATLAS_W: u32 = 2 * (LADOS[0] + 2);

/// O lado (com margem) do mapa `k`.
const fn lado_com_margem(k: usize) -> u32 {
    LADOS[k] + 2
}

/// Arrumação em PRATELEIRAS (os lados não crescem): `(x0, y0)` de cada mapa e a altura do atlas.
const fn prateleiras() -> ([u32; NIVEIS], [u32; NIVEIS], u32) {
    let (mut xs, mut ys) = ([0u32; NIVEIS], [0u32; NIVEIS]);
    let (mut x, mut y, mut h) = (0u32, 0u32, 0u32);
    let mut k = 0;
    while k < NIVEIS {
        let s = lado_com_margem(k);
        if x + s > ATLAS_W {
            y += h;
            x = 0;
            h = 0;
        }
        xs[k] = x;
        ys[k] = y;
        x += s;
        if s > h {
            h = s;
        }
        k += 1;
    }
    (xs, ys, y + h)
}

/// A coluna do canto de cada nível no atlas.
pub const X0: [u32; NIVEIS] = prateleiras().0;
/// A linha do canto de cada nível no atlas.
pub const Y0: [u32; NIVEIS] = prateleiras().1;
/// A altura do atlas.
pub const ATLAS_H: u32 = prateleiras().2;
const _: () = assert!(ATLAS_H <= 2048, "o WebGL2 só garante 2048");

/// ⭐ **Um panorama equiretangular** — `rgb` linha a linha, a linha `0` em CIMA (o polo `+y`).
#[derive(Clone, Debug, PartialEq)]
pub struct Panorama {
    pub largura: u32,
    pub altura: u32,
    pub rgb: Vec<Rgb>,
}

impl Panorama {
    /// Lê um OpenEXR (o RGB da primeira camada; o alfa é ignorado).
    ///
    /// # Errors
    /// Quando o ficheiro não é um EXR que o leitor da casa decodifica.
    pub fn de_exr(bytes: &[u8]) -> Result<Self, String> {
        use ph2d_imageio::ImageImporter;
        let img = ph2d_imageio_exr::ExrImporter
            .import(bytes, &ph2d_imageio::ImportOpts::default())
            .map_err(|e| format!("{e}"))?;
        let ph2d_imageio::DecodedImage::FlatHdr(b) = img else {
            return Err("o EXR não é uma imagem HDR plana".to_owned());
        };
        Ok(Self {
            largura: b.width,
            altura: b.height,
            rgb: b
                .pixels
                .iter()
                .map(|p| [p.r(), p.g(), p.b()].map(|c| if c.is_finite() { c.max(0.0) } else { 0.0 }))
                .collect(),
        })
    }

    /// Um céu da mesma radiância `l` em toda a direcção — a fixtura do forno.
    #[must_use]
    pub fn chapado(l: Rgb) -> Self {
        Self {
            largura: 128,
            altura: 64,
            rgb: vec![l; 128 * 64],
        }
    }

    /// ⭐ **A direcção → o ponto do panorama**, em pixels contínuos (o centro do texel `i` está em
    /// `i + 0,5`). A convenção do Blender com `y` para cima: `u` cresce com `atan2(z, x)`, a linha `0`
    /// é o polo `+y`.
    #[must_use]
    pub fn ponto(&self, d: [f32; 3]) -> (f32, f32) {
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1.0e-20);
        let u = 0.5 + d[2].atan2(d[0]) / core::f32::consts::TAU;
        let v = (d[1] / len).clamp(-1.0, 1.0).acos() / core::f32::consts::PI;
        (u * self.largura as f32, v * self.altura as f32)
    }

    /// O texel `(x, y)`, com `x` a dar a volta e `y` preso.
    fn texel(&self, x: i64, y: i64) -> Rgb {
        let w = i64::from(self.largura);
        let x = x.rem_euclid(w);
        let y = y.clamp(0, i64::from(self.altura) - 1);
        self.rgb[(y * w + x) as usize]
    }

    /// A leitura bilinear no ponto contínuo `(px, py)`.
    #[must_use]
    pub fn bilinear(&self, px: f32, py: f32) -> Rgb {
        let (fx, fy) = (px - 0.5, py - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0);
        let (x0, y0) = (x0 as i64, y0 as i64);
        let a = self.texel(x0, y0);
        let b = self.texel(x0 + 1, y0);
        let c = self.texel(x0, y0 + 1);
        let d = self.texel(x0 + 1, y0 + 1);
        [0, 1, 2].map(|i| {
            let cima = a[i] + (b[i] - a[i]) * tx;
            let baixo = c[i] + (d[i] - c[i]) * tx;
            cima + (baixo - cima) * ty
        })
    }

    /// A radiância pela direcção `d`, lida bilinear.
    #[must_use]
    pub fn radiancia(&self, d: [f32; 3]) -> Rgb {
        let (px, py) = self.ponto(d);
        self.bilinear(px, py)
    }

    /// O ângulo sólido do texel da linha `y`.
    #[must_use]
    pub fn angulo_solido(&self, y: u32) -> f64 {
        let pi = std::f64::consts::PI;
        let h = f64::from(self.altura);
        let (t0, t1) = (pi * f64::from(y) / h, pi * f64::from(y + 1) / h);
        2.0 * pi / f64::from(self.largura) * (t0.cos() - t1.cos())
    }

    /// A direcção do centro do texel `(x, y)`.
    #[must_use]
    pub fn direcao(&self, x: u32, y: u32) -> [f32; 3] {
        let phi = ((f64::from(x) + 0.5) / f64::from(self.largura) - 0.5) * std::f64::consts::TAU;
        let theta = (f64::from(y) + 0.5) / f64::from(self.altura) * std::f64::consts::PI;
        [
            (theta.sin() * phi.cos()) as f32,
            theta.cos() as f32,
            (theta.sin() * phi.sin()) as f32,
        ]
    }

    /// ⭐ **A radiância MÉDIA sobre a esfera** — que é também a irradiância normalizada média sobre
    /// todas as normais. É o que o chamador usa para pôr dois céus à mesma luz.
    #[must_use]
    pub fn media(&self) -> Rgb {
        let mut s = [0.0f64; 3];
        for y in 0..self.altura {
            let w = self.angulo_solido(y);
            for x in 0..self.largura {
                let c = self.rgb[(y * self.largura + x) as usize];
                for i in 0..3 {
                    s[i] += f64::from(c[i]) * w;
                }
            }
        }
        s.map(|v| (v / (4.0 * std::f64::consts::PI)) as f32)
    }
}

/// ⭐ **Direcção → o quadrado octaédrico `[-1, 1]²`**, com o polo `+y` no centro. ⚠️ Invariante à
/// escala: a direcção não precisa de vir normalizada. `sinal(0) = +1`, aqui e no WGSL.
#[must_use]
pub fn oct(d: [f32; 3]) -> (f32, f32) {
    let s = d[0].abs() + d[1].abs() + d[2].abs();
    let s = if s > 0.0 { s } else { 1.0 };
    let (a, b) = (d[0] / s, d[2] / s);
    if d[1] < 0.0 {
        let sa = if a >= 0.0 { 1.0 } else { -1.0 };
        let sb = if b >= 0.0 { 1.0 } else { -1.0 };
        ((1.0 - b.abs()) * sa, (1.0 - a.abs()) * sb)
    } else {
        (a, b)
    }
}

/// O inverso do [`oct`] — e, fora do quadrado, a DOBRA: o ponto `(1 + e, t)` é `(1 − e, −t)`.
#[must_use]
pub fn de_oct(a: f32, b: f32) -> [f32; 3] {
    let (mut a, mut b) = (a, b);
    if a.abs() > 1.0 {
        a = a.signum() * 2.0 - a;
        b = -b;
    }
    if b.abs() > 1.0 {
        b = b.signum() * 2.0 - b;
        a = -a;
    }
    let y = 1.0 - a.abs() - b.abs();
    let (x, z) = if y < 0.0 {
        let sa = if a >= 0.0 { 1.0 } else { -1.0 };
        let sb = if b >= 0.0 { 1.0 } else { -1.0 };
        ((1.0 - b.abs()) * sa, (1.0 - a.abs()) * sb)
    } else {
        (a, b)
    };
    let l = (x * x + y * y + z * z).sqrt();
    [x / l, y / l, z / l]
}

/// ⭐⭐ **O CÉU PRÉ-FILTRADO** — o atlas que sobe para a placa e que a CPU lê do mesmo modo.
///
/// ⚠️ **Em MEIA precisão** (`Rgba16Float`: metade da memória, o formato de mapa de ambiente de todo
/// celular) — e a CPU lê os MESMOS meios-floats, logo a paridade não perde nada no arredondamento.
/// Os céus embarcados vêm presos a `32 000`, dentro do `65 504` do `f16`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ceu {
    /// `ATLAS_W × ATLAS_H` texels RGBA (o `a` é `1`), linha a linha.
    atlas: Vec<[half::f16; 4]>,
    media: Rgb,
    /// O sol tirado do panorama ([`Ceu::com_sol`]); o atlas é então o do céu SEM ele.
    sol: Option<Sol>,
}

impl Ceu {
    /// ⭐ Pré-filtra o panorama — trabalho de CPU, em paralelo; o chamador corre-o fora do quadro.
    #[must_use]
    pub fn novo(p: &Panorama) -> Self {
        Self {
            atlas: prefiltro::atlas(p)
                .into_iter()
                .map(|t| t.map(half::f16::from_f32))
                .collect(),
            media: p.media(),
            sol: None,
        }
    }

    /// ⭐⭐ **O céu com o sol À PARTE** — o atlas é o do panorama sem o disco ([`Panorama::separa_sol`])
    /// e o sol vem em [`Ceu::sol`]: `atlas + sol` é o panorama inteiro. A média é a do panorama INTEIRO.
    #[must_use]
    pub fn com_sol(p: &Panorama) -> Self {
        let (sem, sol) = p.separa_sol();
        Self {
            sol,
            media: p.media(),
            ..Self::novo(&sem)
        }
    }

    /// O sol, se este céu foi montado com ele à parte.
    #[must_use]
    pub fn sol(&self) -> Option<&Sol> {
        self.sol.as_ref()
    }

    /// Os texels do atlas, `ATLAS_W × ATLAS_H` RGBA `f16` — o que sobe para a placa.
    #[must_use]
    pub fn atlas(&self) -> &[[half::f16; 4]] {
        &self.atlas
    }

    /// A radiância média do panorama (ver [`Panorama::media`]).
    #[must_use]
    pub fn media(&self) -> Rgb {
        self.media
    }

    fn ler(&self, x: u32, y: u32) -> Rgb {
        let t = self.atlas[(y * ATLAS_W + x) as usize];
        [t[0].to_f32(), t[1].to_f32(), t[2].to_f32()]
    }

    /// A leitura bilinear do nível `k`, no ponto octaédrico `(a, b)`.
    /// ⚠️ A MESMA ordem de contas do `sky_bilinear` do [`wgsl`].
    fn bilinear(&self, k: usize, (a, b): (f32, f32)) -> Rgb {
        let n = lado_com_margem(k) - 2;
        let nf = n as f32;
        let fx = ((a * 0.5 + 0.5) * nf - 0.5).clamp(-0.5, nf - 0.5);
        let fy = ((b * 0.5 + 0.5) * nf - 0.5).clamp(-0.5, nf - 0.5);
        let (x0, y0f) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0f);
        let ix = (x0 as i32 + 1) as u32 + X0[k];
        let iy = (y0f as i32 + 1) as u32 + Y0[k];
        let p00 = self.ler(ix, iy);
        let p10 = self.ler(ix + 1, iy);
        let p01 = self.ler(ix, iy + 1);
        let p11 = self.ler(ix + 1, iy + 1);
        [0, 1, 2].map(|i| {
            let cima = p00[i] + (p10[i] - p00[i]) * tx;
            let baixo = p01[i] + (p11[i] - p01[i]) * tx;
            cima + (baixo - cima) * ty
        })
    }

    /// ⭐ **A radiância pré-filtrada** pela direcção `dir` (no referencial do céu), para o lóbulo GGX
    /// de `α`: linear em `√α` entre os dois níveis vizinhos.
    #[must_use]
    pub fn radiance(&self, dir: [f32; 3], alpha: f32) -> Rgb {
        let r = alpha.clamp(0.0, 1.0).sqrt() * (NIVEIS - 1) as f32;
        let k0 = (r as usize).min(NIVEIS - 2);
        let t = r - k0 as f32;
        let ab = oct(dir);
        let a = self.bilinear(k0, ab);
        let b = self.bilinear(k0 + 1, ab);
        [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
    }

    /// ⭐ **A irradiância normalizada** `E(n)/π` (no referencial do céu) — o ÚLTIMO nível.
    ///
    /// ⭐⭐ **Não é um atalho, é a mesma integral:** com `α = 1` a NDF do GGX é constante (`1/π`), logo
    /// o núcleo do pré-filtro `(R·l)·D(h)` vira `(R·l)/π` — a média pesada pelo cosseno, que é
    /// `E(n)/π`. Foi a prova de mutação que o mostrou (02/10): trocar a irradiância pela radiância a
    /// `α = 1` deixava o gate verde ao byte. O mapa próprio que aqui esteve era redundante e menos
    /// fiel (máximo `7,2 %` contra a soma crua; o nível exacto dá `≤ 2,6 %`).
    #[must_use]
    pub fn irradiance(&self, n: [f32; 3]) -> Rgb {
        self.bilinear(NIVEIS - 1, oct(n))
    }
}

/// O mundo → o referencial do céu girado `giro = (cos θ, sin θ)` em torno de `+y`.
/// ⚠️ A mesma conta do `sky_gira` do [`wgsl`].
#[must_use]
pub fn gira(d: [f32; 3], giro: [f32; 2]) -> [f32; 3] {
    let [c, s] = giro;
    [c * d[0] - s * d[2], d[1], s * d[0] + c * d[2]]
}

/// ⭐ **O céu como o material o vê** — girado e com a sua força (um fator linear).
#[derive(Clone, Copy, Debug)]
pub struct Orientado<'a> {
    pub ceu: &'a Ceu,
    /// `(cos θ, sin θ)` do giro em torno de `+y`.
    pub giro: [f32; 2],
    pub forca: f32,
    /// O peso do [`Ceu::sol`] (`1` = o panorama inteiro; `0` = o céu sem o disco).
    pub sol: f32,
}

impl ph2d_material::Environment for Orientado<'_> {
    fn radiance(&self, dir: [f32; 3], alpha: f32) -> Rgb {
        let d = gira(dir, self.giro);
        let c = self.ceu.radiance(d, alpha);
        let s = self.ceu.sol().map_or([0.0; 3], |s| s.radiance(d, alpha));
        [0, 1, 2].map(|i| (c[i] + s[i] * self.sol) * self.forca)
    }
    fn irradiance(&self, n: [f32; 3]) -> Rgb {
        let d = gira(n, self.giro);
        let c = self.ceu.irradiance(d);
        let s = self.ceu.sol().map_or([0.0; 3], |s| s.irradiance(d));
        [0, 1, 2].map(|i| (c[i] + s[i] * self.sol) * self.forca)
    }
}

#[cfg(test)]
mod tests;
