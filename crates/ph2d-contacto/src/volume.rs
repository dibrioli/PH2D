//! ⭐ **A peça como um volume de distância** — o campo amostrado numa grelha regular, lido com
//! interpolação trilinear. É onde os raios da [`crate::Grade`] marcham.

/// Um campo de distância amostrado em `n³` pontos, de `lo` a `hi` (os cantos são amostras).
#[derive(Clone, Debug)]
pub struct Volume {
    pub lo: [f32; 3],
    pub hi: [f32; 3],
    pub n: usize,
    /// `x` mais depressa, depois `y`, depois `z`.
    pub d: Vec<f32>,
}

impl Volume {
    /// Amostra `campo` (avalia um lote de pontos de uma vez) na caixa `lo..hi`.
    pub fn de(
        lo: [f32; 3],
        hi: [f32; 3],
        n: usize,
        campo: impl FnOnce(&[[f32; 3]]) -> Vec<f32>,
    ) -> Self {
        let n = n.max(2);
        let mut pts = Vec::with_capacity(n * n * n);
        for k in 0..n {
            for j in 0..n {
                for i in 0..n {
                    let t = [i, j, k].map(|c| c as f32 / (n - 1) as f32);
                    pts.push([0, 1, 2].map(|e| lo[e] + (hi[e] - lo[e]) * t[e]));
                }
            }
        }
        let d = campo(&pts);
        assert_eq!(d.len(), pts.len(), "o campo devolve um valor por ponto");
        Self { lo, hi, n, d }
    }

    /// A aresta de um passo da grelha (a maior das três).
    #[must_use]
    pub fn passo(&self) -> f32 {
        (0..3)
            .map(|e| (self.hi[e] - self.lo[e]) / (self.n - 1) as f32)
            .fold(0.0, f32::max)
    }

    /// A distância em `p`, trilinear. Fora da caixa: um limite INFERIOR da distância à peça (a peça
    /// está dentro da caixa) — o maior entre a distância à caixa e o valor da borda menos o que falta.
    #[must_use]
    pub fn distancia(&self, p: [f32; 3]) -> f32 {
        let q = [0, 1, 2].map(|e| p[e].clamp(self.lo[e], self.hi[e]));
        let fora = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
        let dq = self.trilinear(q);
        if fora > 0.0 { fora.max(dq - fora) } else { dq }
    }

    /// O gradiente (diferenças centrais de meio passo), normalizado.
    #[must_use]
    pub fn normal(&self, p: [f32; 3]) -> [f32; 3] {
        let h = 0.5 * self.passo();
        let mut g = [0.0f32; 3];
        for (e, ge) in g.iter_mut().enumerate() {
            let (mut a, mut b) = (p, p);
            a[e] += h;
            b[e] -= h;
            *ge = self.distancia(a) - self.distancia(b);
        }
        let l = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        if l > 1.0e-12 {
            g.map(|c| c / l)
        } else {
            [0.0, 1.0, 0.0]
        }
    }

    fn trilinear(&self, p: [f32; 3]) -> f32 {
        let n = self.n;
        let mut i0 = [0usize; 3];
        let mut f = [0.0f32; 3];
        for e in 0..3 {
            let u = (p[e] - self.lo[e]) / (self.hi[e] - self.lo[e]) * (n - 1) as f32;
            let u = u.clamp(0.0, (n - 1) as f32);
            let i = (u.floor() as usize).min(n - 2);
            i0[e] = i;
            f[e] = u - i as f32;
        }
        let at = |i: usize, j: usize, k: usize| self.d[(k * n + j) * n + i];
        let mut s = 0.0;
        for dz in 0..2 {
            for dy in 0..2 {
                for dx in 0..2 {
                    let w = (if dx == 1 { f[0] } else { 1.0 - f[0] })
                        * (if dy == 1 { f[1] } else { 1.0 - f[1] })
                        * (if dz == 1 { f[2] } else { 1.0 - f[2] });
                    s += w * at(i0[0] + dx, i0[1] + dy, i0[2] + dz);
                }
            }
        }
        s
    }
}
