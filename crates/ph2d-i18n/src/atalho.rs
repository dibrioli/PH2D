//! ⭐ **A tecla dos atalhos, por sistema operativo** — a porta única.
//!
//! A tabela escreve sempre [`NA_TABELA`] (`Ctrl+`); no macOS o ecrã diz `Cmd+`. É verdade nos dois: o despacho de
//! teclas da shell aceita Ctrl **ou** ⌘ em todo o acorde (`input_dispatch/keyboard.rs`, `input_handlers.rs`:
//! `control_key() || super_key()`). Só o acorde (`Ctrl+X`) se troca: «Hold Ctrl» sem `+` é a tecla Control mesmo.
//! Censo: `crates/ph2d-i18n/tests/it/a_tecla_dos_atalhos_vem_da_porta.rs` (nenhum `Cmd+` cru no repo).

use std::borrow::Cow;

/// Para que sistema se escreve o rótulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sistema {
    /// macOS: o acorde diz `Cmd+`.
    Mac,
    /// Linux e Windows: o acorde diz `Ctrl+`, como a tabela.
    Outro,
}

impl Sistema {
    /// O sistema desta compilação.
    pub const DESTE: Self = if cfg!(target_os = "macos") {
        Self::Mac
    } else {
        Self::Outro
    };
}

/// Como a tabela escreve a tecla do acorde.
pub const NA_TABELA: &str = "Ctrl+";
const NO_MAC: &str = "Cmd+";

/// `texto` (com acordes escritos como [`NA_TABELA`]) dito para `sistema`.
#[must_use]
pub fn atalho_em(sistema: Sistema, texto: &str) -> Cow<'_, str> {
    match sistema {
        Sistema::Mac if texto.contains(NA_TABELA) => Cow::Owned(texto.replace(NA_TABELA, NO_MAC)),
        _ => Cow::Borrowed(texto),
    }
}

/// `texto` dito para o sistema desta compilação — a porta de quem escreve um atalho fora da tabela.
#[must_use]
pub fn atalho(texto: &str) -> Cow<'_, str> {
    atalho_em(Sistema::DESTE, texto)
}

/// A entrada `key` da tabela dita para `sistema`: a forma trocada guarda-se UMA vez por processo (a tabela é finita).
pub(crate) fn da_tabela(sistema: Sistema, key: &str) -> &'static str {
    let texto = crate::tabela(key);
    match atalho_em(sistema, texto) {
        Cow::Borrowed(s) => s,
        Cow::Owned(dita) => {
            static FEITAS: std::sync::Mutex<
                std::collections::BTreeMap<&'static str, &'static str>,
            > = std::sync::Mutex::new(std::collections::BTreeMap::new());
            let mut feitas = FEITAS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            feitas
                .entry(texto)
                .or_insert_with(|| Box::leak(dita.into_boxed_str()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_acorde_diz_cmd_so_no_mac() {
        let t = "Undo \u{00b7} Ctrl+Z · Redo Ctrl+Shift+Z · Hold Ctrl";
        assert_eq!(
            atalho_em(Sistema::Mac, t),
            "Undo \u{00b7} Cmd+Z · Redo Cmd+Shift+Z · Hold Ctrl"
        );
        assert_eq!(atalho_em(Sistema::Outro, t), t);
        assert!(matches!(atalho_em(Sistema::Mac, "Save"), Cow::Borrowed(_)));
    }

    /// *Mutação: `da_tabela` sem o `atalho_em` ⇒ o Mac diz Ctrl+.* (No Linux tirar a porta do `tr` não se vê: é a mesma
    /// palavra.)
    #[test]
    fn a_tabela_passa_pela_porta() {
        let k = "chrome.menu.undo_cmd_z";
        assert_eq!(da_tabela(Sistema::Mac, k), "Undo \u{00b7} Cmd+Z");
        assert_eq!(da_tabela(Sistema::Outro, k), "Undo \u{00b7} Ctrl+Z");
        assert!(
            std::ptr::eq(da_tabela(Sistema::Mac, k), da_tabela(Sistema::Mac, k)),
            "guarda-se uma vez"
        );
        assert_eq!(crate::tr(k), da_tabela(Sistema::DESTE, k));
    }
}
