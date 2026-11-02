//! **A BOOLEANA VIVA** — o fato da cena que a ④ lê: há um grupo booleano na seleção, e com que
//! operação? (F50: a booleana é sempre viva, como na Figma — a chave «Live» saiu.)
//!
//! ⚠️ A verdade mora no ECS (`ph2d_ecs::VecBoolGroup`), não aqui: isto é a projeção que a shell
//! publica por frame. O painel não alcança o mundo — e não deve: se alcançasse, haveria duas
//! respostas para *"esta seleção é uma booleana viva?"*.

use std::cell::{Cell, RefCell};

thread_local! {
    /// A operação do grupo booleano vivo da seleção deste frame (`None` = não há grupo).
    static GROUP_OP: Cell<Option<u8>> = const { Cell::new(None) };
    /// A fileira do verbo por forma: *(verbo aceso, nome da forma)*.
    static SHAPE_ROW: RefCell<Option<(u8, String)>> = const { RefCell::new(None) };
}

/// Publica a operação do grupo booleano da seleção deste frame (shell → painel); o código é o
/// discriminante de `VecBoolGroup::op`.
pub fn set_bool_group(op: Option<u8>) {
    GROUP_OP.with(|c| c.set(op));
}

/// A operação do grupo booleano selecionado — acende o botão dela na ④.
#[must_use]
pub(crate) fn bool_group_op() -> Option<u8> {
    GROUP_OP.with(Cell::get)
}

/// Há booleana viva selecionada? É isto que oferece o **Flatten** com uma forma só.
#[must_use]
pub(crate) fn bool_group_selected() -> bool {
    bool_group_op().is_some()
}

/// **O verbo da forma selecionada** dentro de uma booleana viva (shell → painel).
///
/// `None` faz a fileira **não existir**, e ele carrega a regra inteira — irmão exacto do
/// `frame_clip`, e pela mesma razão: quem alcança o mundo é a shell, e uma segunda resposta a
/// *"esta forma pode escolher um verbo?"* divergiria da primeira no dia seguinte.
///
/// A shell devolve `None` em quatro casos, e cada um é uma decisão:
///
/// - não há **primário** na seleção — o verbo é de UMA forma, e o primário é a que o dedo
///   apontou (tocar um filho seleciona o grupo inteiro, e é por isso que a contagem não serve:
///   ⚠️ exigi-la tornava a fileira **inalcançável por clique**, que foi o defeito de 22/08);
/// - a forma não está numa booleana viva;
/// - ela é a **BASE** — o verbo dela é inerte, e um controlo inerte pintado como vivo é pior que
///   controlo nenhum;
/// - o grupo está numa **RECEITA** (`Trim`/`Crop`/`Merge`/`MinusBack`), que é verbo da pilha
///   inteira e ignora os das formas.
pub fn set_bool_shape_row(row: Option<(u8, String)>) {
    SHAPE_ROW.with(|c| *c.borrow_mut() = row);
}

/// O verbo da forma selecionada — `None` = a fileira não é oferecida.
#[must_use]
pub(crate) fn bool_shape_row() -> Option<(u8, String)> {
    SHAPE_ROW.with(|c| c.borrow().clone())
}
