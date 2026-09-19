//! **O DOMÍNIO de uma distribuição** — a caixa do plano em que um gerador de `motion.*` põe os
//! seus pontos, e a régua com que ele mede *quão fundo* cada ponto está nela.
//!
//! ## ⛔⛔ O que esta crate JÁ FOI, e porque encolheu
//!
//! Ela nasceu (doc 89, folha 01) como a **FORMA** do domínio: um param `Shape` com
//! `Rect`/`Circle`/`Ring` mais um `Hole`, partilhado por quatro nós — `motion.grid`,
//! `motion.lattice`, `motion.scatter` e `motion.distribute_poisson`. Duas leis viviam aqui, e a
//! distinção entre elas está registada no §5.0 do roteador: *um RETICULADO recorta (a contagem
//! cai) e um AMOSTRADOR redistribui (a contagem sobrevive)*.
//!
//! **Ordem do dono, 2026-09-19: *«tire de todos»*.** Os dois params saíram dos quatro cartões, e
//! com eles saiu tudo o que só a forma não-retangular alcançava — o corte (`carve`), o círculo, o
//! anel, o buraco, os rótulos e a trigonometria própria que o sorteio radial exigia.
//!
//! ⚠️ **O que FICA é o que os dois amostradores continuam a precisar**, e não um resto: a caixa
//! (`half_extents`, que a grelha de vizinhança do `motion.scatter` indexa), a régua
//! ([`Region::radial`]) e a **densidade graduada** ([`Region::density`]), que é o `Density
//! Falloff` do `motion.scatter` e do `motion.distribute_poisson` — uma feature que o dono nunca
//! pediu para tirar e que a fileira do meio da cena `=93` demonstra.
//!
//! ⚠️ **Apagar a crate inteira teria levado essa feature com ela**, que é o erro que a §5.0 chama
//! de *«tirar o que estava de graça sem ninguém reconferir»*. O corte é pela forma, não pelo
//! ficheiro.
//!
//! ## A régua
//!
//! [`Region::radial`] normaliza a distância à borda: **`0` no centro, `1` na fronteira**. É dela
//! que a profundidade e a densidade se derivam.
//!
//! ⚠️ **A métrica é a de CHEBYSHEV normalizada por eixo** — a caixa é o conjunto
//! `max(|x|/hw, |y|/hh) ≤ 1` —, a mesma que o `motion.falloff` usa no `shape = Rect`. Ela era um
//! dos três ramos e é agora o único; o valor que ela devolve **não mudou um bit**.

#![forbid(unsafe_code)]

/// O piso da densidade graduada — e ele nomeia o recurso: no
/// `motion.distribute_poisson` a densidade vira o **raio** (`r = radius/d`), logo
/// um `d` que chegue a zero pede um raio infinito, e o número de células a varrer
/// cresce com o quadrado dele. A `0,2` o raio máximo é `5×` o mínimo e a varredura
/// cabe numa vizinhança fixa; abaixo disso o custo deixa de ser limitado.
pub const MIN_DENSITY: f32 = 0.2;

/// **A REGIÃO em que uma distribuição põe os seus pontos** — o rectângulo centrado na origem, como
/// todo gerador desta família.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Region {
    /// Meia-largura e meia-altura.
    hw: f32,
    hh: f32,
}

impl Region {
    /// Constrói a região a partir da extensão crua do nó.
    ///
    /// ⚠️ **Totalizado na entrada**, como o `param_as_count` do `nodegraph`: uma extensão negativa
    /// ou `NaN` vem de um param dirigido por fio, e o único comportamento honesto é a caixa
    /// degenerada — nunca um pânico e nunca uma região vazia em silêncio.
    #[must_use]
    pub fn rect(w: f32, h: f32) -> Self {
        let half = |v: f32| if v.is_finite() { v.abs() * 0.5 } else { 0.0 };
        Self {
            hw: half(w),
            hh: half(h),
        }
    }

