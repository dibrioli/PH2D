//! ⭐⭐⭐ **AS PORTAS DAS TAGS** (TOP-20 #9) — irmão de [`super`] por tecto de LOC.
//!
//! ⚠️ **São DOIS níveis, e é isso que os põe juntos aqui e separados lá dentro:** os *chips* de um
//! objecto vêm do instantâneo POR OBJECTO, e a árvore vem do **DOCUMENTO** — a segunda tem porta
//! própria porque a caixa de escolha do alvo de uma *Signal Action* vive num objecto que pode não
//! ter `Tags` nenhum.

use ph2d_editor_core::screens::hero::{InspectorTagRow, InspectorTagsInfo};

thread_local! {
    /// TAGS — os chips DESTE objecto (TOP-20 #9). ⚠️ A árvore do projecto **não** vem aqui: ela
    /// é do DOCUMENTO e tem porta própria ([`CURRENT_TAG_TREE`]).
    pub(crate) static CURRENT_INSPECTOR_TAGS:
        std::cell::RefCell<Option<InspectorTagsInfo>> = const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **A ÁRVORE DE TAGS DO PROJECTO** — a lista inteira, na ordem dela.
    ///
    /// ⛔⛔ **Porta própria, e não um campo do [`CURRENT_INSPECTOR_TAGS`]**, e a razão é a SEGUNDA
    /// superfície: a secção *Signal Actions* escolhe uma tag como alvo, e um objecto com
    /// `SignalActions` **pode não ter `Tags` nenhum** — com a lista dentro do instantâneo por
    /// objecto, a caixa de escolha do alvo abriria vazia exactamente no caso normal.
    ///
    /// ⚠️ *Não foi erro de leitura: era a forma certa enquanto houve um consumidor só.* O segundo
    /// consumidor é que revela o NÍVEL a que um dado pertence — a árvore é documento, como a
    /// `VecScene` e o `FlipDoc`, e não dado de uma entidade.
    pub(crate) static CURRENT_TAG_TREE:
        std::cell::RefCell<Vec<InspectorTagRow>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn set_current_inspector_tags(info: Option<InspectorTagsInfo>) {
    CURRENT_INSPECTOR_TAGS.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_tags() -> Option<InspectorTagsInfo> {
    CURRENT_INSPECTOR_TAGS.with(|c| c.borrow().clone())
}

pub fn set_current_tag_tree(rows: Vec<InspectorTagRow>) {
    CURRENT_TAG_TREE.with(|c| *c.borrow_mut() = rows);
}

pub(crate) fn current_tag_tree() -> Vec<InspectorTagRow> {
    CURRENT_TAG_TREE.with(|c| c.borrow().clone())
}
