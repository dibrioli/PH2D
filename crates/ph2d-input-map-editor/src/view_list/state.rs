//! O estado da vista A: a busca por tecla, o aviso do Add recusado, o arrasto de um número e o rolar.

use ph2d_input::{ActionId, Key};

/// Qual dos dois números de uma linha com manche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Dead,
    Press,
}

/// Um arrasto de número em curso (scrub).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scrub {
    pub action: ActionId,
    pub zone: Zone,
    pub start_x: f32,
    pub start: f32,
    /// A largura do campo: arrastar a largura inteira percorre 0 → 1.
    pub width: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListState {
    /// O filtro POR TECLA (o chip dentro da busca).
    pub search_key: Option<Key>,
    /// O porquê de o último Add ter sido recusado (chave i18n).
    pub refused: Option<&'static str>,
    pub scrub: Option<Scrub>,
    /// A linha sob o ponteiro (mostra × nas teclas e o lixo).
    pub hover_row: Option<ActionId>,
    /// O alvo sob o ponteiro (a dica).
    pub hover_target: Option<ph2d_a11y::NodeId>,
    /// Pedido à escuta partilhada: `Some(Some(a))` arma, `Some(None)` desarma. A família drena-o.
    pub listen_request: Option<Option<ActionId>>,
    /// Quanto a lista rolou (px, ≥ 0) e quanto pode rolar.
    pub scroll: f32,
    pub scroll_max: f32,
}
