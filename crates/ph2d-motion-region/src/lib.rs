//! **O DOMÍNIO de uma distribuição** — a região do plano em que um gerador de
//! `motion.*` põe os seus pontos (doc 89, folha 01: as células de `form`/`fill`
//! do `motion.grid`, de `form` do `motion.lattice` e do domínio circular/anel do
//! `motion.scatter`).
//!
//! Quatro nós faziam a MESMA pergunta — *"onde é que isto cabe?"* — e nenhum a
//! sabia fazer: os quatro só sabiam **retângulo**. A referência resolve-o com um
//! dropdown (C4D *Grid Array* `form`, Niagara *Sphere Location*, VFXG
//! `Position (Circle/Torus)`), e a nossa composição resolvia-o com três nós
//! (`falloff → cull`) que **cortam**. Aqui a região é um valor, e ela vive numa
//! porta só.
//!
//! ## ⚠️ Duas leis, e é por isso que esta crate expõe DUAS perguntas
//!
//! **Um RETICULADO recorta.** `motion.grid` e `motion.lattice` põem os pontos numa
//! rede: a rede não se dobra para caber num círculo, então uma forma não-retangular
//! só pode **remover** o que fica de fora — e a contagem cai. É [`Region::contains`].
//!
//! **Um AMOSTRADOR redistribui.** `motion.scatter` e `motion.distribute_poisson`
//! atiram dardos: mudar a região muda **onde o dardo cai**, e a contagem (ou o raio
//! mínimo) sobrevive intacta. É [`Region::sample`].
//!
//! ⚠️ **A cadeia `falloff → cull` só sabe fazer a primeira**, e por isso ela nunca
//! foi resposta para o amostrador: pedir 400 pontos num círculo e receber 314 é
//! outra coisa. É a mesma distinção que o `motion.distribute_radial` pagou quando
//! ganhou `start_angle`/`end_angle` — *"e ele EMPACOTA, não recorta"*.
//!
//! ## ⚠️ Uma CASCA é um ANEL — e é por isso que não há um param `fill`
//!
//! O C4D tem `form` **e** `fill` (*Solid* / *Shell*), e a célula do `motion.grid`
//! pedia os dois. Um `fill = Shell` é *"fica só a camada de fora"*, que é
//! exactamente um **anel com o buraco grande**: [`SHAPE_RING`] com `inner` a subir
//! encolhe a banda até à casca, e `inner` intermédios dão espessuras que o par
//! *Solid/Shell* não sabe exprimir. **O superset ganha a dedup** — dois params
//! seriam duas portas para o mesmo número, e um deles teria de decidir quem manda
//! quando discordassem.
//!
//! ## A régua
//!
//! [`Region::radial`] normaliza a distância à borda: **`0` no centro, `1` na
//! fronteira**, para as três formas. É a partir dela que tudo o resto se deriva —
//! o corte, o sorteio e a **densidade graduada** ([`Region::depth`]).
//!
//! ⚠️ **`Rect` é o de sempre, ao bit.** O sorteio retangular é literalmente
//! `(u − ½)·w`, `(v − ½)·h` — a mesma expressão que o `motion.scatter` tinha
//! escrita à mão —, e `contains` de um `Rect` cujo `inner` é `0` aceita a rede
//! inteira. Um documento que nunca ouviu falar de `shape` coze byte a byte o que
//! cozia.

#![forbid(unsafe_code)]

/// A chave do param **da forma do domínio**.
pub const SHAPE: &str = "shape";
/// A chave do param **do buraco do anel**, em fração do raio.
pub const INNER: &str = "inner";

/// Retângulo — o domínio de sempre, e o default de todo nó que adopta esta lei.
pub const SHAPE_RECT: i32 = 0;
/// Disco (elipse, quando `width != height`).
pub const SHAPE_CIRCLE: i32 = 1;
/// Anel: o disco menos um buraco concêntrico de `inner` do raio.
pub const SHAPE_RING: i32 = 2;

/// Os rótulos de [`SHAPE`], na ordem em que o número os indexa.
///
/// ⚠️ **Apendar é a única operação legal aqui** — um documento autorado guarda o
/// NÚMERO, não o nome, e reordenar mudaria a forma de toda cena já salva.
pub const SHAPE_LABELS: &[&str] = &["Rect", "Circle", "Ring"];

/// O buraco mais fino que um anel pode ter sem a banda deixar de ter área. Acima
/// disto o anel seria uma linha, e o sorteio por área dividiria por zero.
const MAX_INNER: f32 = 0.98;

/// O piso da densidade graduada — e ele nomeia o recurso: no
/// `motion.distribute_poisson` a densidade vira o **raio** (`r = radius/d`), logo
/// um `d` que chegue a zero pede um raio infinito, e o número de células a varrer
/// cresce com o quadrado dele. A `0,2` o raio máximo é `5×` o mínimo e a varredura
/// cabe numa vizinhança fixa; abaixo disso o custo deixa de ser limitado.
pub const MIN_DENSITY: f32 = 0.2;

