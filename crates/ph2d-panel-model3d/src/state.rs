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

use ph2d_field::RadiusBound;
use std::cell::{Cell, RefCell};

thread_local! {
    static CURRENT: RefCell<Option<ModelSnapshot>> = const { RefCell::new(None) };
    static INTENTS: RefCell<Vec<ModelIntent>> = const { RefCell::new(Vec::new()) };
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Uma linha do painel: **um nó do documento com raio editável**.
#[derive(Clone, Debug, PartialEq)]
pub struct RadiusRow {
    /// O índice do nó na arena — a identidade da linha, e de onde saem os ids dos widgets.
    pub node: u32,
    /// A chave i18n do que este nó é (`panel.model3d.kind.*`).
    ///
    /// ⚠️ Uma **chave**, nunca um rótulo pronto: HR-15. Quem traduz é o painel.
    pub kind_key: &'static str,
    pub radius: f32,
    /// Até onde este raio vai, e **de que natureza é o limite** — parede ou sugestão.
    pub bound: RadiusBound,
}

/// O que o painel precisa de saber sobre o modelo neste quadro.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelSnapshot {
    /// Uma linha por nó com raio editável, na ordem da arena.
    pub rows: Vec<RadiusRow>,
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
    SetRadius { node: u32, radius: f32 },
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
