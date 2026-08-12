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

/// **COMO o número do slider de força vira o peso do dab.**
///
/// ⚠️ **É o E13 do estudo, e a razão está escrita na FONTE:** o
/// `brush_strength` do Blender (`sculpt.cc:2337-2339`) traz o comentário
/// *"Primary strength input; square it to make lower values more sensitive"* —
/// o slider é a RAIZ, não o peso.
///
/// ⚠️ **Sozinho isto já torna o chip `B` legítimo pelo §3 do plano:** a meio
/// curso ele deposita `0,25` contra `0,50` — o dobro de diferença, muito acima
/// do piso de paridade de 1 ULP.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrengthCurve {
    /// O número do slider **É** o peso. O SculptGL e o que nós shipávamos.
    #[default]
    Linear,
    /// O peso é o **quadrado** do slider — a faixa baixa ganha resolução.
    Squared,
}

impl StrengthCurve {
    /// O peso que este slider significa.
    #[must_use]
    pub fn resolve(self, slider: f32) -> f32 {
        match self {
            Self::Linear => slider,
            Self::Squared => slider * slider,
        }
    }
}

/// **COMO um verbo que APERTA puxa o vértice** — a primeira metade imperativa
/// de um modo, e a que o atlas de divergência media em `5,776e-4`.
///
/// O `Pinch.js:52-58` soma `(centro − v) · f` **cru**, em 3D. Nós projetamos na
/// tangente antes de somar, e a projeção é o que separa *apertar* de *apertar e
/// afundar junto*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LateralPull {
    /// Projeta na tangente do plano de área antes de puxar — **o nosso**.
    ///
    /// ⚠️ **Isto NÃO é a lei do Blender, e chamá-la de `B` seria o erro que
    /// esta wave acabou de corrigir noutro lugar.** O `pinch.cc:39-60` monta um
    /// frame `(X ao longo do TRAÇO, Z na normal)` e devolve `x_disp + z_disp`,
    /// *"the Y component is removed"* — ele descarta a tangente PERPENDICULAR
    /// ao traço e **guarda** a componente normal, que é quase o oposto do que a
    /// nossa projeção faz. São **três** leis, não duas. Fechar a dele pede o
    /// frame do traço dentro do [`crate::Dab`] — wave própria, nomeada aqui em
    /// vez de contrabandeada num `match` que diria `B` sem ser.
    Tangential,
    /// Puxa em 3D, sem projetar — **a da referência**.
    Direct,
}

/// **Quantos lados do plano um achatamento morde** — a segunda metade
/// imperativa, e a maior do atlas (`1,717e-3` no Flatten).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneReach {
    /// Corta a crista **e** enche o vale.
    ///
    /// É a leitura do Blender, lida no cabeçalho do `plane.cc`: *"The Plane
    /// brush translates the vertices towards the brush plane"*, com **Height**
    /// para o que está acima e **Depth** para o que está abaixo — dois lados,
    /// um knob cada. O nosso é este com os dois knobs no máximo.
    Bilateral,
    /// Um lado só, e o outro é `continue` — **a da referência**.
    ///
    /// O `Flatten.js:64` faz `if (distToPlane * comp > 0.0) continue`, então o
    /// *Flatten* do SculptGL **é** o nosso `Fill` (ou o nosso `Scrape`, sob
    /// Ctrl). É o `continue` que torna o verbo auto-limitado.
    OneSided,
}

/// **COMO um vértice de PERFIL entra no dab** — o terceiro eixo, e o único que
/// acrescenta uma capacidade em vez de restaurar paridade.
///
/// ⚠️ **A linha E12 do doc 20 conflita DOIS consumidores, e o gate depende de
/// distingui-los.** *"O front-face é binário?"* tem três respostas, e elas não
/// falam da mesma coisa:
///
/// - **nós**: um filtro **binário** na ESTIMATIVA DO PLANO (o `front` do
///   `fit_plane_over` — um vértice de costas entra com peso zero na normal e no
///   centro de área). O **dab não filtra nada**;
/// - **o Blender** (`sculpt.cc:7283-7295`): `factors[i] *= max(dot, 0)`, ou seja
///   pesa **o FATOR DE CADA VÉRTICE** do dab;
/// - **o SculptGL**: o `_culling`, um **checkbox do usuário desligado de
///   fábrica** em dez tools.
///
/// ⇒ Este eixo é o **do FATOR**. A metade do plano é outra pergunta, fica onde
/// está, e é o `S` quem depende dela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontFace {
    /// O dab não pesa por orientação — todo vértice da pegada entra inteiro.
    ///
    /// É o que nós e o SculptGL fazem (lá o `_culling` existe e nasce
    /// **desligado**), e portá-lo ligado seria divergir com a ferramenta em
    /// silêncio.
    Ignored,
    /// O fator de cada vértice é escalado por `max(n · olho, 0)` — **o
    /// Blender**.
    ///
    /// Um vértice de perfil pesa zero e um de frente pesa cheio, com a
    /// transição CONTÍNUA: é o *pouso macio* na silhueta, contra o degrau de um
    /// filtro binário.
    Continuous,
}

