//! ⭐⭐⭐ **`ph2d-panel-tags`** — o painel das TAGS (TOP-20 #9, W4), `Panel<State>` tipado (ADR-0029).
//!
//! A **taxonomia do projecto**: criar uma tag, aninhá-la, renomeá-la, apagá-la (com a subárvore e a
//! pertença, num passo de undo), e escolher na cena todos os objectos que lhe pertencem.
//!
//! # ⚠️ Ele NÃO é a secção *Tags* do Inspector, e a diferença é o SUJEITO
//!
//! A secção responde *«que tags este objecto tem»* e escreve no componente dele; este responde
//! *«que tags o projecto tem»* e escreve na **árvore**, que não está no mundo. São dois documentos,
//! e é por isso que os dois gestos viajam por acções diferentes
//! ([`InspectorTagsEdit`](ph2d_editor_core::action_bus::EditorAction::InspectorTagsEdit) contra
//! [`TagTreeEdit`](ph2d_editor_core::action_bus::EditorAction::TagTreeEdit)).
//!
//! # ⚠️ Categoria MUNDO, como o de física
//!
//! Ele não é gateado por ferramenta nem docado à selecção: é sempre alcançável e edita o
//! **documento**. Nasce FECHADO porque a maioria dos projectos não tem tag nenhuma — e a porta dele
//! num projecto vazio é o *Window → Tags*, que é exactamente onde o artista carrega em *+ New* para
//! fazer a primeira.
//!
//! # ⚠️ Ele não conhece a folha das tags
//!
//! O instantâneo chega em `u64` + `String` (`TagsPanelInfo`), como o do Inspector. Quem conhece a
//! [`ph2d_tags::TagTree`](https://docs.rs) é a shell — inclusive a **frase de cada recusa**, que
//! nasce ao lado da lei que a produziu e atravessa este painel sem ser interpretada.

#![forbid(unsafe_code)]

pub mod ids;
mod paint;
mod rows;
mod seam;
pub mod state;

pub use state::{TagsPanelState, current_tags, last_content_h, set_born_tag, set_current_tags};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// Marcador de tamanho zero que implementa o contrato tipado do painel de tags.
///
/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro e
/// entra em pânico se ele faltar.
pub struct TagsPanel;

impl Panel for TagsPanel {
    type State = TagsPanelState;

    const ID: &'static str = "tags";
    const NODE_ID: NodeId = ph2d_editor_core::ids::TAGS_PANEL;
    /// ⛔ **Nasce FECHADO** — a maioria dos projectos não tem tag nenhuma, e um painel que se abre
    /// sozinho para todos é chrome que se dispensa em vez de se procurar.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.tags.title");
    /// ⭐ O glifo `tag` já existia em `docs/design/icons/` e **nenhum painel o usava** — é o
    /// assunto exacto, e o `no_two_panels_share_a_glyph` fica verde.
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Tag;
    /// ⚠️ **Uma árvore não cabe na faixa de baixo** (240 px): ali ela mostra duas linhas. As duas
    /// colunas, como o painel de física e o do esqueleto.
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;

    fn paint(state: &mut TagsPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut TagsPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        seam::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        seam::populate(store);
    }
}
