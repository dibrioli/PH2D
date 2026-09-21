//! **AS ESCOLHAS NOMEADAS do painel 3D** — os enums que um chip escreve.
//!
//! Irmão do [`super::state`], e o corte foi forçado pelo teto de LOC dos painéis
//! (653 contra 600). ⭐ **Mas ele é de ASSUNTO:** lá mora *o estado inteiro que o
//! painel autora* — o pincel, os slots, o retrato, os intents —, aqui as
//! **escolhas discretas** que um chip escreve, cada uma com o seu `ALL` e o seu
//! rótulo i18n.
//!
//! ⚠️ **A lei comum às duas:** o `ALL` é a ordem em que os chips são pintados, e o
//! id de cada chip é indexado por essa posição — **nunca** pelo discriminante. Um
//! `ALL` reordenado com índices por discriminante dá um chip rotulado `Fast`
//! escrevendo `Even Grid`: pintado, vivo sob o mouse, e a mentir.

/// **COM QUE PROFUNDIDADE O PAINEL SE MOSTRA** (§2 do plano).
///
/// ⚠️ **Isto não são dois conjuntos de features — é divulgação progressiva do
/// MESMO estado**, e essa escolha é o que impede duas fontes de verdade. Em
/// `Pro` o artista não ganha números novos: ele ganha *acesso* aos números que o
/// verbo e o modo já haviam armado por ele.
///
/// ⚠️ **A regra de quem pode ser `Pro`, e ela é testável:** só uma row cujo
/// valor **o slot do verbo já traz** ([`VerbSlot::for_verb`]). Esconder um
/// número que a ferramenta escolheu bem é divulgação progressiva; esconder um
/// que nasce neutro e tem de ser fornecido é amputação — o artista ficaria com
/// uma ferramenta que não faz o que o nome dela diz e sem nada na tela
/// explicando por quê.
///
/// ⚠️ **Ela é NECESSÁRIA e não suficiente, e é isso que o falloff custou:** a
/// curva nasce no slot do verbo, logo *podia* ser `Pro` — e era, e o
/// smoke reprovou (*"não dá a opção de escolher o falloff e deveria dar"*).
/// Quem decide a segunda metade é a REFERÊNCIA, medida e não lembrada: no
/// Blender a curva é *dobrada* (`DEFAULT_CLOSED` com cabeçalho à vista, mais um
/// popover no cabeçalho de ferramenta), nunca *ausente*. **Dobrar é divulgação
/// progressiva; sumir sem rastro é amputação**, e o nosso `Pro` fazia o segundo.
///
/// ⚠️ **`Ord` é a lei inteira:** uma row aparece quando `nível do painel >=
/// nível da row`. Escrito como dois `if`s (um por lado) o terceiro degrau nasce
/// fora da regra.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiLevel {
    /// O que TODO pincel tem: o verbo, a referência, o raio, a força e a
    /// **CURVA**.
    ///
    /// ⚠️ **Isto dizia *"o vocabulário do SculptGL"* e a frase custou um
    /// smoke.** O SculptGL **não tem** seletor de curva — a dele é fixa —, então
    /// herdar o vocabulário dele apagava do Basic um controle que a nossa malha
    /// tem **doze** vezes e que a OUTRA referência trata como primeiro-classe (na
    /// interface do Blender o painel de queda não está dentro da secção
    /// *Advanced* das definições do pincel, e no cabeçalho de ferramenta ele é um
    /// popover sempre visível). *Um vocabulário herdado
    /// descreve a ferramenta de onde veio, não a que se está a construir.*
    #[default]
    Basic,
    /// Mais os knobs que o modo tinha armado.
    Pro,
}

impl UiLevel {
    /// A ordem em que os chips são pintados. **É** a ordem do enum.
    pub const ALL: [Self; 2] = [Self::Basic, Self::Pro];

    /// Chave i18n do rótulo.
    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => ph2d_i18n::tr("panel.sculpt3d.ui_level.basic"),
            Self::Pro => ph2d_i18n::tr("panel.sculpt3d.ui_level.pro"),
        }
    }

    /// **Uma coisa que exige `needs` aparece neste nível?** A porta única — o
    /// pintor a consulta para desenhar e o gate de costura para varrer.
    pub fn shows(self, needs: Self) -> bool {
        needs <= self
    }
}

/// **QUAL MOTOR DE RETOPOLOGIA o botão chama** — e os dois são de espécie
/// diferente, não um melhor e um pior.
///
/// ⭐ **Medido lado a lado, na mesma peça e no mesmo gesto** (2026-08-21, esfera
/// com bico, `detail = 0,5`):
///
/// | | quads | irregulares | bordo | relógio |
/// |---|---|---|---|---|
/// | [`Self::Global`] | **100 %** | 19 | **0** | ~330 ms |
/// | [`Self::Local`] | 63 % | não conta | 0 | **~70 ms** |
///
/// ⚠️ **O `Local` não é um modo de compatibilidade.** Ele é o porte fiel do
/// *Instant Meshes* (BSD-3, `ph2d-quadflow`), que é o clássico rápido e robusto da
/// família: a grade dele segue a curvatura, ele responde em sub-segundo e **nunca
/// precisa de um layout global fechar**. O `Global` entrega a promessa inteira —
/// 100 % de quads e a contagem de irregulares perto do chão topológico — e paga
/// isso em relógio e em recusas nomeadas quando o traçado não fecha.
///
/// ⛔ **Ele esteve escondido atrás de `PH2D_RETOPO_LEGACY=1` durante toda a wave
/// do pivô**, ou seja: alcançável só por quem soubesse o nome da variável.
/// *Um motor que o painel não oferece não existe para o artista.*
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RetopoMode {
    /// A cadeia **global** — campo cruzado, patches, quantização inteira.
    #[default]
    Global,
    /// O porte do **Instant Meshes** — local, rápido, robusto.
    Local,
}

