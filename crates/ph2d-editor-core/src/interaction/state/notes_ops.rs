//! ⭐⭐ **As NOTAS de um painel** — criar, recolorir, apagar, duplicar e mover (ordem do dono,
//! 2026-09-30: *«Coloque os 10 pontinhos de arrastar também nas notas. Botão direito sobre as
//! notas devem ter no menu a mudança de cor das notas, opções de apagar e duplicar.»*).
//!
//! ⚠️ **Irmão do `chrome_ops` pelo tecto de LOC e pelo assunto:** as três operações de antes
//! (ler, empurrar, recolorir) viviam lá; as três novas são as que PERMUTAM a lista, e é essa
//! permutação que tem uma lei própria.
//!
//! ## ⛔ A lei da permutação: o TEXTO viaja com a nota
//!
//! A nota `i` pinta-se na ranhura `i`, e o título e o corpo dela são as caixas de texto
//! `NOTE_TITLE_IDS[i]` / `NOTE_BODY_IDS[i]` — o texto mora na CAIXA, não na [`NoteData`]. ⇒ toda
//! operação que muda a posição de uma nota (apagar a de cima, duplicar, arrastar) tem de levar as
//! caixas com ela ([`WidgetStore::permute_note_texts`]), senão a nota de baixo sobe e herda o texto
//! da que foi apagada.
//!
//! ⚠️ **As caixas são partilhadas entre a Galeria de widgets e o Inspector** (as mesmas doze
//! ranhuras — pré-existente). Permutar as notas de um painel permuta também o texto das do outro
//! nas mesmas ranhuras; a Galeria é superfície de demonstração, e a dívida fica nomeada no handoff.

use super::{InteractiveState, WidgetStore};
use crate::ids::{NOTE_BODY_IDS, NOTE_TITLE_IDS, NOTES_PER_PANEL};
use crate::interaction::types::NoteData;
use crate::widget::TextInputState;
use ph2d_a11y::NodeId;

/// ⭐ **Um arrasto de nota pela pega** — a irmã do [`super::SectionDrag`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NoteDrag {
    pub panel: NodeId,
    pub index: usize,
    pub down_x: f32,
    pub down_y: f32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    /// Vira `true` depois do limiar das abas — um Down+Up parado na pega não move nada.
    pub active: bool,
}

