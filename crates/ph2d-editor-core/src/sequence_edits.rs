//! ⭐⭐⭐ **O vocabulário da secção SEQUENCE** (TOP-20 #19, W3) — o que o painel mostra de uma
//! cutscene e o que ele edita.
//!
//! Irmão do [`crate::hud_edits`] e do [`crate::particles_edits`]: um instantâneo que a shell
//! publica por quadro, e um `enum` de edição que volta pelo barramento.
//!
//! # ⭐⭐ Porque o controlo é um SELECTOR e não um campo de texto
//!
//! O conjunto das cutscenes é **conhecido** — elas são os containers do documento da timeline, e a
//! aba *Containers* já as lista. Um campo de texto obrigaria o artista a escrever um nome que tem
//! de casar **exactamente** (`"porta"` contra `"Porta"` é uma cutscene que não corre), e o produto
//! ficaria com a doença que esta casa nomeia em três sítios: *uma ferramenta que não faz nada e não
//! diz porquê é indistinguível de uma partida*. ⇒ um chip com a lista, como o verbo do
//! [`crate::action_bus::EditorAction::InspectorActionEdit`] e o barramento do áudio.
//!
//! ⚠️ **E a edição carrega o NOME, nunca o índice** — a mesma lei que o
//! [`ph2d_ecs::SequencePlayer`] escreve no cabeçalho dele: apagar o container de cima renumera os
//! de baixo, e um índice guardado passaria a tocar a cutscene do vizinho **em silêncio**. O
//! selector escolhe por posição porque é assim que se pinta uma lista; o que viaja é o nome.
//!
//! # ⭐⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No cutscenes in the timeline yet` | não há container nenhum — faz-se um em *Timeline → + Container* |
//! | `No cutscene chosen` | o campo está em branco: o objecto tem o componente e não toca nada |
//! | `That cutscene is gone` | o nome guardado já não existe — e ele **FICA**, para não se perder |
//! | `This object has no Timer` | sem relógio ele é inerte, e o descritor exige-o (`requires`) |
//! | `The clock is stopped` | o relógio da cena não anda ⇒ o `Timer` não conta |
//! | `Not running` | o relógio existe, está parado: falta um *Start Timer* |
//! | `The timer is shorter than the cutscene` | ela nunca chega ao fim — o defeito mais caro deste componente |
//! | `Now: 1.20 s of 2.00` | o instante VIVO — leitura, nunca edição: ele não é documento |
//!
//! ⚠️ **Sem eles, uma cutscene exactamente como o artista a pediu lê-se como partida, com todos os
//! campos certos no ecrã** — a lição do projéctil, do mover de vista de cima e do emissor.

/// Uma edição da secção **SEQUENCE**.
///
/// ⚠️ **Uma variante só, e isso é o componente inteiro:** o [`ph2d_ecs::SequencePlayer`] tem um
/// campo. O relógio é do `Timers`, que tem secção própria — *duas secções a editar o mesmo relógio
/// seriam duas respostas à mesma pergunta*.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SequenceFieldEdit {
    /// A cutscene que este objecto toca — o **nome**. Vazio = calado.
    Container(String),
}

/// **O que o painel mostra da cutscene deste objecto** — o instantâneo que a shell publica por
/// quadro.
///
/// ⚠️ **Ele traz TRÊS factos que não são campos do componente**: quais cutscenes existem (senão o
/// selector não tem o que mostrar), se o relógio deste objecto anda (senão ela é inerte) e o
/// instante vivo. *Os três são a diferença entre um painel e uma folha de campos.*
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorSequenceInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// O nome guardado no componente, **cru**: é ele que o aviso do órfão mostra.
    pub container: String,
    /// As cutscenes que existem no documento — as opções do selector, pela ordem do documento.
    pub nomes: Vec<String>,
    /// O índice que o nome resolve, ou `None` (em branco **ou** órfão — o painel separa os dois
    /// pelo [`Self::container`] estar vazio).
    pub escolhido: Option<usize>,
    /// Quanto dura a cutscene escolhida, em segundos. `0` = nenhuma escolhida.
    pub duracao_da_cutscene: f64,
    /// O objecto tem `Timers`? ⚠️ O descritor **exige-o**, mas o artista pode removê-lo.
    pub tem_relogio: bool,
    /// O relógio `0` — o da sequência — está a correr?
    pub a_correr: bool,
    /// Quanto dura o relógio `0`, em segundos.
    pub duracao_do_relogio: f64,
    /// O instante VIVO da cutscene, em segundos — leitura, nunca edição.
    pub t: f64,
    /// O relógio da CENA está a tocar? Sem ele nenhum `Timer` conta.
    pub clock_playing: bool,
    /// ⭐⭐⭐ **A vista da timeline deixa esta cutscene correr?**
    ///
    /// ⚠️ **`false` = as duas vistas de EDIÇÃO** (a aba *Keys*, ou o interior de um container):
    /// elas solam o que o animador está a autorar e congelam o relógio da cena. Sem esta linha, um
    /// artista que abrisse a timeline veria a cutscene dele parar **com todos os campos certos** —
    /// e concluiria que o componente está partido.
    pub vista_deixa_correr: bool,
    /// Quantos objectos estão escolhidos — a secção edita o primário, e di-lo.
    pub selected_count: usize,
}

impl InspectorSequenceInfo {
    /// **O nome guardado, aparado** — `None` quando está em branco.
    #[must_use]
    pub fn nome(&self) -> Option<&str> {
        let t = self.container.trim();
        (!t.is_empty()).then_some(t)
    }

    /// ⛔ **O nome guardado não existe no documento** — a cutscene não corre, e o nome fica.
    ///
    /// ⚠️ É diferente de *«não escolheu nenhuma»*, e as duas frases do painel são diferentes: uma
    /// diz ao artista que falta escolher, a outra que **alguém apagou** o que ele tinha escolhido.
    #[must_use]
    pub fn orfao(&self) -> bool {
        self.nome().is_some() && self.escolhido.is_none()
    }

    /// ⭐⭐⭐ **O relógio acaba antes da cutscene** — ela nunca chega ao fim.
    ///
    /// ⚠️ **Só é uma pergunta quando há cutscene E relógio**, e a folga de um milissegundo é
    /// deliberada: os dois números vêm de unidades diferentes (o container em `f64` de segundos, o
    /// relógio em microssegundos inteiros), e um `<` cru acusaria um empate exacto que o artista
    /// escreveu de propósito.
    #[must_use]
    pub fn relogio_curto(&self) -> bool {
        self.escolhido.is_some()
            && self.tem_relogio
            && self.duracao_do_relogio + 1e-3 < self.duracao_da_cutscene
    }
}
