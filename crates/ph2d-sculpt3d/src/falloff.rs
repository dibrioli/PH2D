//! **A CURVA DO PINCEL** — irmã do [`super::brush`], e o corte é de assunto.
//!
//! ⚠️ Ela saiu do `brush.rs` porque a curva deixou de ser um detalhe do pincel
//! e virou **o que um MODO DE REFERÊNCIA escolhe** (`crate::ref_mode`): é o
//! achado D1 do `docs/3D/20_divergencias_tools.md`, o que muda o pincel em
//! `1,08x a 1,44x` ao longo do raio. Um assunto com consumidor próprio ganha
//! arquivo próprio; o `brush.rs` fica com *que ferramenta está na mao*.
//!
//! O tipo e´ **re-exportado pela raiz**, entao nenhum caminho de chamador muda.

/// A curva de peso do pincel, do centro (`t = 0`) à borda (`t = 1`).
///
/// A MESMA família que o Painter 2D já expõe, e de propósito: um artista que
/// aprendeu *Sharper* pintando não devia reaprendê-la esculpindo. A curva
/// **customizada** (o `ParamWidget::Curve` que o repo já possui) é a 6ª e entra
/// quando houver painel — construir aqui um segundo editor de curva seria a
/// segunda resposta que o `04.1` proíbe em letra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Falloff {
    /// `(1 − t²)²` — **C¹ na borda** (valor e derivada zeram em `t = 1`), e é
    /// isso que faz um traço não deixar degrau na fronteira do pincel.
    #[default]
    Smooth,
    /// `√(1 − t²)` — o perfil de uma esfera: cheio no miolo, tangente vertical
    /// na borda. Deposita mais massa que o Smooth com o mesmo raio.
    Sphere,
    /// `(1 − t²)⁴` — pico estreito, ombro que morre cedo. É o falloff de quem
    /// quer detalhe pequeno com pincel grande.
    Sharper,
    /// `1` até a borda, e nada além. Um disco duro; o degrau é a feature.
    Constant,
    /// `√(1 − t)` — sobe rápido na borda e achata no miolo, o oposto do Sharper.
    Root,
    /// `3t⁴ − 4t³ + 1` — **a curva da REFERÊNCIA**, e a única desta família que
    /// não foi escolhida por desenho: ela é a que as dez tools de geometria do
    /// SculptGL usam, e é o que a paridade bit-a-bit exige que exista aqui.
    ///
    /// ⚠️ **Ela é mais CHEIA que a `Smooth`, e a diferença é visível:** a meio
    /// raio dá `0,6875` contra `0,5625` — **1,22×** —, e a razão cresce até
    /// `1,44×` a `7/8` do raio. Um artista que trocar de uma para a outra vê o
    /// pincel engordar, não um detalhe numérico.
    ///
    /// ⚠️ **Ela é `C¹` nas DUAS pontas** (derivada `12t²(t − 1)`, que zera em
    /// `t = 0` e em `t = 1`) — daí o nome: um platô no miolo e um pouso sem
    /// degrau na borda. A `Smooth` só é plana na borda.
    ///
    /// ⚠️ **E o valor sai da PORTA ÚNICA** [`crate::ref_kernels::falloff`], em
    /// `f64`, arredondado uma vez: uma segunda cópia da quártica aqui seria a
    /// forma exata de a paridade divergir do porte que ela mede.
    Plateau,
}

impl Falloff {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 6] = [
        Self::Smooth,
        Self::Sphere,
        Self::Sharper,
        Self::Constant,
        Self::Root,
        Self::Plateau,
    ];

    /// O peso a uma distância normalizada `t`. **Porta única** — todo verbo, a
    /// simetria e o cursor perguntam a esta função.
    ///
    /// Sem transcendental exceto a raiz (HR-5): `sqrt` é instrução de hardware,
    /// não chamada de libm.
    #[must_use]
    pub fn weight(self, t: f32) -> f32 {
        // O NaN é peneirado explicitamente: sem isso ele escorre pelas fórmulas
        // e sai como peso NaN num vértice, que é como uma malha inteira vira
        // `NaN` a partir de um dab com raio zero em algum lugar.
        if !t.is_finite() || t >= 1.0 {
            return 0.0;
        }
        let t = t.max(0.0);
        match self {
            Self::Smooth => {
                let u = 1.0 - t * t;
                u * u
            }
            Self::Sphere => (1.0 - t * t).sqrt(),
            Self::Sharper => {
                let u = 1.0 - t * t;
                let u2 = u * u;
                u2 * u2
            }
            Self::Constant => 1.0,
            Self::Root => (1.0 - t).sqrt(),
            // ⚠️ **Em `f64`, e não uma quártica escrita aqui em `f32`.** É a
            // aritmética do original (um `Float32Array` do JS lê `f32 → f64`,
            // calcula em `f64` e arredonda UMA vez), e é o que faz esta curva
            // servir de peça de paridade em vez de parecer com ela.
            Self::Plateau => crate::ref_kernels::falloff(f64::from(t)) as f32,
        }
    }

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Smooth => "Smooth",
            Self::Sphere => "Sphere",
            Self::Sharper => "Sharper",
            Self::Constant => "Constant",
            Self::Root => "Root",
            Self::Plateau => "Plateau",
        }
    }
}