    /// **AS MEIAS-EXTENSÕES da caixa.**
    ///
    /// Existe para quem precisa de INDEXAR o espaço da região (a grelha de vizinhança do
    /// `motion.scatter`): sem isto o consumidor teria de reconstruir a caixa a partir dos params,
    /// que é a segunda cópia da lei que este tipo existe para ter numa só.
    #[must_use]
    pub fn half_extents(&self) -> [f32; 2] {
        [self.hw, self.hh]
    }

    /// **A régua: a distância à fronteira, normalizada.** `0` no centro, `1` na borda, `> 1` fora
    /// — a distância de **Chebyshev** normalizada por eixo.
    ///
    /// ⚠️ **Um eixo de extensão zero é «tudo está na borda»**, não uma divisão por zero: uma caixa
    /// achatada não tem interior, e devolver `∞` faria a densidade apagar tudo em vez de deixar a
    /// linha.
    #[must_use]
    pub fn radial(&self, p: [f32; 2]) -> f32 {
        let ax = if self.hw > 0.0 {
            (p[0] / self.hw).abs()
        } else {
            1.0
        };
        let ay = if self.hh > 0.0 {
            (p[1] / self.hh).abs()
        } else {
            1.0
        };
        ax.max(ay)
    }

    /// **`p` cai dentro da região?**
    #[must_use]
    pub fn contains(&self, p: [f32; 2]) -> bool {
        self.radial(p) <= 1.0
    }

    /// **Quão FUNDO `p` está** — `1` no coração da região, `0` na fronteira, e negativo fora. É a
    /// variável da densidade graduada.
    #[must_use]
    pub fn depth(&self, p: [f32; 2]) -> f32 {
        1.0 - self.radial(p)
    }

    /// **A DENSIDADE em `p`** — `1` em toda a parte quando `falloff` é `0`, e graduada do coração
    /// para a fronteira quando não é.
    ///
    /// `falloff = 1` leva a densidade ao piso [`MIN_DENSITY`] na borda; valores intermédios
    /// interpolam. ⚠️ **O piso não é conforto: é o que torna o custo do amostrador adaptativo
    /// limitado** (ver [`MIN_DENSITY`]).
    ///
    /// ⚠️ **`falloff = 0` devolve `1,0` por RAMO**, e não pela aritmética: um `1 + 0·(x − 1)` não
    /// é `1` em `f32` para todo `x`, e o default deste knob tem de reduzir ao nó que shipava **ao
    /// bit**, não a um ULP dele.
    #[must_use]
    pub fn density(&self, p: [f32; 2], falloff: f32) -> f32 {
        // `is_nan() ||` e não uma comparação negada: sobre um tipo parcialmente
        // ordenado a negação esconde o caso `NaN`, e aqui ele existe (um `falloff`
        // dirigido por fio). A guarda é a mesma, escrita de forma legível — o mesmo
        // molde do `history_samples` do `motion.emitter`.
        if falloff.is_nan() || falloff <= 0.0 {
            return 1.0;
        }
        let k = falloff.min(1.0);
        let d = self.depth(p).clamp(0.0, 1.0);
        // No coração vale 1; na fronteira vale `1 − k·(1 − MIN_DENSITY)`.
        let floor = 1.0 - k * (1.0 - MIN_DENSITY);
        (floor + (1.0 - floor) * d).clamp(MIN_DENSITY, 1.0)
    }

    /// **O SORTEIO uniforme por ÁREA** — a pergunta do AMOSTRADOR. `u` e `v` são dois números
    /// independentes em `[0,1)`.
    ///
    /// ⚠️ **É a expressão literal que os nós tinham escrita à mão** — `(u − ½)·w` — para o
    /// caminho de sempre não mover um bit.
    #[must_use]
    pub fn sample(&self, u: f32, v: f32) -> [f32; 2] {
        [(u - 0.5) * self.hw * 2.0, (v - 0.5) * self.hh * 2.0]
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
