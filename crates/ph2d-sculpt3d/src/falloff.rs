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
/// **customizada** (o `ParamWidget::Curve` que o repo já possui) entra quando
/// houver painel de curva — construir aqui um segundo editor seria a segunda
/// resposta que o `04.1` proíbe em letra.
///
/// # As NOVE do Blender entraram, e a leitura corrigiu a minha própria tabela
///
/// O `blenkernel/intern/brush.cc:1489-1610` traz as nove fórmulas analíticas do
/// `BRUSH_CURVE_*` em forma fechada, escritas em `u = 1 − t`. Lidas uma a uma —
/// e **não** de memória — TRÊS das nossas já eram exatamente as dele:
///
/// | preset | em `u` | em `t` | nossa |
/// |---|---|---|---|
/// | `CONSTANT`  | `1`                  | `1`           | [`Falloff::Constant`] |
/// | `LIN`       | `u`                  | `1 − t`       | [`Falloff::Linear`] |
/// | `SHARP`     | `u²`                 | `(1 − t)²`    | [`Falloff::Sharp`] |
/// | `POW4`      | `u⁴`                 | `(1 − t)⁴`    | [`Falloff::Pow4`] |
/// | `ROOT`      | `√u`                 | `√(1 − t)`    | [`Falloff::Root`] |
/// | `SPHERE`    | `√(2u − u²)`         | `√(1 − t²)`   | [`Falloff::Sphere`] |
/// | `INVSQUARE` | `u(2 − u)`           | `1 − t²`      | [`Falloff::InvSquare`] |
/// | `SMOOTH`    | `3u² − 2u³`          | smoothstep    | [`Falloff::Smoothstep`] |
/// | `SMOOTHER`  | `u³(6u² − 15u + 10)` | quíntica      | [`Falloff::Smoother`] |
///
/// ⚠️ **A `Sphere` é a MESMA curva, e eu tinha escrito o contrário.** A nota da
/// §7.0 do plano dizia *"o `SPHERE` dele é a esfera em `u`, o nosso em `t`"* —
/// a álgebra desmente: `2u − u² = u(2 − u)`, e com `u = 1 − t` isso é
/// `(1 − t)(1 + t) = 1 − t²`. Uma diferença de FORMA não é uma diferença de
/// CURVA, e um par de expressões só se compara depois de reduzido.
///
/// ⚠️ **DUAS armadilhas de NOME, e as duas mordem em silêncio.** O Blender
/// rotula o `POW4` de **"Sharper"** e o nosso [`Falloff::Sharper`] é
/// `(1 − t²)⁴`, outra curva — por isso a nova se chama [`Falloff::Pow4`], o
/// identificador dele. E ele rotula o `SMOOTH` de **"Smooth"**, enquanto o
/// nosso [`Falloff::Smooth`] é `(1 − t²)²` — daí [`Falloff::Smoothstep`], que é
/// o nome matemático e não colide. *Duas entradas com o mesmo rótulo são dois
/// botões que o painel pinta lado a lado.*
///
/// ⚠️ **E o que NÃO se lê no `brush.cc` é qual delas um pincel VESTE:** o
/// `curve_preset` de um `Brush` zero-inicializado é `BRUSH_CURVE_CUSTOM = 0`, e
/// o `brush_init_data` semeia a *curvemapping* dele com `CURVE_PRESET_SMOOTH` —
/// uma bézier editável, **nenhuma das nove**. As nove são o que o artista pode
/// ESCOLHER; a de fábrica é outra coisa. É por isso que
/// `VerbProfile::falloff` do modo `B` continua `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Falloff {
    /// `(1 − t²)²` — **C¹ na borda** (valor e derivada zeram em `t = 1`), e é
    /// isso que faz um traço não deixar degrau na fronteira do pincel.
    #[default]
    Smooth,
    /// `√(1 − t²)` — o perfil de uma esfera: cheio no miolo, tangente vertical
    /// na borda. Deposita mais massa que o Smooth com o mesmo raio.
    ///
    /// ⚠️ **É o `BRUSH_CURVE_SPHERE` do Blender, ao pé da letra** — ele o
    /// escreve `√(2u − u²)`, que reduz a esta expressão (tabela acima).
    Sphere,
    /// `(1 − t²)⁴` — pico estreito, ombro que morre cedo. É o falloff de quem
    /// quer detalhe pequeno com pincel grande.
    Sharper,
    /// `1` até a borda, e nada além. Um disco duro; o degrau é a feature.
    ///
    /// ⚠️ **É o `BRUSH_CURVE_CONSTANT` do Blender**, que também não faz nada
    /// além de recusar o que passa do raio.
    Constant,
    /// `√(1 − t)` — sobe rápido na borda e achata no miolo, o oposto do Sharper.
    ///
    /// ⚠️ **É o `BRUSH_CURVE_ROOT` do Blender** (`√u`).
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

    // ── As seis que faltavam do catálogo do Blender ──────────────────────────
    //
    // Apendadas de propósito: os seis chips que o artista já aprendeu não mudam
    // de lugar na fileira. Nada aqui é serializado (o pincel não viaja no
    // documento), então a ORDEM é conforto, não compatibilidade.
    /// `1 − t` — a rampa. O `BRUSH_CURVE_LIN`.
    Linear,
    /// `(1 − t)²` — o `BRUSH_CURVE_SHARP`.
    ///
    /// ⚠️ **Não confundir com [`Falloff::Sharper`]** (`(1 − t²)⁴`, da família do
    /// Painter): esta cai reto do centro, aquela tem platô e ombro curto.
    Sharp,
    /// `(1 − t)⁴` — o `BRUSH_CURVE_POW4`, que o Blender **rotula "Sharper"**.
    ///
    /// ⚠️ O nome dele já é nosso e significa outra curva, então esta veste o
    /// identificador em vez do rótulo. É a mais estreita das nove.
    Pow4,
    /// `1 − t²` — o `BRUSH_CURVE_INVSQUARE`, escrito lá como `u(2 − u)`.
    ///
    /// É a [`Falloff::Sphere`] ao quadrado, ou a [`Falloff::Smooth`] pela raiz:
    /// as três são a mesma parábola vista com expoentes diferentes.
    InvSquare,
    /// `3u² − 2u³` com `u = 1 − t` — a **smoothstep**, e o `BRUSH_CURVE_SMOOTH`.
    ///
    /// ⚠️ **O Blender a rotula "Smooth"**, que aqui já é `(1 − t²)²`; o nome
    /// matemático desempata sem inventar nada.
    Smoothstep,
    /// `u³(6u² − 15u + 10)` com `u = 1 − t` — a **smootherstep** de Perlin, o
    /// `BRUSH_CURVE_SMOOTHER`.
    ///
    /// C² nas duas pontas: é a curva que um escultor reconhece como *o pincel
    /// do Blender* quando quer que a pressão suba e desça sem quina nenhuma.
    Smoother,
}