impl RetopoMode {
    /// A ordem em que os chips são pintados. **É** a ordem do enum.
    pub const ALL: [Self; 2] = [Self::Global, Self::Local];

    /// Chave i18n do rótulo.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Global => ph2d_i18n::tr("panel.sculpt3d.retopo_mode.global"),
            Self::Local => ph2d_i18n::tr("panel.sculpt3d.retopo_mode.local"),
        }
    }

    // ⛔⛔⛔ **`uses_adaptive` FOI APAGADA em 2026-08-31, e não por limpeza cosmética: ela
    // MENTIA.** Dizia *«este motor consome a densidade adaptativa? só o local — e é por isso
    // que o painel avisa quando o knob não é zero no outro»*, e o `Follow Curvature` chega
    // **aos dois** motores desde 2026-08-21 (o comentário no sítio da chamada, em
    // `sculpt3d/panel.rs`, já o dizia). ⚠️ **E ela não tinha UM leitor** — nem o aviso que o
    // doc dela descrevia existia.
    //
    // ⚠️ **É a espécie ÓRFÃ, não a MORTA** (memória `an_orphan_id_and_a_dead_knob…`): a cura
    // de um órfão é apagar, a de um knob morto é ligar o braço — e tratá-las ao contrário
    // constrói consumidor para um controlo que não existe. ⛔ O perigo aqui era o texto: quem
    // greppasse `uses_adaptive` lia *«o `Curvature` é inerte no motor de omissão»* e concluía
    // o contrário do que o produto faz. *Uma afirmação falsa que ninguém executa continua a
    // ser lida.*
}

/// ⭐⭐⭐⭐ **A RESOLUÇÃO DA TINTA nesta peça** — os chips `Mesh` · `2×` · `4×`
/// · `8×`.
///
/// A cor deste app mora nos VÉRTICES, logo *a resolução da tinta é a da malha*
/// — e é isso que o `Mesh` diz: o caminho de sempre, **ao bit**. Os outros três
/// armam um plano de amostras (`ph2d_mesh_colors::Tinta`) com `2^k` intervalos
/// por aresta, e a tinta passa a ter resolução própria.
///
/// ⛔⛔ **Chips e não uma PISTA, e a razão não é gosto:** o domínio tem quatro
/// valores nomeados e eles são potências de dois — uma pista contínua ofereceria
/// posições que o motor arredonda, que é o *«aceita e mente»* que esta casa já
/// pagou três vezes. *Um selector discreto sobre um domínio discreto não tem
/// onde mentir.*
///
/// ⚠️ **O `Mesh` é uma OPÇÃO e não a ausência das outras** — ele é o estado de
/// fábrica, e tem de ser alcançável de volta: sem um chip para ele, armar a
/// tinta fina seria irreversível.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DetalheDaTinta {
    /// A cor tem a resolução da MALHA — o caminho de sempre, byte a byte.
    #[default]
    Malha,
    /// `2` intervalos por aresta.
    Duas,
    /// `4` intervalos por aresta.
    Quatro,
    /// `8` intervalos por aresta.
    Oito,
    /// `16` intervalos por aresta — o mais fino que o produto oferece, e o
    /// tecto é MEDIDO (ver `ph2d_app_sculpt3d::tinta_da_peca::NIVEL_MAX`).
    ///
    /// ⭐ **Ordem do dono (2026-09-20): *«acrescente a opção de 16x»***, e o
    /// degrau só entrou depois de medido — a nota que aqui estava dizia
    /// *«um degrau novo aqui mede-se antes de se escrever»*, e a medição está
    /// na tabela daquela constante.
    Dezasseis,
}

impl DetalheDaTinta {
    /// A ordem em que os chips são pintados. **É** a ordem do enum.
    pub const ALL: [Self; 5] = [
        Self::Malha,
        Self::Duas,
        Self::Quatro,
        Self::Oito,
        Self::Dezasseis,
    ];

    /// Chave i18n do rótulo.
    pub fn label(self) -> &'static str {
        match self {
            Self::Malha => ph2d_i18n::tr("panel.sculpt3d.tinta_detalhe.malha"),
            Self::Duas => ph2d_i18n::tr("panel.sculpt3d.tinta_detalhe.duas"),
            Self::Quatro => ph2d_i18n::tr("panel.sculpt3d.tinta_detalhe.quatro"),
            Self::Oito => ph2d_i18n::tr("panel.sculpt3d.tinta_detalhe.oito"),
            Self::Dezasseis => ph2d_i18n::tr("panel.sculpt3d.tinta_detalhe.dezasseis"),
        }
    }

    /// ⭐ **O `k` que o motor pede** — `None` é *«sem plano»*.
    ///
    /// ⚠️ **A ponte é AQUI e é uma só**, com gate de ida-e-volta: o painel não
    /// conhece a `ph2d-mesh-colors` e não vai passar a conhecer, e sem a volta
    /// uma ponte que colapsasse dois chips num `k` deixaria um deles morto.
    #[must_use]
    pub fn nivel(self) -> Option<u8> {
        match self {
            Self::Malha => None,
            Self::Duas => Some(1),
            Self::Quatro => Some(2),
            Self::Oito => Some(3),
            Self::Dezasseis => Some(4),
        }
    }

    /// A volta da [`Self::nivel`].
    #[must_use]
    pub fn do_nivel(nivel: Option<u8>) -> Self {
        Self::ALL
            .into_iter()
            .find(|d| d.nivel() == nivel)
            .unwrap_or_default()
    }
}
