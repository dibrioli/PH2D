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
/// ⚠️ **São TRÊS leis, e cada uma remove uma coisa diferente de `centro − v`.**
/// A tabela é o resumo, e o doc de cada variante traz o arquivo e a linha:
///
/// | lei | remove | fonte |
/// |---|---|---|
/// | [`Self::Direct`] | nada (3D cru) | `Pinch.js:52-58` · `Crease.js:59-61` |
/// | [`Self::Tangential`] | a componente **NORMAL** | `crease.cc:112` |
/// | [`Self::AcrossStroke`] | a componente **AO LONGO DO TRAÇO** | `pinch.cc:39-60` |
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LateralPull {
    /// Projeta na tangente do plano de área antes de puxar — **o `crease.cc` do
    /// Blender, e não uma invenção nossa**.
    ///
    /// ⚠️ **O doc que estava aqui dizia o contrário — *"o nosso"*, *"isto NÃO é
    /// a lei do Blender"* — e a fonte o desmente numa linha.** O
    /// `crease.cc:110-112` faz exatamente esta projeção, com o porquê ao lado:
    ///
    /// > *"The vertices are pinched towards a **line** instead of a single
    /// > point. Without this we get a 'flat' surface surrounding the pinch."*
    ///
    /// Remover a componente normal faz o puxão apontar para o **EIXO** que passa
    /// pelo centro, não para o ponto — que é a frase do comentário dele.
    ///
    /// ⚠️ **O erro tinha uma causa nomeável, e ela vale para toda leitura de
    /// fonte alheia:** a nota anterior descrevia o `pinch.cc` a partir do
    /// COMENTÁRIO dele (*"Project the displacement into the X vector (aligned to
    /// the stroke)"*) — e esse comentário é **falso no próprio Blender**. O
    /// código monta `X = cross(area_no, grab_delta)`, que é **perpendicular** ao
    /// traço. Ler o comentário e não o `cross` inverteu o mapa inteiro: nós
    /// dizíamos coincidir com ninguém e divergir do Blender, quando na verdade
    /// **coincidimos com o `crease.cc`** e faltava-nos o `pinch.cc`.
    Tangential,
    /// Puxa em 3D, sem projetar — **o SculptGL**.
    Direct,
    /// Remove a componente **ao longo do traço** — o `pinch.cc:39-60`.
    ///
    /// O frame de lá é `X = normalize(cross(area_no, grab_delta))` (perpendicular
    /// ao traço, no plano tangente) e `Z = area_no`; o resultado é
    /// `x_disp + z_disp`, com *"the Y component is removed"* — e o `Y` é
    /// `cross(area_no, X)`, ou seja **a direção do traço**.
    ///
    /// ⇒ Passando o pincel ao longo de uma linha, o barro é espremido dos dois
    /// lados **para** a linha e nunca arrastado ao longo dela: é o que faz um
    /// vinco em vez de um funil. É também a razão de o `S` e o `B` do Pinch se
    /// parecerem antes desta lei existir — as duas eram apertos radiais.
    ///
    /// ⚠️ **Ela precisa de DIREÇÃO, e sem direção não há dab.** A referência
    /// recusa explicitamente (`pinch.cc:188-195`: *"delay the first daub because
    /// grab delta is not setup"*, e `return` se `grab_delta` é zero), e nós
    /// fazemos o mesmo — o [`crate::Dab::path`] nasce em zero no primeiro dab de
    /// um traço, e um tap solto neste modo **não aperta nada**. Inventar um eixo
    /// ali seria escolher uma direção que o artista não desenhou.
    AcrossStroke,
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
/// ⚠️ **O puxão lateral NÃO mora aqui, e a ausência é o achado de 2026-08-15:**
/// os outros dois eixos são fatos sobre o MODO (o Blender achata dos dois lados
/// em toda tool de plano; ele pesa por face em toda tool de carimbo), mas a lei
/// lateral dele é **por FERRAMENTA** — o `pinch.cc` remove a componente ao longo
/// do traço e o `crease.cc` remove a normal. Um campo aqui teria de responder
/// duas coisas com um valor, e a porta é a [`RefMode::lateral_for`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelLaw {
    /// Flatten.
    pub plane: PlaneReach,
    /// TODO verbo de carimbo — é um fator por vértice.
    pub front_face: FrontFace,
}

