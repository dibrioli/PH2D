//! Os canais laterais do CARTÃO (shell → painel): as opções de cada selector, o texto inteiro de
//! cada param de texto, e a curva de cada row `Ease`. Cortados de `snapshot.rs` no tecto de LOC.

use super::EaseView;
use std::cell::RefCell;

thread_local! {
    /// ⭐⭐ **O TEXTO INTEIRO de cada param de texto** — `(nó, param, valor)`, publicado pela
    /// shell ao lado do snapshot.
    ///
    /// ⛔⛔ **Existe porque o [`CardParam`] guarda o texto TRUNCADO** ([`RowText`], o que cabe na
    /// row) e semear uma caixa de edição com ele **destruiria o resto da string** no `Enter` —
    /// que é, uma letra acima, a armadilha que o `param_edit` já nomeia para o número
    /// (*«a row mostra `0.50` e comitar essa leitura destruiria um `0.503`»*).
    ///
    /// ⚠️ **E é um canal lateral, não um campo:** o [`GraphNodeView`] é construído em **31
    /// sítios** desta árvore, e apender-lhe um campo é a forma que a memória do repo nomeia
    /// (*«tipo construído em N sítios prefere componente opcional a campo apendado»*). Aqui o
    /// molde já existe — é o mesmo dos outros seis publicadores acima.
    static CARD_TEXTS: RefCell<Vec<(u32, &'static str, String)>> =
        const { RefCell::new(Vec::new()) };

    /// ⭐⭐⭐ **AS OPÇÕES DE CADA SELECTOR** — o que a LISTA de um enum, de uma fonte publicada
    /// ou de um canal oferece, e em qual delas o nó está (report do Enio, 2026-09-07: *«se
    /// clicar no centro (nome) abre-se um dropdown»*).
    ///
    /// ⚠️ **Um canal lateral, e não um campo do [`CardParam`], pela mesma razão do
    /// [`CARD_TEXTS`]** — mas aqui há uma segunda: uma lista de nomes por row é exactamente a
    /// alocação por quadro que a medição do doc 103 recusou. Daí a [`CardChoices`] ter duas
    /// caras: um enum do registry viaja como `&'static` (**zero** alocação, e são 138 das 143
    /// rows de selector do catálogo), e só as listas VIVAS — o que o artista desenhou, o que a
    /// corrente de cima cozinhou — pagam `String`s, que são meia dúzia.
    static CARD_CHOICES: RefCell<Vec<(u32, &'static str, CardChoices, u16)>> =
        const { RefCell::new(Vec::new()) };
}

/// **O que a lista de um selector oferece** — em duas caras, porque as duas fontes têm custos
/// diferentes e a diferença é medida (ver [`CARD_CHOICES`]).
#[derive(Clone, Debug, PartialEq)]
pub enum CardChoices {
    /// Rótulos que vivem no binário — os `labels` de um `ParamWidget::Enum`.
    Static(&'static [&'static str]),
    /// Rótulos VIVOS — as formas que o artista desenhou, as colunas que a corrente cozinhou.
    Live(Vec<String>),
}

impl CardChoices {
    /// Os rótulos, uma cópia só — chamada **ao ABRIR** a lista, nunca por quadro.
    pub(crate) fn labels(&self) -> Vec<String> {
        match self {
            // ⚠️ **A tradução das opções acontece AQUI e em mais sítio nenhum desta lista** — e é de
            // graça, porque este método corre ao ABRIR e não por quadro (ver o doc acima).
            Self::Static(l) => l.iter().map(|s| ph2d_i18n::tr(s).to_string()).collect(),
            Self::Live(l) => l.clone(),
        }
    }
}

/// Publica as opções de cada selector de cada cartão (shell → painel). Ver [`CARD_CHOICES`].
pub fn set_card_choices(choices: Vec<(u32, &'static str, CardChoices, u16)>) {
    CARD_CHOICES.with(|c| *c.borrow_mut() = choices);
}

/// As opções de `(nó, param)` e o índice do que está escolhido — `None` quando a shell não as
/// publicou (o param não é um selector, ou o cartão não está na vista).
///
/// ⚠️ **O índice pode ser `>= labels.len()`, e isso é a resposta certa:** significa *nenhuma
/// das opções* — uma coluna escrita à mão, uma forma que ainda não foi desenhada. A lista abre
/// sem nada marcado, que é honesto; inventar uma marca diria que o nó está numa opção em que
/// ele não está.
pub(crate) fn card_choices_of(node: u32, param: &str) -> Option<(Vec<String>, u16)> {
    CARD_CHOICES.with(|c| {
        c.borrow()
            .iter()
            .find(|(n, p, _, _)| *n == node && *p == param)
            .map(|(_, _, ch, cur)| (ch.labels(), *cur))
    })
}

/// Publica o texto INTEIRO dos params de texto de cada cartão (shell → painel). Ver
/// [`CARD_TEXTS`]: o cartão desenha o truncado e a caixa de edição abre com este.
pub fn set_card_texts(texts: Vec<(u32, &'static str, String)>) {
    CARD_TEXTS.with(|c| *c.borrow_mut() = texts);
}

thread_local! {
    /// ⭐ As CURVAS de easing de cada cartão (shell → painel, ciclo 17) — o editor gráfico lê-as
    /// para desenhar e para saber de onde parte um arrasto, como a caixa de texto lê os textos.
    static CARD_EASES: RefCell<Vec<(u32, &'static str, EaseView)>> = const { RefCell::new(Vec::new()) };
}

/// Publica a curva de easing de cada row `Ease` dos cartões (shell → painel).
pub fn set_card_eases(eases: Vec<(u32, &'static str, EaseView)>) {
    CARD_EASES.with(|c| *c.borrow_mut() = eases);
}

/// A curva de `(nó, param)` — `None` quando a shell não a publicou.
pub(crate) fn card_ease_of(node: u32, param: &str) -> Option<EaseView> {
    CARD_EASES.with(|c| {
        c.borrow()
            .iter()
            .find(|(n, p, _)| *n == node && *p == param)
            .map(|(_, _, v)| *v)
    })
}

/// O texto inteiro de `(nó, param)` — `None` quando a shell não o publicou.
pub(crate) fn card_text_of(node: u32, param: &str) -> Option<String> {
    CARD_TEXTS.with(|c| {
        c.borrow()
            .iter()
            .find(|(n, p, _)| *n == node && *p == param)
            .map(|(_, _, v)| v.clone())
    })
}
