/// O gerador do rough.js (`math.js`, Park–Miller com `Math.imul`): `seed ← 48271·seed` em 32 bits,
/// `next = (seed & 0x7fffffff) / 2³¹`.
///
/// Diferença única e deliberada: no rough.js a semente `0` cai no `Math.random` (traço diferente a
/// cada desenho). Aqui a `0` vale `1` — um elemento do quadro desenha sempre igual.
#[derive(Clone, Debug)]
pub struct Random {
    seed: i32,
}

impl Random {
    pub fn new(seed: u32) -> Self {
        Self {
            seed: if seed == 0 { 1 } else { seed as i32 },
        }
    }

    pub fn next(&mut self) -> f64 {
        self.seed = 48271i32.wrapping_mul(self.seed);
        f64::from(0x7fff_ffff & self.seed) / 2_147_483_648.0
    }
}
