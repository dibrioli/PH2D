//! O **retrato** que o shell publica a cada quadro, e os **intents** que o painel devolve.
//!
//! # Por que intents, e não `ToolPanelEvent`
//!
//! Todo painel acoplado a uma ferramenta encaminha `EditorAction::ToolPanelEvent`. Este edita um
//! **documento**, não uma ferramenta — e não há tool para onde encaminhar. Inventar uma para o pipe
//! existente encaixar seria uma tool que não é uma tool, e ainda por cima mexeria no `Tool=12`, que
//! está congelado.
//!
//! Segue-se então o painel que já resolveu isto: o de **física** (ADR-0131 D8) empurra intents numa
//! fila e o shell drena-as. *O shell continua a ser a única coisa que toca no documento.*

use ph2d_field::Bound;
use std::cell::{Cell, RefCell};

thread_local! {
    static CURRENT: RefCell<Option<ModelSnapshot>> = const { RefCell::new(None) };
    static INTENTS: RefCell<Vec<ModelIntent>> = const { RefCell::new(Vec::new()) };
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Uma linha do painel: **uma dimensão do objeto selecionado**.
#[derive(Clone, Debug, PartialEq)]
pub struct ParamRow {
    /// **A entidade** — a identidade da linha, e o que o intent devolve ao shell.
    ///
    /// ⚠️ Bits de entidade, e não o índice do nó na arena cozida. O índice muda quando a árvore
    /// muda (apagar um filho renumera tudo acima dele), e um controle a meio de um arrasto passaria
    /// a escrever noutro nó — em silêncio, porque um índice válido nunca parece errado.
    ///
    /// ⚠️ **Os ids dos widgets NÃO saem daqui**: eles vêm da POSIÇÃO da linha na lista, que é o que
    /// o `populate` consegue cunhar às cegas (ver `MAX_ROWS`). A posição escolhe o controle; a
    /// entidade escolhe o nó. São duas perguntas e têm duas respostas.
    pub entity: u64,
    /// **Que número daquele nó** — posição, escala ou uma dimensão da forma.
    ///
    /// ⚠️ Um índice cru serviria, com o painel a saber que «0..2 é a posição e o resto são
    /// dimensões». Uma convenção implícita entre duas crates sobrevive até alguém acrescentar uma
    /// linha no meio — e aí o controle escreve noutro número, em silêncio. Ver
    /// [`ph2d_field::Param`].
    pub param: ph2d_field::Param,
    /// A chave i18n do NOME desta dimensão (`field.dim.*`).
    ///
    /// ⚠️ Uma **chave**, nunca um rótulo pronto: HR-15. Quem traduz é o painel.
    pub key: &'static str,
    pub value: f32,
    /// ⭐ **O piso da linha** — e ele existe porque nem toda grandeza começa em zero.
    ///
    /// ⚠️ **Este campo é uma correção.** Sem ele o painel punha zero em todas as linhas, e uma
    /// **posição** negativa era impossível de digitar: o espelho do controle reescrevia `-0,5` para
    /// `0` e a peça saltava para a origem, sem mensagem nenhuma. Um número que a UI recusa em
    /// silêncio é a pior forma de recusa — e ela sobreviveu a um smoke porque o valor experimentado
    /// era positivo.
    pub lo: f32,
    /// ⭐ **Esta linha pode ser mexida agora?** `false` ⇒ ela é pintada como um **facto** (rótulo e
    /// número), sem slider, sem campo e **sem entrada no índice de acerto**.
    ///
    /// ⚠️ *Uma affordance que não pode ser honrada é pior do que nenhuma* — é a lei que este arquivo
    /// já aplica ao texto puro e à fileira de operações vazia. O caso de hoje é o terceiro ângulo na
    /// trava de cardan, onde ele deixa de ser um eixo independente
    /// ([`ph2d_field::xform::rotation_axis_is_free`], que é a **mesma** porta que recusa a escrita).
    ///
    /// ⛔ **E não é «esconder a linha»**: o valor continua a ser um facto que o artista precisa de
    /// ler, e uma linha que aparece e desaparece faria o painel saltar de tamanho a cada travessia.
    pub live: bool,
    /// ⭐ **O número desta linha é INTEIRO** — quantas cópias, e não quanto.
    ///
    /// ⚠️ Três coisas mudam de uma vez, e é por isso que é um campo e não uma dedução do valor: o
    /// passo do arrasto é **1** (e não um centésimo do curso), o número mostra-se **sem casas** (não
    /// existe meia cópia), e o piso é **1**. Deduzir *"parece inteiro, logo é"* daria uma linha que
    /// muda de comportamento quando o valor calha em `3,0`.
    pub integral: bool,
    /// Até onde ele vai, e **de que natureza é o limite**.
    ///
    /// ⚠️ [`Bound::Hard`] é a **parede do documento** (um filete que não cabe); [`Bound::Soft`] é o
    /// alcance do **gesto**, que a vista escolhe — uma largura de caixa não tem teto físico, e
    /// inventar um seria escrever um limite que a física não pede; [`Bound::Wrap`] é a própria
    /// **representação** — meia volta, num ângulo.
    pub bound: Bound,
    /// ⭐⭐⭐ **A SECÇÃO a que esta linha pertence** — a chave i18n de um cabeçalho a desenhar
    /// **antes** dela, ou `None` para continuar a secção anterior (2026-08-30).
    ///
    /// # ⛔ O report que a obrigou
    ///
    /// Enio: *«nada aparece torcido, apesar dos sliders terem algum efeito. O modificador deveria
    /// ter sua própria seção no painel»*. As linhas de um modificador eram **acrescentadas ao fim da
    /// mesma lista** das dimensões da forma, sem nada a separá-las: com uma casca e uma torção
    /// empilhadas, o painel mostrava seis números seguidos e **nenhum dizia de quem era**.
    ///
    /// ⚠️ *Um controle que o artista não consegue atribuir é um controle que ele não usa* — e o
    /// sintoma lê-se exactamente como uma feature partida: ele arrasta o que julga ser o da torção,
    /// vê outra coisa mudar, e conclui que a torção não funciona.
    pub section: Option<&'static str>,
    /// ⭐⭐⭐ **OS NOMES DE UMA ESCOLHA** — vazio ⇒ a linha é um número (Enio, 2026-08-31).
    ///
    /// Quando não é vazio, o valor da linha é o **índice** escolhido e estas são as chaves i18n dos
    /// nomes, na ordem. O painel pinta uma fileira de botões em vez de um slider, e o índice do que
    /// for premido é o que o intent leva.
    ///
    /// ⚠️ **Chaves, nunca rótulos** (HR-15) — a mesma lei do [`ParamRow::key`].
    ///
    /// ⛔ **Ela substitui o slider, não o acompanha:** um eixo com slider *e* botões seria o mesmo
    /// facto em dois controlos, e os dois podem discordar. Ver [`ph2d_field::Span::Choice`].
    pub choices: &'static [&'static str],
}

