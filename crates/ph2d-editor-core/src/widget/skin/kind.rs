//! **O CATÁLOGO** — quais tipos uma forma pode vestir, e o que cada um É.
//!
//! ⚠️ **Irmão do pintor, e o corte é de ASSUNTO:** o `skin.rs` responde *como um tipo aparece*
//! (um `match`, um braço por tipo, cada um terminando no pintor nativo); aqui mora *que tipos
//! existem* — a lista, o número que viaja no documento, o identificador que o codegen escreve, a
//! chave i18n que o artista lê, e quem consome uma cor. As duas perguntas crescem por motivos
//! diferentes: a primeira com o catálogo do design system, a segunda com o formato de arquivo.
//!
//! ⚠️ **`skin/`, e não `src/widget/`**: o `ph2d-widget-sync` varre os `*.rs` do TOPO daquela pasta
//! para gerar o bloco de `mod`, e um arquivo solto ali viraria um "widget" sem showcase e sem
//! a11y — a cicatriz do `command_palette`.

/// **Que widget do catálogo esta forma veste.**
///
/// ⚠️ O código de cada variante é um **literal explícito**, nunca a ordem do enum: o número viaja
/// no documento, e reordenar o enum re-pintaria em silêncio toda arte já salva. Acrescentar um
/// tipo é acrescentar um código novo no fim — nunca reciclar um.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WidgetKind {
    Button,
    Toggle,
    Checkbox,
    Slider,
    ProgressBar,
    Tag,
    TextInput,
    Card,
    SectionHeader,
    ListItem,
    Spinner,
    Divider,
    NumberInput,
    LevelMeter,
    ColorSwatch,
    IconButton,
    /// Uma faixa de abas — N opções, uma marcada.
    Tabs,
    /// Um grupo de rádio — N opções, uma marcada.
    RadioGroup,
    /// Uma segmentada que se adapta à largura — N opções, uma marcada.
    SegmentedAdaptive,
    /// Um select — N opções, uma marcada, e as outras **escondidas até se pedir**.
    Dropdown,
}

impl WidgetKind {
    /// Todos os tipos que uma forma pode vestir hoje, na ordem em que o painel os oferece.
    ///
    /// # A fronteira é ESTRUTURAL, não um orçamento — e o teste dela ficou mais FINO
    ///
    /// ⚠️ **A primeira formulação desta lei era *"tem um `Vec`?"*, e ela é NECESSÁRIA mas não
    /// SUFICIENTE.** O levantamento de 2026-08-08 contou os quatro tipos ausentes que não têm
    /// `Vec` — `ColorSwatch`, `NumberInput`, `IconButton`, `LevelMeter` — e chamou aos quatro
    /// *omissão de fiação*. Medido campo a campo, dois deles pedem um parâmetro que **nem o
    /// desenho, nem os tokens, nem o estado vivo determinam**: a `rgba` de uma swatch (o valor que
    /// ela existe para mostrar) e o `IconGlyph` de um botão de ícone (*qual* ícone?).
    ///
    /// A lei, então, é: **todo parâmetro tem de ser determinado pelo retângulo, pelo rótulo, pelos
    /// tokens ou pelo estado vivo.** Sob ela os ausentes se separam em três, não em dois:
    ///
    /// | natureza | tipos | o que falta |
    /// |---|---|---|
    /// | vestível **hoje** | os dezanove desta lista | nada |
    ///
    /// ⚠️ **A tabela ESVAZIOU-SE, e é isso que o canal existia para fazer.** Os dois que pediam um
    /// parâmetro por-tipo — a `ColorSwatch` e o `IconButton` — entraram pelo
    /// [`SkinParam`](super::SkinParam), um campo com neutro cada; e a família da LISTA
    /// (`Tabs` · `RadioGroup` · `SegmentedAdaptive` · `Dropdown`) entrou pelo MESMO canal, com a
    /// lista a vir dos FILHOS que o artista desenhou (ver [`Self::takes_options`]).
    ///
    /// ⚠️ **A premissa que a família da LISTA derrubou está escrita aqui de propósito:** o
    /// levantamento de 2026-08-08 chamou-lhe *estrutural*, por oposição a *omissão de fiação*, e a
    /// leitura implícita era que o canal de side-metadata não a alcançava. Nada no molde do canal
    /// exige que o campo seja **pequeno** — `options: &[String]` é um campo com neutro (a lista
    /// vazia) exactamente como `rgba: Option<_>` é.
    pub const ALL: [WidgetKind; 20] = [
        WidgetKind::Button,
        WidgetKind::Toggle,
        WidgetKind::Checkbox,
        WidgetKind::Slider,
        WidgetKind::ProgressBar,
        WidgetKind::Tag,
        WidgetKind::TextInput,
        WidgetKind::Card,
        WidgetKind::SectionHeader,
        WidgetKind::ListItem,
        WidgetKind::Spinner,
        WidgetKind::Divider,
        WidgetKind::NumberInput,
        WidgetKind::LevelMeter,
        WidgetKind::ColorSwatch,
        WidgetKind::IconButton,
        WidgetKind::Tabs,
        WidgetKind::RadioGroup,
        WidgetKind::SegmentedAdaptive,
        WidgetKind::Dropdown,
    ];