impl RefMode {
    /// **ESTE MODO DIZ ALGUMA COISA SOBRE ESTE VERBO?** — a porta única da lei
    /// anti-chip-morto do §3 do plano: *um chip existe se e somente se o modo
    /// existe*.
    ///
    /// ⚠️ **A PRIMEIRA versão desta função era derivada — *"este modo duplica um
    /// anterior?"* — e ela OFERECIA o `L`.** O motivo é o achado: o `L` não é
    /// uma duplicata do `B`, ele é **`B` sem o `strength²`** (o `profile_l`
    /// devolve `None`, então a `StrengthCurve` cai no `Linear` do
    /// [`VerbProfile::SILENT`]). Ou seja: hoje o `L` já significa alguma coisa,
    /// e essa coisa **não é literatura** — é um acidente da tabela vazia. Um
    /// chip que funciona e mente sobre o próprio nome é pior que um chip
    /// ausente.
    ///
    /// ⇒ A resposta do `L` é uma **AFIRMAÇÃO SOBRE O QUE FOI CONSTRUÍDO**, e a
    /// direção de falha é a segura: esquecer de virá-la deixa a feature
    /// **invisível**, nunca errada. O gate
    /// `a_mode_is_offered_only_where_it_is_not_a_duplicate_of_an_earlier_one`
    /// cobra a metade perigosa — *dois modos oferecidos nunca são o mesmo* — e
    /// o irmão dele fixa a razão do `L` em vez de a deixar como prosa.
    ///
    /// ⚠️ **E ela DEIXOU DE SER UNIVERSAL na W4**, que é a wave que o parágrafo
    /// acima previa. O `S` e o `B` respondem por TODO verbo porque as fontes
    /// deles são motores completos; a LITERATURA não é um motor — ela é um
    /// paper por assunto, e o primeiro portado (Taubin 1995) fala do **Smooth**
    /// e de mais nada. Uma resposta universal aqui daria quinze chips sem
    /// conteúdo pelo preço do que tem um.
    #[must_use]
    pub const fn declares(self, verb: Verb) -> bool {
        match self {
            // A tabela lida das fontes (§7 do plano) e a lei de kernel.
            //
            // ⚠️ **MENOS o Clay Strips, e a incoerência era MEDÍVEL:** o
            // SculptGL **não tem** esta ferramenta — o censo
            // `the_census_of_offered_chips` já dizia isso pela tabela de
            // DEFAULTS (`profile_s` devolve `None`) — e mesmo assim a tabela da
            // LEI dizia que o `S` a governava. O preço, medido na sonda
            // `measure_whether_the_strip_deposits_on_back_facing_clay` com o
            // olho rasante: a faixa punha **2,3× mais barro nas costas do que
            // na frente** (39,86 contra 17,18), porque o `front_face:
            // Ignored` do `S` é fiel ao `Brush.js` (o `_culling` nasce
            // desligado) e **não é a lei desta ferramenta**. O
            // `clay_strips.cc::calc_faces` aplica o `calc_front_face` como
            // terceira linha da cadeia de fatores.
            //
            // ⚠️ **A lei de uma referência só vale para as ferramentas que ela
            // TEM** — é a mesma frase que a coluna de defaults já honrava, e
            // agora as duas tabelas concordam.
            // ⚠️ **E nem o Blob, pela MESMA frase:** o `Crease.js` é o único
            // parente que o SculptGL tem, e ele não carrega o `invert_strength`
            // que faz de um Blob um Blob. Uma referência governa as ferramentas
            // que ela TEM.
            // ⚠️ **E nem o Clay Thumb**, pela terceira vez a mesma frase: o
            // `clay_thumb.cc` é do Blender e o SculptGL não tem parente dele.
            // ⚠️ **E nem o Multiplane Scrape**, a quarta: `multiplane_scrape.cc`,
            // e o SculptGL não tem uma ferramenta de DOIS planos.
            // ⚠️ **E nem o Slide Relax**, a quinta: `relax.cc` é do Blender, e o
            // SculptGL não tem verbo que mexa na MALHA sem mexer na forma.
            Self::S => !matches!(
                verb,
                Verb::ClayStrips
                    | Verb::Blob
                    | Verb::ClayThumb
                    | Verb::MultiplaneScrape
                    | Verb::SlideRelax
            ),
            // A lei de kernel (bilateral · tangencial · front-face contínuo) e
            // a `StrengthCurve::Squared` do E13.
            Self::B => true,
            // ⚠️ **TAUBIN 1995 no Smooth, e mais nada** — o par λ|μ de
            // [`crate::Brush::passes`], que é a única literatura portada.
            //
            // ⚠️ **Escrita e não DERIVADA de `passes().len() > 1`, e eu escrevi
            // a derivada primeiro:** ela é elegante, casa hoje, e é **falsa em
            // geral** — o Kelvinlets da W5 é um campo de deslocamento de UM
            // passe, e sob a derivada o chip dele nasceria mudo com todos os
            // gates verdes. *Fazer dois passes* e *declarar uma lei* são
            // perguntas diferentes que hoje têm a mesma resposta; o gate
            // `the_literature_mode_is_offered_exactly_where_it_declares_a_law`
            // pina a coincidência, e o dia em que ela cair é o dia em que
            // alguém tem de vir aqui decidir.
            //
            // ⚠️ **O Sharpen fica FORA de propósito:** o paper descreve um
            // passa-baixa que não encolhe, e não diz nada sobre afiar — dar-lhe
            // o par seria pôr o nome de uma fonte numa lei que ela não declara.
            Self::L => matches!(verb, Verb::Smooth) || self.field(verb).is_some(),
        }
    }

