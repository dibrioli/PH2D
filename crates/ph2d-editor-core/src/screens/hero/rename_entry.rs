//! ⭐ **A ENTRADA NO MODO RENOMEAR** — a porta que arma o campo de texto da Hierarquia.
//!
//! # Por que isto é um ficheiro irmão
//!
//! Corte feito na **INTEGRAÇÃO de 2026-09-10**, e o vermelho era **ACUMULADO**: o
//! `screens/hero.rs` chegou a `709 LOC` contra o tecto de `700` somando três linhas que fecharam
//! no mesmo dia — **nenhuma delas o estoura sozinha**, e por isso nenhum dos três portões de fecho
//! o podia ver. ⛔ A cura de um tecto de LOC é **cortar por responsabilidade**, nunca acrescentar
//! uma entrada ao `FILE_OVERAGE_OK` (`CLAUDE.md` §5.0: *uma catraca sem censo de obsolescência não
//! desce — ela vira licença*).
//!
//! E a responsabilidade separa-se limpa: o `hero.rs` é o **ecrã** (a `HeroScreen`, a selecção, o
//! despacho de eventos, a árvore de a11y); isto aqui é o **estado de um widget de texto** e não
//! toca na `HeroScreen` em sítio nenhum — recebe só o [`WidgetStore`].
//!
//! ⚠️ **O par público/privado ficou como estava**, de propósito. Hoje o `open_rename` tem **um**
//! chamador (o wrapper acima dele) e a indirecção lê-se como resíduo — mas ela é anterior a este
//! corte e a decisão de a colapsar é de quem tem o report que a criou, não da integração.

use crate::ids;

/// Shared entry-path for rename mode (right-click "Rename..." +
/// long-press). Wipes any leftover text from a prior rename session
/// (Cancel / Blur paths don't necessarily clear), reinstalls the
/// TextInput state as `Focused`, and parks focus on the field. The
/// host's `pending_rename_seed` drain fills the buffer with the
/// entity's current `Name` on the next frame.
///
/// Side-table safety: `HIER_RENAME_INPUT` has no associated
/// `widget_color` / `panel_z` / `panel_scroll` / `tooltip` entries,
/// so the force-overwrite `store.register` (vs `register_if_absent`)
/// only resets buffer / caret / state — the intended effect.
pub fn open_rename_public(store: &mut crate::interaction::WidgetStore) {
    open_rename(store)
}

fn open_rename(store: &mut crate::interaction::WidgetStore) {
    store.register(
        ids::HIER_RENAME_INPUT,
        crate::interaction::InteractiveState::TextInput {
            state: crate::widget::TextInputState::Focused,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(ids::HIER_RENAME_INPUT));
    // Esc aborts the rename (it used to be spelled as this id, hardcoded inside
    // `dispatch_key`; the field says so itself now).
    store.mark_cancel_on_escape(ids::HIER_RENAME_INPUT);
}