/// **A REGIÃO em que uma distribuição põe os seus pontos** — centrada na origem,
/// como todo gerador desta família.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Region {
    shape: i32,
    /// Meia-largura e meia-altura. Um `Circle` num `w != h` é uma ELIPSE, e é o
    /// que a referência faz: a forma herda a caixa, não a substitui.
    hw: f32,
    hh: f32,
    /// O buraco, em fração do raio. Lido só pelo [`SHAPE_RING`].
    inner: f32,
}

impl Region {
    /// Constrói a região a partir dos params crus do nó.
    ///
    /// ⚠️ **Totalizado na entrada**, como o `param_as_count` do `nodegraph`: um
    /// `shape` fora da escada, um `inner` `NaN` ou uma extensão negativa vêm de um
    /// param dirigido por fio, e o único comportamento honesto é o retângulo de
    /// sempre — nunca um pânico e nunca uma região vazia em silêncio.
    #[must_use]
    pub fn of(shape: f32, w: f32, h: f32, inner: f32) -> Self {
        let shape = if shape.is_finite() {
            match shape.round() as i32 {
                SHAPE_CIRCLE => SHAPE_CIRCLE,
                SHAPE_RING => SHAPE_RING,
                _ => SHAPE_RECT,
            }
        } else {
            SHAPE_RECT
        };
        let half = |v: f32| if v.is_finite() { v.abs() * 0.5 } else { 0.0 };
        let inner = if inner.is_finite() {
            inner.clamp(0.0, MAX_INNER)
        } else {
            0.0
        };
        Self {
            shape,
            hw: half(w),
            hh: half(h),
            inner,
        }
    }

    /// O retângulo de sempre — o caminho que **não** paga nada por esta lei existir.
    #[must_use]
    pub fn is_rect(&self) -> bool {
        self.shape == SHAPE_RECT
    }

    /// **A régua: a distância à fronteira, normalizada.** `0` no centro, `1` na
    /// borda, `> 1` fora.
    ///
    /// - `Rect` — distância de **Chebyshev** normalizada por eixo (a caixa é o
    ///   conjunto `max(|x|/hw, |y|/hh) ≤ 1`, e é a mesma métrica que o
    ///   `motion.falloff` usa no `shape = Rect`).
    /// - `Circle`/`Ring` — o raio **elíptico** `hypot(x/hw, y/hh)`.
    ///
    /// ⚠️ **Um eixo de extensão zero é «tudo está na borda»**, não uma divisão por
    /// zero: uma caixa achatada não tem interior, e devolver `∞` faria o corte
    /// apagar tudo em vez de deixar a linha.
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
        if self.shape == SHAPE_RECT {
            ax.max(ay)
        } else {
            (ax * ax + ay * ay).sqrt()
        }
    }

    /// **`p` cai dentro da região?** — a pergunta do RETICULADO.
    #[must_use]
    pub fn contains(&self, p: [f32; 2]) -> bool {
        let r = self.radial(p);
        if self.shape == SHAPE_RING {
            r >= self.inner && r <= 1.0
        } else {
            r <= 1.0
        }
    }

    /// **Quão FUNDO `p` está** — `1` no coração da região, `0` na fronteira, e
    /// negativo fora. É a variável da densidade graduada.
    ///
    /// ⚠️ **Num anel o coração é o MEIO DA BANDA, não o centro do disco** — que é
    /// um buraco. Por isso o anel tem duas fronteiras e a profundidade sobe e desce
    /// entre elas, enquanto o disco e a caixa têm uma só.
    #[must_use]
    pub fn depth(&self, p: [f32; 2]) -> f32 {
        let r = self.radial(p);
        if self.shape == SHAPE_RING {
            let band = (1.0 - self.inner).max(f32::EPSILON);
            let t = (r - self.inner) / band;
            // `1 − |2t − 1|`: `0` nas duas bordas, `1` no meio, negativo fora.
            1.0 - (2.0 * t - 1.0).abs()
        } else {
            1.0 - r
        }
    }

    /// **A DENSIDADE em `p`** — `1` em toda a parte quando `falloff` é `0`, e
    /// graduada do coração para a fronteira quando não é.
    ///
    /// `falloff = 1` leva a densidade ao piso [`MIN_DENSITY`] na borda; valores
    /// intermédios interpolam. ⚠️ **O piso não é conforto: é o que torna o custo do
    /// amostrador adaptativo limitado** (ver [`MIN_DENSITY`]).
    ///
    /// ⚠️ **`falloff = 0` devolve `1,0` por RAMO**, e não pela aritmética: um
    /// `1 + 0·(x − 1)` não é `1` em `f32` para todo `x`, e o default deste knob tem
    /// de reduzir ao nó que shipava **ao bit**, não a um ULP dele.
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

    /// **O SORTEIO uniforme por ÁREA** — a pergunta do AMOSTRADOR. `u` e `v` são
    /// dois números independentes em `[0,1)`.
    ///
    /// ⚠️ **Uniforme por ÁREA e não pelo raio.** Um `r = u·R` amontoa os pontos no
    /// centro (um anel fino a raio `ρ` tem área `∝ ρ`), e `r = R·√u` é a correção
    /// aceite. No anel a mesma conta corre entre os dois raios: `√(i² + u(1 − i²))`.
    ///
    /// ⚠️ **`Rect` é a expressão literal que os nós tinham escrita à mão** —
    /// `(u − ½)·w` — para o default não mover um bit.
    #[must_use]
    pub fn sample(&self, u: f32, v: f32) -> [f32; 2] {
        if self.shape == SHAPE_RECT {
            return [(u - 0.5) * self.hw * 2.0, (v - 0.5) * self.hh * 2.0];
        }
        let i = if self.shape == SHAPE_RING {
            self.inner
        } else {
            0.0
        };
        let r = (i * i + u.clamp(0.0, 1.0) * (1.0 - i * i)).max(0.0).sqrt();
        let (c, s) = unit_cycles(v);
        [r * self.hw * c, r * self.hh * s]
    }
}

