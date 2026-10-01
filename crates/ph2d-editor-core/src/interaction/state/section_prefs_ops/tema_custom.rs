//! ⭐⭐ **O TEMA CUSTOM — a combinação de temas dos cartões que o artista GRAVA e volta a pôr.**
//!
//! Ordem do dono (2026-09-30): *«Ainda não temos um botão para resetar todos os themes dos cards e
//! nem um botão para salvar o theme custom.»* Três verbos no menu do botão direito do título de uma
//! secção, e os três agem sobre TODOS os cartões — de todos os painéis com secções (desde 2026-09-30
//! o Vector também), não sobre a secção do pedido:
//!
//! - **Reset All Card Themes** — todas as secções voltam ao tema do app.
//! - **Save Custom Theme** — a combinação de agora fica guardada.
//! - **Load Custom Theme** — a combinação guardada volta a ser a de agora.
//!
//! ⚠️ **A escolha de cada secção já se gravava sozinha** (`~/.ph2d/sections.txt`, desde 29/09): o
//! que faltava não era persistir o estado, era **um sítio para onde voltar** depois de experimentar
//! — é por isso que o custom é uma cópia SEPARADA do mapa vivo, e o *Reset* não lhe toca.
//!
//! ⚠️ **`Some(vazio)` é uma gravação legítima** («todas no tema do app»), diferente de `None`
//! (nunca gravou). O ficheiro distingue os dois pela linha `custom=1`, senão um custom vazio lia-se
//! como ausente e o *Load* depois de reabrir o app deixava de fazer o que fazia antes de fechar.

use super::super::WidgetStore;
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;
use std::collections::BTreeMap;

impl WidgetStore {
    /// ⭐ **Reset All Card Themes** — todas as secções voltam ao tema do app. O custom guardado fica.
    pub fn reset_section_themes(&mut self) {
        if !self.section_prefs.themes.is_empty() {
            self.section_prefs.themes.clear();
            self.section_prefs.dirty = true;
        }
    }

    /// ⭐ **Save Custom Theme** — guarda a combinação de agora (substitui a anterior).
    pub fn save_custom_section_themes(&mut self) {
        let agora = Some(self.section_prefs.themes.clone());
        if self.section_prefs.custom != agora {
            self.section_prefs.custom = agora;
            self.section_prefs.dirty = true;
        }
    }

    /// ⭐ **Load Custom Theme** — a combinação guardada volta a ser a de agora. Devolve `false`
    /// quando nada foi gravado (e aí o estado não se mexe).
    pub fn load_custom_section_themes(&mut self) -> bool {
        let Some(custom) = self.section_prefs.custom.clone() else {
            return false;
        };
        if self.section_prefs.themes != custom {
            self.section_prefs.themes = custom;
            self.section_prefs.dirty = true;
        }
        true
    }

    /// O custom guardado, se houver.
    #[must_use]
    pub fn custom_section_themes(&self) -> Option<&BTreeMap<NodeId, Theme>> {
        self.section_prefs.custom.as_ref()
    }

    /// ⭐ **A combinação de agora É a guardada?** — a marca do menu acende a linha *Load Custom
    /// Theme* com isto. `false` quando nada foi gravado.
    #[must_use]
    pub fn section_themes_are_the_custom(&self) -> bool {
        self.section_prefs.custom.as_ref() == Some(&self.section_prefs.themes)
    }
}

/// Acrescenta ao texto do `sections.txt` as linhas do custom (nada se nunca foi gravado).
pub(super) fn escreve(store: &WidgetStore, s: &mut String) {
    use std::fmt::Write as _;
    let Some(custom) = store.custom_section_themes() else {
        return;
    };
    let _ = writeln!(s, "custom=1");
    for (id, t) in custom {
        let _ = writeln!(s, "custom.{}={}", id.0, t.id());
    }
}

/// O custom a ser lido, linha a linha, do `sections.txt`.
#[derive(Default)]
pub(super) struct Leitura {
    marcado: bool,
    temas: BTreeMap<NodeId, Theme>,
}

impl Leitura {
    /// Uma linha `chave=valor` que não era da ordem nem de um tema vivo. As ilegíveis saltam.
    pub(super) fn linha(&mut self, k: &str, v: &str) {
        if k == "custom" {
            self.marcado = true;
        } else if let Some(id) = k.strip_prefix("custom.")
            && let (Ok(id), Some(t)) = (id.parse(), Theme::from_id(v.trim()))
        {
            self.temas.insert(NodeId(id), t);
        }
    }

    /// O custom lido: `None` sem a linha `custom=1`.
    pub(super) fn fim(self) -> Option<BTreeMap<NodeId, Theme>> {
        self.marcado.then_some(self.temas)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{restore_section_prefs_text, section_prefs_text};
    use super::*;

    const A: NodeId = NodeId(1);
    const B: NodeId = NodeId(2);

    /// ⭐⭐ **Reset limpa as secções e deixa o custom; Load devolve a combinação guardada.**
    /// *Mutação: o `reset` a limpar também o custom ⇒ o `load` devolve `false`.*
    #[test]
    fn reset_limpa_e_load_devolve_a_combinacao_guardada() {
        let mut s = WidgetStore::with_capacity(4);
        assert!(
            !s.load_custom_section_themes(),
            "sem gravar, nao ha' o que por"
        );
        s.set_section_theme(A, Some(Theme::Oled));
        s.set_section_theme(B, Some(Theme::Candy));
        s.save_custom_section_themes();
        assert!(s.section_themes_are_the_custom());
        let _ = s.take_section_prefs_dirty();

        s.reset_section_themes();
        assert!(s.section_themes().is_empty(), "o reset devolve tudo ao app");
        assert!(s.take_section_prefs_dirty(), "o reset grava");
        assert!(!s.section_themes_are_the_custom());

        assert!(s.load_custom_section_themes());
        assert_eq!(s.section_theme(A), Some(Theme::Oled));
        assert_eq!(s.section_theme(B), Some(Theme::Candy));
        assert!(s.take_section_prefs_dirty(), "o load grava");
    }

    /// ⭐ **Um custom VAZIO é uma gravação**, e atravessa o ficheiro como tal. *Mutação: escrever o
    /// `custom=1` só com entradas ⇒ o vazio volta como «nunca gravou».*
    #[test]
    fn o_custom_atravessa_o_ficheiro_mesmo_vazio() {
        let mut a = WidgetStore::with_capacity(4);
        a.set_section_theme(A, Some(Theme::Light));
        a.save_custom_section_themes();
        a.set_section_theme(A, Some(Theme::Sunset));
        let mut b = WidgetStore::with_capacity(4);
        restore_section_prefs_text(&mut b, &section_prefs_text(&a));
        assert_eq!(b.section_theme(A), Some(Theme::Sunset), "o vivo volta");
        assert_eq!(
            b.custom_section_themes().and_then(|c| c.get(&A)).copied(),
            Some(Theme::Light),
            "o custom volta"
        );

        let mut vazio = WidgetStore::with_capacity(4);
        vazio.save_custom_section_themes();
        let mut c = WidgetStore::with_capacity(4);
        restore_section_prefs_text(&mut c, &section_prefs_text(&vazio));
        assert!(c.custom_section_themes().is_some_and(BTreeMap::is_empty));

        // e quem nunca gravou volta sem custom
        let mut d = WidgetStore::with_capacity(4);
        restore_section_prefs_text(&mut d, &section_prefs_text(&WidgetStore::with_capacity(4)));
        assert!(d.custom_section_themes().is_none());
    }
}