/// Um verbo que o gizmo oferece: a chave i18n do rótulo, e se ele é o ativo.
///
/// ⚠️ O painel **não conhece o enum dos modos** — ele vive no shell, com o gizmo. Aqui chega uma
/// lista, e o intent devolve a POSIÇÃO. É o que mantém a contagem de verbos numa fonte só
/// (`Mode::ALL`): acrescentar um lá faz o painel seguir sem uma linha de mudança.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModeChip {
    pub key: &'static str,
    pub active: bool,
}

/// O que o painel precisa de saber sobre o modelo neste quadro.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelSnapshot {
    /// ⭐ Os verbos do gizmo, na ordem do seletor. Vazio ⇒ o seletor não é pintado.
    pub modes: Vec<ModeChip>,
    /// ⭐ Os referenciais de eixo (global / local), na mesma forma e pela mesma razão.
    pub frames: Vec<ModeChip>,
    /// ⭐⭐ **O que o LAÇO faz ao que apanha** (W112) — somar à selecção, ou tirar dela.
    ///
    /// ⛔ **É um chip e não um modificador, e a lista das quatro saídas está medida** (doc 06
    /// §79.3): o `Alt` é roubado pelo gestor de janelas, o arrasto com o botão direito **é** o
    /// Orbit, e um modificador lido a meio troca o verbo de um gesto já começado. O chip é o único
    /// sem colisão — e o único **descobrível**, que é o que decide num módulo cujo dono aprende as
    /// ferramentas ao usá-las.
    ///
    /// ⚠️ Vazio ⇒ não é pintado.
    pub selects: Vec<ModeChip>,
    /// ⭐ **A porta de criar** — hoje **um** chip, que abre a paleta de formas (W100).
    ///
    /// ⚠️ Continua a ser uma fileira e não um `bool`, e não é indecisão: o `paint_chips` já sabe
    /// desenhar, medir e registar uma fileira, e um botão avulso seria um sexto caminho de pintura
    /// neste arquivo. Nenhum fica «ativo»: é uma ação, não um modo.
    pub adds: Vec<ModeChip>,
    /// ⭐ As operações booleanas.
    ///
    /// ⚠️ **Vazio quando não há o que combinar**, e é de propósito: um controle que aparece e não faz
    /// nada é pior do que um que não aparece. Ele mostra-se quando uma operação está selecionada
    /// (e aí o ativo diz qual ela é) ou quando há dois nós irmãos escolhidos.
    pub ops: Vec<ModeChip>,
    /// ⭐⭐⭐ **O VERBO DA FORMA ESCOLHIDA** — com que operação ela dobra sobre o resultado das
    /// anteriores (`Inherit` · `Add` · `Subtract` · `Intersect`).
    ///
    /// ⚠️ **Não é a fileira [`Self::ops`] outra vez, e o sujeito é a diferença:** aquela é a
    /// operação **do grupo**, que passa a ser o *padrão* de quem não se pronunciou; esta é a escolha
    /// **desta forma**. É por isso que ela vem com o [`Self::verb_subject`] — a fileira **nomeia o
    /// próprio sujeito**, que é a cura que o vetorial pagou em 2026-08-22 quando o artista escolheu
    /// o verbo sem saber de qual forma o painel falava.
    ///
    /// ⚠️ **O primeiro chip é `Inherit`**, e é ele que faz o modelo caber na fileira: sem um gesto
    /// que devolva a forma à herança, escolher um verbo uma vez seria irreversível.
    ///
    /// ⚠️ Vazio ⇒ não é pintada: a base não dobra sobre nada, e a raiz da peça não tem irmãos.
    pub verbs: Vec<ModeChip>,
    /// O **nome** da forma de que a fileira [`Self::verbs`] fala. `None` ⇒ não há fileira.
    pub verb_subject: Option<String>,
    /// ⭐⭐⭐ **O CARÁCTER da mistura** — `Fillet` · `Chamfer` · `Organic` (W99).
    ///
    /// ⚠️ **Três e não quatro:** a aresta **viva** não é um carácter, é o **raio zero** — e o slider
    /// do raio já o exprime. Um quarto chip seria uma segunda porta para o mesmo facto.
    ///
    /// ⚠️ Ela aparece onde há **mistura**: numa operação (o carácter do filete dela, que é o padrão
    /// dos filhos calados) e numa forma que se junta ao resto. Vazio ⇒ não é pintada.
    pub characters: Vec<ModeChip>,
    /// ⭐ **Os modificadores** — casca e afastamento. São **interruptores**: `active` diz que o nó
    /// já tem um daquela natureza, e clicar tira-o.
    ///
    /// ⚠️ Interruptor e não ação, ao contrário das formas: um modificador é um **estado** do nó, e
    /// um botão que só acrescenta deixaria o artista sem forma de o tirar a não ser desfazendo.
    pub mods: Vec<ModeChip>,
    /// ⭐ **Sair para um arquivo**, por nível de resolução.
    ///
    /// ⚠️ Os três são **ações**, não um modo: nenhum fica aceso. Um seletor de qualidade guardado
    /// obrigaria o artista a lembrar em que ficou, e a resposta certa está na peça que ele tem à
    /// frente — não numa preferência de ontem.
    pub exports: Vec<ModeChip>,
    /// ⭐ As ações sobre o objeto escolhido (duplicar, apagar). Vazio quando não há nenhum — e aí a
    /// fileira não é pintada, pela mesma razão da de operações.
    pub acts: Vec<ModeChip>,
    /// ⭐⭐ **AS SEIS VISTAS NOMEADAS** (W47) — frente, trás, direita, esquerda, topo, base.
    ///
    /// ⚠️ O `active` é **derivado da orientação da câmera**, não de um modo guardado: um arrasto de
    /// um pixel já solta a vista, e o chip tem de apagar com ele. Um espelho de estado ficaria aceso
    /// sobre uma vista que já não é aquela.
    ///
    /// ⚠️ **Sempre pintadas**, ao contrário das fileiras que dependem da seleção: olhar a peça de
    /// frente não precisa de nada escolhido.
    pub views: Vec<ModeChip>,
    /// ⭐ **Os gestos de câmera que não são uma vista** (W47): a **lente** (convergente/paralela) e o
    /// **enquadrar**.
    ///
    /// ⚠️ Eles existiam só como TECLAS (`Numpad5`, `Home`) — isto é, para quem já sabia que existem.
    /// A lei da casa (W34) diz que o painel oferece exatamente o que o gesto faz, e a **câmera**
    /// nunca tinha passado por ela.
    pub camera: Vec<ModeChip>,
    /// ⭐ **As dimensões do objeto selecionado.** Vazio quando não há nada selecionado — e aí o
    /// painel diz-lo, em vez de mostrar uma lista de tudo que ninguém pediu.
    ///
    /// ⚠️ **Mudou de significado na W10** (antes era *uma linha por nó, com o raio dele*). A divisão
    /// passou a ser a da casa: a **Hierarquia** mostra a estrutura, o **painel** mostra os números
    /// do que está escolhido. Uma lista de todos os nós competia com a Hierarquia e não tinha onde
    /// pôr as outras dimensões.
    pub rows: Vec<ParamRow>,
    /// ⭐ **O nome do nó ISOLADO, se houver** (W44) — `None` quando se vê a peça inteira.
    ///
    /// ⚠️ **Ele é publicado independentemente da SELEÇÃO, e essa é a correção.** Até aqui o único
    /// sinal de isolamento era o `active` do chip da fileira de ações, e ele comparava o nó isolado
    /// com o **escolhido**: isolar `A` e depois escolher `B` apagava o chip, e nada na tela dizia
    /// que metade da peça estava fora de vista por decisão de alguém. Pior, a fileira inteira
    /// desaparece quando o escolhido não se destaca da peça (a **raiz**, ou nada) — e aí não havia
    /// indicador nenhum.
    ///
    /// *Um estado da VISTA não se pode anunciar através de um controle da SELEÇÃO.*
    ///
    /// ⚠️ É o **nome** e não um `bool`: *"estás a ver só uma parte"* deixa o artista à procura de
    /// qual; o nome é o que ele reconhece na Hierarquia.
    pub isolated: Option<String>,
    /// Quantos nós o documento tem **ao todo** — inclusive os sem raio.
    ///
    /// ⚠️ Ele existe para o rodapé poder dizer *"8 nós, 3 com raio"* em vez de deixar o artista
    /// concluir que o resto do modelo desapareceu.
    pub node_count: usize,
    /// Quanto custou o último traçado, em milissegundos.
    ///
    /// ⭐ É o número que responde *"isto ainda é interativo?"*, e é por isso que ele fica **no
    /// painel** e não só no terminal: quem mexe num raio é quem paga o traçado seguinte.
    pub last_trace_ms: f32,
    /// ⭐⭐ **A CHAVE i18n do rótulo da vista actual** (`Front`, `User`, …) — a face do pulldown da
    /// área (ver [`crate::area_bar`]).
    ///
    /// ⚠️ **É uma LEITURA, e é derivada da CÂMERA**, nunca do último botão apertado: orbitar a
    /// vista de cima faz dela *User*, que é o que ela passou a ser. Quem a resolve é o shell
    /// (`field3d_views::label_key`), porque só ele tem a câmera.
    ///
    /// ⚠️ Vazia por omissão — um retrato que ninguém publicou não tem vista para nomear, e o
    /// `area_bar` cai num rótulo genérico em vez de pedir `tr("")`.
    pub view_label: &'static str,
}

