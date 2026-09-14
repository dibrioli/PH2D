//! **O modelo da secção TAGS** (TOP-20 #9, W3) — snapshot e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** — o mesmo padrão dos outros sete.
//!
//! # ⚠️ Este instantâneo é DO OBJECTO, e a árvore do projecto NÃO está nele
//!
//! [`InspectorTagsInfo::on_object`] responde *«que tags este objecto tem»* — os chips. A outra
//! pergunta, *«que tags o projecto tem»*, é do DOCUMENTO e viaja por uma porta própria
//! (`ph2d_panel_inspector::set_current_tag_tree`).
//!
//! ⛔⛔ **A árvore ESTEVE aqui dentro, e foi a segunda superfície que mostrou o nível certo:** a
//! secção *Signal Actions* escolhe uma tag como alvo, e um objecto com `SignalActions` pode não ter
//! `Tags` nenhum — com a lista dentro deste instantâneo (que é `None` nesse caso), a caixa de
//! escolha do alvo abriria vazia exactamente no caso normal. ⚠️ *Não foi erro de leitura: era a
//! forma certa enquanto houve um consumidor só.*
//!
//! # ⚠️ O que este snapshot NÃO tem
//!
//! A contagem de objectos por tag. Ela é `O(mundo)` por tag (`ph2d_ecs::tags::tagged`), e o
//! Inspector repinta a cada quadro; o sítio dela é o painel *Tags* (W4), que a pede uma vez.

/// Uma tag, como o Inspector a lê.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorTagRow {
    /// A identidade durável (`ph2d_tags::TagId`), já em `u64` — o `editor-core` não conhece a folha
    /// das tags, e não precisa: o que ele faz com isto é devolvê-lo numa edição.
    pub id: u64,
    /// `"Enemy/Flying"` — o caminho inteiro, que é o que o balão mostra.
    pub path: String,
    /// O último nível (`"Flying"`) — o que o chip mostra. ⚠️ **Derivado na construção do snapshot**,
    /// e não aqui: quem sabe cortar um caminho é a álgebra de caminhos, não o painel.
    pub label: String,
    /// `0` = raiz. A caixa de escolha usa-a para indentar a lista como uma árvore.
    pub depth: usize,
}

/// Snapshot da secção TAGS da entidade selecionada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorTagsInfo {
    pub entity_bits: u64,
    /// As tags DESTE objecto, pela ordem da árvore.
    pub on_object: Vec<InspectorTagRow>,
    /// O objecto já não aceita mais tags (`ph2d_ecs::TAGS_MAX`) — a secção esconde a caixa e diz
    /// porquê, em vez de oferecer um gesto que vai ser recusado.
    pub full: bool,
    /// Quantas entidades estão selecionadas. ⚠️ A secção **não se espalha** sobre a seleção: ela
    /// mostra as tags da primária, e diz quando há mais.
    pub selected_count: usize,
}

/// **Uma edição da secção TAGS.**
///
/// ⚠️ **`Create` carrega TEXTO e as outras um id**, e a distinção é a que faz a caixa de escolha ter
/// uma porta só: escrever um nome que não existe e carregar em *Create “…”* cria a tag **e** marca
/// o objecto, num gesto; escolher uma da lista só marca.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagsFieldEdit {
    /// Marca o objecto com esta tag (id da árvore). Uma tag que já lá está é no-op.
    Add(u64),
    /// Tira esta tag do objecto. ⛔ **Não apaga a tag da árvore** — o painel *Tags* (W4) é quem o faz.
    Remove(u64),
    /// Cria a tag com este caminho (ou devolve a que já existe, dobrada) **e** marca o objecto.
    Create(String),
}
