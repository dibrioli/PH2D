//! **O modelo da secção SIGNAL ACTIONS** (TOP-20 #5, W3) — snapshot e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** — mesmo padrão dos outros sete.
//!
//! # ⚠️ O verbo viaja como TAG, e não como enum
//!
//! O `ph2d-editor-core` é chrome e **não depende do `ph2d-ecs`** (ADR-0029: uma crate de painel
//! fala com a shell por snapshot). ⇒ o verbo atravessa a fronteira como o `u8` que o
//! `SignalVerb::tag()` produz, e quem o volta a fazer verbo é a shell, com o `from_tag`.
//!
//! ⚠️ **O RÓTULO também atravessa** (`verb_labels`), e é o que impede o painel de escrever a lista
//! uma segunda vez: cinco rótulos copiados aqui envelheciam no primeiro verbo novo, e o artista
//! leria o nome errado sobre o botão certo.

/// Uma linha da tabela, como o Inspector a lê.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorActionRow {
    /// O nome do sinal que dispara. **Vazio = nunca.**
    pub on: String,
    /// O NOME do objecto que sofre. **Vazio = este objecto.**
    pub target: String,
    /// A posição do verbo em `SignalVerb::ALL`.
    pub verb_tag: u8,
    /// O parâmetro do verbo. **Vazio = todos**, para os verbos que o lêem.
    pub arg: String,
    /// Este verbo LÊ o `arg`? **Derivado na shell**, do próprio verbo.
    ///
    /// ⚠️ **Vem no snapshot em vez de ser re-derivado aqui**, e é a mesma lei do rótulo: o painel
    /// não conhece o enum, e re-derivá-lo seria uma segunda resposta a *«este campo serve para
    /// alguma coisa?»* — a que o artista vê seria a que envelhece.
    pub uses_arg: bool,
    /// ⭐⭐⭐ **A tag escolhida como alvo** (TOP-20 #9, W3b). `None` = o alvo é por NOME, e o
    /// [`Self::target`] é que manda.
    ///
    /// ⚠️ **`Some(0)` é «por tag, e ainda não escolheu qual»** — o `TagId(0)` nunca é dado pela
    /// árvore. Ele alcança ninguém, tal como uma tag apagada, mas **são duas histórias diferentes**
    /// e a secção conta cada uma: uma é uma linha por acabar, a outra é uma linha que partiu.
    pub target_tag: Option<u64>,
    /// O CAMINHO da tag alvo, para a linha o mostrar.
    ///
    /// ⚠️ **Vazio com [`Self::target_tag`] a `Some(n)`, `n != 0`, significa que a tag foi
    /// APAGADA** — ver [`Self::target_tag_missing`]. O painel não conhece a árvore, e sem este
    /// campo ele só teria um número para desenhar.
    pub target_tag_path: String,
}

impl InspectorActionRow {
    /// ⛔ **Esta linha nunca dispara** — o nome do sinal está vazio.
    ///
    /// ⚠️ **Derivado, nunca guardado.** É a causa nº 1 de *«não acontece nada»* nesta secção, e o
    /// painel escreve-a em WARN por isso.
    #[must_use]
    pub fn never_fires(&self) -> bool {
        self.on.trim().is_empty()
    }

    /// O alvo, para leitura: o nome, ou *este objecto* quando vazio.
    ///
    /// ⚠️ **Só faz sentido no modo NOME** — com o alvo por tag, um `target` vazio não quer dizer
    /// *este objecto*; quer dizer que o campo do nome não está a ser lido por ninguém.
    #[must_use]
    pub fn target_is_self(&self) -> bool {
        self.target_tag.is_none() && self.target.trim().is_empty()
    }

    /// ⭐ **O alvo é por TAG?**
    #[must_use]
    pub fn target_is_tag(&self) -> bool {
        self.target_tag.is_some()
    }

    /// ⛔ **A tag alvo foi APAGADA da árvore** — a linha continua a existir e já não alcança
    /// ninguém. *Uma acção que deixou de acertar por causa de um gesto noutro painel é a forma mais
    /// silenciosa de «não acontece nada».*
    #[must_use]
    pub fn target_tag_missing(&self) -> bool {
        self.target_tag.is_some_and(|t| t != 0) && self.target_tag_path.is_empty()
    }

    /// ⚠️ **Por tag, e ainda SEM tag escolhida** — uma linha por acabar, não uma linha partida.
    #[must_use]
    pub fn target_tag_unset(&self) -> bool {
        self.target_tag == Some(0)
    }
}

/// Snapshot da secção SIGNAL ACTIONS da entidade selecionada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorActionInfo {
    pub entity_bits: u64,
    /// As linhas, na ordem em que o componente as guarda — **que é a ordem em que elas correm**.
    pub rows: Vec<InspectorActionRow>,
    /// Os rótulos dos verbos, na ordem de `SignalVerb::ALL`. ⚠️ Ver o doc do módulo.
    pub verb_labels: Vec<String>,
    /// Quantas entidades estão selecionadas — a secção **não** se espalha (o índice só significa
    /// alguma coisa na lista da primária).
    pub selected_count: usize,
}

/// Uma edição de um campo da secção SIGNAL ACTIONS.
///
/// ⚠️ **O `u8` do primeiro campo é o ÍNDICE na lista**, e o do `Verb` é a **tag** do verbo. Dois
/// `u8` com significados diferentes na mesma família — e é por isso que os nomes dos campos os
/// distinguem em vez de se chamarem os dois `n`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionFieldEdit {
    /// Cria uma linha vazia. ⚠️ Ela nasce **sem nome de sinal**, e o painel di-lo em WARN: um
    /// default que disparasse em alguma coisa faria um `+` mudar a cena.
    Add,
    /// Retira a linha deste índice.
    Remove(u8),
    /// `(linha, nome do sinal)`.
    On(u8, String),
    /// `(linha, NOME do objecto alvo)` — vazio volta a ser *este objecto*.
    Target(u8, String),
    /// `(linha, posição do verbo em `SignalVerb::ALL`)`.
    Verb(u8, u8),
    /// `(linha, parâmetro)`.
    Arg(u8, String),
    /// ⭐⭐⭐ **`(linha, por TAG?)`** — vira o alvo entre NOME e TAG (TOP-20 #9, W3b).
    ///
    /// ⚠️ **Virar para TAG perde a tag anterior, por construção**: o `SignalTarget` é um enum, e o
    /// ramo `Named` não guarda tag nenhuma. ⛔ Guardar as duas respostas ao mesmo tempo é
    /// exactamente o que o doc do `SignalTarget` recusa — *«com os dois, «a quem?» teria duas
    /// respostas escritas ao mesmo tempo e o painel teria de escolher uma»*.
    TargetMode(u8, bool),
    /// **`(linha, id da tag alvo)`** — só faz sentido com a linha já no modo TAG.
    TargetTag(u8, u64),
}