    /// **QUE CAMPO ELÁSTICO este modo entrega a este verbo** — o `l-mode` da
    /// família que AGARRA ([`crate::kelvinlet`], de Goes & James 2017).
    ///
    /// ⚠️ **`None` é o mundo que já shipava, e não um buraco:** só o `L`
    /// responde, e só nos verbos cuja lei o paper de facto declara.
    ///
    /// ⚠️ **Esta pergunta move uma COLUNA da [`crate::GripLaw`]**, e é por isso
    /// que ela é uma porta e não um `match` no meio do laço do dab: um campo
    /// **é** o falloff, então quem o recebe carrega o peso no ALVO. Perguntada
    /// em dois lugares, o dia em que um verbo novo entrasse na família seria o
    /// dia em que ele aplicaria o perfil duas vezes — o defeito que o
    /// [`Verb::Move`] já pagou uma vez, medido em `0,12226` contra `0,22500`.
    #[must_use]
    pub const fn field(self, verb: Verb) -> Option<Field> {
        match self {
            // ⚠️ **O modo decide SE, o verbo decide QUAL** — ver
            // [`Verb::elastic_field`]. As duas perguntas viviam neste `match` e
            // isso deixava um par (verbo, campo) ERRADO ser escrito: o alvo do
            // verbo casa o próprio variante, então um par trocado o mandava para
            // o modo que já shipava **enquanto a pegada continuava a do campo**
            // (o [`crate::Brush::query_radius`] pergunta só `is_some`). O
            // resultado era um `l-mode` sem a lei e com o alcance, e um gate
            // comportamental não o distinguia de um `l-mode` são — medido: a
            // mutação passou nos 193.
            Self::L => verb.elastic_field(),
            Self::S | Self::B => None,
        }
    }

    /// Os modos que o painel deve pintar para este verbo, na ordem do `ALL`.
    pub fn offered_for(verb: Verb) -> impl Iterator<Item = Self> {
        Self::ALL.into_iter().filter(move |m| m.declares(verb))
    }

