//! ⭐ **Os VOCABULÁRIOS que as acções carregam** — o segundo irmão por assunto do
//! [`super::action_bus`], pelo mesmo motivo e com a mesma cerca do primeiro
//! ([`super::action_bus_queue`]).
//!
//! # ⚠️ Porque saem estes três, e não o `EditorAction`
//!
//! O `action_bus.rs` voltou ao tecto de 700 LOC em 2026-08-30, ao ganhar o menu do cartão da
//! biblioteca. ⛔ **O corte óbvio continua a ser o errado**: o `EditorAction` cresce por
//! **acrescento no meio**, e este repo corre linhas paralelas em worktrees — mover ~600 linhas
//! poria toda linha que acrescenta uma acção em conflito textual com esta.
//!
//! ⇒ saem os **enums companheiros**: os tipos que as variantes *carregam*, que vivem nas duas
//! pontas do ficheiro (onde ninguém escreve) e que formam um assunto só — *o vocabulário de uma
//! acção, ao lado da acção que o transporta*. Eles são re-exportados pelo `action_bus`, então quem
//! escreve `action_bus::TransportCmd` continua a escrevê-lo e **nenhum chamador muda**.

/// Modifier-key context for a [`super::action_bus::EditorAction::SelectSprite`] event
/// (Fase 0b — image-tools multi-select). The hero/panel side resolves
/// the OS keyboard modifier into this enum before pushing; the shell
/// dispatches the matching [`crate::screens::hero::GizmoStateGroup`]
/// API call. Stays a plain enum (no `bitflags`) — modifiers in PH2D
/// don't compose meaningfully (Shift+Cmd-click on the same element
/// has no defined semantics in this version).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SelectModifier {
    /// No modifier. Replaces the entire selection with the clicked
    /// sprite as the new primary. Most common path.
    #[default]
    Replace,
    /// Shift held. Adds the clicked sprite to the selection without
    /// dropping current sprites. If already selected, no-op (Shift
    /// re-click is idempotent; use [`Self::Toggle`] for off-on).
    Add,
    /// Cmd (macOS) / Ctrl (Linux/Windows) held. Toggles the clicked
    /// sprite in the selection — adds if absent, removes if present.
    /// Removing the primary promotes the oldest extra to primary.
    Toggle,
}

/// Os verbos do menu de um cartão da biblioteca (plano `docs/Components/07`, etapa C).
///
/// ⚠️ **É a FONTE** da tabela do menu: um verbo novo aqui aparece no censo do gate, nunca numa
/// segunda lista escrita à mão.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AssetCardAction {
    /// Pôr uma cópia na cena — o mesmo verbo do duplo-clique, sem ponto de queda.
    Instantiate,
    /// ⭐⭐ **Quem usa isto?** — a metade que o Godot chama *Owners*, e a pergunta que precede
    /// *«posso apagar?»* (plano 07 D9).
    ///
    /// ⚠️ **Ela SELECCIONA em vez de listar.** Uma lista diz um número; uma selecção põe o artista
    /// em cima dos objectos, com o gizmo e o Inspector já apontados. O número vai na voz.
    SelectUsers,
    /// ⭐⭐ **Tirar da biblioteca** (report do Enio, 2026-08-30). A lei das duas metades vive em
    /// `shells/desktop/src/instance_unmake.rs`.
    RemoveFromLibrary,
}

/// The three TopBar transport commands. Kept a small copy enum so the
/// chrome layer stays free of the `Playhead` type (that lives in the shell).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TransportCmd {
    /// Start the clock rolling forward (`playhead.play()`).
    Play,
    /// Halt the clock where it is (`playhead.pause()`).
    Pause,
    /// Rewind the clock to the start and stop (`playhead.rewind()` + pause).
    Reset,
}

/// ⭐⭐ **O que se pode fazer a um catálogo** (plano 07, wave A3).
///
/// ⚠️ **Os ids são `u128` CRUS.** Esta camada é chrome e não conhece o `ph2d-asset-index`; quem os
/// interpreta é o shell, que é o dono da taxonomia. É a mesma cerca do [`AssetCardAction`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogVerb {
    /// Cria um catálogo dentro de `parent` (ou na raiz). O nome nasce gerado e único — quem o
    /// escolhe é o gesto seguinte, o renomear.
    New {
        /// `None` = na raiz.
        parent: Option<u128>,
    },
    /// Renomeia um catálogo. ⚠️ Só o ÚLTIMO nível — mover é outro gesto, e o modelo recusa um nome
    /// com separador.
    Rename {
        /// Quem.
        id: u128,
        /// O rótulo novo.
        name: String,
    },
    /// Apaga um catálogo **e os descendentes**. ⛔ Nunca apaga um asset: eles voltam a
    /// *Unassigned*.
    Delete {
        /// Quem.
        id: u128,
    },
    /// ⭐ Põe um asset num catálogo — o que a queda de um cartão numa linha faz.
    Assign {
        /// O endereço do asset, no vocabulário de chrome.
        asset: crate::interaction::drag_payload::DragPayload,
        /// O catálogo de destino. `None` = tirar de qualquer catálogo (*Unassigned*).
        catalog: Option<u128>,
    },
}
