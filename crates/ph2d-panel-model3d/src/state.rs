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
    /// **Qual dimensão daquele nó** — a posição na lista que o documento devolve.
    ///
    /// ⚠️ A posição, e não o nome: é o que um controle cunhado às cegas consegue guardar, e é o que
    /// a porta de escrita recebe. Reordenar a lista do documento reordena os controles.
    pub index: usize,
    /// A chave i18n do NOME desta dimensão (`field.dim.*`).
    ///
    /// ⚠️ Uma **chave**, nunca um rótulo pronto: HR-15. Quem traduz é o painel.
    pub key: &'static str,
    pub value: f32,
    /// Até onde ele vai, e **de que natureza é o limite**.
    ///
    /// ⚠️ [`Bound::Hard`] é a **parede do documento** (um filete que não cabe); [`Bound::Soft`] é o
    /// alcance do **gesto**, que a vista escolhe — uma largura de caixa não tem teto físico, e
    /// inventar um seria escrever um limite que a física não pede.
    pub bound: Bound,
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
    /// ⭐ As formas que se podem **acrescentar**. ⚠️ Nenhuma fica «ativa»: são ações, não um modo —
    /// clicar numa cria uma forma e o seletor volta ao mesmo sítio.
    pub adds: Vec<ModeChip>,
    /// ⭐ As operações booleanas.
    ///
    /// ⚠️ **Vazio quando não há o que combinar**, e é de propósito: um controle que aparece e não faz
    /// nada é pior do que um que não aparece. Ele mostra-se quando uma operação está selecionada
    /// (e aí o ativo diz qual ela é) ou quando há dois nós irmãos escolhidos.
    pub ops: Vec<ModeChip>,
    /// ⭐ **As dimensões do objeto selecionado.** Vazio quando não há nada selecionado — e aí o
    /// painel diz-lo, em vez de mostrar uma lista de tudo que ninguém pediu.
    ///
    /// ⚠️ **Mudou de significado na W10** (antes era *uma linha por nó, com o raio dele*). A divisão
    /// passou a ser a da casa: a **Hierarquia** mostra a estrutura, o **painel** mostra os números
    /// do que está escolhido. Uma lista de todos os nós competia com a Hierarquia e não tinha onde
    /// pôr as outras dimensões.
    pub rows: Vec<ParamRow>,
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
}

/// Uma edição que o painel pede e o shell executa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModelIntent {
    /// Escrever uma dimensão do nó — a **posição** dela na lista do documento, e o valor.
    SetParam {
        entity: u64,
        index: usize,
        value: f32,
    },
    /// Trocar o verbo do gizmo, pela **posição** no seletor.
    SetGizmoMode { slot: usize },
    /// Trocar o referencial dos eixos, pela **posição** no seletor.
    SetGizmoFrame { slot: usize },
    /// Acrescentar uma forma à peça, pela **posição** no seletor.
    AddShape { slot: usize },
    /// Aplicar uma operação booleana ao que está selecionado, pela **posição** no seletor.
    ApplyOp { slot: usize },
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
