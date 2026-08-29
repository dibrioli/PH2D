//! **ESTA FORMA TEM TRAÇO?** — a resposta que o painel usa para desenhar a caixa de marcar da
//! secção *Stroke* (plano 34), e a **única** deste painel.
//!
//! ⚠️⚠️ **Ela SUBSTITUI o `TokenBindings::stroke_exists`**, que respondia à mesma pergunta noutro
//! sítio. Duas respostas à mesma pergunta divergem no dia em que uma delas ganhar uma condição — e
//! aqui isso seria visível de imediato: a caixa diria *"tem traço"* e a row de token do traço não
//! apareceria, ou o contrário.
//!
//! ⚠️ **O `Option` é a metade que importa.** `None` = a selecção não tem uma resposta *(nada
//! selecionado, ou selecção múltipla)* e a linha **não é pintada** — a mesma lei do `resize_box`:
//! *uma caixa que descreve um objecto que não está lá é pior que caixa nenhuma.*
//!
//! ⛔ **O painel não alcança a cena, e não deve.** Se alcançasse, a resposta que DESENHA a caixa
//! divergiria da que HONRA o clique, e o artista descobriria a divergência clicando.

use std::cell::Cell;

thread_local! {
    /// `Some(tem)` para uma selecção com resposta; `None` quando não há o que descrever.
    static STROKE_PRESENT: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Publica a resposta deste frame (shell → painel).
pub fn set_stroke_present(v: Option<bool>) {
    STROKE_PRESENT.with(|c| c.set(v));
}

/// A forma selecionada tem traço? `None` = a linha não é pintada.
#[must_use]
pub(crate) fn stroke_present() -> Option<bool> {
    STROKE_PRESENT.with(Cell::get)
}

/// ⭐ **Com que TINTA o traço desenha** (plano 35, wave D) — espelho panel-local do `StrokePaint`
/// da cena, pela MESMA razão que o [`super::FillKind`] o é: o painel não depende da crate do
/// documento.
///
/// ⛔ **Duas variantes, e não as cinco do preenchimento.** O renderer de traço não desenha
/// gradiente; um modelo que representa o que o desenho não faz produz estado inalcançável GRAVADO
/// (plano 35 §2.1). Quando um gradiente no traço for pedido, isto ganha uma variante.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StrokePaintKind {
    /// Uma cor.
    Solid,
    /// Uma arte repetida.
    Pattern,
    /// ⭐⭐ **Um PINCEL** — a arte PERCORRE o contorno (plano 36).
    ///
    /// ⚠️ **O chip dele só é PINTADO quando o desenho existir** (W4). Acrescentar a variante aqui
    /// na W1 é o que faz o modelo dar a volta e o diagnóstico nomeá-lo; pintar o chip antes de a
    /// linha desenhar seria a 4.ª condição da costura falhada de propósito — *um chip que muda o
    /// tipo para algo invisível é o defeito que esta linha já recebeu de report três vezes*.
    Brush,
}

thread_local! {
    /// `Some(k)` para uma selecção cujo traço tem uma tinta; `None` quando não há o que descrever.
    static STROKE_PAINT_KIND: Cell<Option<StrokePaintKind>> = const { Cell::new(None) };
}

/// Publica a tinta do traço deste quadro (shell -> painel).
pub fn set_stroke_paint_kind(v: Option<StrokePaintKind>) {
    STROKE_PAINT_KIND.with(|c| c.set(v));
}

/// A tinta do traço neste quadro. `None` = a fileira **não é pintada** — a mesma lei do
/// [`stroke_present`]: *uma fileira que descreve um objecto que não está lá é pior que fileira
/// nenhuma*. Sem traço não há tinta de traço a escolher.
#[must_use]
pub(crate) fn stroke_paint_kind() -> Option<StrokePaintKind> {
    STROKE_PAINT_KIND.with(Cell::get)
}
