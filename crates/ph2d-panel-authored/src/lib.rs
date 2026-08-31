#![forbid(unsafe_code)]
//! `ph2d-panel-authored` — **o painel que o artista DESENHOU, vivo** (plano UI/UX §4 W8b.2).
//!
//! # O que esta crate é, em uma frase
//!
//! A W8b.1 fez a árvore autorada descrever um painel e o app escrever o código dele. Esta crate é
//! o outro lado: a tabela emitida está **compilada** ([`generated`]) e vira `populate`/`paint`/
//! `apply_event` sobre os widgets do catálogo.
//!
//! # A fronteira que o §2 protege, e que esta crate não pode cruzar
//!
//! *O desenho é a PELE, o widget é o COMPORTAMENTO, o token é a PONTE.* Nada aqui implementa
//! *drag*, *foco*, *teclado* ou *a11y*: cada row é pintada por
//! [`paint_widget_skin_with`](ph2d_editor_core::widget::paint_widget_skin_with) — a MESMA porta que
//! a prévia de canvas percorre — e o gesto é resolvido pelo dispatch genérico do
//! `WidgetStore`. Uma linha de comportamento aqui seria o interpretador que o §2 recusou.
//!
//! # UMA TABELA, QUATRO CONSUMIDORES
//!
//! `paint`, `populate`, `event` e a varredura de seam percorrem [`rows::rows`]. É a lei do
//! `SECTIONS` do painel de física, e é ela que torna **estrutural** o requisito mais duro da W8b —
//! *o gerado passa os gates que o repo cobra de código escrito à mão*: uma row não pode ser
//! pintada-e-não-registada, porque as duas metades leem a mesma lista.
//!
//! # O TÍTULO é do artista; o ID é da plumbing
//!
//! O que a moldura nomeia é o **título** ([`generated::PANEL_TITLE`]) — o que o artista lê na
//! barra. O [`AuthoredPanel::ID`] é `"authored"`, fixo.
//!
//! ⚠️ **Derivá-lo do desenho foi construído e desfeito**, e a razão é uma consequência que só
//! aparece a jusante: o `ID` é a chave do mapa de visibilidade e o nome que TODA lista de painéis
//! do shell carrega como literal (o gate de registro, o fallback de z-order). Com ele derivado,
//! renomear a moldura deixaria uma entrada órfã no mapa, tornaria aquelas listas impossíveis de
//! escrever, e o app passaria a pensar *"é outro painel"* quando o artista só trocou um rótulo.
//! `generated::PANEL_ID` continua sendo emitido e **lido** (o gate do golden compilado o confere):
//! ele é o slug, e é a identidade que a fatia multi-painel vai usar para nomear a crate.

pub mod generated;
pub mod ids;
mod paint;
mod populate;
pub mod rows;
pub mod state;

mod event;

pub use state::{AuthoredIntent, AuthoredPanelState, drain_intents};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Marcador de tamanho zero que implementa o contrato tipado do painel autorado.
///
/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Name>Panel` deste arquivo e
/// entra em pânico se ele faltar.
pub struct AuthoredPanel;

/// **A chave de visibilidade deste painel**, para quem não quer o trait em escopo.
///
/// ⚠️ Ela **resolve** `<AuthoredPanel as Panel>::ID` em vez de repetir o valor: uma const paralela
/// seria a segunda resposta a *"como este painel se chama no mapa de visibilidade?"*, e ela
/// divergiria no dia em que o artista renomeasse a moldura — com o interruptor a alternar a
/// visibilidade de um painel que não existe, e todos os gates verdes (a cicatriz do painel de
/// física, W2b).
#[must_use]
pub fn visibility_key() -> &'static str {
    <AuthoredPanel as Panel>::ID
}

impl Panel for AuthoredPanel {
    type State = AuthoredPanelState;

    /// ⚠️ Fixo — ver o § *"O TÍTULO é do artista; o ID é da plumbing"* no topo. O nome que a
    /// moldura dá aparece no TÍTULO, que é o que o artista lê.
    const ID: &'static str = "authored";
    const NODE_ID: NodeId = ph2d_editor_core::ids::AUTHORED_PANEL;
    /// Fechado até ser pedido — o interruptor mora na seção **Frame** do painel Vector, ao lado do
    /// gesto que criou a moldura.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: &'static str = "Authored UI";
    /// ⚠️ **Um painel de COLUNA não cabe na faixa de baixo.** Ela tem 240 px de altura e a
    /// largura da área: uma lista de propriedades ali fica com duas linhas visíveis. ⇒ as duas
    /// colunas, e o gesto que o levaria ao fundo não é oferecido (decisão D1).
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    /// ⭐ **DECLARA que flutua** (D1): janela FLUTUANTE: ela lê `blender_picker_offset` + `panel_resize_delta` e clampa-os na própria crate.
    ///
    /// ⚠️ A declaração descreve o que este painel **FAZ**, e o gate
    /// `a_docked_panel_never_reaches_the_drawing_area` impede-a de mentir nos dois sentidos —
    /// quem não declara **não pode** publicar um rect sobre a área de desenho.
    const CAN_FLOAT: bool = true;

    fn paint(state: &mut AuthoredPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut AuthoredPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
