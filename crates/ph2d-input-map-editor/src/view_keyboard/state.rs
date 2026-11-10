//! O estado da vista B: a acção aberta, a sub-vista (teclado · comando), o alvo sob o ponteiro e o Add no lugar.

use ph2d_a11y::NodeId;
use ph2d_editor_core::registry::hash_node_id;
use ph2d_input::{ActionId, Binding, Key};

/// O desenho de baixo: o teclado ou o comando.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sub {
    #[default]
    Keyboard,
    Gamepad,
}

/// Um alvo da vista B.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KbHit {
    Row(ActionId),
    Add,
    Remove,
    Sub(Sub),
    Key(Key),
    /// Um botão ou uma metade de manche do comando desenhado.
    Pad(Binding),
    /// O × do ladrilho `n` da acção aberta.
    TileX(usize),
    /// O ladrilho tracejado que escuta.
    Listen,
    BarDead,
    BarPress,
}

const BASE: NodeId = hash_node_id("input_map_kb.target");
const STRIDE: u64 = 1 << 20;

const fn off(kind: u64, n: u64) -> NodeId {
    NodeId(BASE.0.wrapping_add(kind * STRIDE).wrapping_add(n))
}

const fn pad_code(b: Binding) -> u64 {
    match b {
        Binding::Key(k) => k.0 as u64,
        Binding::PadButton(p) => p as u64,
        Binding::PadAxis { axis, positive } => STRIDE / 2 + (axis as u64) * 2 + positive as u64,
    }
}

impl KbHit {
    #[must_use]
    pub const fn id(self) -> NodeId {
        match self {
            Self::Row(a) => off(1, a.0 as u64),
            Self::Add => off(2, 0),
            Self::Remove => off(2, 1),
            Self::Sub(s) => off(3, s as u64),
            Self::Key(k) => off(4, k.0 as u64),
            Self::Pad(b) => off(5, pad_code(b)),
            Self::TileX(n) => off(6, n as u64),
            Self::Listen => off(7, 0),
            Self::BarDead => off(7, 1),
            Self::BarPress => off(7, 2),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KbState {
    /// A acção aberta (por id: apagar outra não a troca).
    pub open: Option<ActionId>,
    pub sub: Sub,
    /// O alvo sob o ponteiro (o pé do teclado, o × dos ladrilhos, a dica).
    pub hover: Option<KbHit>,
    /// O campo do nome novo está aberto na lista («+ Add»).
    pub adding: bool,
    /// O nome acabado de pedir: abre-se quando a família o criar.
    pub select_name: Option<String>,
    /// Os alvos da última pintura (para achar o `KbHit` de um `NodeId`).
    pub(crate) targets: Vec<KbHit>,
    /// A barra do curso da última pintura (x, largura): o arrasto das marcas lê-a.
    pub(crate) bar: Option<(f32, f32)>,
    /// Onde o ponteiro está (o × aparece com o ponteiro sobre o ladrilho).
    pub(crate) pointer: Option<(f32, f32)>,
    /// As barras da última pintura: cada alvo desenhado e as acções que o usam (o censo da lei «uma barra por acção»).
    pub bars: Vec<(KbHit, Vec<ActionId>)>,
}

impl KbState {
    #[must_use]
    pub fn hit_of(&self, id: NodeId) -> Option<KbHit> {
        self.targets.iter().copied().find(|h| h.id() == id)
    }
}
