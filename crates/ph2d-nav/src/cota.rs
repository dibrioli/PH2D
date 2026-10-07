//! ⭐ (W19, plano 30 §28.1–§28.2) **A COTA INFERIOR do custo de um caminho** — o heurístico de toda
//! procura ponderada, a saída cedo da fase geral, a poda dos atalhos e o «alvo à vista» leem-na daqui,
//! e de mais nenhum sítio.
//!
//! # A lei
//!
//! Seja `K` a união FECHADA dos polígonos que custam MENOS que `1` (as áreas baratas) e `w ≤ 1` o
//! menor custo da tabela; fora de `K` todo polígono custa `≥ 1`. Um caminho de `a` a `b` de
//! comprimento `L ≥ ℓ` (`ℓ` um mínimo geométrico: a recta, ou o heurístico de um intervalo):
//!
//! - que NÃO toca `K` custa `≥ L ≥ ℓ`;
//! - que toca anda FORA de `K` pelo menos `D = d(a, K) + d(K, b)` (a ida e a volta), e o resto a `≥ w`:
//!
//! ```text
//! custo ≥ L_fora + w · L_dentro = w · L + (1 − w) · L_fora ≥ w · ℓ + (1 − w) · min(ℓ, D)
//! ```
//!
//! que também cobre o 1.º caso (`D ≥ ℓ` ⇒ `ℓ`). É [`inferior`]. Vale para QUALQUER número de áreas
//! baratas (um caminho que passa por várias anda `D` fora de todas), e medir `D` até à CAIXA de cada
//! polígono (`≤` a distância ao polígono) só a torna mais pequena. Sem área barata na malha `D = ∞` e
//! a cota é `ℓ`, ao bit; com o chão comum abaixo de `1`, `D = 0` e é `w · ℓ` — a cota global da W7,
//! que punha UMA estrada num canto a pesar em TODAS as procuras (`24 176 → 85 744` nós por consulta).
//!
//! # Consistente, não só admissível
//!
//! `h(x) = inferior(w, |x b|, d(x, K) + d(K, b))` é 1-Lipschitz (mínimo e soma de distâncias): um troço
//! fora de `K` custa `≥` o que mede `≥` o que `h` muda. Dentro de `K`, `d(x, K) = 0` e
//! `|x b| ≥ d(K, b)`, logo `h(x) = w · |x b| + (1 − w) · d(K, b)` muda `w` por metro — e o troço custa
//! `≥ w` por metro.

use crate::cost::cost_of;
use crate::geom::{V2, dist};
use crate::mesh::NavMesh;

/// A cota inferior: `w · ℓ + (1 − w) · min(ℓ, d_fora)` (ver o cabeçalho). `d_fora ≥ ℓ` dá `ℓ` exacto.
#[inline]
pub(crate) fn inferior(w: f64, l: f64, d_fora: f64) -> f64 {
    if d_fora >= l {
        l
    } else {
        w * l + (1.0 - w) * d_fora
    }
}

/// As áreas baratas de UMA tabela de custos sobre UMA malha: o `w` e as caixas de `K`.
#[derive(Clone, Debug, Default)]
pub(crate) struct Cota {
    w: f64,
    /// O chão comum custa menos que `1`: `K` é quase a malha toda, `D = 0`.
    chao: bool,
    caixas: Vec<[V2; 2]>,
}

impl Cota {
    pub(crate) fn nova(mesh: &NavMesh, costs: &[f64], global: bool) -> Self {
        let mut c = Self::default();
        c.refaz(mesh, costs, global);
        c
    }

    /// Para outra consulta (os buffers ficam).
    pub(crate) fn refaz(&mut self, mesh: &NavMesh, costs: &[f64], global: bool) {
        self.w = costs.iter().copied().fold(1.0, f64::min);
        self.chao = global || cost_of(costs, 0) < 1.0;
        self.caixas.clear();
        if self.w < 1.0 && !self.chao {
            for (a, &c) in costs.iter().enumerate().skip(1) {
                if c < 1.0 {
                    self.caixas
                        .extend(mesh.caixas_da_area(a as u16).iter().map(|x| x.1));
                }
            }
        }
    }

    /// O menor custo da tabela (`≤ 1`).
    #[inline]
    pub(crate) fn w(&self) -> f64 {
        self.w
    }

    /// `d(p, K)` — `∞` sem área barata na malha.
    pub(crate) fn distancia(&self, p: V2) -> f64 {
        if self.chao {
            return 0.0;
        }
        let mut m2 = f64::INFINITY;
        for [lo, hi] in &self.caixas {
            let dx = (lo[0] - p[0]).max(p[0] - hi[0]).max(0.0);
            let dy = (lo[1] - p[1]).max(p[1] - hi[1]).max(0.0);
            m2 = m2.min(dx * dx + dy * dy);
        }
        m2.sqrt()
    }

    /// A cota de qualquer caminho de `a` a `b` (sem atalhos).
    pub(crate) fn entre(&self, a: V2, b: V2) -> f64 {
        inferior(self.w, dist(a, b), self.distancia(a) + self.distancia(b))
    }
}
