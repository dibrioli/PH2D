//! **OS TRÊS MODOS DE REFERÊNCIA** — de qual fonte um verbo herda o que ele é.
//!
//! Plano: [`docs/3D/21_plano_modos_e_ferramentas.md`]. Levantamento que o
//! fundamenta: [`docs/3D/20_divergencias_tools.md`] (D1-D27).
//!
//! # O que este módulo é, e o que ele NÃO é
//!
//! Um modo governa **duas metades**, e separá-las é o que impede a explosão
//! combinatória (16 verbos × 3 modos × 2 níveis de UI = 96 caminhos):
//!
//! - a **declarativa** — a tabela de números que o verbo ARMA (curva, força,
//!   raio, `accumulate`). É **este módulo**, e é `const`;
//! - a **imperativa** — onde o modo escolhe uma FUNÇÃO diferente (laplaciano ×
//!   HC × Taubin; deslocamento × Kelvinlets; plano × MLS). É um braço no
//!   `compute_target`/[`crate::GripLaw`], e chega wave a wave.
//!
//! ⚠️ **A metade declarativa é a que mais muda o que o artista vê**, e isso é
//! medido, não opinião: os quatro achados mais visíveis do doc 20 (D1-D4) são
//! **tabelas**, não algoritmos — a curva sozinha muda o pincel em **1,08× a
//! 1,44×** ao longo do raio.
//!
//! # ⚠️ O que o app rodava ANTES desta tabela: um TERCEIRO que ninguém escolheu
//!
//! Nós shipávamos o **kernel** do SculptGL (medido: `1,00×` a `5,960e-8` em
//! Draw/Clay/Fill/Scrape/Inflate — doc 20 §11.1, *"o kernel é idêntico ao ULP;
//! o produto não"*) com **defaults NOSSOS**: curva `Smooth` e força `0,5` em
//! tudo. Isso não era o s-mode nem o b-mode. Era um terceiro, e é a causa raiz
//! do D1-D4.
//!
//! ⇒ O que este módulo entrega não é *"nada muda"*: é que **o default do app
//! deixa de ser um terceiro sem nome e passa a ser uma REFERÊNCIA nomeada.**
//!
//! # ⚠️ [`RefMode::S`] é o CONTRATO DE PARIDADE; o default é PRODUTO
//!
//! Os treze kernels do [`crate::ref_kernels`] são gateados bit a bit contra o JS
//! **executando**. Um oráculo executável com paridade ao ULP é raro, e é ele que
//! torna seguro trocar de alvo depois — dá para adotar a curva do Blender, o HC
//! e os Kelvinlets *sabendo qual bit deixou de ser idêntico e qual não foi
//! tocado*. Por isso o gate afirma sobre o `S`, e o **default** pode mudar sem
//! que o contrato se mova. O precedente é o `PH2D_FLIP_NEW_ENGINE=0` do Flip: o
//! motor antigo continua vivo e testado como rota de bissecção.
//!
//! # ⚠️ `None` é uma AFIRMAÇÃO, não um buraco
//!
//! [`Verb::profile`] devolve `Option`, e cada `None` é um fato sobre a fonte:
//! *esta referência não tem resposta para este verbo* (o SculptGL não tem
//! Sharpen; ele não declara força para Drag/Twist/LocalScale). Isso alimenta
//! direto a lei anti-chip-morto do plano §3 — **um chip existe se e somente se
//! o perfil existe** —, então não há uma segunda lista de "quais modos oferecer"
//! para derivar da tabela.
//!
//! E é também o que impede o pior erro possível aqui: **inventar** um número e
//! shipá-lo com a autoridade de uma referência que não o declara.

use crate::brush::Verb;
use crate::falloff::Falloff;

/// De qual fonte um verbo herda o que ele é.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub enum RefMode {
    /// **SculptGL** (MIT) — a referência portada 1:1 e gateada ao ULP.
    #[default]
    S,
    /// **Blender** (GPL — só comportamento, nunca código).
    B,
    /// **A literatura publicada** — cada l-mode tem paper, ano e critério de
    /// aceitação. Ver o plano §4; não há l-mode "de gosto".
    L,
}

