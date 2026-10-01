//! **As notas de um painel — as faces de cada ranhura** (2026-09-30; para QUALQUER painel desde
//! 2026-10-01).
//!
//! Uma nota ocupa uma RANHURA pela posição dela na lista do painel (a nota `i` pinta-se na
//! ranhura `i`). Cada ranhura tem cinco ids: o fundo (botão direito ⇒ menu da nota), o título,
//! o corpo, a pega de 10 pontos que a arrasta e o botão que a minimiza.
//!
//! ⚠️ **Vivem aqui e não no `widget::showcase`** porque apagar, duplicar e mover uma nota PERMUTA
//! as caixas de texto das ranhuras — é uma operação do `WidgetStore` (`interaction`), e
//! `interaction` não pode ler `widget` (DAG dos módulos de topo).
//!
//! ⛔ **A pertença de um id a uma nota lê-se por [`note_index_in`], nunca por uma faixa de
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

/// Quantas notas cabem num painel — o comprimento das tabelas.
pub const NOTES_PER_PANEL: usize = NOTE_SLOT_IDS.len();

/// O sal do botão de MINIMIZAR de cada ranhura do Inspector, derivado da pega (o idioma da
/// [`super::grip_de`]: um XOR por uma constante ímpar fixa é uma bijecção, sem tabela à mão).
const SAL_DA_DOBRA: u64 = 0x0D0B_7A1E_4F3C_9A65;

/// ⭐⭐ **As faces das ranhuras de UM painel** (2026-10-01).
///
/// ⛔ Até 30/09 as doze ranhuras eram UMA tabela para os painéis que pintavam notas, e o texto de
/// uma nota mora na CAIXA da ranhura ⇒ a nota `0` de dois painéis lia o mesmo texto. Desde
/// 2026-10-01 (ordem do dono: *«a possibilidade de criar notas deve existir em quaisquer
/// painéis»*) **todo painel tem as suas**, derivadas do id dele por [`note_ids`].
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
    /// ⭐ O botão de MINIMIZAR (2026-10-01: *«crie um botão nas notas que possibilite minimizar»*).
    pub fold: [NodeId; NOTES_PER_PANEL],
}

/// Uma das faces de uma nota — o que um id de nota É.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NoteFace {
    Slot,
    Title,
    Body,
    Grip,
    Fold,
}

impl NoteIds {
    /// As tabelas com a face de cada uma.
    #[must_use]
    pub const fn faces(&self) -> [(NoteFace, &[NodeId; NOTES_PER_PANEL]); 5] {
        [
            (NoteFace::Slot, &self.slot),
            (NoteFace::Title, &self.title),
            (NoteFace::Body, &self.body),
            (NoteFace::Grip, &self.grip),
            (NoteFace::Fold, &self.fold),
        ]
    }

    /// `(ranhura, face)` de `id` neste painel.
    #[must_use]
    pub fn find(&self, id: NodeId) -> Option<(usize, NoteFace)> {
        self.faces()
            .into_iter()
            .find_map(|(face, t)| t.iter().position(|x| *x == id).map(|i| (i, face)))
    }
}

const fn com_sal(t: [NodeId; NOTES_PER_PANEL], sal: u64) -> [NodeId; NOTES_PER_PANEL] {
    let mut out = [NodeId(0); NOTES_PER_PANEL];
    let mut i = 0;
    while i < NOTES_PER_PANEL {
        out[i] = NodeId(t[i].0 ^ sal);
        i += 1;
    }
    out
}

/// As ranhuras do INSPECTOR — as tabelas de sempre (os nomes `insp_note_*` são delas).
pub const INSP_NOTES: NoteIds = NoteIds {
    slot: NOTE_SLOT_IDS,
    title: NOTE_TITLE_IDS,
    body: NOTE_BODY_IDS,
    grip: NOTE_GRIP_IDS,
    fold: com_sal(NOTE_GRIP_IDS, SAL_DA_DOBRA),
};

/// ⭐ **O sal de um painel** — o *splitmix64* do id dele, forçado a ímpar (nunca zero ⇒ nunca as
/// ranhuras do Inspector). Painéis diferentes dão sais diferentes salvo colisão do hash, que o gate
/// `os_ids_das_notas_dos_paineis_registados_sao_distintos` mede sobre os painéis REAIS.
const fn sal_do_painel(panel: NodeId) -> u64 {
    let mut z = panel.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) | 1
}