impl Falloff {
    /// Todos, na ordem em que a UI os lista.
    ///
    /// ⚠️ **O tamanho se CONTA** — o seam
    /// `the_panel_offers_every_falloff_the_engine_has` compara este `ALL` com o
    /// array de ids do painel, então uma curva que entre aqui e não lá nasce
    /// inalcançável e o gate fica vermelho em vez de o chip sumir em silêncio.
    pub const ALL: [Self; 12] = [
        Self::Smooth,
        Self::Sphere,
        Self::Sharper,
        Self::Constant,
        Self::Root,
        Self::Plateau,
        Self::Linear,
        Self::Sharp,
        Self::Pow4,
        Self::InvSquare,
        Self::Smoothstep,
        Self::Smoother,
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

            // As seis do Blender, escritas em `u = 1 − t` porque é assim que o
            // `brush.cc` as escreve — traduzir a forma abriria espaço para uma
            // divergência que só um gate de valor pegaria.
            Self::Linear => 1.0 - t,
            Self::Sharp => {
                let u = 1.0 - t;
                u * u
            }
            Self::Pow4 => {
                let u = 1.0 - t;
                let u2 = u * u;
                u2 * u2
            }
            Self::InvSquare => {
                let u = 1.0 - t;
                u * (2.0 - u)
            }
            Self::Smoothstep => {
                let u = 1.0 - t;
                3.0 * u * u - 2.0 * u * u * u
            }
            Self::Smoother => {
                let u = 1.0 - t;
                u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
            }
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
            Self::Linear => "Linear",
            Self::Sharp => "Sharp",
            Self::Pow4 => "Pow4",
            Self::InvSquare => "Inv Square",
            Self::Smoothstep => "Smoothstep",
            Self::Smoother => "Smoother",
        }
    }
}