impl RefMode {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 3] = [Self::S, Self::B, Self::L];

    /// A letra que o chip mostra.
    ///
    /// ⚠️ **Letras, e não o nome do produto.** O artista não sabe o que é o
    /// SculptGL, e o nome de um produto de terceiro num botão é ruído que
    /// envelhece; o significado vive no readout e no tooltip. É uma linha de
    /// i18n trocar para os nomes por extenso.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::S => "S",
            Self::B => "B",
            Self::L => "L",
        }
    }
}

/// **O que uma referência DECLARA sobre um verbo.**
///
/// ⚠️ **A struct CRESCE wave a wave, e um campo só entra com o consumidor
/// dele.** Os campos da metade imperativa (`front_face`, `hardness`,
/// `normal_radius_factor`, `strength_curve`, `plane_side`, …) chegam junto com
/// os kernels que os leem — um campo que ninguém lê é estado morto, e este repo
/// varre isso a cada wave.
///
/// ⚠️ **Cada `Option` é uma afirmação sobre a FONTE**, nunca um valor esquecido:
/// `None` quer dizer *a referência é silenciosa aqui*, e quem arma deixa o
/// número do artista onde está.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VerbProfile {
    /// A curva do pincel — D1/E1, o maior número do estudo.
    pub falloff: Falloff,
    /// A força de fábrica — D3/E2. `None` = a fonte não declara.
    pub strength: Option<f32>,
    /// O raio de fábrica como **fração do raio-base da fonte** — D4/E3.
    ///
    /// ⚠️ Fração e não pixels: o raio-base do SculptGL é `50` e o nosso
    /// `radius_px` também nasce em `50`, mas guardar `25` aqui congelaria a
    /// escolha do artista no dia em que ele mudasse o raio-base. Uma fração
    /// continua verdadeira em qualquer raio — é a mesma lei do
    /// `sss_scatter` (fração do maior lado) e do `Falloff` (distância
    /// normalizada).
    pub radius_factor: Option<f32>,
    /// O Accumulate de fábrica. `None` = a fonte não declara o campo.
    pub accumulate: Option<bool>,
}

impl VerbProfile {
    /// O perfil que **não afirma nada** — a base sobre a qual as entradas da
    /// tabela são escritas, para uma linha nova nascer explícita no que declara.
    const SILENT: Self = Self {
        falloff: Falloff::Plateau,
        strength: None,
        radius_factor: None,
        accumulate: None,
    };
}

/// O raio de fábrica do SculptGL, em pixels — `SculptBase`/`Brush.js:11`. É o
/// denominador de todo [`VerbProfile::radius_factor`] da coluna `S`.
const S_BASE_RADIUS_PX: f32 = 50.0;

