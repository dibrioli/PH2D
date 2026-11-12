//! O estado da vista A: a busca por tecla, o aviso do Add recusado, o arrasto de um número e o rolar.

use ph2d_input::{ActionId, Key};

/// Qual dos dois números de uma linha com manche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Dead,
    Press,
}

impl Zone {
    /// O maior valor que o mapa guarda para este número (`InputAction::set_zone`).
    #[must_use]
    pub const fn max(self) -> f32 {
        match self {
            Self::Dead => ph2d_input::MAX_DEAD_ZONE,
            Self::Press => 1.0,
        }
    }
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
    /// O ponteiro andou: soltar sem andar abre o campo para escrever.
    pub moved: bool,
}

/// O número a ser escrito (clique sem arrasto): `Enter` ou clicar fora aplica, `Esc` desiste.
#[derive(Clone, Debug, PartialEq)]
pub struct Typing {
    pub action: ActionId,
    pub zone: Zone,
    pub buffer: String,
}

/// A barra da lista na última pintura: o polegar e as medidas que o arrasto converte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Bar {
    pub thumb: ph2d_editor_core::zones::Rect,
    pub track_h: f32,
    pub content_h: f32,
    pub visible_h: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListState {
    /// O filtro POR TECLA (o chip dentro da busca).
    pub search_key: Option<Key>,
    /// O porquê de o último Add ter sido recusado (chave i18n).
    pub refused: Option<&'static str>,
    pub scrub: Option<Scrub>,
    pub typing: Option<Typing>,
    /// O caret, a selecção e o valor do foco do número aberto (`field.rs`).
    pub caret: crate::field::Caret,
    /// O último número que ficou no limite: a frase Warn ao lado dele, até à próxima edição válida.
    pub warn: Option<(ActionId, Zone, crate::field::Clamp)>,
    /// A linha sob o ponteiro (mostra × nas teclas e o lixo).
    pub hover_row: Option<ActionId>,
    /// O alvo sob o ponteiro (a dica).
    pub hover_target: Option<ph2d_a11y::NodeId>,
    /// Pedido à escuta partilhada: `Some(Some(a))` arma, `Some(None)` desarma. A família drena-o.
    pub listen_request: Option<Option<ActionId>>,
    /// Quanto a lista rolou (px, ≥ 0) e quanto pode rolar.
    pub scroll: f32,
    pub scroll_max: f32,
    pub(crate) bar: Option<Bar>,
    /// O polegar agarrado: o y do ponteiro e o rolar ao carregar.
    pub thumb_drag: Option<(f32, f32)>,
    /// Onde cada byte de fronteira de um campo de texto caiu na última pintura (o clique põe o caret ali).
    pub(crate) text_stops: Vec<(ph2d_a11y::NodeId, Vec<(usize, f32)>)>,
    /// O arrasto de selecção num campo de texto: o campo e a âncora (o byte do Down).
    pub text_drag: Option<(ph2d_a11y::NodeId, usize)>,
    /// A ação a trazer à vista na próxima pintura (a busca da faixa escolheu-a).
    pub reveal: Option<ActionId>,
}

impl ListState {
    /// O polegar da barra da última pintura (`None` = a lista cabe).
    #[must_use]
    pub fn thumb(&self) -> Option<ph2d_editor_core::zones::Rect> {
        self.bar.map(|b| b.thumb)
    }
}
