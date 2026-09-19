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

/// ⭐⭐⭐ **A QUEM esta linha acerta** — o vocabulário do PAINEL (suplente #24, 2026-09-19).
///
/// ⚠️ **Ele existe porque o alvo deixou de ser uma pergunta de SIM/NÃO.** Até 2026-09-19 eram dois
/// modos e a shell mandava um `bool`; com o outro lado do disparo são três, e um segundo `bool` ao
/// lado do primeiro daria quatro estados para três respostas — um deles impossível.
///
/// ⚠️ **A POSIÇÃO em [`Self::ALL`] é o que atravessa a fronteira**, como a do verbo: o
/// `ph2d-editor-core` é chrome e não vê o `ph2d-ecs` (ADR-0029), logo quem volta a fazer disto um
/// `SignalTarget` é a shell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActionTargetMode {
    /// O objecto com aquele NOME — vazio = *este objecto*. O de sempre, e o default.
    #[default]
    Name,
    /// Todos os que pertencem a uma TAG.
    Tag,
    /// ⭐ **Quem bateu** — o outro lado do contacto que publicou o sinal.
    Other,
}

impl ActionTargetMode {
    /// Todos, em ordem — **a fonte da iteração** do segmentado. ⚠️ A posição é a tag.
    pub const ALL: [ActionTargetMode; 3] = [Self::Name, Self::Tag, Self::Other];

    /// A posição em [`Self::ALL`].
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O modo desta posição, ou o primeiro. ⚠️ **A POSIÇÃO NO ARRAY É A TAG.**
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }
}

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
    /// ⭐⭐⭐ **Este verbo tem ALVO?** — `false` só no `Restart Run`, que age sobre a CORRIDA e não
    /// sobre uma entidade.
    ///
    /// ⚠️ **Derivado na shell como o irmão [`Self::uses_arg`]**, e pela mesma lei: pintar a escolha
    /// de quem sofre para um verbo que a deita fora é um controlo morto — a família que a caça de
    /// 2026-08-30 mediu em 34 controlos.
    pub uses_target: bool,
    /// ⭐⭐⭐ **A QUEM ela acerta** — a posição em [`ActionTargetMode::ALL`] (suplente #24).
    ///
    /// ⚠️ **É ELE que manda**, e os campos abaixo são a carga de um dos modos: o [`Self::target`]
    /// só é lido em `Name`, e o [`Self::target_tag`] só em `Tag`. *Duas respostas escritas ao mesmo
    /// tempo é o que o doc do `SignalTarget` recusa no modelo.*
    pub target_mode: u8,
    /// ⭐⭐⭐ **A cerca: DE QUEM o sinal tem de vir** — a posição em `SignalFrom::ALL` (suplente #24).
    ///
    /// ⚠️ **Tag e não booleano**, como o verbo e pela mesma razão: o painel não conhece o enum, e a
    /// terceira cerca (*«de quem pertence à tag X»*) não obrigaria a mudar esta fronteira.
    pub from_tag: u8,
    /// ⭐⭐⭐ **A tag escolhida como alvo** (TOP-20 #9, W3b). `None` = a linha não está no modo `Tag`.
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
        self.modo() == ActionTargetMode::Name && self.target.trim().is_empty()
    }

    /// ⭐⭐ **O modo do alvo** — a PORTA da pergunta *«a quem?»*, e os três predicados abaixo saem
    /// dela. ⛔ Ler o `target_tag` para responder seria a segunda resposta.
    #[must_use]
    pub fn modo(&self) -> ActionTargetMode {
        ActionTargetMode::from_tag(self.target_mode)
    }

    /// ⭐ **O alvo é por TAG?**
    #[must_use]
    pub fn target_is_tag(&self) -> bool {
        self.modo() == ActionTargetMode::Tag
    }

    /// ⭐ **O alvo é QUEM BATEU?** (suplente #24)
    #[must_use]
    pub fn target_is_other(&self) -> bool {
        self.modo() == ActionTargetMode::Other
    }

    /// ⭐⭐ **Esta linha só reage ao PRÓPRIO golpe** — a cerca `Myself`.
    ///
    /// ⚠️ **`1` e não um nome:** a posição em `SignalFrom::ALL`, que é o que atravessa a fronteira.
    /// O painel não conhece o enum, e há gate na lei a prender a ordem daquele array.
    #[must_use]
    pub fn from_is_myself(&self) -> bool {
        self.from_tag == 1
    }

    /// ⛔ **A tag alvo foi APAGADA da árvore** — a linha continua a existir e já não alcança
    /// ninguém. *Uma acção que deixou de acertar por causa de um gesto noutro painel é a forma mais
    /// silenciosa de «não acontece nada».*
    #[must_use]
    pub fn target_tag_missing(&self) -> bool {
        self.target_is_tag()
            && self.target_tag.is_some_and(|t| t != 0)
            && self.target_tag_path.is_empty()
    }

    /// ⚠️ **Por tag, e ainda SEM tag escolhida** — uma linha por acabar, não uma linha partida.
    #[must_use]
    pub fn target_tag_unset(&self) -> bool {
        self.target_is_tag() && self.target_tag == Some(0)
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
    /// ⭐⭐⭐ **`(linha, posição em [`ActionTargetMode::ALL`])`** — vira o alvo entre NOME, TAG e
    /// QUEM BATEU (TOP-20 #9 W3b · suplente #24).
    ///
    /// ⚠️ **Virar de modo perde a carga do anterior, por construção**: o `SignalTarget` é um enum, e
    /// o ramo `Named` não guarda tag nenhuma. ⛔ Guardar as duas respostas ao mesmo tempo é
    /// exactamente o que o doc do `SignalTarget` recusa — *«com os dois, «a quem?» teria duas
    /// respostas escritas ao mesmo tempo e o painel teria de escolher uma»*.
    ///
    /// ⚠️ **Era um `bool` até 2026-09-19**, e passou a tag quando o terceiro modo chegou: dois
    /// booleanos dariam quatro estados para três respostas, um deles impossível.
    TargetMode(u8, u8),
    /// **`(linha, id da tag alvo)`** — só faz sentido com a linha já no modo TAG.
    TargetTag(u8, u64),
    /// ⭐⭐⭐ **`(linha, posição em `SignalFrom::ALL`)`** — a CERCA, *de quem o sinal tem de vir*
    /// (suplente #24).
    From(u8, u8),
}