/// **A metade IMPERATIVA de um modo** — onde a LEI do kernel difere, e não só
/// um número de tabela.
///
/// ⚠️ **Ela é derivada UMA vez do modo e perguntada onde o verbo decide**, e não
/// um `match mode` espalhado por três braços do `compute_target`: três cópias da
/// mesma pergunta são três lugares onde o quarto verbo nasce sem a resposta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelLaw {
    /// Pinch · Magnify · o termo lateral do Crease.
    pub lateral: LateralPull,
    /// Flatten.
    pub plane: PlaneReach,
    /// TODO verbo de carimbo — é um fator por vértice.
    pub front_face: FrontFace,
}

impl RefMode {
    /// A lei de kernel que este modo manda.
    ///
    /// ⚠️ **O `B` e o `L` devolvem o que o app JÁ shipava**, de propósito: esta
    /// wave não muda o produto, ela torna o `S` verdadeiro. Antes dela o app
    /// rodava um kernel que não era o de referência nenhuma em três verbos, com
    /// um chip que dizia `S` — o mesmo defeito que a W0 curou nos DEFAULTS,
    /// aqui na LEI.
    #[must_use]
    pub const fn kernel(self) -> KernelLaw {
        match self {
            Self::S => KernelLaw {
                lateral: LateralPull::Direct,
                plane: PlaneReach::OneSided,
                front_face: FrontFace::Ignored,
            },
            Self::B | Self::L => KernelLaw {
                lateral: LateralPull::Tangential,
                plane: PlaneReach::Bilateral,
                // ⚠️ **É o único dos três eixos em que o `B` ACRESCENTA** em vez
                // de guardar o que o app já fazia — os outros dois nasceram
                // preservando o produto, este liga uma lei que ninguém tinha.
                front_face: FrontFace::Continuous,
            },
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
    ///
    /// ⚠️ **`None` é uma AFIRMAÇÃO, e ela ficou mais forte quando o `brush.cc`
    /// foi lido** (§7.0 do plano). As nove fórmulas do `BRUSH_CURVE_*` são
    /// legíveis e **já estão todas em [`Falloff`]** — mas elas são o que o
    /// artista pode ESCOLHER, não o que um pincel VESTE: o `curve_preset` de um
    /// `Brush` zero-inicializado é `BRUSH_CURVE_CUSTOM = 0`, e o
    /// `brush_init_data` semeia a *curvemapping* dele com `CURVE_PRESET_SMOOTH`
    /// — uma bézier editável, **nenhuma das nove**.
    ///
    /// ⇒ Declarar aqui *"o Blender usa a Smooth"* seria inventar um número e
    /// vesti-lo com o nome de outro produto. E a tabela por-TOOL (a força e o
    /// raio de fábrica do Clay Strips) não é lida de fonte nenhuma: desde o 4.3
    /// ela vive dentro de um `.blend` binário de assets.
    pub falloff: Option<Falloff>,
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
    /// **Como o slider de força vira peso** — o E13.
    ///
    /// ⚠️ Não é `Option`: toda referência responde a esta pergunta, porque toda
    /// referência transforma o slider de ALGUMA maneira — e *"linearmente"* é
    /// uma resposta, não uma omissão.
    pub strength_curve: StrengthCurve,
}

impl VerbProfile {
    /// O perfil que **não afirma nada** — a base sobre a qual as entradas da
    /// tabela são escritas, para uma linha nova nascer explícita no que declara.
    const SILENT: Self = Self {
        falloff: None,
        strength: None,
        radius_factor: None,
        accumulate: None,
        strength_curve: StrengthCurve::Linear,
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
            falloff: Some(Falloff::Plateau),
            strength: Some(0.5),
            radius_factor: Some(1.0),
            accumulate: Some(true),
            ..VerbProfile::SILENT
        },
        // `Inflate.js:9-11` — o mais fraco do catálogo, e por um motivo: inflar
        // é a operação que estoura mais rápido.
        Verb::Inflate => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.3),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Smooth.js:10-13` — e ⚠️ o `_tangent = false` dele é o
        // `smoothTangent`, **código vivo que nenhuma UI do original alcança**;
        // é o E7 do estudo e a wave W4 daqui.
        Verb::Smooth => VerbProfile {
            falloff: Some(Falloff::Plateau),
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
            falloff: Some(Falloff::Plateau),
            strength: Some(0.75),
            radius_factor: Some(1.0),
            accumulate: Some(true),
            ..VerbProfile::SILENT
        },
        // `Pinch.js:9-11`.
        Verb::Pinch | Verb::Magnify => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.75),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Crease.js:9-11` — ⚠️ **raio 25, metade do resto**: um vinco é fino
        // por definição, e é a divergência D4 mais visível do catálogo.
        Verb::Crease => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.75),
            radius_factor: Some(25.0 / S_BASE_RADIUS_PX),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Masking.js:13-16` — força CHEIA, e o nosso default já concorda.
        Verb::Mask => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(1.0),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Move.js:10-11` — ⚠️ **raio 150, TRÊS vezes o resto**: puxar é um
        // gesto de região, e um Move com raio de pincel de detalhe é o que faz
        // um artista concluir que a ferramenta não funciona.
        Verb::Move => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(1.0),
            radius_factor: Some(150.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `Drag.js:10` — mesmo raio do Move e ⚠️ **sem `_intensity` declarada**:
        // o `None` é a fonte sendo silenciosa, não um número esquecido.
        Verb::SnakeHook => VerbProfile {
            falloff: Some(Falloff::Plateau),
            radius_factor: Some(150.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `Twist.js:10` — raio 75, e também sem força declarada.
        Verb::Twist => VerbProfile {
            falloff: Some(Falloff::Plateau),
            radius_factor: Some(75.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `LocalScale.js:8` — raio-base, sem força.
        Verb::LocalScale => VerbProfile {
            falloff: Some(Falloff::Plateau),
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
            RefMode::B => profile_b(self),
            // ⚠️ Chega nas waves W4/W5/W7, um paper por vez. Enquanto for `None`
            // **nenhum chip é oferecido**, que é a lei anti-chip-morto valendo
            // por construção em vez de por disciplina.
            RefMode::L => None,
        }
    }
}

/// **A coluna `B`** — o que o Blender DECLARA, e só isso.
///
/// ⛔ **Ela não traz DEFAULTS, e a ausência é MEDIDA — o arquivo foi trazido e
/// respondeu que a resposta não está nele** (§7.0 do plano). O
/// `BKE_brush_sculpt_reset`, onde a força, o raio e a curva de fábrica de cada
/// tool viviam, **não existe mais em C** (`git grep` sobre a árvore: zero):
/// desde o Blender 4.3 os pincéis são ASSETS, num `.blend` binário. O que
/// sobrou, o `brush_defaults()` (`brush.cc:597`), copia de um `Brush def = {}`
/// — **um** conjunto para todas as tools, não uma tabela por ferramenta.
///
/// ⇒ *"a força de fábrica do Clay Strips"* não é lida de fonte nenhuma. Isto
/// **não é uma lacuna do nosso clone**: é onde o Blender passou a guardar a
/// resposta, e trazer mais arquivos não muda.
///
/// ✅ **O que ela traz é LIDO literalmente** (`sculpt.cc:2337-2339`): o slider é
/// a RAIZ do peso, com o comentário do próprio Blender ao lado — *"square it to
/// make lower values more sensitive"*. Vale para TODA tool: ele mora no
/// `brush_strength`, que é o funil de todas elas — e é por isso que este `match`
/// não tem braços por verbo.
const fn profile_b(_verb: Verb) -> Option<VerbProfile> {
    Some(VerbProfile {
        strength_curve: StrengthCurve::Squared,
        ..VerbProfile::SILENT
    })
}

#[cfg(test)]
#[path = "ref_mode_tests.rs"]
mod tests;
