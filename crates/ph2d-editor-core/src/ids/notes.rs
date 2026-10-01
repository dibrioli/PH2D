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

/// ⭐⭐ **As quatro faces das ranhuras de UM painel** (2026-10-01).
///
/// ⛔ Até aqui as doze ranhuras eram UMA tabela para os dois painéis que pintam notas, e o texto de
/// uma nota mora na CAIXA da ranhura ⇒ a nota `0` da Galeria e a nota `0` do Inspector **liam o
/// mesmo texto**, e apagar uma nota num painel permutava o texto das do outro. A dívida estava
/// nomeada no handoff de 30/09 («curar pede ids por painel — é modelo, não fiação»); a cura é isto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NoteIds {
    /// Fundo de cada ranhura (botão direito ⇒ menu da nota).
    pub slot: [NodeId; NOTES_PER_PANEL],
    /// Título editável.
    pub title: [NodeId; NOTES_PER_PANEL],
    /// Corpo editável.
    pub body: [NodeId; NOTES_PER_PANEL],
    /// A pega de 10 pontos.
    pub grip: [NodeId; NOTES_PER_PANEL],
}

impl NoteIds {
    /// As quatro tabelas, pela ordem fundo · título · corpo · pega.
    #[must_use]
    pub const fn faces(&self) -> [&[NodeId; NOTES_PER_PANEL]; 4] {
        [&self.slot, &self.title, &self.body, &self.grip]
    }
}

/// As ranhuras do INSPECTOR — as tabelas de sempre (os nomes `insp_note_*` são delas).
pub const INSP_NOTES: NoteIds = NoteIds {
    slot: NOTE_SLOT_IDS,
    title: NOTE_TITLE_IDS,
    body: NOTE_BODY_IDS,
    grip: NOTE_GRIP_IDS,
};

/// O sal das ranhuras da Galeria — o mesmo idioma da pega derivada de uma secção
/// ([`super::grip_de`]): um XOR por uma constante ímpar fixa é uma bijecção, logo as doze ranhuras
/// da Galeria nunca colidem com as do Inspector por acaso, e não há uma segunda tabela à mão.
/// ⚠️ A distinção dos 96 ids é gateada (`os_noventa_e_seis_ids_das_notas_sao_distintos`).
const SAL_DA_GALERIA: u64 = 0x6A1E_77E2_0F0C_A5D3;

const fn com_sal(t: [NodeId; NOTES_PER_PANEL]) -> [NodeId; NOTES_PER_PANEL] {
    let mut out = [NodeId(0); NOTES_PER_PANEL];
    let mut i = 0;
    while i < NOTES_PER_PANEL {
        out[i] = NodeId(t[i].0 ^ SAL_DA_GALERIA);
        i += 1;
    }
    out
}

/// As ranhuras da GALERIA de widgets — derivadas das do Inspector pelo [`SAL_DA_GALERIA`].
pub const GAL_NOTES: NoteIds = NoteIds {
    slot: com_sal(NOTE_SLOT_IDS),
    title: com_sal(NOTE_TITLE_IDS),
    body: com_sal(NOTE_BODY_IDS),
    grip: com_sal(NOTE_GRIP_IDS),
};

/// ⭐⭐ **Os painéis que PINTAM notas, e as ranhuras de cada um** — a única lista.
///
/// ⛔ Ela é também a resposta a *«este painel oferece Create Note?»*: até 2026-10-01 o botão
/// direito oferecia-o a todo painel fora de uma lista de EXCLUSÃO, e só estes dois pintam notas ⇒
/// no Vector, na Física, no Áudio… a nota nascia no store e **nunca aparecia** (um botão mudo).
pub const NOTE_HOSTS: [(NodeId, NoteIds); 2] = [(INSP_PANEL, INSP_NOTES), (GAL_PANEL, GAL_NOTES)];

/// As ranhuras de `panel`, se ele pinta notas.
#[must_use]
pub fn note_ids(panel: NodeId) -> Option<&'static NoteIds> {
    NOTE_HOSTS.iter().find(|(p, _)| *p == panel).map(|(_, n)| n)
}

/// ⭐ **A nota a que um id pertence** — qualquer das quatro faces (fundo · título · corpo · pega),
/// de qualquer painel que pinte notas. É a pergunta do botão direito: sobre o título de uma nota
/// ele abre o menu DELA, não o de criar. (O PAINEL do menu é o que está sob o cursor.)
#[must_use]
pub fn note_index_of(id: NodeId) -> Option<usize> {
    NOTE_HOSTS.iter().find_map(|(_, n)| {
        n.faces()
            .iter()
            .find_map(|tabela| tabela.iter().position(|x| *x == id))
    })
}

/// A nota cuja PEGA é `id` — só a pega arrasta; o título e o corpo são caixas de texto.
#[must_use]
pub fn note_of_grip(id: NodeId) -> Option<usize> {
    NOTE_HOSTS
        .iter()
        .find_map(|(_, n)| n.grip.iter().position(|x| *x == id))
}

/// Uma caixa de texto de nota (título ou corpo) de QUALQUER painel?
#[must_use]
pub fn is_note_text(id: NodeId) -> bool {
    NOTE_HOSTS
        .iter()
        .any(|(_, n)| n.title.contains(&id) || n.body.contains(&id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toda_face_de_uma_nota_responde_pela_nota_dela() {
        for (_, n) in NOTE_HOSTS {
            for i in 0..NOTES_PER_PANEL {
                for t in n.faces() {
                    assert_eq!(note_index_of(t[i]), Some(i));
                }
                assert_eq!(note_of_grip(n.grip[i]), Some(i));
                assert_eq!(note_of_grip(n.title[i]), None);
                assert!(is_note_text(n.title[i]) && is_note_text(n.body[i]));
                assert!(!is_note_text(n.slot[i]) && !is_note_text(n.grip[i]));
            }
        }
        assert_eq!(note_index_of(INSP_SAMPLE_TEXT), None);
    }

    /// ⭐⭐ **Os dois painéis têm ranhuras PRÓPRIAS** — os 96 ids são distintos (sem isto a nota `0`
    /// da Galeria e a do Inspector liam o mesmo texto). *Mutação: o `SAL_DA_GALERIA` a `0` ⇒ as
    /// duas tabelas coincidem.*
    #[test]
    fn os_noventa_e_seis_ids_das_notas_sao_distintos() {
        let mut todos: Vec<NodeId> = NOTE_HOSTS
            .iter()
            .flat_map(|(_, n)| n.faces().into_iter().flatten().copied())
            .collect();
        assert_eq!(todos.len(), 96);
        todos.sort_by_key(|i| i.0);
        todos.dedup();
        assert_eq!(todos.len(), 96);
        assert_eq!(note_ids(INSP_PANEL), Some(&INSP_NOTES));
        assert_eq!(note_ids(GAL_PANEL), Some(&GAL_NOTES));
        assert_eq!(note_ids(HIER_PANEL), None);
    }
}
