//! `Panel` trait — typed contract each panel crate implements.
//!
//! ADR-0029 §4.3 — chosen over `dyn Any`-based registry. Rationale:
//! compile-time check of `Panel ↔ State` relationship. Refactor of
//! state struct (rename, split, mover fields) caught by rustc, not
//! by runtime smoke. Cost: ~50 LOC of glue in [`crate::panel::erased`]
//! that does the downcast once at install. Worth the cost for
//! foundation that lasts years.

use super::EventOutcome;
use super::PaintCtx;
use super::PanelHostInternal;
use crate::interaction::{WidgetEvent, WidgetStore};
use ph2d_a11y::NodeId;

/// A panel implementation. Each `ph2d-panel-*` crate defines exactly
/// one type implementing this trait + a `pub static MANIFEST: PanelManifest`
/// constructed via [`super::PanelManifest::for_panel::<Self>()`].
///
/// `State` is the panel's per-instance retained state — what was
/// previously a `pub(super) struct InspectorState`/`HierarchyState`/etc.
/// living flat on `HeroScreen`. Now owned by `ErasedPanel<Self>` and
/// passed typed to each fn.
pub trait Panel: Sized + 'static {
    /// Per-instance retained state. Allocated once when the panel
    /// is registered, owned by the registry, passed `&mut` to each
    /// fn. `Default` enforced so `register_all_panels()` can build
    /// the registry without painel-specific constructors.
    type State: Default + Send + 'static;

    /// Short stable identifier for logs/MCP/ADR refs. Snake-case
    /// (`"inspector"`, `"widget_gallery"`).
    const ID: &'static str;

    /// `NodeId` of the panel's outer rect — used by `paint_hero_screen`
    /// to look up the manifest by `z_order`'s panel_id.
    const NODE_ID: NodeId;

    /// Default `visible` value when the host doesn't supply one.
    const DEFAULT_VISIBLE: bool;

    /// ⭐⭐ **O NOME que o artista lê** — a etiqueta da aba quando este painel divide um encaixe
    /// com outro (spec §2, regra 1).
    ///
    /// ⛔ **Sem default, de propósito.** Um default derivado do [`Self::ID`] daria *"Tokens"* onde
    /// o menu *Window* diz *"Design Tokens"* e *"Sculpt3d"* onde ele diz *"Sculpt 3D"* — e o
    /// artista teria **dois nomes para o mesmo painel**, um em cada superfície. O gate
    /// `the_tab_and_the_menu_call_a_panel_the_same_thing` mede exactamente isso, conduzido pela
    /// tabela [`crate::screens::hero::menu_bar::MODULE_TRUTHS`].
    const TITLE: &'static str;

    /// ⭐⭐ **ONDE ESTE PAINEL PODE ESTAR** (decisão **D1**).
    ///
    /// O default é [`SlotSet::ANY_DOCK`] — as duas colunas e a faixa de baixo, **nunca o centro**.
    /// Um painel de propriedades que declare `SlotSet::RIGHT` **não consegue** ser posto sobre a
    /// viewport: não há valor que o exprima. *É um `Constraint`, não uma verificação.*
    ///
    /// ⚠️ **Tem default para os 24 painéis não terem de mudar todos no mesmo commit** — quem
    /// precisa de ser mais estreito que o default declara-o.
    const ALLOWED_SLOTS: crate::screens::slot::SlotSet = crate::screens::slot::SlotSet::ANY_DOCK;

    /// ⭐⭐ **Em QUAL dos seis encaixes ele está.**
    ///
    /// ⛔⛔ **Sem default desde 2026-08-30, e a razão é uma medição:** com um default de
    /// `RightTop`, **20 dos 21** painéis registados declaravam-no — e **três mentiam**
    /// (`hierarchy` publica a coluna da ESQUERDA, `timeline` e `flip_frames` a faixa de BAIXO).
    /// *Uma declaração que ninguém confronta com a realidade é decoração*, e ela só passou a
    /// custar quando as abas começaram a derivar dela quem divide o quê.
    ///
    /// ⚠️ Tem de estar dentro de [`Self::ALLOWED_SLOTS`] (gate
    /// `every_panel_is_born_inside_its_own_allowed_slots`) **e** conter o rect que o painel
    /// publica (gate `the_slot_a_panel_declares_is_where_it_paints`).
    const DEFAULT_SLOT: crate::screens::slot::Slot;

    /// ⭐⭐ **Ele pode sair para uma janela própria?** (decisão **D1**.)
    ///
    /// ⛔ **`false` por omissão, e é a metade que importa:** um painel que não flutua **nunca**
    /// publica um rect por cima da área de desenho, e há gate a medi-lo sobre o quadro real
    /// (`ph2d-panel-registry-init/tests/a_docked_panel_never_reaches_the_drawing_area.rs`).
    ///
    /// ⚠️ **Declarar `true` é declarar que o artista o ARRASTA** — o Grid Snap, a galeria de
    /// widgets e o `authored` têm rect próprio com clamp na crate deles, e é por isso que eles o
    /// declaram. *A declaração descreve o que o painel FAZ, e o gate impede-a de mentir.*
    const CAN_FLOAT: bool = false;

    /// Paint the panel for one frame. State + host are typed; the
    /// orchestrator dispatches through `ErasedPanel` which downcasts
    /// once at install.
    fn paint(state: &mut Self::State, ctx: &mut PaintCtx);

    /// Handle one `WidgetEvent`. Returns [`EventOutcome`] declaring
    /// whether the event was consumed exclusively, observed (side
    /// effect, no exclusivity), or ignored.
    fn apply_event(
        state: &mut Self::State,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome;

    /// Pre-register the panel's widget `NodeId`s against the
    /// `WidgetStore` at construction time. Matches the existing
    /// `pub fn populate(&mut WidgetStore)` shape.
    fn populate(store: &mut WidgetStore);
}
