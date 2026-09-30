//! **As notas de um painel — as quatro faces de cada ranhura, numa tabela só** (2026-09-30).
//!
//! Uma nota ocupa uma RANHURA pela posição dela na lista do painel (a nota `i` pinta-se na
//! ranhura `i`). Cada ranhura tem quatro ids: o fundo (botão direito ⇒ menu da nota), o título,
//! o corpo e a pega de 10 pontos que a arrasta.
//!
//! ⚠️ **Vivem aqui e não no `widget::showcase`** porque apagar, duplicar e mover uma nota PERMUTA
//! as caixas de texto das ranhuras — é uma operação do `WidgetStore` (`interaction`), e
//! `interaction` não pode ler `widget` (DAG dos módulos de topo).
//!
//! ⛔ **A pertença de um id a uma nota lê-se por [`note_index_of`], nunca por uma faixa de
//! números.** Até 2026-09-30 o botão direito perguntava `800..=811` — os ids das notas são hashes
//! FNV e nunca lá caíam, logo o menu da nota NUNCA abria (e o título e o corpo, registados por cima
//! do fundo, ganhavam o clique de qualquer maneira).

use super::*;

/// Fundo de cada ranhura de nota.
pub const NOTE_SLOT_IDS: [NodeId; 12] = [
    INSP_NOTE_SLOT_0,
    INSP_NOTE_SLOT_1,
    INSP_NOTE_SLOT_2,
    INSP_NOTE_SLOT_3,
    INSP_NOTE_SLOT_4,
    INSP_NOTE_SLOT_5,
    INSP_NOTE_SLOT_6,
    INSP_NOTE_SLOT_7,
    INSP_NOTE_SLOT_8,
    INSP_NOTE_SLOT_9,
    INSP_NOTE_SLOT_10,
    INSP_NOTE_SLOT_11,
];

/// Título editável de cada ranhura.
pub const NOTE_TITLE_IDS: [NodeId; 12] = [
    INSP_NOTE_TITLE_0,
    INSP_NOTE_TITLE_1,
    INSP_NOTE_TITLE_2,
    INSP_NOTE_TITLE_3,
    INSP_NOTE_TITLE_4,
    INSP_NOTE_TITLE_5,
    INSP_NOTE_TITLE_6,
    INSP_NOTE_TITLE_7,
    INSP_NOTE_TITLE_8,
    INSP_NOTE_TITLE_9,
    INSP_NOTE_TITLE_10,
    INSP_NOTE_TITLE_11,
];

/// Corpo editável de cada ranhura.
pub const NOTE_BODY_IDS: [NodeId; 12] = [
    INSP_NOTE_BODY_0,
    INSP_NOTE_BODY_1,
    INSP_NOTE_BODY_2,
    INSP_NOTE_BODY_3,
    INSP_NOTE_BODY_4,
    INSP_NOTE_BODY_5,
    INSP_NOTE_BODY_6,
    INSP_NOTE_BODY_7,
    INSP_NOTE_BODY_8,
    INSP_NOTE_BODY_9,
    INSP_NOTE_BODY_10,
    INSP_NOTE_BODY_11,
];

/// ⭐ A pega de 10 pontos de cada ranhura — a mesma pega das secções (ordem do dono, 2026-09-30:
/// *«Coloque os 10 pontinhos de arrastar também nas notas»*).
pub const NOTE_GRIP_IDS: [NodeId; 12] = [
    hash_node_id("insp_note_grip_0"),
    hash_node_id("insp_note_grip_1"),
    hash_node_id("insp_note_grip_2"),
    hash_node_id("insp_note_grip_3"),
    hash_node_id("insp_note_grip_4"),
    hash_node_id("insp_note_grip_5"),
    hash_node_id("insp_note_grip_6"),
    hash_node_id("insp_note_grip_7"),
    hash_node_id("insp_note_grip_8"),
    hash_node_id("insp_note_grip_9"),
    hash_node_id("insp_note_grip_10"),
    hash_node_id("insp_note_grip_11"),
];

/// Quantas notas cabem num painel — o comprimento das quatro tabelas.
pub const NOTES_PER_PANEL: usize = NOTE_SLOT_IDS.len();

/// ⭐ **A nota a que um id pertence** — qualquer das quatro faces (fundo · título · corpo · pega).
/// É a pergunta do botão direito: sobre o título de uma nota ele abre o menu DELA, não o de criar.
#[must_use]
pub fn note_index_of(id: NodeId) -> Option<usize> {
    [NOTE_SLOT_IDS, NOTE_TITLE_IDS, NOTE_BODY_IDS, NOTE_GRIP_IDS]
        .iter()
        .find_map(|tabela| tabela.iter().position(|n| *n == id))
}

/// A nota cuja PEGA é `id` — só a pega arrasta; o título e o corpo são caixas de texto.
#[must_use]
pub fn note_of_grip(id: NodeId) -> Option<usize> {
    NOTE_GRIP_IDS.iter().position(|n| *n == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toda_face_de_uma_nota_responde_pela_nota_dela() {
        for i in 0..NOTES_PER_PANEL {
            assert_eq!(note_index_of(NOTE_SLOT_IDS[i]), Some(i));
            assert_eq!(note_index_of(NOTE_TITLE_IDS[i]), Some(i));
            assert_eq!(note_index_of(NOTE_BODY_IDS[i]), Some(i));
            assert_eq!(note_index_of(NOTE_GRIP_IDS[i]), Some(i));
            assert_eq!(note_of_grip(NOTE_GRIP_IDS[i]), Some(i));
            assert_eq!(note_of_grip(NOTE_TITLE_IDS[i]), None);
        }
        assert_eq!(note_index_of(INSP_SAMPLE_TEXT), None);
    }

    #[test]
    fn os_quarenta_e_oito_ids_das_notas_sao_distintos() {
        let mut todos: Vec<NodeId> =
            [NOTE_SLOT_IDS, NOTE_TITLE_IDS, NOTE_BODY_IDS, NOTE_GRIP_IDS].concat();
        let n = todos.len();
        todos.sort_by_key(|i| i.0);
        todos.dedup();
        assert_eq!(todos.len(), n);
    }
}
