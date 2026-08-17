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
/// | `POW4`      | `u⁴`                 | `(1 − t)⁴`    | [`Falloff::Sharper`] |
/// | `ROOT`      | `√u`                 | `√(1 − t)`    | [`Falloff::Root`] |
/// | `SPHERE`    | `√(2u − u²)`         | `√(1 − t²)`   | [`Falloff::Sphere`] |
/// | `INVSQUARE` | `u(2 − u)`           | `1 − t²`      | [`Falloff::InvSquare`] |
/// | `SMOOTH`    | `3u² − 2u³`          | smoothstep    | [`Falloff::Smooth`] |
/// | `SMOOTHER`  | `u³(6u² − 15u + 10)` | quíntica      | [`Falloff::Smoother`] |
///
/// ⚠️ **A `Sphere` é a MESMA curva, e eu tinha escrito o contrário.** A nota da
/// §7.0 do plano dizia *"o `SPHERE` dele é a esfera em `u`, o nosso em `t`"* —
/// a álgebra desmente: `2u − u² = u(2 − u)`, e com `u = 1 − t` isso é
/// `(1 − t)(1 + t) = 1 − t²`. Uma diferença de FORMA não é uma diferença de
/// CURVA, e um par de expressões só se compara depois de reduzido.
///
/// ⚠️ **DUAS armadilhas de NOME, e a cura foi APAGAR a curva inventada, não
/// batizá-la de outra coisa.** O Blender rotula o `POW4` de **"Sharper"** e o
/// `SMOOTH` de **"Smooth"**, e até 2026-08-16 este enum gastava esses dois
/// nomes em curvas que **referência nenhuma tem** — `(1 − t²)⁴` e `(1 − t²)²` —
/// com as leis do Blender escondidas sob [`Falloff::Pow4`] e
/// [`Falloff::Smoothstep`], os identificadores dele. A wave anterior leu isso
/// como uma colisão de RÓTULO e resolveu inventando um segundo nome; ⚠️ **o
/// que ela não perguntou é de onde vinham as duas curvas ocupantes.** Elas não
/// estão no `brush.cc`, não estão no SculptGL (que é o [`Falloff::Plateau`],
/// `3d⁴ − 4d³ + 1`) e não trazem medição nenhuma atrás — *uma entrada de
/// catálogo sem referência e sem número é um botão que ninguém pode julgar*.
/// Apagadas as duas, a colisão **dissolve**: sobra um nome por lei, e o nome é
/// o que o Blender escreve na tela.
///
/// ⚠️ **E o que o `brush.cc` NÃO diz é qual delas um pincel VESTE — a fonte
/// não responde e o ORÁCULO respondeu.** A leitura estática era que o
/// `curve_preset` de um `Brush` zero-inicializado é `BRUSH_CURVE_CUSTOM = 0` e
/// que o `brush_init_data` semeia a *curvemapping* com `CURVE_PRESET_SMOOTH`,
/// logo *"uma bézier editável, nenhuma das nove"*. **Medido no Blender 5.2 a
/// correr** (`docs/3D/ferramentas/blender_sculpt_oracle.py`), o pincel de
/// fábrica reporta `curve_distance_falloff_preset = SMOOTH`, e o perfil que ele
/// deposita é a **analítica**: a `r/R = 0,258` ele move `0,417503` de um pico de
/// `0,5` ⇒ razão **0,835**, contra **0,8348** de `3u² − 2u³` e **0,94** do
/// spline de quatro pontos. *Um pincel não nasce zero-inicializado; ele nasce do
/// arquivo de startup.* É por isso que o perfil do modo `B` declara
/// [`Falloff::Smooth`] em vez de `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Falloff {
    /// `3u² − 2u³` com `u = 1 − t` — a **smoothstep**, e o `BRUSH_CURVE_SMOOTH`
    /// do Blender, que é o preset do pincel de fábrica dele.
    ///
    /// ⚠️ **Ela SUBSTITUIU uma curva inventada que vestia este nome** (`(1 − t²)²`,
    /// escolhida por desenho e que referência nenhuma tem). O nome é do Blender,
    /// então a lei tem de ser a dele: um artista que escolhe *Smooth* aqui e no
    /// Blender tem de receber o mesmo peso — medido, as duas divergiam **3,5% a
    /// meio raio** (0,8789 contra 0,84375) e o nome não dizia isso.
    #[default]
    Smooth,
    /// `√(1 − t²)` — o perfil de uma esfera: cheio no miolo, tangente vertical
    /// na borda. Deposita mais massa que o Smooth com o mesmo raio.
    ///
    /// ⚠️ **É o `BRUSH_CURVE_SPHERE` do Blender, ao pé da letra** — ele o
    /// escreve `√(2u − u²)`, que reduz a esta expressão (tabela acima).
    Sphere,
    /// `(1 − t)⁴` — o `BRUSH_CURVE_POW4`, que o Blender **rotula "Sharper"**.
    /// A mais estreita das nove.
    ///
    /// ⚠️ **Ela SUBSTITUIU uma segunda curva inventada** (`(1 − t²)⁴`), e aqui a
    /// divergência era grande: a meio raio a nossa dava **0,7725** contra os
    /// **0,3164** do Blender — **2,4×** —, ou seja *Sharper* era o nome de uma
    /// curva GORDA num menu cujo dono a usa para detalhe fino.
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
    /// `1 − t²` — o `BRUSH_CURVE_INVSQUARE`, escrito lá como `u(2 − u)`.
    ///
    /// É a [`Falloff::Sphere`] ao quadrado, ou a [`Falloff::Smooth`] pela raiz:
    /// as três são a mesma parábola vista com expoentes diferentes.
    InvSquare,
    /// `u³(6u² − 15u + 10)` com `u = 1 − t` — a **smootherstep** de Perlin, o
    /// `BRUSH_CURVE_SMOOTHER`.
    ///
    /// C² nas duas pontas: é a curva que um escultor reconhece como *o pincel
    /// do Blender* quando quer que a pressão suba e desça sem quina nenhuma.
    Smoother,

    // ── As duas que a wave de paridade tinha DESALOJADO ──────────────────────
    //
    // ⚠️ **Elas voltam com nome PRÓPRIO, e é o ponto inteiro.** Os rótulos
    // `Smooth` e `Sharper` são do Blender e a wave de paridade fez a coisa
    // certa ao pôr as leis dele por baixo deles. O que ela não fez foi guardar
    // as leis que saíram — e elas não eram redundantes com nada do catálogo:
    // *nenhuma* das dez de hoje é `(1 − t²)ⁿ`.
    //
    // ⚠️ **O que essa família tem, e a razão de o artista sentir falta:
    // DERIVADA ZERO no centro.** `d/dt (1 − t²)ⁿ = −2nt(1 − t²)ⁿ⁻¹`, que é
    // exatamente `0` em `t = 0` — o dab pousa como um DOMO, e o vértice do meio
    // sobe quase igual aos vizinhos. As de `u = 1 − t` do Blender caem retas do
    // centro (`Sharper` a **−4**), e sobre uma malha discreta um ápice de cone
    // *é* um vértice puxado para longe do anel dele. Medido em
    // `measure_raycast_feedback::measure_which_falloff_spikes` (guarda-chuva
    // sobre os vértices tocados, dividido pelo relevo para não premiar a curva
    // mais rala): `Sharper` **10,77** contra **3,43** do `Plateau` de fábrica.
    //
    // Apendadas ao FIM pelo motivo dos seis irmãos acima: os chips que o
    // artista já aprendeu não mudam de lugar na fileira.
    /// `(1 − t²)²` — o **domo**: topo chato, ombro largo, pouso liso.
    ///
    /// ⚠️ **É a lei que o rótulo `Smooth` carregava até 2026-08-15**, e é por
    /// isso que ela existe: a wave de paridade não a substituiu por uma
    /// equivalente, ela a apagou. A `Smooth` de hoje (a smoothstep do Blender)
    /// é próxima em VALOR — 0,5625 contra 0,5 a meio raio — e a diferença que
    /// importa é de FORMA, não de altura.
    ///
    /// ⚠️ **Não confundir com [`Falloff::InvSquare`]** (`1 − t²`), que é esta
    /// pela raiz: aquela chega à borda com derivada `−2`, esta com `0`.
    Dome,
    /// `(1 − t²)⁴` — o domo APERTADO: o mesmo topo chato num ombro curto.
    ///
    /// ⚠️ **É a lei que o rótulo `Sharper` carregava até 2026-08-15** — a curva
    /// que o artista pedia quando queria detalhe *sem* espetar. Ela é a razão
    /// de esta família voltar: a `Sharper` de hoje é a mais AGUDA das doze
    /// (`f'(0) = −4`) e esta é a mais CHEIA de topo chato depois da `Dome`,
    /// então trocar uma pela outra sob o mesmo nome não trocou a intensidade,
    /// trocou o que o dab É.
    Dome4,
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
        Self::InvSquare,
        Self::Smoother,
        Self::Dome,
        Self::Dome4,
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
                let u = 1.0 - t;
                3.0 * u * u - 2.0 * u * u * u
            }
            Self::Sphere => (1.0 - t * t).sqrt(),
            Self::Sharper => {
                let u = 1.0 - t;
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
            Self::InvSquare => {
                let u = 1.0 - t;
                u * (2.0 - u)
            }
            Self::Smoother => {
                let u = 1.0 - t;
                u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
            }

            // ⚠️ **Escritas em `u = 1 − t²`, e não em `1 − t`** — é o que as
            // separa das seis do Blender acima e o que lhes dá a derivada zero
            // no centro. Ver o doc das variantes.
            Self::Dome => {
                let u = 1.0 - t * t;
                u * u
            }
            Self::Dome4 => {
                let u = 1.0 - t * t;
                let u2 = u * u;
                u2 * u2
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
            Self::InvSquare => "Inv Square",
            Self::Smoother => "Smoother",
            Self::Dome => "Dome",
            Self::Dome4 => "Dome 4",
        }
    }
}

#[cfg(test)]
#[path = "falloff_tests.rs"]
mod tests;
