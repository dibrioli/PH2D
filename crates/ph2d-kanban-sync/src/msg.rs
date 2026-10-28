//! O que viaja: as escritas, o retrato de quem entra, e as mensagens do Kanban.

use ph2d_board_sync::{Lamport, PeerId};
use ph2d_kanban_model::{KanbanFile, KanbanId, Record, RecordId};
use serde::{Deserialize, Serialize};

use crate::presence::KanbanPresence;

/// O alvo de um campo: o cabeçalho ou um registo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Target {
    Header,
    Record(RecordId),
}

/// Uma mudança que viaja. `kanban` é sempre o id DA SESSÃO (o do anfitrião, ou sorteado por quem o
/// partilhou a meio); cada par traduz para o seu.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum KWrite {
    /// Um registo que nasceu: todos os campos com este carimbo.
    Born {
        kanban: KanbanId,
        rec: Record,
        at: Lamport,
    },
    /// Um campo (índice em `Record::fields`/`Header::fields`, os bytes de lá).
    Set {
        kanban: KanbanId,
        target: Target,
        field: u8,
        value: Vec<u8>,
        at: Lamport,
    },
    /// Um elemento de um campo-conjunto: posto (`Some(valor)`) ou tirado (`None`). `idx` = a posição
    /// em que entrou (ordena os que entraram com o mesmo carimbo).
    Elem {
        kanban: KanbanId,
        target: Target,
        field: u8,
        key: Vec<u8>,
        value: Option<Vec<u8>>,
        idx: u32,
        at: Lamport,
    },
    /// Um Kanban que entrou na sessão a meio (um elemento novo no Quadro, ou partilhado depois).
    Shared { kanban: KanbanId, bytes: Vec<u8> },
    /// Os bytes de um ficheiro (uma vez por conteúdo).
    File(KanbanFile),
}

impl KWrite {
    pub(crate) fn at(&self) -> Option<Lamport> {
        match self {
            KWrite::Born { at, .. } | KWrite::Set { at, .. } | KWrite::Elem { at, .. } => Some(*at),
            KWrite::Shared { .. } | KWrite::File(_) => None,
        }
    }
}

/// Um registo de elemento: o valor (`None` = tirado), o carimbo, e a posição de entrada.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Elem {
    pub value: Option<Vec<u8>>,
    pub at: Lamport,
    pub idx: u32,
}

pub(crate) type FieldKey = (KanbanId, Target, u8);
pub(crate) type ElemKey = (KanbanId, Target, u8, Vec<u8>);

/// Tudo o que quem entra precisa: os Kanbans (postcard) e os carimbos, compactos e sem perda
/// ([`crate::snapshot`]). Os ficheiros vão a seguir, um por mensagem.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KSnapshot {
    pub(crate) kanbans: Vec<(KanbanId, Vec<u8>)>,
    pub(crate) stamps: crate::snapshot::Stamps,
    pub(crate) clock: u64,
}

/// As mensagens do Kanban — dentro de `ph2d_board_sync::Msg::Kanban` ou de [`crate::SoloMsg::K`].
/// ⛔ Só se ACRESCENTA no fim: o índice da variante é o byte que viaja.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum KMsg {
    /// Quem entrou na sala do Quadro pede os Kanbans dele.
    Want,
    Snapshot(Box<KSnapshot>),
    Writes(Vec<KWrite>),
    Presence(KanbanPresence),
    /// Quem saiu (a presença dele some).
    Gone(PeerId),
    /// Um bloco do retrato grande (`part` de `of`, os bytes postcard do [`KSnapshot`] aos pedaços):
    /// juntos, valem um [`KMsg::Snapshot`].
    SnapshotPart { part: u32, of: u32, bytes: Vec<u8> },
}

impl KMsg {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("postcard escreve qualquer mensagem")
    }

    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<KMsg> {
        postcard::from_bytes(bytes).ok()
    }
}
