#![forbid(unsafe_code)]
//! **A lei de acumulação fractal** — fBm, e a costura que a faz fechar no tempo.
//!
//! ## O que esta folha É, e o que ela deliberadamente NÃO é
//!
//! Ela carrega a **LEI** (como as oitavas se somam) e **não** o ruído de base.
//! Isso não é escrúpulo de arquitetura: os consumidores usam ruídos
//! **diferentes de propósito** — o `motion.noise` usa ruído de **GRADIENTE**
//! (Perlin 2002, cujo doc explica por quê: o campo é exatamente zero em cada
//! ponto da grade, então os extremos caem ENTRE eles e não há padrão de grade) e
//! o `force.curl` usa ruído de **VALOR** (mais barato, e o potencial `ψ` do
//! curl noise do Bridson não precisa da diferença). Unificar o ruído de base
//! **mudaria a aparência** dos dois; unificar a lei não muda nada.
//!
//! ## Por que isto é uma crate
//!
//! A mesma lei tinha **duas implementações, e elas já divergiam**: o
//! `motion.noise` desloca o seed a cada oitava (*"para as oitavas serem
//! independentes, não cópias escaladas de um campo, que bateriam visivelmente"*)
//! e o `force.curl` não; um multiplica a coordenada, o outro a frequência; um
//! tem os três tipos, o outro só o fBm; um clampa o ganho, o outro crava
//! `amp *= 0.5` no laço. A família de forças ia ganhar a **terceira** cópia ao
//! herdar o cluster de params que a família de animadores já tinha.
//!
//! ⚠️ **O SEED fica FORA da lei, e é isso que a torna adotável sem mudar
//! desenho.** Quem decide se as oitavas se decorrelacionam é o ruído de base —
//! ele recebe o índice da oitava e faz o que já fazia. Uma lei que deslocasse o
//! seed sozinha mudaria o `force.curl`; uma que nunca o deslocasse mudaria o
//! `motion.noise`. Nenhuma das duas é a lei.
//!
//! Livre de transcendentais (HR-5): `floor`, multiplicação, soma e comparação.

/// A retificação por-oitava — o que se soma, antes de somar.
///
/// ⚠️ Aplicá-la **POR OITAVA** (e não à soma final) é o que dá à turbulência as
/// suas dobras e ao ridged as suas veias afiadas; à soma final ela seria só um
/// valor absoluto no fim, que é outra imagem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NoiseType {
    /// `Σ aᵢ·noise` — com sinal, bipolar `[-1, 1]`. O campo à deriva, o default.
    #[default]
    Fbm,
    /// `Σ aᵢ·|noise|` — retificado, unipolar `[0, 1]`. Dobras e bolhas afiadas
    /// (a "turbulence" original do Perlin: fumaça, mármore).
    Turbulence,
    /// `Σ aᵢ·(1−|noise|)²` — turbulência invertida, unipolar `[0, 1]`. Cristas
    /// afiadas (montanhas, veias).
    Ridged,
}

impl NoiseType {
    /// **A FAIXA NATURAL da soma** — o que este tipo produz ANTES de qualquer
    /// ganho, e o número que um controle de `Min`/`Max` precisa de saber.
    ///
    /// ⚠️ **É por causa dele que a aritmética óbvia do artista está ERRADA.** A
    /// conta que toda a gente faz — `amplitude = (max − min)/2` — assume que o
    /// campo é bipolar, e **dois dos três tipos não são**: `Turbulence` e `Ridged`
    /// retificam por oitava e saem em `[0, 1]`. Com essa conta, trocar o `type` de
    /// `Fbm` para `Ridged` **METADE a excursão e desloca o centro**, em silêncio,
    /// sem que nenhum número do painel tenha mudado. A folha 06 registou isso como
    /// *"armadilha real"* no irmão `motion.oscillator`, cujo `Spike` tem
    /// exactamente a mesma polaridade — e é a mesma cura nos dois sítios.
    ///
    /// Ver [`gain_offset_for_range`], que é a outra metade da lei.
    #[must_use]
    pub const fn natural_range(self) -> (f32, f32) {
        match self {
            // Σ aᵢ·noise — com sinal.
            Self::Fbm => (-1.0, 1.0),
            // Σ aᵢ·|noise| e Σ aᵢ·(1−|noise|)² — retificados.
            Self::Turbulence | Self::Ridged => (0.0, 1.0),
        }
    }

    /// O índice que um param `f32` carrega (o enum do painel guarda o índice).
    /// Um índice que não existe cai no `Fbm`, nunca num pânico.
    #[must_use]
    pub fn from_index(i: f32) -> Self {
        match i.round() as i32 {
            1 => Self::Turbulence,
            2 => Self::Ridged,
            _ => Self::Fbm,
        }
    }
}