/// Uma edição que o painel pede e o shell executa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModelIntent {
    /// Escrever uma dimensão do nó — a **posição** dela na lista do documento, e o valor.
    SetParam {
        entity: u64,
        param: ph2d_field::Param,
        value: f32,
    },
    /// Trocar o verbo do gizmo, pela **posição** no seletor.
    SetGizmoMode { slot: usize },
    /// Trocar o referencial dos eixos, pela **posição** no seletor.
    SetGizmoFrame { slot: usize },
    /// ⭐⭐ **Trocar o que o LAÇO faz ao que apanha** (W112), pela **posição** no seletor.
    SetLassoMode { slot: usize },
    /// ⭐⭐⭐ **ABRIR a paleta de formas** (W100) — e repare que ele **não traz forma nenhuma**.
    ///
    /// ⚠️ Ele substituiu um `AddShape { slot }`, e a diferença é a que faz o catálogo poder
    /// crescer: a fileira de chips corta em `MAX_MODES` (8) e já tinha 8, então a forma nº 9 seria
    /// pintada e morta. Quem escolhe passa a ser a paleta genérica da casa (a mesma do `Ctrl+K` e
    /// do `+` do Inspector), que tem busca, categorias e rolagem — e o pick volta pelo canal dela,
    /// não por aqui. *Um painel de dock não é sítio para um catálogo de 60 itens.*
    OpenShapes,
    /// Aplicar uma operação booleana ao que está selecionado, pela **posição** no seletor.
    ApplyOp { slot: usize },
    /// ⭐⭐ **Escrever o verbo da forma escolhida**, pela **posição** no seletor — e a posição `0` é
    /// o `Inherit`, que **apaga** o verbo em vez de escrever um.
    SetVerb { slot: usize },
    /// ⭐⭐ **Trocar o CARÁTER da mistura**, pela **posição** no seletor — e o **número não se mexe**.
    SetCharacter { slot: usize },
    /// Liga ou desliga um modificador do nó, pela **posição** no seletor.
    ToggleMod { slot: usize },
    /// Escrever a peça num arquivo, pela **posição** no seletor de resolução.
    Export { slot: usize },
    /// Uma ação sobre o objeto escolhido, pela **posição** no seletor.
    Act { slot: usize },
    /// ⭐ **Pôr a câmera numa vista nomeada** (W47), pela **posição** no seletor.
    SetView { slot: usize },
    /// ⭐ **Um gesto de CÂMERA que não é uma vista** (W47) — a lente, e o enquadrar.
    Camera { slot: usize },
}

