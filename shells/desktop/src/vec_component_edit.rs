//! **O QUE UM CLIQUE NA SECÇÃO *Prefab* PEDE** — a tabela `id → verbo`, e mais nada.
//!
//! # ⛔⛔ Este ficheiro tinha 473 linhas e um MOTOR dentro (F4.6c, 2026-09-07)
//!
//! Ele foi a shell do sistema de instâncias **do vetor**: `create_main`, `place_instance`,
//! `detach`, `reset_overrides`, `instance_count`, `cascade_offset`, `arm_instance`,
//! `arm_detached`, `main_content_box` — um segundo motor de instância, com o `VecInstance` no ECS
//! por trás dele. O modelo GERAL (ADR-0164) responde hoje por todos, e *dois motores para o mesmo
//! estado é pior que um motor lento*: o que sobra aqui é a **tradução do clique**, que é a única
//! coisa que era de facto do painel vetorial.
//!
//! Quem executa é [`crate::vec_component_general`], que passa o verbo ao dreno único
//! (`instance_verbs::drain`) — onde vivem a voz de cada recusa, a cascata da cópia e a lei *«a
//! selecção segue para a cópia»*.
//!
//! # ⚠️ As duas variantes que morreram com o motor
//!
//! `PieceVisible(row)` e `Variant(axis, value)` endereçavam controlos que só o motor vetorial
//! pintava (a lista de peças e a fileira de eixos). **As capacidades não se perderam** — esconder
//! uma peça de UMA cópia e pintá-la são hoje o `Visibility`/`Sprite` da própria peça
//! ([`crate::instance_piece_override_tests`], escrito como pré-condição desta fatia), e a versão
//! que uma cópia é vive no cartão do Inspector (F5).
//!
//! ⚠️ *Um verbo que não tem executor não fica no enum a fingir* — enquanto ficava, o
//! `general_verb` devolvia `None` e o clique morria em silêncio, que é o defeito que esta secção
//! caçou três vezes.

/// O que um clique num verbo de componente PEDE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ComponentEdit {
    Create,
    /// ⭐⭐⭐ **Edit Prefab** — abre a RECEITA desta cópia (2026-09-07). Ver
    /// [`crate::instance_verbs::Verb::Edit`]: no modelo geral a receita está escondida do canvas,
    /// e até aqui só o cartão da biblioteca lá chegava.
    Edit,
    Place,
    /// ⭐⭐ **Instantiate Linked** (2026-09-07) — a cópia divide a ARTE da receita.
    PlaceLinked,
    Detach,
    Reset,
    /// **Apply to Prefab** — as diferenças desta cópia passam a ser a receita.
    UpdateMain,
    /// **Swap Prefab** — arma o conta-gotas; o clique seguinte no canvas escolhe o prefab.
    Swap,
}

/// Este id é um verbo de componente? Porta única do roteador.
#[must_use]
pub(crate) fn component_edit_for_id(id: ph2d_editor::NodeId) -> Option<ComponentEdit> {
    match id {
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_CREATE => Some(ComponentEdit::Create),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_EDIT => Some(ComponentEdit::Edit),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_PLACE => Some(ComponentEdit::Place),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_PLACE_LINKED => {
            Some(ComponentEdit::PlaceLinked)
        }
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_DETACH => Some(ComponentEdit::Detach),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_RESET => Some(ComponentEdit::Reset),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_UPDATE_MAIN => {
            Some(ComponentEdit::UpdateMain)
        }
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_SWAP => Some(ComponentEdit::Swap),
        _ => None,
    }
}