    /// O código que viaja no documento. Estável para sempre.
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            WidgetKind::Button => 1,
            WidgetKind::Toggle => 2,
            WidgetKind::Checkbox => 3,
            WidgetKind::Slider => 4,
            WidgetKind::ProgressBar => 5,
            WidgetKind::Tag => 6,
            WidgetKind::TextInput => 7,
            WidgetKind::Card => 8,
            WidgetKind::SectionHeader => 9,
            WidgetKind::ListItem => 10,
            WidgetKind::Spinner => 11,
            WidgetKind::Divider => 12,
            WidgetKind::NumberInput => 13,
            WidgetKind::LevelMeter => 14,
            WidgetKind::ColorSwatch => 15,
            WidgetKind::IconButton => 16,
            WidgetKind::Tabs => 17,
            WidgetKind::RadioGroup => 18,
            WidgetKind::SegmentedAdaptive => 19,
            WidgetKind::Dropdown => 20,
        }
    }

    /// **Este tipo CONSOME uma cor?** — a porta única do canal [`SkinParam::rgba`].
    ///
    /// ⚠️ Ela é perguntada por quem MONTA o parâmetro (o construtor de spec do painel gerado, o
    /// construtor de peles do canvas), nunca por quem pinta: o braço de pintura já sabe o que
    /// desenha. Sem ela, os dois construtores leriam o preenchimento de TODA forma vestida — e o
    /// dia em que um segundo tipo passasse a mostrar a cor, o `Slider` mudaria de tinta junto,
    /// porque ninguém teria escrito onde a fronteira estava.
    ///
    /// ⚠️ **Um tipo por enquanto, e a lista é o ponto.** *"É só a swatch"* escrito num `if` no
    /// chamador é a regra que o segundo chamador nasce sem.
    #[must_use]
    pub const fn takes_colour(self) -> bool {
        matches!(self, WidgetKind::ColorSwatch)
    }

    /// **Este tipo CONSOME um ícone?** — a porta única do canal [`SkinParam::icon`].
    ///
    /// Irmã exata de [`Self::takes_colour`], perguntada pelos mesmos dois CONSTRUTORES e nunca
    /// pelo pintor. Ler a geometria de toda forma vestida custaria uma normalização por-quadro
    /// por-widget para um glifo que catorze dos dezasseis tipos não desenham.
    ///
    /// [`SkinParam::icon`]: super::SkinParam::icon
    #[must_use]
    pub const fn takes_icon(self) -> bool {
        matches!(self, WidgetKind::IconButton)
    }

    /// **Este tipo é feito de OPÇÕES?** — a família de LISTA.
    ///
    /// ⚠️ **É a pergunta que muda a leitura da ÁRVORE, e não só a da pele.** Um controle de
    /// opções não tem um valor: ele tem N rótulos e um índice marcado, e o lugar nativo desses
    /// rótulos num editor vetorial são os **FILHOS que o artista desenhou** — a árvore já
    /// exprime contenção, e ele já os nomeia na Hierarquia.
    ///
    /// A consequência é uma LEI de posse: **um controle que toma opções POSSUI os seus filhos**,
    /// então a varredura não pode descê-los como rows. Sem ela, desenhar três abas dentro de uma
    /// faixa daria uma faixa **e mais três linhas soltas** no painel, cada uma um controle que o
    /// artista não pediu.
    #[must_use]
    pub const fn takes_options(self) -> bool {
        matches!(
            self,
            WidgetKind::Tabs
                | WidgetKind::RadioGroup
                | WidgetKind::SegmentedAdaptive
                | WidgetKind::Dropdown
        )
    }

    /// **Este tipo esconde as opções até se pedir?** — a porta única da SEGUNDA superfície.
    ///
    /// ⚠️ **É a pergunta que divide a família de LISTA em duas, e a divisão é de LAYOUT, não de
    /// gosto.** Uma faixa de abas, um grupo de rádio e uma segmentada desenham as N opções DENTRO
    /// do retângulo que lhes foi dado — quem os pinta acabou o trabalho quando a função retorna.
    /// Um dropdown desenha **um chip** e guarda a lista para uma superfície que tem de aparecer
    /// **por cima de tudo o que for pintado depois dela**.
    ///
    /// Isso torna-o o único do catálogo que um painel não consegue pintar num passe só: o corpo é
    /// recortado (senão as rows escapavam pelo cabeçalho), e uma lista aberta na última row seria
    /// **cortada pelo próprio recorte** — ou, sem recorte, desenhada por baixo das rows seguintes.
    /// Quem responde `true` aqui exige do painel um **passe diferido**, e é por isso que a pergunta
    /// tem nome em vez de ser um `matches!` escrito dentro do laço de pintura.
    ///
    /// ⚠️ **A pele NUNCA o pinta aberto** — nem no painel, nem na prévia do canvas. Ela desenha o
    /// chip; a lista é do passe de quem tem `hit_index` para registar as opções, que a prévia não
    /// tem e (§2 do plano) não deve ter.
    #[must_use]
    pub const fn defers_a_popover(self) -> bool {
        matches!(self, WidgetKind::Dropdown)
    }

    /// A tradução de volta. **`None` é o caso que este canal existe para suportar**: um documento
    /// autorado por um build mais novo carrega um código que este não conhece, e a resposta certa
    /// é *desenhe a forma* — nunca um retângulo vazio, e nunca recusar o arquivo.
    #[must_use]
    pub fn from_code(code: u16) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.code() == code)
    }

    /// **O IDENTIFICADOR do variante, tal como o Rust o escreve** — o que o codegen do W8b põe no
    /// código gerado (`WidgetKind::Slider`).
    ///
    /// ⚠️ **Ela mora aqui porque o dono da lista é quem pode respondê-la.** O gerador
    /// (`ph2d-ui-codegen`) **não depende** desta crate de propósito: sem alcance ao catálogo ele
    /// não CONSEGUE ter opinião sobre o que um `Slider` é, e uma tabela `código → nome` do lado
    /// dele seria uma segunda resposta que driftaria no dia em que um tipo entrasse — em silêncio,
    /// porque um número desconhecido não falha, ele só não casa.
    ///
    /// ⚠️ **É o identificador, NÃO um rótulo:** ele atravessa para dentro de código-fonte, então
    /// traduzi-lo ou embelezá-lo produziria um arquivo que não compila. O nome que o artista lê é
    /// o [`Self::i18n_key`].
    #[must_use]
    pub const fn ident(self) -> &'static str {
        match self {
            WidgetKind::Button => "Button",
            WidgetKind::Toggle => "Toggle",
            WidgetKind::Checkbox => "Checkbox",
            WidgetKind::Slider => "Slider",
            WidgetKind::ProgressBar => "ProgressBar",
            WidgetKind::Tag => "Tag",
            WidgetKind::TextInput => "TextInput",
            WidgetKind::Card => "Card",
            WidgetKind::SectionHeader => "SectionHeader",
            WidgetKind::ListItem => "ListItem",
            WidgetKind::Spinner => "Spinner",
            WidgetKind::Divider => "Divider",
            WidgetKind::NumberInput => "NumberInput",
            WidgetKind::LevelMeter => "LevelMeter",
            WidgetKind::ColorSwatch => "ColorSwatch",
            WidgetKind::IconButton => "IconButton",
            WidgetKind::Tabs => "Tabs",
            WidgetKind::RadioGroup => "RadioGroup",
            WidgetKind::SegmentedAdaptive => "SegmentedAdaptive",
            WidgetKind::Dropdown => "Dropdown",
        }
    }

    /// A chave i18n do nome deste tipo, para o painel.
    #[must_use]
    pub const fn i18n_key(self) -> &'static str {
        match self {
            WidgetKind::Button => "panel.vector.widget.kind.button",
            WidgetKind::Toggle => "panel.vector.widget.kind.toggle",
            WidgetKind::Checkbox => "panel.vector.widget.kind.checkbox",
            WidgetKind::Slider => "panel.vector.widget.kind.slider",
            WidgetKind::ProgressBar => "panel.vector.widget.kind.progress",
            WidgetKind::Tag => "panel.vector.widget.kind.tag",
            WidgetKind::TextInput => "panel.vector.widget.kind.text_input",
            WidgetKind::Card => "panel.vector.widget.kind.card",
            WidgetKind::SectionHeader => "panel.vector.widget.kind.section",
            WidgetKind::ListItem => "panel.vector.widget.kind.list_item",
            WidgetKind::Spinner => "panel.vector.widget.kind.spinner",
            WidgetKind::Divider => "panel.vector.widget.kind.divider",
            WidgetKind::NumberInput => "panel.vector.widget.kind.number_input",
            WidgetKind::LevelMeter => "panel.vector.widget.kind.level_meter",
            WidgetKind::ColorSwatch => "panel.vector.widget.kind.color_swatch",
            WidgetKind::IconButton => "panel.vector.widget.kind.icon_button",
            WidgetKind::Tabs => "panel.vector.widget.kind.tabs",
            WidgetKind::RadioGroup => "panel.vector.widget.kind.radio_group",
            WidgetKind::SegmentedAdaptive => "panel.vector.widget.kind.segmented_adaptive",
            WidgetKind::Dropdown => "panel.vector.widget.kind.dropdown",
        }
    }
}