/// O shell publica o retrato antes de pintar.
pub fn publish(snapshot: ModelSnapshot) {
    CURRENT.with(|c| *c.borrow_mut() = Some(snapshot));
}

/// O que o painel lê. Vazio até o primeiro `publish` — e um modelo vazio é uma resposta legítima
/// (é o que uma cena sem peça reportaria de qualquer forma).
#[must_use]
pub fn current() -> ModelSnapshot {
    CURRENT.with(|c| c.borrow().clone().unwrap_or_default())
}

/// A mesma porta, aberta para quem **testa a ponte do shell** — que é o único consumidor externo
/// legítimo: um gate da costura tem de poder encenar o que o painel faria, e o caminho real
/// (arrastar um widget) não existe fora de um app.
///
/// ⚠️ Fora de teste, quem empurra é o `apply_event` deste painel e mais ninguém.
pub fn push_intent_for_test(intent: ModelIntent) {
    push_intent(intent);
}

pub(crate) fn push_intent(intent: ModelIntent) {
    INTENTS.with(|q| q.borrow_mut().push(intent));
}

/// O shell drena as edições uma vez por quadro.
#[must_use]
pub fn drain_intents() -> Vec<ModelIntent> {
    INTENTS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

pub(crate) fn set_last_content_h(h: f32) {
    LAST_CONTENT_H.with(|c| c.set(h));
}

/// A altura que o conteúdo ocupou — o shell usa-a para dimensionar o encaixe.
#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(Cell::get)
}

/// Estado retido do painel. Vazio: tudo o que ele mostra é do documento, e um espelho local seria
/// uma segunda verdade a divergir.
#[derive(Default)]
pub struct Model3dPanelState;