    /// **A lei que governa ESTE verbo sob este modo** — a porta do produto.
    ///
    /// ⚠️ **[`Self::kernel`] responde outra pergunta:** *que lei este modo
    /// declara*. Ela continua pública porque os gates comparam leis entre si,
    /// mas quem esculpe pergunta por VERBO — um modo não pode governar uma
    /// ferramenta que a referência dele não tem.
    ///
    /// Um verbo que o modo não declara cai na lei da referência que o **TEM**.
    /// Hoje o único caso é o [`Verb::ClayStrips`] sob o `S`, e o destino é o
    /// `B` por ser a referência dele (`clay_strips.cc`); o `L` só é oferecido
    /// onde declara, então o caminho dele aqui é inalcançável pelo produto e
    /// existe para o `match` ser total.
    #[must_use]
    pub fn kernel_for(self, verb: Verb) -> KernelLaw {
        if self.declares(verb) {
            self.kernel()
        } else {
            Self::B.kernel()
        }
    }

    /// **COMO ESTE MODO APERTA ESTE VERBO** — a porta do puxão lateral, e a
    /// única; o [`KernelLaw`] não a carrega e o doc dele diz por quê.
    ///
    /// ⚠️ **Ela é por VERBO porque a referência é por verbo.** O Blender tem
    /// duas ferramentas nesta família e **duas leis diferentes** — o `pinch.cc`
    /// remove a componente ao longo do traço, o `crease.cc` remove a normal —, e
    /// enquanto isto era um valor só do modo o chip `B` do Pinch vestia a lei do
    /// *crease*. O preço estava medido no report do Enio antes de eu o nomear:
    /// *"Pinch em B e S bons mas idênticos ou quase idênticos"*, e a sonda
    /// `measure_pinch_family_modes` concorda — em força `1,00`, onde o `x²` do
    /// `B` é a identidade, o que sobrava entre os dois era **0,0125 r no pior
    /// vértice, 9 % do pico**, com a normal do `B` em `0,00000` exato contra
    /// `0,00129` do `S`. Dois apertos radiais separados por um arredondamento.
    ///
    /// ⚠️ **Com a lei certa em cada um, o mesmo número é `0,1342 r` — 99 % do
    /// pico, e 10,7× o que era.** Os dois chips deixaram de ser o mesmo aperto
    /// com forças diferentes e passaram a ser duas leis.
    ///
    /// ⚠️ **E a lei da referência responde de graça o que o `l-mode` tentava e
    /// piorava:** apertar para uma LINHA em vez de um PONTO não colapsa a
    /// vizinhança radialmente, então quase não remove volume — medido,
    /// **`−0,0119` contra `−0,9066`** (10⁻⁴ de `V`), **76× menos** que o `S`. O
    /// campo elástico existia para "deixar de remover volume" e removia 4,8×
    /// mais; a resposta estava na fonte o tempo todo.
    ///
    /// ⚠️ **O `Magnify` fica no `Tangential` e isso é uma AUSÊNCIA declarada:**
    /// o Blender não tem esta ferramenta (o oposto do pinch lá é o Inflate, que
    /// é outro verbo), então não há `pinch.cc` a portar e o `B` dele herda a lei
    /// da família — nomeado aqui em vez de escolhido em silêncio.
    #[must_use]
    pub const fn lateral_for(self, verb: Verb) -> LateralPull {
        // Um verbo que o modo não declara cai na lei da referência que o TEM —
        // a mesma regra do [`Self::kernel_for`], e por isso é o mesmo `if`.
        let m = if self.declares(verb) { self } else { Self::B };
        match m {
            Self::S | Self::L => LateralPull::Direct,
            Self::B => match verb {
                // `pinch.cc` — a ferramenta **Pinch** do Blender.
                Verb::Pinch => LateralPull::AcrossStroke,
                // `crease.cc:112` — as ferramentas **Crease** e **Blob**; o
                // Magnify entra aqui pela ausência que o doc acima nomeia.
                _ => LateralPull::Tangential,
            },
        }
    }

