#![forbid(unsafe_code)]
//! `ph2d-panel-tokens` — **a tabela de cor do design system, autorável** (plano UI/UX W6, degrau 1).
//!
//! # A decisão que este painel realiza
//!
//! O §2 do plano decide que *o desenho é a PELE, o widget é o COMPORTAMENTO, e o token é a PONTE*.
//! Este é o primeiro degrau dessa escada, e ele **não tem código de pintura novo**: o caminho
//! `tokens.json → build.rs → consts → 44 widgets` já existe, e `ColorToken::resolve` já é a porta
//! única por onde todos passam. Editar um valor ali re-veste **o app inteiro** — não uma prévia,
//! não um mockup: a janela em que o artista está a clicar.
//!
//! # Categoria MUNDO, e a razão não é arrumação
//!
//! A tabela de tokens não é propriedade de nada que se selecione. Uma seção do Inspector precisaria
//! de uma seleção para existir, e *"de que cor é a superfície deste editor?"* não tem sujeito. É a
//! categoria do painel de física (ADR-0131 D8), e o abridor é a tecla **`T`** pelo mesmo motivo que
//! o `W` existe.
//!
//! # O painel não guarda cópia de token nenhum
//!
//! A lista é `ColorToken::ALL`, o valor sai de `resolve`, o "está autorado?" sai de
//! `overrides::color_override`, e o modo vem do host. Um snapshot publicado por frame — o padrão do
//! painel de física — seria uma **segunda cópia da tabela de cor**, num painel cuja razão de
//! existir é ser a autoridade sobre ela.
//!
//! # UM escritor
//!
//! O painel **lê** a camada de override e emite [`TokensIntent`]s; quem escreve é a shell, que já é
//! obrigada a fazê-lo no read-back do picker (o estado do picker mora no store que ela possui).
//! Dois escritores para a mesma tabela dariam um que esquece de marcar o projeto como sujo.

pub mod ids;
mod paint;
mod paint_num;
mod populate;
pub mod state;

mod event;

pub use state::{TokensIntent, TokensPanelState, drain_intents, last_content_h, last_visible_h};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Marcador de tamanho zero que implementa o contrato tipado do painel de Tokens.
///
/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Name>Panel` deste arquivo e
/// entra em pânico se ele faltar.
pub struct TokensPanel;

impl Panel for TokensPanel {
    type State = TokensPanelState;

    const ID: &'static str = "tokens";
    const NODE_ID: NodeId = ph2d_editor_core::ids::TOKENS_PANEL;
    /// Fechado até ser pedido. Re-vestir o app é uma sessão, não o estado normal de trabalho — e
    /// um painel que se abre sozinho é chrome que se dispensa em vez de se procurar.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: &'static str = "Design Tokens";
    /// ⚠️ **Um painel de COLUNA não cabe na faixa de baixo.** Ela tem 240 px de altura e a
    /// largura da área: uma lista de propriedades ali fica com duas linhas visíveis. ⇒ as duas
    /// colunas, e o gesto que o levaria ao fundo não é oferecido (decisão D1).
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;

    fn paint(state: &mut TokensPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut TokensPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
