//! **O dither ordenado** (Bayer 1973): a matriz `n × n` (n = 2, 4, 8) presa à grelha do CANVAS — a
//! célula `(x, y)` lê o degrau `M[y mod n][x mod n]`. A matriz é aritmética, pela recorrência
//! publicada `M₂ₙ = [[4M, 4M + 2], [4M + 3, 4M + 1]]` a partir de `M₁ = [0]`.
//! Medido no Trial 1.3.18.6 (o Gradient da dinâmica, Bayer 4×4): as células acendem na ordem
//! (0,0) → (2,2) → (2,0) → (0,2) → (1,1) → (3,3) — `oracle_trial_dynamics_velocity.json`.

/// As matrizes do Aseprite (o dropdown do Gradient, na ordem dele).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dither {
    #[default]
    None,
    Bayer2,
    Bayer4,
    Bayer8,
}

impl Dither {
    pub const ALL: [Self; 4] = [Self::None, Self::Bayer2, Self::Bayer4, Self::Bayer8];

    /// O lado da matriz (`None` = 1: sem dither).
    #[must_use]
    pub const fn side(self) -> u32 {
        match self {
            Self::None => 1,
            Self::Bayer2 => 2,
            Self::Bayer4 => 4,
            Self::Bayer8 => 8,
        }
    }

    /// O degrau da célula `(x, y)` do canvas, em `0..side²`.
    #[must_use]
    pub fn level(self, x: i32, y: i32) -> u32 {
        let n = self.side() as i32;
        bayer(
            n.trailing_zeros(),
            x.rem_euclid(n) as u32,
            y.rem_euclid(n) as u32,
        )
    }

    /// A célula `(x, y)` mostra a cor de destino com a fracção `t ∈ [0, 1]` do caminho andada: as
    /// `⌊t · side²⌋` células de degrau mais baixo da matriz.
    #[must_use]
    pub fn reaches(self, x: i32, y: i32, t: f64) -> bool {
        self.level(x, y) < self.lit(t)
    }

    /// Quantos degraus da matriz a fracção `t` acende: `⌊t · side²⌋`.
    #[must_use]
    pub fn lit(self, t: f64) -> u32 {
        let n2 = self.side() * self.side();
        (t.clamp(0.0, 1.0) * f64::from(n2)).floor() as u32
    }
}

/// `M` de lado `2^k` em `(i, j)` pela recorrência: os bits de `i` e `j`, do mais baixo ao mais alto,
/// dão os dígitos de base 4 do degrau, do mais alto ao mais baixo.
fn bayer(k: u32, i: u32, j: u32) -> u32 {
    (0..k).fold(0, |acc, b| {
        let (x, y) = ((i >> b) & 1, (j >> b) & 1);
        let digit = match (x, y) {
            (0, 0) => 0,
            (1, 1) => 1,
            (1, 0) => 2,
            _ => 3,
        };
        acc + digit * 4u32.pow(k - 1 - b)
    })
}
