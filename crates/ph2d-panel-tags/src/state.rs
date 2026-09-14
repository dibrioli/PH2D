//! O estado do painel TAGS — o que ele guarda entre quadros, e a porta por onde a shell publica.
//!
//! ⚠️ **A divisão é a das irmãs:** o que é do GESTO (a linha em mãos, a linha a ser renomeada) vive
//! na `State` tipada do painel; o que é do DOCUMENTO chega por thread-local, publicado pela shell
//! ANTES de o painel pintar. Um instantâneo dentro da `State` seria o painel a guardar uma cópia do
//! documento, que envelhece à primeira edição feita noutro sítio.

use ph2d_editor_core::TagsPanelInfo;

/// O estado retido do painel.
#[derive(Clone, Debug, Default)]
pub struct TagsPanelState {
    /// **A linha em mãos** — o sujeito dos verbos da barra.
    ///
    /// ⚠️ Ela é do PAINEL e não da cena: escolher uma tag aqui não mexe na selecção de objectos (o
    /// que faz isso é o verbo *Select*, explícito). ⛔ Fundir as duas faria passar o rato sobre a
    /// taxonomia desfazer a selecção do artista.
    pub focus: Option<u64>,
    /// A linha que está com o campo de renomear aberto por cima.
    pub renaming: Option<u64>,
}

thread_local! {
    /// O instantâneo publicado pela shell, uma vez por quadro.
    static CURRENT: std::cell::RefCell<TagsPanelInfo> =
        const { std::cell::RefCell::new(TagsPanelInfo { rows: Vec::new(), problem: None }) };

    /// ⭐⭐ **A tag que o último gesto CRIOU** — consumida uma vez, pelo `paint`, para abrir o campo
    /// de renomear por cima dela.
    ///
    /// ⚠️ **Um disparo só (`take`), e é isso que a torna correcta:** se ela ficasse posta, todo
    /// quadro seguinte reabriria o campo e o artista nunca conseguiria sair dele.
    static BORN: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };

    /// A altura do corpo pintado no último quadro — o `paint` apara a rolagem contra ela.
    static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// **A porta da shell** — publica a árvore e a última recusa. Uma vez por quadro.
pub fn set_current_tags(info: TagsPanelInfo) {
    CURRENT.with(|c| *c.borrow_mut() = info);
}

/// **A tag acabada de nascer** — a shell escreve-a no quadro em que o `Create` foi aplicado.
pub fn set_born_tag(tag: Option<u64>) {
    BORN.with(|c| c.set(tag));
}

pub(crate) fn take_born_tag() -> Option<u64> {
    BORN.with(std::cell::Cell::take)
}

pub(crate) fn with_current<R>(f: impl FnOnce(&TagsPanelInfo) -> R) -> R {
    CURRENT.with(|c| f(&c.borrow()))
}

/// A árvore como o painel a leu no último quadro — o que os gates e a shell precisam de conferir.
#[must_use]
pub fn current_tags() -> TagsPanelInfo {
    CURRENT.with(|c| c.borrow().clone())
}

pub(crate) fn set_last_content_h(h: f32) {
    LAST_CONTENT_H.with(|c| c.set(h));
}

/// A altura do corpo pintado no último quadro.
#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(std::cell::Cell::get)
}