/// **O GANHO E O DESLOCAMENTO que levam uma faixa NATURAL à faixa PEDIDA.**
///
/// Dado que a forma produz `[lo, hi]` e o artista quer `[min, max]`, a resposta é
/// a única afim que leva as duas pontas às duas pontas:
/// `gain = (max − min)/(hi − lo)` e `offset = min − lo·gain`.
///
/// ⚠️ **É UMA lei e não três**, e é isso que apaga o caso especial: para um campo
/// bipolar ela dá a conta que o artista já fazia (`(max−min)/2` e `(min+max)/2`) e
/// para um unipolar dá a que ele NÃO faria (`max−min` e `min`). Escrever as duas
/// como ramos seria escrever duas leis que concordam hoje e divergem no dia em que
/// aparecer uma terceira polaridade.
///
/// ⚠️ **Uma faixa natural degenerada (`hi == lo`) devolve ganho ZERO e o offset em
/// `min`** — uma forma constante não pode ser esticada até uma faixa, e a resposta
/// honesta é o piso dela, nunca um `inf`.
///
/// `min > max` é aceite e **INVERTE** o resultado (o ganho sai negativo). Isso é
/// deliberado: é o *"Min>Max auto-invertido"* que a Cavalry documenta como default
/// inteligente, e cai fora da fórmula sem um `if`.
#[must_use]
pub fn gain_offset_for_range(natural: (f32, f32), min: f32, max: f32) -> (f32, f32) {
    let (lo, hi) = natural;
    let span = hi - lo;
    if span == 0.0 {
        return (0.0, min);
    }
    let gain = (max - min) / span;
    (gain, min - lo * gain)
}

/// A forma da soma fractal: quantas oitavas, o quanto a frequência sobe e a
/// amplitude cai a cada uma, e como cada oitava é retificada.
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    /// Quantas oitavas somar (o mínimo efetivo é 1 — zero oitavas não é um campo).
    pub octaves: u32,
    /// **Lacunarity**: o multiplicador de frequência por oitava. `2.0` é o
    /// universal (cada oitava divide o tamanho da feição pela metade).
    pub lacunarity: f32,
    /// O ganho/persistência: o multiplicador de amplitude por oitava. `0.5` típico.
    pub roughness: f32,
    /// A retificação por oitava.
    pub ty: NoiseType,
}

impl Default for Spec {
    /// O neutro é o fBm clássico: 1 oitava, lacunarity 2, ganho 0,5.
    fn default() -> Self {
        Self {
            octaves: 1,
            lacunarity: 2.0,
            roughness: 0.5,
            ty: NoiseType::Fbm,
        }
    }
}

/// A soma fractal em `(x, y)`, normalizada pela amplitude total (o resultado fica
/// limitado qualquer que seja a contagem de oitavas).
///
/// `base` é o ruído de UMA oitava: recebe o ponto **já escalado** e o **índice da
/// oitava** — é por esse índice que um consumidor decorrelaciona as oitavas (o
/// `motion.noise` desloca o seed; o `force.curl` ignora), sem que a lei tenha
/// opinião sobre isso.
///
/// ⚠️ **A coordenada é escalada por multiplicação repetida** (`x *= lacunarity`),
/// não por `x * lacunarityᵏ`. Com `lacunarity = 2` as duas são **exatas e
/// idênticas** em IEEE-754 (escalar por potência de dois não arredonda), que é o
/// que torna esta folha adotável byte a byte por quem usava a outra forma; fora
/// das potências de dois elas divergem, e a forma repetida é a que os dois
/// consumidores já usavam no caminho que mais anda.
pub fn eval(spec: Spec, x: f32, y: f32, base: impl Fn(f32, f32, u32) -> f32) -> f32 {
    let gain = spec.roughness.clamp(0.0, 1.0);
    let (mut px, mut py) = (x, y);
    let (mut amp, mut sum, mut total) = (1.0f32, 0.0f32, 0.0f32);
    for o in 0..spec.octaves.max(1) {
        let n = base(px, py, o);
        let shaped = match spec.ty {
            NoiseType::Fbm => n,
            NoiseType::Turbulence => n.abs(),
            NoiseType::Ridged => {
                let r = 1.0 - n.abs();
                r * r
            }
        };
        sum += amp * shaped;
        total += amp;
        amp *= gain;
        px *= spec.lacunarity;
        py *= spec.lacunarity;
    }
    sum / total
}

/// **A costura que faz o campo FECHAR no tempo** — os dois instantes a amostrar e
/// o peso da mistura, para um laço de `loop_len` segundos. `loop_len <= 0` = sem
/// laço (o peso é zero e os dois instantes são o mesmo, então o consumidor pode
/// pular a segunda amostra).
///
/// ⚠️ **O tempo WRAPA primeiro, e é isso que fecha.** Misturar `campo(t)` com
/// `campo(t − L)` não fecha: são campos diferentes nas duas pontas. Lido em
/// `τ = frac(t/L)·L` o campo já é periódico, e a segunda amostra é `τ − L`; em
/// `τ = 0` o resultado é `campo(0)`, e em `τ → L` o peso vai a 1 e a segunda
/// amostra vale `campo(0)` de novo — a costura fecha no MESMO número.
///
/// ⚠️ **O peso é smoothstep, não linear, e não é enfeite:** com peso linear o
/// VALOR fecha mas a DERIVADA salta na costura, e um salto de derivada num campo
/// de movimento lê como um tranco a cada volta. Com `w' = 0` nas duas pontas os
/// dois lados têm a mesma inclinação (`campo'(0)`) ⇒ o ciclo é C¹.
///
/// Livre de transcendentais (HR-5): a técnica que anda num CÍRCULO do espaço de
/// ruído (com `sin`/`cos`) está fora por essa razão.
#[must_use]
pub fn loop_times(t: f32, loop_len: f32) -> (f32, f32, f32) {
    if loop_len <= 0.0 {
        return (t, t, 0.0);
    }
    let u = t / loop_len;
    let u = u - u.floor();
    let tau = u * loop_len;
    (tau, tau - loop_len, u * u * (3.0 - 2.0 * u))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