/// **A coluna `S`** — lida das fontes do SculptGL, arquivo e linha ao lado.
///
/// ⚠️ **Todo número aqui foi LIDO, nenhum foi afinado.** As tools de geometria
/// do original compartilham a quártica `3t⁴ − 4t³ + 1` (a nossa
/// [`Falloff::Plateau`], que sai da porta única [`crate::ref_kernels::falloff`]
/// em `f64`), e o que difere entre elas é força, raio e `accumulate`.
const fn profile_s(verb: Verb) -> Option<VerbProfile> {
    // Um `radius_factor` é `_radius / 50`.
    let p = match verb {
        // `Brush.js:11-16` — `_radius 50 · _intensity 0.5 · _clay true ·
        // _accumulate true`. A tool `Brush` do original é a nossa **Draw E
        // Clay** (o `_clay` é um checkbox dela, ligado de fábrica).
        //
        // ⚠️ **A força do Draw é a ÚNICA que já batia com a nossa** (0,5). As
        // divergências do D3 são as outras: 0,75 em quatro tools e 0,30 no
        // Inflate.
        Verb::Draw | Verb::Clay => VerbProfile {
            strength: Some(0.5),
            radius_factor: Some(1.0),
            accumulate: Some(true),
            ..VerbProfile::SILENT
        },
        // `Inflate.js:9-11` — o mais fraco do catálogo, e por um motivo: inflar
        // é a operação que estoura mais rápido.
        Verb::Inflate => VerbProfile {
            strength: Some(0.3),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Smooth.js:10-13` — e ⚠️ o `_tangent = false` dele é o
        // `smoothTangent`, **código vivo que nenhuma UI do original alcança**;
        // é o E7 do estudo e a wave W4 daqui.
        Verb::Smooth => VerbProfile {
            strength: Some(0.75),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Flatten.js:9-11` — ⚠️ e a família do plano é mais forte que um
        // default: o `Flatten.js` **não declara `_accumulate`** e o kernel dele
        // pergunta `this._accumulate === false`, que em `undefined` é FALSO ⇒
        // ele lê o vivo **sempre**, sem checkbox. Nós temos o interruptor, então
        // o honesto é nascer armado.
        Verb::Flatten | Verb::Fill | Verb::Scrape => VerbProfile {
            strength: Some(0.75),
            radius_factor: Some(1.0),
            accumulate: Some(true),
            ..VerbProfile::SILENT
        },
        // `Pinch.js:9-11`.
        Verb::Pinch | Verb::Magnify => VerbProfile {
            strength: Some(0.75),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Crease.js:9-11` — ⚠️ **raio 25, metade do resto**: um vinco é fino
        // por definição, e é a divergência D4 mais visível do catálogo.
        Verb::Crease => VerbProfile {
            strength: Some(0.75),
            radius_factor: Some(25.0 / S_BASE_RADIUS_PX),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Masking.js:13-16` — força CHEIA, e o nosso default já concorda.
        Verb::Mask => VerbProfile {
            strength: Some(1.0),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Move.js:10-11` — ⚠️ **raio 150, TRÊS vezes o resto**: puxar é um
        // gesto de região, e um Move com raio de pincel de detalhe é o que faz
        // um artista concluir que a ferramenta não funciona.
        Verb::Move => VerbProfile {
            strength: Some(1.0),
            radius_factor: Some(150.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `Drag.js:10` — mesmo raio do Move e ⚠️ **sem `_intensity` declarada**:
        // o `None` é a fonte sendo silenciosa, não um número esquecido.
        Verb::SnakeHook => VerbProfile {
            radius_factor: Some(150.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `Twist.js:10` — raio 75, e também sem força declarada.
        Verb::Twist => VerbProfile {
            radius_factor: Some(75.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `LocalScale.js:8` — raio-base, sem força.
        Verb::LocalScale => VerbProfile {
            radius_factor: Some(1.0),
            ..VerbProfile::SILENT
        },
        // ⚠️ **O SculptGL NÃO TEM Sharpen**, e este `None` é essa frase.
        // (O Blender tem o equivalente, `Enhance Details` — doc 20 §9 item 12.)
        Verb::Sharpen => return None,
    };
    Some(p)
}

impl Verb {
    /// **O que a referência `mode` declara sobre este verbo** — ou `None` se ela
    /// não tem resposta para ele.
    ///
    /// ⚠️ É a porta única da metade declarativa: o painel pergunta para saber
    /// **que chips oferecer** e **que valor pintar**, o arming pergunta para
    /// saber **o que escrever**, e o gate pergunta para conferir contra a fonte.
    /// Uma segunda tabela em qualquer um dos três derivaria em silêncio.
    #[must_use]
    pub const fn profile(self, mode: RefMode) -> Option<VerbProfile> {
        match mode {
            RefMode::S => profile_s(self),
            // ⚠️ Chegam nas waves W1 (o `B` declarativo) e W4/W5/W7 (o `L`, um
            // paper por vez). Enquanto forem `None` **nenhum chip é oferecido**,
            // que é a lei anti-chip-morto valendo por construção em vez de por
            // disciplina.
            RefMode::B | RefMode::L => None,
        }
    }
}

#[cfg(test)]
#[path = "ref_mode_tests.rs"]
mod tests;