impl WidgetStore {
    /// Read the per-panel note list. Returns an empty slice when no
    /// notes have been created for the panel.
    pub fn notes_for_panel(&self, panel: NodeId) -> &[NoteData] {
        self.notes_per_panel
            .get(&panel)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Acrescenta uma nota ao painel, no fim da `section` (ou do painel, com `None`). Devolve o
    /// índice dela — `None` quando o painel já tem [`NOTES_PER_PANEL`] notas (uma ranhura por
    /// nota, e as ranhuras são doze).
    pub fn notes_push(
        &mut self,
        panel: NodeId,
        color_idx: u8,
        section: Option<NodeId>,
    ) -> Option<usize> {
        let list = self.notes_per_panel.entry(panel).or_default();
        if list.len() >= NOTES_PER_PANEL {
            return None;
        }
        list.push(NoteData {
            color_idx,
            title: ph2d_i18n::tr_with("chrome.interaction.note_n", &[("n", &(list.len() + 1))]),
            body: String::new(),
            section,
        });
        Some(list.len() - 1)
    }

    /// Update an existing note's color index.
    pub fn note_set_color(&mut self, panel: NodeId, index: usize, color_idx: u8) {
        if let Some(list) = self.notes_per_panel.get_mut(&panel)
            && let Some(note) = list.get_mut(index)
        {
            // ⚠️ Sem teto aqui: a paleta cresceu (2026-09-30) e quem a lê é a porta
            // `panel_chrome::highlighter_rgba`, que cai na última cor em vez de estourar.
            note.color_idx = color_idx;
        }
    }

    /// ⭐ **Apaga a nota `index`** — as de baixo sobem, e o texto delas sobe com elas.
    pub fn note_delete(&mut self, panel: NodeId, index: usize) {
        let Some(list) = self.notes_per_panel.get_mut(&panel) else {
            return;
        };
        if index >= list.len() {
            return;
        }
        list.remove(index);
        let order: Vec<usize> = (0..=list.len()).filter(|&i| i != index).collect();
        self.permute_note_texts(&order);
    }

    /// ⭐ **Duplica a nota `index`** — a cópia nasce logo a seguir, na mesma secção, com a mesma
    /// cor e o mesmo texto. Devolve o índice da cópia; `None` com o painel cheio.
    pub fn note_duplicate(&mut self, panel: NodeId, index: usize) -> Option<usize> {
        let list = self.notes_per_panel.get_mut(&panel)?;
        if index >= list.len() || list.len() >= NOTES_PER_PANEL {
            return None;
        }
        let copia = list[index].clone();
        list.insert(index + 1, copia);
        let n = list.len();
        let order: Vec<usize> = (0..n).map(|k| if k <= index { k } else { k - 1 }).collect();
        self.permute_note_texts(&order);
        Some(index + 1)
    }

    /// ⭐⭐ **Move a nota `from` para a `section`, na posição `rank` entre as notas dessa secção**
    /// (`rank` maior que as que lá estão = a última). É o fim de um arrasto pela pega.
    ///
    /// A posição na LISTA só importa entre notas da mesma secção — é a ordem em que o painel as
    /// pinta no fim dela —, logo uma secção sem notas recebe a nota no fim da lista.
    pub fn note_move(&mut self, panel: NodeId, from: usize, section: Option<NodeId>, rank: usize) {
        let Some(list) = self.notes_per_panel.get(&panel) else {
            return;
        };
        let Some(order) = ordem_depois_de_mover(list, from, section, rank) else {
            return;
        };
        let identidade = order.iter().enumerate().all(|(k, &o)| k == o);
        let mudou_de_seccao = list[from].section != section;
        if identidade && !mudou_de_seccao {
            return;
        }
        let mut nova: Vec<NoteData> = order.iter().map(|&o| list[o].clone()).collect();
        if let Some(n) = order.iter().position(|&o| o == from) {
            nova[n].section = section;
        }
        self.notes_per_panel.insert(panel, nova);
        self.permute_note_texts(&order);
    }

    /// ⛔ **A lei da permutação** (ver o cabeçalho): a ranhura `k` recebe o texto que estava na
    /// ranhura `order[k]`; as ranhuras para lá de `order.len()` ficam vazias.
    ///
    /// ⚠️ Um campo de nota com o teclado larga-o: o texto que ele segurava pode ter mudado de
    /// ranhura, e um cursor numa caixa cujo conteúdo trocou por baixo escreveria na nota errada.
    pub fn permute_note_texts(&mut self, order: &[usize]) {
        let ler = |s: &Self, id: NodeId| match s.get(id) {
            Some(InteractiveState::TextInput { text, .. }) => text.clone(),
            _ => String::new(),
        };
        let titulos: Vec<String> = NOTE_TITLE_IDS.iter().map(|id| ler(self, *id)).collect();
        let corpos: Vec<String> = NOTE_BODY_IDS.iter().map(|id| ler(self, *id)).collect();
        for k in 0..NOTES_PER_PANEL {
            let origem = order.get(k).copied().filter(|&o| o < NOTES_PER_PANEL);
            for (ids, antigos) in [(&NOTE_TITLE_IDS, &titulos), (&NOTE_BODY_IDS, &corpos)] {
                let novo = origem.map(|o| antigos[o].clone()).unwrap_or_default();
                if let Some(InteractiveState::TextInput {
                    state,
                    text,
                    caret,
                    selection_anchor,
                }) = self.get_mut(ids[k])
                {
                    *caret = novo.len();
                    *text = novo;
                    *state = TextInputState::Normal;
                    *selection_anchor = None;
                }
            }
        }
        if self
            .focus_id()
            .is_some_and(|f| NOTE_TITLE_IDS.contains(&f) || NOTE_BODY_IDS.contains(&f))
        {
            self.set_focus(None);
        }
    }

    /// O arrasto de nota em curso, se houver.
    #[must_use]
    pub fn note_drag(&self) -> Option<NoteDrag> {
        self.section_prefs.note_drag
    }

    /// Down primário na pega da nota `index` de `panel`.
    pub fn begin_note_drag(&mut self, panel: NodeId, index: usize, x: f32, y: f32) {
        self.section_prefs.note_drag = Some(NoteDrag {
            panel,
            index,
            down_x: x,
            down_y: y,
            cursor_x: x,
            cursor_y: y,
            active: false,
        });
    }

    /// Avança o cursor; vira `active` depois do limiar das abas (a mesma pergunta do arrasto de
    /// secção: quanto tem a mão de andar para um clique virar um arrasto).
    pub fn update_note_drag(&mut self, x: f32, y: f32) {
        if let Some(d) = self.section_prefs.note_drag.as_mut() {
            d.cursor_x = x;
            d.cursor_y = y;
            if (y - d.down_y).abs() > super::TAB_DRAG_THRESHOLD_PX
                || (x - d.down_x).abs() > super::TAB_DRAG_THRESHOLD_PX
            {
                d.active = true;
            }
        }
    }

    /// Tira o arrasto (no Up).
    pub fn end_note_drag(&mut self) -> Option<NoteDrag> {
        self.section_prefs.note_drag.take()
    }
}

/// A ORDEM da lista depois de mover `from` para `section` na posição `rank` entre as notas dela —
/// `order[k]` é o índice ANTIGO da nota que fica na posição `k`. `None` se `from` não existe.
fn ordem_depois_de_mover(
    list: &[NoteData],
    from: usize,
    section: Option<NodeId>,
    rank: usize,
) -> Option<Vec<usize>> {
    if from >= list.len() {
        return None;
    }
    let mut order: Vec<usize> = (0..list.len()).filter(|&i| i != from).collect();
    let membros: Vec<usize> = order
        .iter()
        .enumerate()
        .filter(|(_, o)| list[**o].section == section)
        .map(|(pos, _)| pos)
        .collect();
    let pos = match membros.get(rank) {
        Some(&p) => p,
        None => membros.last().map_or(order.len(), |&p| p + 1),
    };
    order.insert(pos, from);
    Some(order)
}

#[cfg(test)]
#[path = "notes_ops_tests.rs"]
mod tests;