/// **O CORTE de um reticulado** — fica quem cai dentro de `region`.
///
/// A outra metade da lei (a do amostrador é [`Region::sample`]). Um `motion.grid` ou um
/// `motion.lattice` não se dobra para caber num círculo: a rede é a rede, e a forma só
/// pode remover — **a contagem cai, e é suposto cair**.
///
/// ⚠️ **`Rect` devolve o vector INTACTO, por ramo — e o ramo compra CUSTO, não bits.**
///
/// A primeira versão desta nota dizia que ele era load-bearing para a identidade, e uma
/// **mutação sobreviveu** a apagá-lo: a varredura incondicional dá exactamente o mesmo
/// conjunto. A afirmação encolheu até ao que a máquina de facto faz — o ramo poupa uma
/// varredura e uma realocação no caminho que todo documento de hoje percorre.
///
/// ⚠️ **E o que o torna SEGURO está medido, não assumido:** um ponto da rede podia cair
/// fora da caixa construída da própria extensão, por arredondamento — as duas contas
/// são `((n−1) − (n−1)/2)·g` e `((n−1)·g)/2`, árvores diferentes do mesmo produto real.
/// O gate `no_lattice_point_ever_falls_outside_its_own_box` varre 22 800 pares
/// `(colunas, passo)` e não acha nenhum; se alguém mudar a expressão da extensão, ele
/// fica vermelho **antes** de o corte começar a comer a coluna de fora.
#[must_use]
pub fn carve(points: Vec<[f32; 2]>, region: &Region) -> Vec<[f32; 2]> {
    if region.is_rect() {
        return points;
    }
    points.into_iter().filter(|p| region.contains(*p)).collect()
}

/// Seno parabólico corrigido (Capens/devmaster) em **ciclos**, `[-1, 1]`.
///
/// A cópia da casa: `sin`/`cos` de biblioteca são transcendentais e não são
/// bit-reprodutíveis entre plataformas (HR-5), e o replay-hash corre numa matriz
/// de três sistemas. ⚠️ **Ela vive aqui e não numa crate de trigonometria** porque
/// é isso que as folhas de nó fazem (`motion.emitter/trig.rs` diz-se
/// *"self-contained per node crate"*): o que se partilha é a **LEI** que o artista
/// vê — a região —, não a primitiva aritmética.
fn sin_cycles(phase: f32) -> f32 {
    let f = phase - phase.floor();
    let p = if f < 0.5 {
        let u = f * 2.0;
        4.0 * u * (1.0 - u)
    } else {
        let u = (f - 0.5) * 2.0;
        -4.0 * u * (1.0 - u)
    };
    const Q: f32 = 0.225;
    Q * (p * p.abs() - p) + p
}

/// **A DIREÇÃO de `phase` ciclos — NORMALIZADA**, e a normalização é load-bearing.
///
/// ⚠️ **Um seno aproximado serve para um ÂNGULO e não serve para um RAIO.** O
/// `motion.emitter` usa o mesmo polinómio para apontar uma partícula, e `0,09%` de
/// erro num ângulo é invisível. Aqui o vector unitário **multiplica a extensão da
/// região**: `c² + s²` chega a `1,004`, e um dardo de raio `1` aterra `0,2%` fora
/// do disco — o próprio [`Region::contains`] recusaria um ponto que o
/// [`Region::sample`] acabou de produzir. *A mesma aproximação é boa numa pergunta
/// e errada na outra, e o que muda é o que ela multiplica.*
///
/// O `sqrt` do IEEE é exacto e determinista (HR-5), ao contrário do `sin` que ele
/// está a corrigir.
fn unit_cycles(phase: f32) -> (f32, f32) {
    let (c, s) = (sin_cycles(phase + 0.25), sin_cycles(phase));
    let m = (c * c + s * s).sqrt();
    if m > 0.0 { (c / m, s / m) } else { (1.0, 0.0) }
}

#[cfg(test)]
mod tests;
