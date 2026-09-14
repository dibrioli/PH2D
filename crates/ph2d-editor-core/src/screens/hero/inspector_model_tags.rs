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

// ⛔⛔ **Nem o `TagsFieldEdit` nem o `TagTreeEdit` moram aqui, e a mudança tem um NÚMERO atrás.**
// Os dois vivem em [`crate::tags_edits`], o módulo de vocabulário que a catraca do DAG prescreve
// por escrito — *«os PAYLOADS do Inspector moram em `screens::hero::inspector_model*`; cura:
// descem para um módulo de vocabulário abaixo do `action_bus`»*. A aresta `action_bus → screens`
// estava no tecto (`24`) e o `TagsFieldEdit` desta wave fê-la passar a `25`, **em silêncio**: o
// portão da W3 correu o painel e a shell, e este gate vive no `ph2d-editor-core`.
// ⇒ a cura não foi subir o tecto (a catraca só encolhe): foi **começar a migração** que ela pede.
// ⚠️ Os dois descem juntos porque são o mesmo assunto — *marcar um objecto* e *mudar a taxonomia*
// são as duas metades de um só vocabulário, e separá-los daria dois sítios para a mesma família.
// ⛔⛔ **O `TagTreeEdit` NÃO mora aqui, e a ausência é a decisão.** Ele vive em
// [`crate::action_bus`], ao lado da acção que o carrega — e a razão não é gosto: as irmãs
// (`TagsFieldEdit`, `TimerFieldEdit`, …) são tipos do MODELO, construídos pelos instantâneos
// deste módulo, enquanto o `TagTreeEdit` não tem instantâneo nenhum: ele é só a carga do
// barramento. ⚠️ E foi a catraca do DAG que o disse — o `action_bus → screens` está no tecto
// (`24`), e uma 25.ª referência ali reprova: *a catraca só encolhe*, e a cura é não precisar da
// aresta.
