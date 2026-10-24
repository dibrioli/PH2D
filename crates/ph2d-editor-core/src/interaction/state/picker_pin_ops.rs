//! ⭐ **O PINO e a POSIÇÃO do seletor de cor** (ordem do dono, 24/10).
//!
//! - **O pino** — fixado, um Down fora do seletor deixa de o fechar; outro dono que o abra só lhe
//!   muda o alvo (o seletor é UM, partilhado por `picker_target`). Todo FECHO desafixa — o `×`, a
//!   amostra clicada outra vez, o painel que fecha —, na porta única (`set_picker_target(None)`).
//! - **A posição** — o desvio arrastado (`blender_picker_offset` do seletor, relativo ao encaixe
//!   AO LADO do Inspector, `color_picker_demo::picker_anchor_beside`) grava-se em
//!   `~/.ph2d/sections.txt` pela porta de [`super::section_prefs_ops::section_prefs_text`]: o
//!   formato é do editor e a shell só sabe onde fica o disco — logo a shell não cresce.

use super::WidgetStore;
use crate::ids;

/// A chave do desvio no ficheiro (`picker_offset=dx,dy`).
const CHAVE: &str = "picker_offset";

impl WidgetStore {
    /// O seletor está fixado?
    #[must_use]
    pub fn picker_pinned(&self) -> bool {
        self.picker_pinned
    }

    pub fn set_picker_pinned(&mut self, pinned: bool) {
        self.picker_pinned = pinned;
    }

    /// Aberto e NÃO fixado — o que um light-dismiss de fora do editor (a shell) pergunta.
    #[must_use]
    pub fn picker_dismissable(&self) -> bool {
        self.picker_target.is_some() && !self.picker_pinned
    }
}

/// Escreve o desvio do seletor, se o artista alguma vez o arrastou.
pub(super) fn escreve_posicao(store: &WidgetStore, s: &mut String) {
    use std::fmt::Write as _;
    if let Some((dx, dy)) = store.blender_picker_offset.get(&ids::INSP_BLENDER_PICKER) {
        let _ = writeln!(s, "{CHAVE}={dx},{dy}");
    }
}

/// Lê uma linha `k=v`; `true` se era a do desvio (mesmo ilegível — ela não é de mais ninguém).
pub(super) fn le_posicao(store: &mut WidgetStore, k: &str, v: &str) -> bool {
    if k != CHAVE {
        return false;
    }
    if let Some((dx, dy)) = v.split_once(',')
        && let (Ok(dx), Ok(dy)) = (dx.trim().parse::<f32>(), dy.trim().parse::<f32>())
        && dx.is_finite()
        && dy.is_finite()
    {
        store
            .blender_picker_offset
            .insert(ids::INSP_BLENDER_PICKER, (dx, dy));
    }
    true
}