    /// A lei de kernel que este modo manda.
    ///
    /// ⚠️ **O `B` devolve o que o app JÁ shipava**, de propósito: a W3 não
    /// mudou o produto, ela tornou o `S` verdadeiro. Antes dela o app rodava um
    /// kernel que não era o de referência nenhuma em três verbos, com um chip
    /// que dizia `S` — o mesmo defeito que a W0 curou nos DEFAULTS, aqui na LEI.
    ///
    /// ⚠️ **O `L` deixou de o acompanhar na W4.** Ele partilhava este braço
    /// como FALLBACK — um `match` precisa de um braço e ele não tinha lei
    /// própria —, e era exatamente por isso que não era oferecido. Hoje ele
    /// declara, e o braço dele diz o que o paper diz.
    #[must_use]
    pub const fn kernel(self) -> KernelLaw {
        match self {
            Self::S => KernelLaw {
                plane: PlaneReach::OneSided,
                front_face: FrontFace::Ignored,
            },
            // **O `L` — a LITERATURA**, e desde a W4 ele é uma declaração e não
            // mais o fallback à lei do `B`.
            //
            // ⚠️ **`Ignored` é DECLARADO, não copiado do `S`:** o Taubin é um
            // filtro de MALHA — ele não sabe onde está a câmera, e nenhum passo
            // dele consulta uma direção de vista. Herdar o `Continuous` do `B`
            // faria trocar `S → L` mudar DUAS coisas (o par λ|μ **e** um portão
            // de face que o paper não menciona), e o artista não teria como
            // separar as duas na tela. Assim, a única diferença entre `S` e `L`
            // no Smooth é a que o chip promete.
            //
            // ⚠️ **Os outros dois eixos são INALCANÇÁVEIS por construção** — o
            // `L` só é oferecido no Smooth ([`RefMode::declares`]), que não lê
            // `lateral` nem `plane`, e a porta "aplicar a todos" só carimba onde
            // o modo declara. Eles trazem os valores do `S` porque um `match`
            // precisa de um braço; no dia em que o `L` declarar um verbo de
            // plano, este arm tem de ser reescrito **com a fonte na mão**, e é o
            // gate `the_literature_mode_is_offered_exactly_where_it_declares_a_law`
            // que torna esse dia impossível de passar em silêncio.
            Self::L => KernelLaw {
                plane: PlaneReach::OneSided,
                front_face: FrontFace::Ignored,
            },
            Self::B => KernelLaw {
                plane: PlaneReach::Bilateral,
                // ⚠️ **É o único dos três eixos em que o `B` ACRESCENTA** em vez
                // de guardar o que o app já fazia — os outros dois nasceram
                // preservando o produto, este liga uma lei que ninguém tinha.
                front_face: FrontFace::Continuous,
            },
        }
    }
}

/// **O CAMPO ELÁSTICO que um modo entrega** — qual das leis do
/// [`crate::kelvinlet`] governa o deslocamento.
///
/// ⚠️ **Ele nasceu com UM variante e a aposta pagou na wave seguinte:** a
/// alternativa — um `bool` *"usa Kelvinlet"* — não teria onde dizer QUAL, e os
/// outros três membros teriam nascido num `match` paralelo ao lado desta tabela.
///
/// ⚠️ **E os quatro NÃO são consumidos da mesma forma**, que é o achado da W5-B:
/// [`Self::Twist`] e [`Self::Scale`] entregam um **escalar**
/// ([`crate::kelvinlet::rigid_profile`]) porque o verbo que os recebe já tem a
/// geometria certa — girar sobre a circunferência, escalar ao longo do raio — e
/// somar o deslocamento linearizado a INFLARIA; [`Self::Grab`] e [`Self::Pinch`]
/// entregam o vetor, porque não há geometria de verbo a preservar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// O agarre (eq. 5): a vizinhança acompanha o bico elasticamente, e **mais
    /// à frente do puxão do que ao lado dele**.
    Grab,
    /// A torção: o barro sob o bico gira como corpo rígido e o resto acompanha.
    Twist,
    /// A dilatação radial em torno da âncora.
    Scale,
    /// O aperto de traço zero: espreme num eixo e **espirra** no outro. É o que
    /// separa um vinco de material de um encolhimento.
    Pinch,
}

#[cfg(test)]
#[path = "ref_mode_tests.rs"]
mod tests;