/// ⭐⭐ **As ranhuras de `panel`** — as do Inspector para ele, e para qualquer outro painel as do
/// Inspector com o sal dele. Uma função, e nunca uma tabela de painéis: um painel novo tem notas
/// sem ninguém se lembrar de o pôr numa lista.
#[must_use]
pub const fn note_ids(panel: NodeId) -> NoteIds {
    if panel.0 == INSP_PANEL.0 {
        return INSP_NOTES;
    }
    let s = sal_do_painel(panel);
    NoteIds {
        slot: com_sal(INSP_NOTES.slot, s),
        title: com_sal(INSP_NOTES.title, s),
        body: com_sal(INSP_NOTES.body, s),
        grip: com_sal(INSP_NOTES.grip, s),
        fold: com_sal(INSP_NOTES.fold, s),
    }
}

/// As ranhuras da GALERIA de widgets.
pub const GAL_NOTES: NoteIds = note_ids(GAL_PANEL);

/// ⭐ **A nota de `panel` a que `id` pertence** — qualquer das faces dela. É a pergunta do botão
/// direito (o painel é o que está sob o cursor) e do arrasto pela pega.
#[must_use]
pub fn note_index_in(panel: NodeId, id: NodeId) -> Option<usize> {
    note_ids(panel).find(id).map(|(i, _)| i)
}

/// A nota de `panel` cuja PEGA é `id` — só a pega arrasta; o título e o corpo são caixas de texto.
#[must_use]
pub fn note_of_grip_in(panel: NodeId, id: NodeId) -> Option<usize> {
    note_ids(panel).grip.iter().position(|x| *x == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toda_face_de_uma_nota_responde_pela_nota_dela() {
        for panel in [INSP_PANEL, GAL_PANEL, NodeId(0xABCD)] {
            let n = note_ids(panel);
            for i in 0..NOTES_PER_PANEL {
                for (face, t) in n.faces() {
                    assert_eq!(n.find(t[i]), Some((i, face)));
                    assert_eq!(note_index_in(panel, t[i]), Some(i));
                }
                assert_eq!(note_of_grip_in(panel, n.grip[i]), Some(i));
                assert_eq!(note_of_grip_in(panel, n.title[i]), None);
            }
        }
        assert_eq!(note_index_in(INSP_PANEL, INSP_SAMPLE_TEXT), None);
    }

    /// ⭐⭐ **Cada painel tem ranhuras próprias, sem colisão entre nenhum, e o Inspector guarda as de
    /// sempre.** *Mutação: o
    /// sal a zero ⇒ todo painel recebe as ranhuras do Inspector.*
    #[test]
    fn cada_painel_tem_as_suas_ranhuras() {
        assert_eq!(note_ids(INSP_PANEL).title, NOTE_TITLE_IDS);
        // ⭐ Desde 2026-10-01 QUALQUER painel tem notas: a varredura cobre os painéis de secções e
        //    as superfícies sem secção que a porta de rolagem serve.
        let paineis = [
            INSP_PANEL,
            GAL_PANEL,
            crate::ids::VECTOR_PANEL,
            crate::ids::PHYSICS_PANEL,
            crate::ids::SCULPT3D_PANEL,
            crate::ids::GS_PANEL,
            crate::ids::PAINTER_LAYERS_PANEL,
            crate::ids::WET_TUNING_PANEL,
            crate::ids::AUDIO_EDITOR_PANEL,
            crate::ids::AUDIO_MIXER_PANEL,
            crate::ids::HIER_PANEL,
            crate::ids::PAD_PANEL,
            crate::ids::TOKENS_PANEL,
        ];
        let mut todos: Vec<NodeId> = paineis
            .into_iter()
            .flat_map(|p| {
                note_ids(p)
                    .faces()
                    .into_iter()
                    .flat_map(|(_, t)| t.iter().copied())
                    .collect::<Vec<_>>()
            })
            .collect();
        let n = todos.len();
        assert_eq!(n, paineis.len() * 5 * NOTES_PER_PANEL);
        todos.sort_by_key(|i| i.0);
        todos.dedup();
        assert_eq!(todos.len(), n);
        assert_eq!(note_index_in(GAL_PANEL, NOTE_TITLE_IDS[0]), None);
    }
}
